//! ESP32 family, identified through its ROM bootloader.
//!
//! The eFuse MAC is burned at the factory and is unique per chip, which makes it
//! the one thing that tells two boards behind identical CH340s apart.
//!
//! The protocol work is espflash's, not ours (requirements §11).

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use espflash::connection::{Connection, ResetAfterOperation, ResetBeforeOperation};
use espflash::flasher::{DeviceInfo, Flasher};
use espflash::target::Chip;
use serialport::{FlowControl, UsbPortInfo};
use wuim_core::windevice::WinUsbDevice;

use crate::identity::{esp_variant, identity_key};
use crate::{Applicability, Note, Recognition, TargetIdentity, TargetProbe, id_sources};

/// The ROM loader answers at 115200 baud; there is no reason to go faster for a
/// handful of register reads.
const PROBE_BAUD: u32 = 115_200;

/// Long enough for a board that resets slowly, short enough that a device which
/// is not an ESP32 does not hold the UI.
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

/// Word 3 of the original ESP32's eFuse read window, which carries the package
/// code and the single-core bit (`EFUSE_RD_REG_BASE + 4 * 3`).
const ESP32_EFUSE_WORD3: u32 = 0x3FF5_A000 + 4 * 3;

pub struct Esp32Probe;

impl TargetProbe for Esp32Probe {
    fn family(&self) -> &'static str {
        "esp32"
    }

    fn recognition(&self) -> Recognition {
        // A COM port is all there is to go on. The adapter in front of the
        // board says nothing about the board, so this probe cannot know the
        // device is its own until it has asked.
        Recognition::Fallback
    }

    fn side_effect(&self) -> Note {
        Note {
            code: "probe.esp32.side_effect",
            en: "Resets the board into its ROM bootloader and then back, so the running firmware restarts.",
        }
    }

    fn applicability(&self, device: &WinUsbDevice) -> Applicability {
        if device.com_port.is_none() {
            // Either the board is behind something that is not a serial port, or
            // its driver is not loaded. Either way there is nothing to talk to.
            return Applicability::NotApplicable(Note {
                code: "probe.blocked.no_com_port",
                en: "no COM port, so there is no serial line to talk over",
            });
        }
        Applicability::Supported
    }

    fn probe(&self, device: &WinUsbDevice) -> Result<TargetIdentity> {
        let port_name = device
            .com_port
            .as_deref()
            .ok_or_else(|| anyhow!("device has no COM port"))?;

        let reading = read_device_info(port_name, device)?;
        let info = reading.info;

        let mac = info
            .mac_address
            .ok_or_else(|| anyhow!("the chip did not report a MAC address"))?;
        let variant = match reading.esp32_package_word {
            Some(word3) => {
                let major = info.revision.map(|(major, _)| major).unwrap_or(0);
                esp_variant(esp32_part_name(word3, major))
            }
            None => esp_variant(&info.chip.to_string()),
        };
        let identity_key = identity_key(&variant, &mac)
            .map_err(|e| anyhow!("could not build an identity key from {variant}/{mac}: {e}"))?;

        let mut details = BTreeMap::new();
        details.insert("crystal".into(), info.crystal_frequency.to_string());
        details.insert("flash_size".into(), info.flash_size.to_string());
        if !info.features.is_empty() {
            details.insert("features".into(), info.features.join(", "));
        }
        details.insert("probed_via".into(), port_name.to_owned());

        Ok(TargetIdentity {
            family: self.family(),
            identity_key,
            device_id: mac,
            device_type: variant,
            hardware_revision: info
                .revision
                .map(|(major, minor)| format!("v{major}.{minor}")),
            id_source: id_sources::TARGET_MAC,
            details,
        })
    }
}

/// The part name esptool gives the original ESP32, from its package eFuse.
///
/// espflash's `Chip` is the series, and for the original family that is one name
/// — `esp32` — across parts that esptool distinguishes: `ESP32-D0WD-V3`,
/// `ESP32-PICO-D4`, `ESP32-U4WDH`. From the S2 onwards the series *is* the name,
/// so this is the only place the two disagree.
///
/// Held in step with esptool's `ESP32ROM.get_chip_description()`, because
/// board-identify names a board from that output and one board with two names is
/// the confusion this application exists to remove (R4.27).
fn esp32_part_name(word3: u32, major_revision: u32) -> &'static str {
    let package = ((word3 >> 9) & 0x07) + (((word3 >> 2) & 0x1) << 3);
    let single_core = word3 & 1 != 0;
    let rev3 = major_revision == 3;

    match package {
        0 if single_core => "ESP32-S0WDQ6",
        0 if rev3 => "ESP32-D0WDQ6-V3",
        0 => "ESP32-D0WDQ6",
        1 if single_core => "ESP32-S0WD",
        1 if rev3 => "ESP32-D0WD-V3",
        1 => "ESP32-D0WD",
        2 => "ESP32-D2WD",
        4 => "ESP32-U4WDH",
        5 if rev3 => "ESP32-PICO-V3",
        5 => "ESP32-PICO-D4",
        6 => "ESP32-PICO-V3-02",
        7 => "ESP32-D0WDR2-V3",
        // esptool says "Unknown ESP32" here. That is a sentence, not a name, and
        // this one ends up in an identity key — so the series stands in, which
        // is what every ESP32 was called before the package was read at all.
        _ => "ESP32",
    }
}

