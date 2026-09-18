//! CH32 RISC-V family, identified through a WCH-Link.
//!
//! The factory UUID is burned into the part and is what tells two boards on
//! identical probes apart. The probe's own serial number identifies the probe,
//! not the board wired to its debug pins (requirements §1.2), so it is the UUID
//! that matters here.
//!
//! The protocol work is `ch32rv`'s, not ours (requirements §4.6). That crate
//! also solves the driver question: it opens the probe through nusb, and on
//! Windows falls back to WCH's own vendor driver when nusb cannot claim the
//! interface. So a WCH-Link left on the stock `WCHLink_A64` driver works as it
//! is, which is why requirement R4.12 forbids asking the user to replace it.

use std::collections::BTreeMap;

use anyhow::{Result, anyhow, bail};
use ch32rv_target::{Db, Resolution};
use ch32rv_usb::{UsbDeviceInfo, enumerate};
use ch32rv_wchlink::{ChipInfo, ChipInfoStatus, Speed, WchLink};
use wuim_core::windevice::WinUsbDevice;

use crate::identity::identity_key;
use crate::{Applicability, Note, Recognition, TargetIdentity, TargetProbe, id_sources};

/// WCH's vendor id, and the product id a WCH-Link reports in RISC-V mode.
const VID_WCH: u16 = 0x1a86;
const PID_LINK_RISCV: u16 = 0x8010;

pub struct Ch32Probe;

impl TargetProbe for Ch32Probe {
    fn family(&self) -> &'static str {
        "ch32"
    }

    fn recognition(&self) -> Recognition {
        // A WCH-Link announces itself with WCH's vendor id and a product id
        // that also says which mode it is in.
        Recognition::ByIdentifier
    }

    fn side_effect(&self) -> Note {
        Note {
            code: "probe.ch32.side_effect",
            en: "Halts the target core while the probe reads its UUID, then releases it. \
                 A program already running on the board is interrupted.",
        }
    }

    fn applicability(&self, device: &WinUsbDevice) -> Applicability {
        match device.instance_id.vid_pid() {
            Some((VID_WCH, PID_LINK_RISCV)) => Applicability::Supported,
            // A WCH-Link in ARM mode answers on a different product id and
            // cannot speak this protocol. Switching it back is the user's to
            // do: this application does not reconfigure their probe (R4.15).
            Some((VID_WCH, 0x8012)) => Applicability::Blocked(Note {
                code: "probe.blocked.wchlink_arm_mode",
                en: "the WCH-Link is in ARM mode; switch it to RISC-V mode to identify CH32 parts",
            }),
            _ => Applicability::NotApplicable(Note {
                code: "probe.blocked.not_a_wchlink",
                en: "not a WCH-Link",
            }),
        }
    }

    fn probe(&self, device: &WinUsbDevice) -> Result<TargetIdentity> {
        let serial = device.instance_id.unit.serial();
        let info = find_probe(serial)?;

        let mut link =
            WchLink::open(&info).map_err(|e| anyhow!("could not open the WCH-Link: {e}"))?;

        let probe_info = link
            .probe_info()
            .map_err(|e| anyhow!("the WCH-Link did not report its firmware: {e}"))?;

        // Best effort. SetSpeed is how a probe is asked to run the debug
        // link faster; the original CH549-based WCH-Link does not implement it
        // and answers `82 81 01 ff` whether it is sent before or after attach
        // (measured on firmware 2.6). Nothing here needs the speed, so a refusal
        // is noted and stepped over rather than failing an identification that
        // would otherwise have worked.
        let speed_set = link.set_speed_default(Speed::default()).is_ok();

        let attached = link
            .attach_chip()
            .map_err(|e| anyhow!("no target answered on the debug pins: {e}"))?;

        let chip = link.chip_info();

        // Released whatever happened, so a failed read does not leave the
        // target halted with nothing holding the probe.
        let _ = link.detach_chip();

        let chip = match chip.map_err(|e| anyhow!("could not read the chip info: {e}"))? {
            ChipInfoStatus::Ok(chip) => chip,
            ChipInfoStatus::NoAnswer => {
                bail!("the target did not answer: check the debug wiring and that it has power")
            }
            other => bail!("the probe returned an unusable chip info ({other:?})"),
        };

        let mut identity = build_identity(
            self.family(),
            &probe_info_summary(&probe_info),
            attached.chip_id,
            &chip,
        )?;
        if !speed_set {
            identity.details.insert(
                "debug_speed".into(),
                "default (probe refused SetSpeed)".into(),
            );
        }
        Ok(identity)
    }
}

/// Finds the probe to talk to among those plugged in.
///
/// Matched on the serial number when Windows gave us one, so the right probe is
/// used when several are connected. Enumeration here is `ch32rv`'s, which is
/// separate from the CfgMgr32 listing the rest of the application uses — the
/// two see the same hardware through different APIs and only the serial number
/// is common to both.
fn find_probe(serial: Option<&str>) -> Result<UsbDeviceInfo> {
    let devices = enumerate().map_err(|e| anyhow!("could not enumerate USB devices: {e}"))?;
    let mut candidates = devices
        .into_iter()
        .filter(|d| d.vid() == VID_WCH && d.pid() == PID_LINK_RISCV);

    match serial {
        Some(serial) => candidates
            .find(|d| d.serial() == Some(serial))
            .ok_or_else(|| anyhow!("no WCH-Link with serial {serial} is connected")),
        // Without a serial there is nothing to match on, so this is only safe
        // when exactly one probe is present.
        None => {
            let found: Vec<UsbDeviceInfo> = candidates.collect();
            match found.len() {
                0 => Err(anyhow!("no WCH-Link is connected")),
                1 => Ok(found.into_iter().next().unwrap()),
                n => Err(anyhow!(
                    "{n} WCH-Links are connected and this one reports no serial number, \
                     so there is no way to tell which is which"
                )),
            }
        }
    }
}

