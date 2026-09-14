//! USB device enumeration and property reads through CfgMgr32.
//!
//! CfgMgr32 owns enumeration and identity information. nusb is not used here:
//! on Windows it only reports serial numbers for composite devices bound to
//! usbccgp, which excludes the CH340 (docs/platform-evaluation.ja.md §3).

use anyhow::{Result, anyhow};
use serde::Serialize;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_GETIDLIST_FILTER_ENUMERATOR, CM_GETIDLIST_FILTER_PRESENT, CM_Get_Child,
    CM_Get_DevNode_PropertyW, CM_Get_Device_ID_List_SizeW, CM_Get_Device_ID_ListW, CM_Get_Sibling,
    CM_LOCATE_DEVNODE_NORMAL, CM_Locate_DevNodeW, CM_Open_DevNode_Key, CM_REGISTRY_HARDWARE,
    CONFIGRET, CR_BUFFER_SMALL, CR_SUCCESS,
};
use windows::Win32::Devices::Properties::{
    DEVPKEY_Device_BusReportedDeviceDesc, DEVPKEY_Device_ContainerId, DEVPKEY_Device_DeviceDesc,
    DEVPKEY_Device_FriendlyName, DEVPKEY_Device_LocationPaths, DEVPKEY_Device_Manufacturer,
    DEVPKEY_Device_Service, DEVPROP_TYPE_GUID, DEVPROP_TYPE_STRING, DEVPROP_TYPE_STRING_LIST,
    DEVPROPTYPE,
};
use windows::Win32::Foundation::DEVPROPKEY;
use windows::Win32::System::Registry::{HKEY, KEY_READ, REG_SZ, RegCloseKey, RegQueryValueExW};
use windows::core::PCWSTR;

use crate::instance_id::InstanceId;

/// One USB device as Windows sees it.
///
/// This carries the Windows-sourced half of requirements §3.2 (USB Identity)
/// and §3.3 (Runtime Connection). usbipd state lives in [`crate::usbipd`].
#[derive(Debug, Clone, Serialize)]
pub struct WinUsbDevice {
    pub instance_id: InstanceId,
    /// `DEVPKEY_Device_DeviceDesc`: the generic name the driver supplies.
    pub device_desc: Option<String>,
    /// `DEVPKEY_Device_FriendlyName`: usually includes the COM number.
    pub friendly_name: Option<String>,
    /// `DEVPKEY_Device_BusReportedDeviceDesc`: the USB iProduct string.
    pub bus_reported_device_desc: Option<String>,
    pub manufacturer: Option<String>,
    /// The bound driver, used to decide whether a WinUSB probe is possible (R4.12).
    pub service: Option<String>,
    pub container_id: Option<ContainerId>,
    /// `DEVPKEY_Device_LocationPaths`: the stable identifier of the physical port
    /// (finding F2). Must never be persisted as a device identifier (R7.1).
    pub location_paths: Vec<String>,
    /// COM number, for display only (R7.1).
    pub com_port: Option<String>,
}

impl WinUsbDevice {
    /// Whether this is a hub or root hub, i.e. a node with no VID/PID.
    pub fn is_hub_like(&self) -> bool {
        self.instance_id.vid_pid().is_none()
    }

    /// The best name available for display.
    ///
    /// `FriendlyName` first: it is what Device Manager shows, so it is the name
    /// the user already knows the device by. The USB `iProduct` string is the
    /// fallback because it is often generic — a CH340 calls itself "USB Serial"
    /// while Windows calls it "USB-SERIAL CH340".
    pub fn display_name(&self) -> &str {
        self.friendly_name
            .as_deref()
            .map(strip_com_suffix)
            .or(self.bus_reported_device_desc.as_deref())
            .or(self.device_desc.as_deref())
            .unwrap_or(&self.instance_id.raw)
    }

    /// Whether interface 0 is bound to WinUSB, a precondition for the WCH-Link probe.
    pub fn is_winusb(&self) -> bool {
        self.service
            .as_deref()
            .is_some_and(|s| s.eq_ignore_ascii_case("WinUSB"))
    }
}