/// What one probe read off the chip.
struct Reading {
    info: DeviceInfo,
    /// eFuse word 3, read only for the original ESP32 and only to name the
    /// part. `None` for every other chip, and for a read that did not answer —
    /// in which case the series name still identifies the board perfectly well.
    esp32_package_word: Option<u32>,
}

/// Opens the port, talks to the ROM loader, and puts the board back into a
/// normal boot before letting go.
fn read_device_info(port_name: &str, device: &WinUsbDevice) -> Result<Reading> {
    let serial = serialport::new(port_name, PROBE_BAUD)
        .flow_control(FlowControl::None)
        .timeout(PROBE_TIMEOUT)
        .open_native()
        .with_context(|| format!("could not open {port_name}"))?;

    let (vid, pid) = device.instance_id.vid_pid().unwrap_or((0, 0));
    let port_info = UsbPortInfo {
        // espflash picks its reset sequence from the PID: a board with native
        // USB needs the USB-JTAG-Serial sequence, not the DTR/RTS one.
        vid,
        pid,
        serial_number: device.instance_id.unit.serial().map(str::to_owned),
        manufacturer: device.manufacturer.clone(),
        product: device.bus_reported_device_desc.clone(),
        // The USB topology and interface index are espflash's to ignore; the
        // reset sequence only looks at the PID.
        location: None,
        interface: None,
    };

    let connection = Connection::new(
        serial,
        port_info,
        ResetAfterOperation::HardReset,
        ResetBeforeOperation::DefaultReset,
        PROBE_BAUD,
    );

    // No stub: the eFuse MAC is readable straight from the ROM loader, and
    // skipping the upload keeps the disturbance down to the single reset that
    // getting into the loader already costs.
    let mut flasher = Flasher::connect(connection, false, false, false, None, None)
        .map_err(|e| anyhow!("{port_name} did not answer as an ESP32: {e}"))?;

    let info = flasher.device_info();

    // One more register while the loader is still listening. Best effort: it
    // only refines the name, and espflash has already done the protocol work
    // that makes the read a register read rather than anything of ours (§11).
    let chip = flasher.chip();
    let esp32_package_word = match chip {
        Chip::Esp32 => flasher.connection().read_reg(ESP32_EFUSE_WORD3).ok(),
        _ => None,
    };

    // Reset back into the application regardless of how the read went, so a
    // failed probe does not leave the board sitting in its bootloader.
    if let Err(e) = flasher.connection().reset_after(false, chip)
        && info.is_ok()
    {
        bail!("read the chip but could not reset it back into the application: {e}");
    }

    let info = info.map_err(|e| anyhow!("could not read the chip: {e}"))?;
    Ok(Reading {
        info,
        esp32_package_word,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Package code 1 with the single-core bit clear is the part in almost
    /// every ESP32 module; revision 3 is what a board bought recently reports.
    #[test]
    fn the_common_esp32_is_named_the_way_esptool_names_it() {
        let word3 = 1 << 9; // package 1, dual core
        assert_eq!(esp32_part_name(word3, 3), "ESP32-D0WD-V3");
        assert_eq!(esp32_part_name(word3, 1), "ESP32-D0WD");
        assert_eq!(esp_variant(esp32_part_name(word3, 3)), "esp32-d0wd-v3");
    }

    #[test]
    fn the_single_core_bit_picks_the_s_parts() {
        assert_eq!(esp32_part_name(1 << 9 | 1, 3), "ESP32-S0WD");
        assert_eq!(esp32_part_name(1, 3), "ESP32-S0WDQ6");
    }

    /// The fourth bit of the package code lives apart from the other three.
    #[test]
    fn the_package_code_is_assembled_from_two_places() {
        // Package 5 (PICO) comes from bits 9..11 alone.
        assert_eq!(esp32_part_name(5 << 9, 3), "ESP32-PICO-V3");
        assert_eq!(esp32_part_name(5 << 9, 1), "ESP32-PICO-D4");
        // Bit 2 carries the 8, so 0b1000 is a package this table does not name.
        assert_eq!(esp32_part_name(1 << 2, 3), "ESP32");
    }

    #[test]
    fn an_unnamed_package_falls_back_to_the_series() {
        // 3 is not in esptool's table either.
        assert_eq!(esp32_part_name(3 << 9, 3), "ESP32");
        assert_eq!(esp_variant(esp32_part_name(3 << 9, 3)), "esp32");
    }
}
