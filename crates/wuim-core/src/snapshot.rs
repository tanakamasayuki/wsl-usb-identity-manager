//! One moment in time: the Windows enumeration joined with `usbipd state`.
//!
//! The join key is the Device Instance ID and nothing else. Bus IDs are never
//! used for it (requirement R5.1).

use anyhow::Result;
use serde::Serialize;
use std::collections::HashMap;

use crate::hub::{self, HubPort};
use crate::instance_id::InstanceId;
use crate::usbipd::{self, SharingState, UsbipdDevice};
use crate::windevice::{self, WinUsbDevice, strip_com_suffix};

/// One device, with whatever each side knows about it.
#[derive(Debug, Clone, Serialize)]
pub struct DeviceRow {
    pub instance_id: InstanceId,
    pub name: String,
    /// The node Windows currently exposes. `None` while attached or absent.
    pub windows: Option<WinUsbDevice>,
    pub usbipd: Option<UsbipdDevice>,
    /// The VBoxUSB stub node Windows exposes while the device is attached
    /// (finding F4).
    pub stub: Option<WinUsbDevice>,
    pub identity_basis: IdentityBasis,
}

impl DeviceRow {
    pub fn sharing_state(&self) -> SharingState {
        match &self.usbipd {
            Some(d) => d.sharing_state(),
            // Something usbipd does not track, such as a hub or a child node
            // of a composite device.
            None => SharingState::NotShared,
        }
    }

    pub fn bus_id(&self) -> Option<&str> {
        self.usbipd.as_ref()?.bus_id.as_deref()
    }

    pub fn com_port(&self) -> Option<&str> {
        self.windows.as_ref()?.com_port.as_deref()
    }

    /// The stable identifier of the physical port (finding F2). Shown and
    /// matched against, never persisted as a device identity.
    /// The node this device hangs off, and the port number on it.
    ///
    /// The join for the hub tree, and taken from the device tree rather than
    /// from a location path — a path is a formatted string, and one a device
    /// stops publishing in exactly the case this has to survive.
    ///
    /// **The stub stands in while the device is attached.** Handing a device to
    /// WSL leaves it with no node of its own, but the VBoxUSB stub Windows puts
    /// in its place is on the same socket (finding F4), so it answers the same
    /// question. Without this an attached device drops out of the tree at the
    /// moment its port matters most.
    pub fn parent_instance_id(&self) -> Option<&str> {
        self.port_node()?.parent_instance_id.as_deref()
    }

    pub fn port_address(&self) -> Option<u32> {
        self.port_node()?.address
    }

    /// Whichever node currently occupies the device's port.
    fn port_node(&self) -> Option<&WinUsbDevice> {
        self.windows.as_ref().or(self.stub.as_ref())
    }

    pub fn location_path(&self) -> Option<&str> {
        self.windows
            .as_ref()?
            .location_paths
            .first()
            .map(String::as_str)
    }
}

/// What the identity currently rests on, matching the routes in requirements §4.1.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "basis", content = "value", rename_all = "snake_case")]
pub enum IdentityBasis {
    /// Route 1: a USB serial pins down the transport. No probe needed.
    UsbSerial(String),
    /// Route 2 or 3: no serial, so the port is the only handle there is.
    /// Identifying the unit itself requires a probe (finding F3).
    PortOnly,
}

impl IdentityBasis {
    fn from_instance_id(id: &InstanceId) -> Self {
        match id.unit.serial() {
            Some(serial) => Self::UsbSerial(serial.to_owned()),
            None => Self::PortOnly,
        }
    }

    pub fn label(&self) -> &'static str {
        match self {
            Self::UsbSerial(_) => "serial",
            Self::PortOnly => "port only",
        }
    }

    pub fn needs_probe(&self) -> bool {
        matches!(self, Self::PortOnly)
    }
}