/// A ContainerId together with its UUID version.
///
/// Devices with a serial get a v5 UUID (a deterministic hash of VID/PID/serial);
/// devices without one get a v1 UUID minted on first connection, which makes it
/// a port identifier in disguise (finding F3).
#[derive(Debug, Clone, Serialize)]
pub struct ContainerId {
    pub uuid: String,
    pub version: u8,
}

impl ContainerId {
    /// v5 means the value derives from the device and survives a port change.
    /// Any other version has degraded to identifying the port.
    pub fn is_stable(&self) -> bool {
        self.version == 5
    }
}

/// Strips a trailing ` (COM12)` from a device name.
///
/// Windows puts the COM number inside the friendly name, and usbipd caches that
/// whole string when the device is bound — so the number in it can name a port
/// the device no longer has. The live number is read from the registry and
/// shown on its own, so the copy baked into the name is dropped rather than
/// contradicting it.
pub fn strip_com_suffix(name: &str) -> &str {
    let Some(open) = name.rfind(" (COM") else {
        return name;
    };
    let Some(digits) = name[open + " (COM".len()..].strip_suffix(')') else {
        return name;
    };
    if digits.is_empty() || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return name;
    }
    name[..open].trim_end()
}

/// The well-known "this device belongs to no container" sentinel.
const NULL_CONTAINER_ID: &str = "00000000-0000-0000-ffff-ffffffffffff";

/// Enumerates the USB devices currently connected.
///
/// `CM_GETIDLIST_FILTER_PRESENT` keeps devices that were merely connected in the
/// past out of the result.
pub fn enumerate_present_usb_devices() -> Result<Vec<WinUsbDevice>> {
    let ids = device_id_list("USB")?;
    Ok(ids.iter().filter_map(|id| load_device(id)).collect())
}

/// Lists the Device Instance IDs under an enumerator such as `USB`.
fn device_id_list(enumerator: &str) -> Result<Vec<String>> {
    let filter: Vec<u16> = enumerator
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let flags = CM_GETIDLIST_FILTER_ENUMERATOR | CM_GETIDLIST_FILTER_PRESENT;

    // A device arriving between the size query and the read returns CR_BUFFER_SMALL,
    // so retry a few times before giving up.
    for _ in 0..4 {
        let mut len = 0u32;
        let cr = unsafe { CM_Get_Device_ID_List_SizeW(&mut len, PCWSTR(filter.as_ptr()), flags) };
        if cr != CR_SUCCESS {
            return Err(configret_error("CM_Get_Device_ID_List_SizeW", cr));
        }
        let mut buf = vec![0u16; len as usize];
        let cr = unsafe { CM_Get_Device_ID_ListW(PCWSTR(filter.as_ptr()), &mut buf, flags) };
        if cr == CR_BUFFER_SMALL {
            continue;
        }
        if cr != CR_SUCCESS {
            return Err(configret_error("CM_Get_Device_ID_ListW", cr));
        }
        return Ok(split_multi_sz(&buf));
    }
    Err(anyhow!(
        "CM_Get_Device_ID_ListW kept failing: devices are arriving faster than they can be listed"
    ))
}

fn load_device(instance_id: &str) -> Option<WinUsbDevice> {
    let devinst = locate_devnode(instance_id)?;
    Some(WinUsbDevice {
        instance_id: InstanceId::parse(instance_id),
        device_desc: prop_string(devinst, &DEVPKEY_Device_DeviceDesc),
        friendly_name: prop_string(devinst, &DEVPKEY_Device_FriendlyName),
        bus_reported_device_desc: prop_string(devinst, &DEVPKEY_Device_BusReportedDeviceDesc),
        manufacturer: prop_string(devinst, &DEVPKEY_Device_Manufacturer),
        service: prop_string(devinst, &DEVPKEY_Device_Service),
        container_id: prop_container_id(devinst),
        location_paths: prop_string_list(devinst, &DEVPKEY_Device_LocationPaths),
        com_port: find_com_port(devinst),
    })
}

