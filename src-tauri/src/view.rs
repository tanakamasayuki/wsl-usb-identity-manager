//! The shape the frontend receives.
//!
//! Deliberately a separate type from [`wuim_core::DeviceRow`]: the wire format
//! is something the UI depends on, and pinning it here means the core model can
//! change without silently reshaping what Svelte is reading.

use serde::Serialize;
use wuim_core::snapshot::DeviceRow;
use wuim_probe::notes;

/// One row of either table.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceView {
    /// The join key, and what every command takes to name a device.
    pub instance_id: String,
    pub name: String,
    /// `1a86:7523`, absent for hubs.
    pub vid_pid: Option<String>,

    // Runtime connection. None of this is an identity (requirement R7.1); it is
    // here to be displayed and nothing else.
    pub bus_id: Option<String>,
    pub com_port: Option<String>,
    pub location_path: Option<String>,

    // usbipd state, pre-computed so the table does not have to derive it.
    pub state: &'static str,
    /// usbipd still reports a bus id. True even while the device is attached.
    pub present: bool,
    /// Windows currently exposes a device node we could talk to.
    ///
    /// Distinct from `present`: while a device is attached, usbipd keeps
    /// reporting its bus id but Windows has handed the device to the client, so
    /// nothing here can reach it. This is the signal for "arrived", not
    /// `present`, which never goes false across an attach/detach cycle.
    pub reachable: bool,
    pub shared: bool,
    pub attached: bool,
    /// The client holding the device while attached.
    pub client_address: Option<String>,

    // Identity.
    /// The serial number the device reports, when it reports one. Shown as-is:
    /// "has a serial" tells the user nothing, the value tells them which unit.
    pub serial: Option<String>,
    /// `serial` or `port only`.
    pub identity_basis: &'static str,
    /// Whether the descriptors leave the unit undetermined (finding F3).
    pub needs_probe: bool,

    pub driver: Option<String>,
    /// Probes that could be run, with the side effects to show first (R4.7).
    pub probes: Vec<ProbeOption>,
}

/// A probe the user could choose to run against this device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeOption {
    pub family: &'static str,
    /// Translation key for the side effects to show before running (R4.7).
    pub side_effect: &'static str,
    pub available: bool,
    /// Translation key for why it is unavailable, when it is.
    pub reason: Option<&'static str>,
}

impl DeviceView {
    pub fn from_row(row: &DeviceRow) -> Self {
        let usbipd = row.usbipd.as_ref();
        Self {
            instance_id: row.instance_id.raw.clone(),
            name: row.name.clone(),
            vid_pid: row.instance_id.vid_pid_string(),

            bus_id: row.bus_id().map(str::to_owned),
            com_port: row.com_port().map(str::to_owned),
            location_path: row.location_path().map(str::to_owned),

            state: row.sharing_state().label(),
            present: usbipd.is_some_and(|d| d.is_connected()),
            reachable: row.windows.is_some(),
            shared: usbipd.is_some_and(|d| d.is_shared()),
            attached: usbipd.is_some_and(|d| d.is_attached()),
            client_address: usbipd.and_then(|d| d.client_ip_address.clone()),

            serial: row.instance_id.unit.serial().map(str::to_owned),
            identity_basis: row.identity_basis.label(),
            needs_probe: row.identity_basis.needs_probe(),

            driver: row.windows.as_ref().and_then(|w| w.service.clone()),
            probes: probe_options(row),
        }
    }
}

/// Asks every probe what it makes of the device. Touches nothing.
fn probe_options(row: &DeviceRow) -> Vec<ProbeOption> {
    let Some(device) = row.windows.as_ref() else {
        // Attached or absent: there is no device node to talk to.
        let why = if row.sharing_state() == wuim_core::SharingState::Attached {
            notes::ATTACHED
        } else {
            notes::NOT_CONNECTED
        };
        return wuim_probe::probes()
            .iter()
            .map(|probe| ProbeOption {
                family: probe.family(),
                side_effect: probe.side_effect().code,
                available: false,
                reason: Some(why.code),
            })
            .collect();
    };

    wuim_probe::applicable(device)
        .into_iter()
        .map(|(probe, verdict)| ProbeOption {
            family: probe.family(),
            side_effect: probe.side_effect().code,
            available: verdict.is_supported(),
            reason: verdict.note().map(|n| n.code),
        })
        .collect()
}