/// One hub, with what it says about its own ports.
///
/// Separate from the device rows because a hub is where devices hang rather than
/// something to bind or attach, and because its **ports** are the point: a port
/// with nothing in it has no device node, so only the hub can report it.
#[derive(Debug, Clone, Serialize)]
pub struct HubNode {
    pub instance_id: String,
    pub name: String,
    /// The hub's own position, for placing it in the tree.
    pub location_path: Option<String>,
    /// The node this hub hangs off, and the port number on it.
    pub parent_instance_id: Option<String>,
    pub address: Option<u32>,
    /// `1a86:8094`. Absent on a root hub, which is part of the controller
    /// rather than a device with a vendor.
    pub vid_pid: Option<String>,
    /// `DEVPKEY_Device_Manufacturer`, which for a hub is often the only thing
    /// naming who made it.
    pub manufacturer: Option<String>,
    pub driver_version: Option<String>,
    pub ports: Vec<HubPort>,
}

/// The device list at one point in time.
#[derive(Debug, Clone, Serialize)]
pub struct Snapshot {
    /// Devices usbipd deals with.
    pub devices: Vec<DeviceRow>,
    /// Nodes Windows exposes but usbipd does not list: hubs, interface nodes of
    /// composite devices, and stubs of attached devices.
    pub other_nodes: Vec<WinUsbDevice>,
    /// Every hub, root or otherwise. Empty unless the snapshot was captured
    /// from a real machine: reading it is I/O, so [`Snapshot::join`] leaves it
    /// alone and stays a pure function.
    pub hubs: Vec<HubNode>,
}

impl Snapshot {
    /// Reads the current state from the machine. Probes nothing (R4.6).
    pub fn capture() -> Result<Self> {
        let windows = windevice::enumerate_present_usb_devices()?;
        let usbipd = usbipd::query()?;
        let mut snapshot = Self::join(windows, usbipd);
        snapshot.hubs = snapshot.read_hubs();
        Ok(snapshot)
    }

    /// Asks every enumerated node whether it is a hub, and what its ports say.
    ///
    /// Offering the question to everything rather than guessing from a driver
    /// name or a missing VID/PID: the test is whether the node answers the hub
    /// IOCTLs, which is exactly what the caller needs it to do.
    fn read_hubs(&self) -> Vec<HubNode> {
        let candidates = self
            .devices
            .iter()
            .filter_map(|row| row.windows.as_ref())
            .chain(self.other_nodes.iter());

        let mut hubs: Vec<HubNode> = candidates
            .filter_map(|node| {
                let ports = hub::ports(&node.instance_id.raw)?;
                Some(HubNode {
                    instance_id: node.instance_id.raw.clone(),
                    name: node.display_name().to_owned(),
                    location_path: node.location_paths.first().cloned(),
                    parent_instance_id: node.parent_instance_id.clone(),
                    address: node.address,
                    vid_pid: node.instance_id.vid_pid_string(),
                    manufacturer: node.manufacturer.clone(),
                    driver_version: node.driver_version.clone(),
                    ports,
                })
            })
            .collect();
        hubs.sort_by(|a, b| a.location_path.cmp(&b.location_path));
        hubs
    }