fn locate_devnode(instance_id: &str) -> Option<u32> {
    let wide: Vec<u16> = instance_id
        .encode_utf16()
        .chain(std::iter::once(0))
        .collect();
    let mut devinst = 0u32;
    let cr = unsafe {
        CM_Locate_DevNodeW(
            &mut devinst,
            PCWSTR(wide.as_ptr()),
            CM_LOCATE_DEVNODE_NORMAL,
        )
    };
    (cr == CR_SUCCESS).then_some(devinst)
}

/// Reads a device property as typed raw bytes.
fn prop_raw(devinst: u32, key: &DEVPROPKEY) -> Option<(DEVPROPTYPE, Vec<u8>)> {
    let mut ty = DEVPROPTYPE::default();
    let mut size = 0u32;
    let cr = unsafe { CM_Get_DevNode_PropertyW(devinst, key, &mut ty, None, &mut size, 0) };
    if cr != CR_BUFFER_SMALL || size == 0 {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    let cr = unsafe {
        CM_Get_DevNode_PropertyW(devinst, key, &mut ty, Some(buf.as_mut_ptr()), &mut size, 0)
    };
    if cr != CR_SUCCESS {
        return None;
    }
    buf.truncate(size as usize);
    Some((ty, buf))
}

fn prop_string(devinst: u32, key: &DEVPROPKEY) -> Option<String> {
    let (ty, buf) = prop_raw(devinst, key)?;
    if ty != DEVPROP_TYPE_STRING {
        return None;
    }
    let decoded = decode_utf16(&buf);
    let trimmed = decoded.trim_end_matches('\0').trim();
    (!trimmed.is_empty()).then(|| trimmed.to_owned())
}

fn prop_string_list(devinst: u32, key: &DEVPROPKEY) -> Vec<String> {
    let Some((ty, buf)) = prop_raw(devinst, key) else {
        return Vec::new();
    };
    if ty != DEVPROP_TYPE_STRING_LIST {
        return Vec::new();
    }
    let units: Vec<u16> = buf
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    split_multi_sz(&units)
}

fn prop_container_id(devinst: u32) -> Option<ContainerId> {
    let (ty, buf) = prop_raw(devinst, &DEVPKEY_Device_ContainerId)?;
    if ty != DEVPROP_TYPE_GUID || buf.len() < 16 {
        return None;
    }
    // A GUID is laid out with Data1/Data2/Data3 little-endian and Data4 as bytes.
    let d1 = u32::from_le_bytes([buf[0], buf[1], buf[2], buf[3]]);
    let d2 = u16::from_le_bytes([buf[4], buf[5]]);
    let d3 = u16::from_le_bytes([buf[6], buf[7]]);
    let uuid = format!(
        "{d1:08x}-{d2:04x}-{d3:04x}-{:02x}{:02x}-{:02x}{:02x}{:02x}{:02x}{:02x}{:02x}",
        buf[8], buf[9], buf[10], buf[11], buf[12], buf[13], buf[14], buf[15]
    );
    if uuid == NULL_CONTAINER_ID {
        // The device declares no container at all; reporting a version here
        // would suggest an identity the value does not carry.
        return None;
    }
    Some(ContainerId {
        uuid,
        // The version lives in the top 4 bits of time_hi_and_version.
        version: (d3 >> 12) as u8,
    })
}

/// Finds the COM number for a device.
///
/// For a CH340 the USB device node is itself the COM port; for a CDC device the
/// port lives on a child node (`USB\...&MI_00\...`). Check the node first, then
/// walk its children.
fn find_com_port(devinst: u32) -> Option<String> {
    if let Some(port) = port_name(devinst) {
        return Some(port);
    }
    let mut child = 0u32;
    if unsafe { CM_Get_Child(&mut child, devinst, 0) } != CR_SUCCESS {
        return None;
    }
    loop {
        if let Some(port) = port_name(child) {
            return Some(port);
        }
        let mut sibling = 0u32;
        if unsafe { CM_Get_Sibling(&mut sibling, child, 0) } != CR_SUCCESS {
            return None;
        }
        child = sibling;
    }
}

/// Reads `Device Parameters\PortName` for one device node.
fn port_name(devinst: u32) -> Option<String> {
    /// `CM_Open_DevNode_Key` disposition: open an existing key, never create one.
    const REG_DISPOSITION_OPEN_EXISTING: u32 = 1;

    let mut hkey = HKEY::default();
    let cr = unsafe {
        CM_Open_DevNode_Key(
            devinst,
            KEY_READ.0,
            0,
            REG_DISPOSITION_OPEN_EXISTING,
            &mut hkey,
            CM_REGISTRY_HARDWARE,
        )
    };
    if cr != CR_SUCCESS {
        return None;
    }
    let value = read_sz(hkey, "PortName");
    let _ = unsafe { RegCloseKey(hkey) };
    value.filter(|v| !v.is_empty())
}

fn read_sz(hkey: HKEY, name: &str) -> Option<String> {
    let name: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
    let mut ty = REG_SZ;
    let mut size = 0u32;
    unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut ty),
            None,
            Some(&mut size),
        )
        .ok()
        .ok()?;
    }
    if ty != REG_SZ || size == 0 {
        return None;
    }
    let mut buf = vec![0u8; size as usize];
    unsafe {
        RegQueryValueExW(
            hkey,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut ty),
            Some(buf.as_mut_ptr()),
            Some(&mut size),
        )
        .ok()
        .ok()?;
    }
    buf.truncate(size as usize);
    Some(decode_utf16(&buf).trim_end_matches('\0').to_owned())
}

