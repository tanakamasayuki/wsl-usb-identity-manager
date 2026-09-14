//! The shape the frontend receives.
//!
//! Deliberately a separate type from [`wuim_core::DeviceRow`]: the wire format
//! is something the UI depends on, and pinning it here means the core model can
//! change without silently reshaping what Svelte is reading.

use serde::Serialize;
use wuim_core::UsbIds;
use wuim_core::recall::{Confidence, Recalled};
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
    /// Vendor name from the USB ID Repository. Absent when the vendor never
    /// submitted an entry, which is common enough to be unremarkable.
    pub vendor: Option<String>,
    /// Product name from the same source. Rarer than the vendor name.
    pub usb_product: Option<String>,

    // Runtime connection. None of this is an identity (requirement R7.1); it is
    // here to be displayed and nothing else.
    pub bus_id: Option<String>,
    pub com_port: Option<String>,
    /// `DEVPKEY_Device_LocationPaths`, the stable identifier of the physical
    /// port (finding F2). Displayed, never persisted as an identity (R7.1).
    pub location_path: Option<String>,
    /// The same path as a short hub-port chain, `1-3-3`.
    pub port_chain: Option<String>,

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
    pub driver_version: Option<String>,
    /// Device revision from the descriptor, `2.64`.
    pub revision: Option<String>,
    /// Windows' problem code, present only when there is one.
    pub problem_code: Option<u32>,
    /// Probes that could be run, with the side effects to show first (R4.7).
    pub probes: Vec<ProbeOption>,
    /// Which usbipd operations make sense for the device as it stands.
    pub actions: Actions,
    /// What the stored file says this device is, when it recognises it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
}

/// A recalled or freshly probed identity, with how much it is worth (R4.3).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub identity_key: String,
    pub device_type: String,
    pub device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_revision: Option<String>,
    pub confidence: Confidence,
}

impl Identity {
    pub fn from_recalled(recalled: &Recalled<'_>) -> Self {
        Self {
            identity_key: recalled.device.identity_key.clone(),
            device_type: recalled.device.device_type.clone(),
            device_id: recalled.device.device_id.clone(),
            hardware_revision: recalled.device.hardware_revision.clone(),
            confidence: recalled.confidence,
        }
    }
}

/// The usbipd operations offered for a device.
///
/// Decided here rather than in the UI so the rule lives next to the state it
/// reads. The backend re-checks anyway — this only decides what to show.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actions {
    /// Share it with usbipd. Needs administrator rights.
    pub bind: bool,
    /// Stop sharing it. Needs administrator rights.
    pub unbind: bool,
    /// Hand it to WSL. Only possible once bound.
    pub attach: bool,
    /// Take it back from WSL.
    pub detach: bool,
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
    pub fn from_row(row: &DeviceRow, ids: &UsbIds, identity: Option<Identity>) -> Self {
        let usbipd = row.usbipd.as_ref();
        let (vendor, usb_product) = match row.instance_id.vid_pid() {
            Some((vid, pid)) => (
                ids.vendor(vid).map(str::to_owned),
                ids.product(vid, pid).map(str::to_owned),
            ),
            None => (None, None),
        };
        Self {
            instance_id: row.instance_id.raw.clone(),
            name: row.name.clone(),
            vid_pid: row.instance_id.vid_pid_string(),
            vendor,
            usb_product,

            bus_id: row.bus_id().map(str::to_owned),
            com_port: row.com_port().map(str::to_owned),
            location_path: row.location_path().map(str::to_owned),
            port_chain: row.windows.as_ref().and_then(|w| w.port_chain()),

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
            driver_version: row.windows.as_ref().and_then(|w| w.driver_version.clone()),
            revision: row.windows.as_ref().and_then(|w| w.revision.clone()),
            problem_code: row.windows.as_ref().and_then(|w| w.problem_code),
            probes: probe_options(row),
            actions: actions(row),
            identity,
        }
    }
}

fn actions(row: &DeviceRow) -> Actions {
    let usbipd = row.usbipd.as_ref();
    let connected = usbipd.is_some_and(|d| d.is_connected());
    let shared = usbipd.is_some_and(|d| d.is_shared());
    let attached = usbipd.is_some_and(|d| d.is_attached());

    Actions {
        // Binding an absent device is not something usbipd can do: the bus id
        // it would need does not exist.
        bind: connected && !shared,
        unbind: shared && !attached,
        attach: connected && shared && !attached,
        detach: attached,
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