    /// Joins the two enumerations. Kept free of I/O so it can be tested.
    pub fn join(windows: Vec<WinUsbDevice>, usbipd: Vec<UsbipdDevice>) -> Self {
        let mut by_instance: HashMap<String, WinUsbDevice> = windows
            .into_iter()
            .map(|d| (d.instance_id.raw.to_ascii_lowercase(), d))
            .collect();

        let mut devices = Vec::with_capacity(usbipd.len());
        for entry in usbipd {
            let win = by_instance.remove(&entry.instance_id.to_ascii_lowercase());
            let stub = entry
                .stub_instance_id
                .as_ref()
                .and_then(|s| by_instance.remove(&s.to_ascii_lowercase()));

            let instance_id = InstanceId::parse(&entry.instance_id);
            // While attached there is no Windows node to read a name from, so
            // usbipd's cached description stands in. Stripping its port suffix
            // keeps the name identical to the one shown the rest of the time,
            // instead of the row appearing to rename itself on attach.
            let name = win
                .as_ref()
                .map(|d| d.display_name().to_owned())
                .unwrap_or_else(|| strip_com_suffix(&entry.description).to_owned());

            devices.push(DeviceRow {
                identity_basis: IdentityBasis::from_instance_id(&instance_id),
                instance_id,
                name,
                windows: win,
                usbipd: Some(entry),
                stub,
            });
        }

        let mut other_nodes: Vec<WinUsbDevice> = by_instance.into_values().collect();
        other_nodes.sort_by(|a, b| a.instance_id.raw.cmp(&b.instance_id.raw));

        // Connected devices first, then by bus id, then by name.
        //
        // Bus id order puts devices in the order they hang off the hubs, which
        // is how the user sees them on the desk: everything on one hub lands
        // together. Sorting by name instead scatters identical adapters, which
        // are exactly the ones that need telling apart.
        devices.sort_by(|a, b| {
            let key = |r: &DeviceRow| {
                (
                    !r.usbipd.as_ref().is_some_and(UsbipdDevice::is_connected),
                    bus_id_order(r.bus_id()),
                    r.name.to_lowercase(),
                    r.instance_id.raw.clone(),
                )
            };
            key(a).cmp(&key(b))
        });

        Self {
            devices,
            other_nodes,
            hubs: Vec::new(),
        }
    }

    pub fn connected(&self) -> impl Iterator<Item = &DeviceRow> {
        self.devices
            .iter()
            .filter(|r| r.usbipd.as_ref().is_some_and(UsbipdDevice::is_connected))
    }

    /// Groups of connected devices that share a VID/PID and have no serial, so
    /// nothing in their descriptors tells them apart.
    ///
    /// This is the problem the whole tool exists to solve, so it is surfaced
    /// rather than left for the user to notice.
    pub fn ambiguous_groups(&self) -> Vec<(String, Vec<&DeviceRow>)> {
        let mut groups: HashMap<String, Vec<&DeviceRow>> = HashMap::new();
        for row in self.connected() {
            if !row.identity_basis.needs_probe() {
                continue;
            }
            let Some(key) = row.instance_id.vid_pid_string() else {
                continue;
            };
            groups.entry(key).or_default().push(row);
        }
        let mut out: Vec<(String, Vec<&DeviceRow>)> =
            groups.into_iter().filter(|(_, v)| v.len() > 1).collect();
        out.sort_by(|a, b| a.0.cmp(&b.0));
        out
    }
}