fn decode_utf16(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|c| u16::from_le_bytes([c[0], c[1]]))
        .collect();
    String::from_utf16_lossy(&units)
}

/// Splits a NUL-separated, double-NUL-terminated string list.
fn split_multi_sz(units: &[u16]) -> Vec<String> {
    units
        .split(|&c| c == 0)
        .filter(|s| !s.is_empty())
        .map(String::from_utf16_lossy)
        .collect()
}

fn configret_error(what: &str, cr: CONFIGRET) -> anyhow::Error {
    anyhow!("{what} failed (CONFIGRET = {})", cr.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn multi_sz_splits_on_nul() {
        let units: Vec<u16> = "a\0bb\0\0".encode_utf16().collect();
        assert_eq!(split_multi_sz(&units), vec!["a", "bb"]);
    }

    #[test]
    fn strips_a_trailing_com_number() {
        // usbipd hands back this string with a stale number in it.
        assert_eq!(
            strip_com_suffix("USB-SERIAL CH340 (COM7)"),
            "USB-SERIAL CH340"
        );
        assert_eq!(
            strip_com_suffix("USB-SERIAL CH340 (COM32)"),
            "USB-SERIAL CH340"
        );
    }

    #[test]
    fn leaves_names_without_a_com_number_alone() {
        assert_eq!(strip_com_suffix("USB Serial"), "USB Serial");
        assert_eq!(strip_com_suffix("Arduino Uno"), "Arduino Uno");
        // Not a port number, so not a port suffix.
        assert_eq!(strip_com_suffix("Widget (COMPACT)"), "Widget (COMPACT)");
        assert_eq!(strip_com_suffix("Widget (COM)"), "Widget (COM)");
        // A mid-string port number is left in place; only a suffix is dropped.
        assert_eq!(
            strip_com_suffix("Hub (COM3) downstream"),
            "Hub (COM3) downstream"
        );
    }

    #[test]
    fn container_id_version_5_is_stable() {
        let v5 = ContainerId {
            uuid: "24309343-175d-5c84-a14b-bb9cc186c126".into(),
            version: 5,
        };
        let v1 = ContainerId {
            uuid: "6aa62d84-4e6a-11f1-b04a-c0a5e85bd89d".into(),
            version: 1,
        };
        assert!(v5.is_stable());
        assert!(!v1.is_stable());
    }
}
