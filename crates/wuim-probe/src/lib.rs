//! Target identification probes.
//!
//! A transport — a CH340, a CP210x, a WCH-Link — identifies itself, never the
//! board behind it. Reaching the board means asking it, and asking it has a
//! cost: an ESP32 restarts, a WCH-Link halts the target core. That cost is why
//! probing lives here rather than in `wuim-core`, where the enumeration paths
//! could reach it by accident (requirement R4.6).
//!
//! Everything in this crate runs only on the two triggers of requirement R4.5:
//! an explicit request from the user, or the opt-in window right after a device
//! is plugged in.
//!
//! Adding a family means adding a module and one line in [`probes`], with no
//! change to the families already here (requirement R4.14).

pub mod esp32;
pub mod identity;

use std::collections::BTreeMap;

use anyhow::Result;
use serde::Serialize;
use wuim_core::windevice::WinUsbDevice;

/// What a probe can say about a device *before* touching it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "status", content = "reason", rename_all = "snake_case")]
pub enum Applicability {
    /// This probe recognises the device and can attempt it.
    Supported,
    /// Another family's business, or not a board at all.
    NotApplicable(&'static str),
    /// The right family, but something has to be resolved first — a driver to
    /// assign, an attach to release. The reason is meant to be shown verbatim.
    Blocked(String),
}

impl Applicability {
    pub fn is_supported(&self) -> bool {
        matches!(self, Self::Supported)
    }
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
    /// Anything else worth showing in a detail pane. Not part of the identity.
    pub details: BTreeMap<String, String>,
}

/// One identifiable family of targets.
pub trait TargetProbe: Send + Sync {
    /// Stable family name. Appears in stored data, so treat it as an identifier.
    fn family(&self) -> &'static str;

    /// What this probe does to the device, in one sentence.
    ///
    /// Requirement R4.7 requires showing this before running a probe, so every
    /// implementation has to be able to state its own side effects.
    fn side_effect(&self) -> &'static str;

    /// Decides whether the probe can be attempted. Must not touch the device.
    fn applicability(&self, device: &WinUsbDevice) -> Applicability;

    /// Asks the target what it is. **Has side effects** — see [`Self::side_effect`].
    fn probe(&self, device: &WinUsbDevice) -> Result<TargetIdentity>;
}

/// Every probe that ships, in the order they should be tried.
pub fn probes() -> Vec<Box<dyn TargetProbe>> {
    vec![Box::new(esp32::Esp32Probe)]
}

/// Picks the probes that could be attempted against a device, each with the
/// verdict that chose it, so a caller can explain why nothing applied.
pub fn applicable(device: &WinUsbDevice) -> Vec<(Box<dyn TargetProbe>, Applicability)> {
    probes()
        .into_iter()
        .map(|p| {
            let verdict = p.applicability(device);
            (p, verdict)
        })
        .collect()
}