/// Sort key for a bus id such as `8-1`.
///
/// The parts are compared as numbers, so `8-10` follows `8-9` instead of
/// sorting between `8-1` and `8-2`. An unparseable or absent bus id sorts last.
fn bus_id_order(bus_id: Option<&str>) -> (u32, u32) {
    let Some(bus_id) = bus_id else {
        return (u32::MAX, u32::MAX);
    };
    let (hub, port) = bus_id.split_once('-').unwrap_or((bus_id, ""));
    (
        hub.parse().unwrap_or(u32::MAX),
        port.parse().unwrap_or(u32::MAX),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::usbipd;

    fn usbipd_entry(instance_id: &str, bus_id: Option<&str>, stub: Option<&str>) -> UsbipdDevice {
        let json = format!(
            r#"{{"Devices":[{{"BusId":{},"ClientIPAddress":null,"Description":"test",
            "InstanceId":{},"IsForced":false,"PersistedGuid":null,"StubInstanceId":{}}}]}}"#,
            bus_id.map_or("null".into(), |b| format!("{b:?}")),
            serde_json::to_string(instance_id).unwrap(),
            stub.map_or("null".into(), |s| serde_json::to_string(s).unwrap()),
        );
        usbipd::parse(&json).unwrap().pop().unwrap()
    }

    #[test]
    fn serial_devices_do_not_need_a_probe() {
        let id = InstanceId::parse(r"USB\VID_1A86&PID_55D3\5B5F090816");
        let basis = IdentityBasis::from_instance_id(&id);
        assert_eq!(basis, IdentityBasis::UsbSerial("5B5F090816".into()));
        assert!(!basis.needs_probe());
    }

    #[test]
    fn serialless_devices_need_a_probe() {
        let id = InstanceId::parse(r"USB\VID_1A86&PID_7523\8&7CC2A31&0&1");
        assert_eq!(
            IdentityBasis::from_instance_id(&id),
            IdentityBasis::PortOnly
        );
        assert!(IdentityBasis::from_instance_id(&id).needs_probe());
    }

    #[test]
    fn join_matches_case_insensitively() {
        let entry = usbipd_entry(r"USB\VID_1A86&PID_7523\8&7CC2A31&0&1", Some("16-1"), None);
        let snapshot = Snapshot::join(Vec::new(), vec![entry]);
        assert_eq!(snapshot.devices.len(), 1);
        assert_eq!(snapshot.devices[0].bus_id(), Some("16-1"));
        // Nothing to join against, since the Windows enumeration was empty.
        assert!(snapshot.devices[0].windows.is_none());
    }

    #[test]
    fn ambiguous_groups_need_two_or_more_serialless_devices() {
        let one = usbipd_entry(r"USB\VID_1A86&PID_7523\8&7CC2A31&0&1", Some("16-1"), None);
        let two = usbipd_entry(r"USB\VID_1A86&PID_7523\8&7CC2A31&0&2", Some("16-2"), None);
        let lone = usbipd_entry(r"USB\VID_1A86&PID_55D3\5B5F090816", Some("16-3"), None);

        let single = Snapshot::join(Vec::new(), vec![one.clone(), lone.clone()]);
        assert!(single.ambiguous_groups().is_empty());

        let pair = Snapshot::join(Vec::new(), vec![one, two, lone]);
        let groups = pair.ambiguous_groups();
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].0, "1a86:7523");
        assert_eq!(groups[0].1.len(), 2);
    }

    #[test]
    fn bus_ids_sort_numerically() {
        assert!(bus_id_order(Some("8-2")) < bus_id_order(Some("8-10")));
        assert!(bus_id_order(Some("2-7")) < bus_id_order(Some("8-1")));
        assert!(bus_id_order(Some("9-4")) < bus_id_order(None));
        // An unparseable bus id ties with an absent one, and both sort last.
        // The tie is harmless: a device with a bus id is connected, so the
        // connected/absent key has already separated it from the absent ones.
        assert_eq!(bus_id_order(Some("odd")), bus_id_order(None));
    }

    #[test]
    fn rows_sort_by_bus_id_not_by_name() {
        // Same model, so the name cannot decide the order.
        let later = usbipd_entry(r"USB\VID_1A86&PID_7523\8&A&0&1", Some("8-3"), None);
        let earlier = usbipd_entry(r"USB\VID_1A86&PID_7523\8&A&0&2", Some("8-1"), None);
        let snapshot = Snapshot::join(Vec::new(), vec![later, earlier]);
        assert_eq!(snapshot.devices[0].bus_id(), Some("8-1"));
        assert_eq!(snapshot.devices[1].bus_id(), Some("8-3"));
    }

    #[test]
    fn disconnected_devices_sort_last() {
        let absent = usbipd_entry(r"USB\VID_1A86&PID_7523\8&7CC2A31&0&1", None, None);
        let present = usbipd_entry(r"USB\VID_1A86&PID_55D3\5B5F090816", Some("16-3"), None);
        let snapshot = Snapshot::join(Vec::new(), vec![absent, present]);
        assert!(snapshot.devices[0].bus_id().is_some());
        assert!(snapshot.devices[1].bus_id().is_none());
    }
}
