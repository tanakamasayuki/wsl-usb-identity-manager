//! Target identification probes.
//!
//! A transport — a CH340, a CP210x, a WCH-Link — identifies itself, never the
//! board behind it. Reaching the board means asking it, and asking it has a
//! cost: an ESP32 restarts, a WCH-Link halts the target core. That cost is why
//! probing lives here rather than in `wuim-core`, where the enumeration paths
//! could reach it by accident (requirement R4.6).
//!
//! Every *probe* here runs only on the two triggers of requirement R4.5: an
//! explicit request from the user, or the opt-in window right after a device is
//! plugged in. [`usb_descriptor`] is not a probe and is not subject to that: it
//! reads what Windows already enumerated and sends nothing, so there is no cost
//! to gate.
//!
//! Adding a family means adding a module and one line in [`probes`], with no
//! change to the families already here (requirement R4.14).
//!
//! Probes are ordered by how surely they recognise a device. One that knows the
//! hardware from its VID/PID goes first and, if it claims the device, stops the
//! generic ones being offered at all. Without that, every new family would have
//! to be added to a list of exceptions inside the generic probe, and that list
//! would grow for as long as families do.

pub mod board_ids;
pub mod ch32;
pub mod esp32;
pub mod identity;
pub mod usb_descriptor;

use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;
use wuim_core::windevice::WinUsbDevice;

/// Where a unique id came from.
///
/// The vocabulary is board-identify's, so a device identified by both tools is
/// described the same way by both. It is shown, not acted on: the difference
/// between reading silicon and reading a descriptor is one a person weighs, not
/// one this application branches on.
pub mod id_sources {
    /// Read from the silicon — an ESP32's eFuse MAC. Survives a reflash, and a
    /// bridge in front of it being swapped.
    pub const TARGET_MAC: &str = "target-mac";
    /// Read from the silicon — a CH32's factory UUID, through a debug probe.
    pub const TARGET_CPU_ID: &str = "target-cpu-id";
    /// The serial number in the board's own USB descriptors. Identifies the
    /// unit only because the pair it comes with identifies the board (see
    /// [`crate::usb_descriptor`]).
    pub const USB_SERIAL: &str = "usb-serial";
}

/// A message this crate produces for a person to read.
///
/// It carries a stable `code` rather than only prose, because the GUI is
/// translated and matching on English sentences to translate them breaks the
/// moment the wording is edited. `en` is the same message for logs and the CLI,
/// which are not translated.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub struct Note {
    /// Translation key, e.g. `probe.blocked.no_com_port`.
    pub code: &'static str,
    pub en: &'static str,
}

impl std::fmt::Display for Note {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.en)
    }
}

/// What a probe can say about a device *before* touching it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(tag = "status", content = "reason", rename_all = "snake_case")]
pub enum Applicability {
    /// This probe recognises the device and can attempt it.
    Supported,
    /// Another family's business, or not a board at all.
    NotApplicable(Note),
    /// The right family, but something has to be resolved first — a driver to
    /// assign, an attach to release.
    Blocked(Note),
}

impl Applicability {
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Supported)
    }

    /// The message behind a negative verdict, if there is one.
    pub fn note(&self) -> Option<Note> {
        match self {
            Self::Supported => None,
            Self::NotApplicable(note) | Self::Blocked(note) => Some(*note),
        }
    }
}

/// How a probe decides a device is its business.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Recognition {
    /// The device's own identifiers say so: a VID/PID this probe owns. Certain,
    /// and decided without touching anything.
    ByIdentifier,
    /// Anything of the right shape — a serial port, which could have any board
    /// behind it, or none. Only offered when nothing more specific claimed the
    /// device.
    Fallback,
}

/// Reasons shared by every probe, so the UI only has to translate them once.
pub mod notes {
    use super::Note;

    pub const CLAIMED_BY_ANOTHER: Note = Note {
        code: "probe.blocked.claimed_by_another",
        en: "another probe recognises this hardware",
    };