/// Names the probe the way `ch32rv probe list` does, so the two agree.
fn probe_info_summary(info: &ch32rv_wchlink::ProbeInfo) -> String {
    format!(
        "WCH-Link({:?}) {}.{}",
        info.variant, info.fw_major, info.fw_minor
    )
}

fn build_identity(
    family: &'static str,
    probe_summary: &str,
    chip_id: u32,
    chip: &ChipInfo,
) -> Result<TargetIdentity> {
    let uuid: String = chip.uuid.iter().map(|b| format!("{b:02x}")).collect();

    let db = Db::builtin();
    // Bits 4..7 of the chip id are the silicon revision. The database masks them
    // off before matching (`DEVICE_ID_MASK = 0xFFFF_FF0F`) and stores every SKU
    // with them zeroed, which is what makes them the revision and the rest the
    // part: the top nibble, which might read like one, varies between families
    // rather than between revisions of a part.
    let device_type = match db.resolve_by_chip_id(chip_id) {
        // The database also carries a family, but it groups parts more broadly
        // than their name suggests - a CH32V305 sits under a CH32V307 family -
        // so showing both next to each other reads as a contradiction rather
        // than as extra detail.
        Resolution::Sku(sku) => sku.sku.to_lowercase(),
        // Several parts share this id; the family is as far as it narrows.
        Resolution::Family(family_name, _) => family_name.to_lowercase(),
        Resolution::Unknown => format!("ch32-{chip_id:08x}"),
    };

    let identity_key = identity_key(&device_type, &uuid)
        .map_err(|e| anyhow!("could not build an identity key from {device_type}/{uuid}: {e}"))?;

    let mut details = BTreeMap::new();
    details.insert("chip_id".into(), format!("{chip_id:#010x}"));
    details.insert("flash".into(), format!("{} KiB", chip.flash_bytes / 1024));
    details.insert("probe".into(), probe_summary.to_owned());
    Ok(TargetIdentity {
        family,
        identity_key,
        device_id: uuid,
        device_type,
        id_source: id_sources::TARGET_CPU_ID,
        // The revision, from the one nibble the database ignores.
        hardware_revision: Some(format!("rev {}", (chip_id >> 4) & 0xf)),
        details,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chip(uuid: [u8; 8]) -> ChipInfo {
        ChipInfo {
            flash_bytes: 128 * 1024,
            uuid,
            protection_raw: [0; 4],
            chip_id_echo: 0,
        }
    }

    #[test]
    fn the_uuid_becomes_the_identity() {
        let identity = build_identity(
            "ch32",
            "LinkE 2.12",
            0x3050_0601,
            &chip([0x1f, 0xf9, 0xab, 0xcd, 0x88, 0x0e, 0xbc, 0x48]),
        )
        .unwrap();

        assert_eq!(identity.device_id, "1ff9abcd880ebc48");
        assert!(
            identity.identity_key.ends_with("-1ff9abcd880ebc48"),
            "{}",
            identity.identity_key
        );
        assert_eq!(identity.details["flash"], "128 KiB");
    }

    #[test]
    fn an_unknown_chip_id_still_yields_a_usable_key() {
        let identity = build_identity(
            "ch32",
            "LinkE 2.12",
            0xdead_beef,
            &chip([1, 2, 3, 4, 5, 6, 7, 8]),
        )
        .unwrap();

        // The part is not in the database, but the UUID still identifies the
        // board, so the key is worth having.
        assert_eq!(identity.device_id, "0102030405060708");
        assert!(identity.identity_key.contains("deadbeef"));
    }

    #[test]
    fn the_silicon_revision_comes_from_the_nibble_the_database_ignores() {
        // A real CH32V305RBT6 id is 0x3050_0508; the 2 here is a revision.
        let identity = build_identity("ch32", "LinkE 2.12", 0x3050_0528, &chip([9; 8])).unwrap();
        assert_eq!(identity.hardware_revision.as_deref(), Some("rev 2"));
        assert_eq!(identity.device_type, "ch32v305rbt6");
    }

    /// The part that prompted the move to `ch32rv` 0.8.
    ///
    /// 0.7 had no row for this SKU, so the name came out as `ch32-00600620`,
    /// and its `AttachChip` did not know the V00x family byte at all — the read
    /// this builds on never got as far as returning. Pinned here because the id
    /// was measured on the part rather than read off a datasheet.
    #[test]
    fn a_ch32v006_resolves_to_its_sku() {
        let identity = build_identity(
            "ch32",
            "LinkE 2.12",
            0x0060_0620,
            &chip([0x1f, 0xf9, 0xab, 0xcd, 0x88, 0x0e, 0xbc, 0x48]),
        )
        .unwrap();

        assert_eq!(identity.device_type, "ch32v006k8u6");
        assert_eq!(identity.hardware_revision.as_deref(), Some("rev 2"));
        assert_eq!(identity.identity_key, "ch32v006k8u6-1ff9abcd880ebc48");
    }
}