    pub const NOT_CONNECTED: Note = Note {
        code: "probe.blocked.not_connected",
        en: "the device is not connected",
    };
    pub const ATTACHED: Note = Note {
        code: "probe.blocked.attached",
        en: "attached to WSL, so Windows cannot reach the device",
    };
}

/// What a successful probe learned about the target.
///
/// Mirrors the Application Identity of requirements §3.4.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TargetIdentity {
    /// The family that produced this, e.g. `esp32`.
    pub family: &'static str,
    /// The persisted key, `<variant>-<unique-id>` (requirements §4.5).
    pub identity_key: String,
    /// The unique id as the chip reports it, e.g. a MAC address.
    pub device_id: String,
    /// The chip variant, e.g. `esp32-s3`.
    pub device_type: String,
    pub hardware_revision: Option<String>,
    /// Where [`Self::device_id`] came from; one of [`id_sources`].
    pub id_source: &'static str,
    /// Anything else worth showing in a detail pane. Not part of the identity.
    pub details: BTreeMap<String, String>,
}

/// One identifiable family of targets.
pub trait TargetProbe: Send + Sync {
    /// Stable family name. Appears in stored data, so treat it as an identifier.
    fn family(&self) -> &'static str;

    /// How surely this probe knows a device is its own. See [`Recognition`].
    fn recognition(&self) -> Recognition;

    /// What this probe does to the device, in one sentence.
    ///
    /// Requirement R4.7 requires showing this before running a probe, so every
    /// implementation has to be able to state its own side effects.
    fn side_effect(&self) -> Note;

    /// Decides whether the probe can be attempted. Must not touch the device.
    fn applicability(&self, device: &WinUsbDevice) -> Applicability;

    /// Asks the target what it is. **Has side effects** — see [`Self::side_effect`].
    fn probe(&self, device: &WinUsbDevice) -> Result<TargetIdentity>;
}

/// Every probe that ships, most specific first.
///
/// The order is the contract: [`Recognition::ByIdentifier`] probes come before
/// [`Recognition::Fallback`] ones, and a test below holds that to it.
pub fn probes() -> Vec<Box<dyn TargetProbe>> {
    vec![Box::new(ch32::Ch32Probe), Box::new(esp32::Esp32Probe)]
}

/// What each probe makes of a device, most specific first.
///
/// A probe that recognised the hardware from its identifiers has the last word.
/// A WCH-Link exposes a serial port of its own, which to a generic serial probe
/// is indistinguishable from an adapter with a board behind it; letting that
/// probe run anyway would send a reset and a sync to a debug probe's console
/// for no possible gain. "Recognised" includes being blocked — an ARM-mode
/// WCH-Link is still a WCH-Link, and still not something to poke at.
pub fn applicable(device: &WinUsbDevice) -> Vec<(Box<dyn TargetProbe>, Applicability)> {
    let verdicts: Vec<(Box<dyn TargetProbe>, Applicability)> = probes()
        .into_iter()
        .map(|p| {
            let verdict = p.applicability(device);
            (p, verdict)
        })
        .collect();

    let claimed = verdicts.iter().any(|(probe, verdict)| {
        probe.recognition() == Recognition::ByIdentifier
            && !matches!(verdict, Applicability::NotApplicable(_))
    });

    verdicts
        .into_iter()
        .map(|(probe, verdict)| {
            if claimed && probe.recognition() == Recognition::Fallback {
                let note = notes::CLAIMED_BY_ANOTHER;
                (probe, Applicability::NotApplicable(note))
            } else {
                (probe, verdict)
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The order in `probes()` is what makes the claim rule work: a fallback
    /// listed first would be offered before the probe that actually knows the
    /// hardware.
    #[test]
    fn specific_probes_come_before_the_fallbacks() {
        let mut seen_fallback = false;
        for probe in probes() {
            match probe.recognition() {
                Recognition::Fallback => seen_fallback = true,
                Recognition::ByIdentifier => assert!(
                    !seen_fallback,
                    "{} recognises by identifier but is listed after a fallback",
                    probe.family()
                ),
            }
        }
    }
}
