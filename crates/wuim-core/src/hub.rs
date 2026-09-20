//! Asking a USB hub about its own ports.
//!
//! [`crate::windevice`] answers "what is plugged in". It cannot answer "what is
//! port 3 doing", and the two are different questions: a port with nothing in it
//! has no device node, so a tree built from device nodes cannot show it at all.
//! The hub knows, and says so through
//! `IOCTL_USB_GET_NODE_CONNECTION_INFORMATION_EX`.
//!
//! **What the hub does not know is whether a port has power.** Measured on a WCH
//! `1a86:8094` with the VBUS drop confirmed on the bench: with the port dead,
//! every byte of every readable structure was identical to the powered reading,
//! `Present` stayed true, the problem code stayed empty and the COM port stayed
//! assigned. The only call that noticed was `SetCommState` — a write, which
//! configures the line and restarts a board wired for auto-reset, so it is not
//! something to poll with. Per-port power is therefore tracked by remembering
//! what was switched, never by observation.

use anyhow::{Context, Result, anyhow};
use serde::Serialize;
use std::os::windows::io::AsRawHandle;

use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_GET_DEVICE_INTERFACE_LIST_PRESENT, CM_Get_Device_Interface_List_SizeW,
    CM_Get_Device_Interface_ListW, CR_SUCCESS,
};
use windows::Win32::Devices::Usb::{
    GUID_DEVINTERFACE_USB_HUB, IOCTL_USB_GET_NODE_CONNECTION_INFORMATION_EX,
    IOCTL_USB_GET_NODE_INFORMATION, USB_NODE_CONNECTION_INFORMATION_EX, USB_NODE_INFORMATION,
};
use windows::Win32::Foundation::HANDLE;
use windows::Win32::System::IO::DeviceIoControl;
use windows::core::PCWSTR;

/// What a hub says about one of its ports.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubPort {
    /// 1-based, and the same number `vhfilter --switch-port` takes — the port
    /// numbering in `DEVPKEY_Device_LocationPaths` agrees with it, so a device's
    /// `#USB(n)` hop is the port to switch.
    pub port: u32,
    /// The raw `ConnectionStatus`: 0 is nothing connected, 1 is connected, and
    /// the rest are failure states such as over-current.
    pub status: u32,
    pub status_name: &'static str,
    /// True when the hub reports a device on the port.
    pub connected: bool,
    /// `idVendor:idProduct` of whatever is on the port, when there is one.
    ///
    /// Only to line the port up against a row in the device list; the device's
    /// own enumeration is the authority on what it is.
    pub vid_pid: Option<String>,
}

/// Whether a device node is a hub, and what its ports are doing.
///
/// `None` for anything that is not a hub. Publishing the hub interface is the
/// test, rather than a driver name or a missing VID/PID: it is the same thing
/// the IOCTLs below need, so a node that passes it is one they will work on.
pub fn ports(instance_id: &str) -> Option<Vec<HubPort>> {
    let path = interface_path(instance_id).ok()?;
    let handle = open(&path).ok()?;
    let count = port_count(&handle).ok()?;
    (1..=count)
        .map(|port| connection(&handle, port).ok())
        .collect()
}

/// The `\\?\usb#...` path the hub answers IOCTLs on.
///
/// A device instance id is not openable; the interface it publishes is.
fn interface_path(instance_id: &str) -> Result<Vec<u16>> {
    let mut id: Vec<u16> = instance_id.encode_utf16().chain(Some(0)).collect();
    let mut size = 0u32;
    // SAFETY: the id is NUL-terminated and outlives the call.
    let ret = unsafe {
        CM_Get_Device_Interface_List_SizeW(
            &mut size,
            &GUID_DEVINTERFACE_USB_HUB,
            PCWSTR(id.as_mut_ptr()),
            CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
        )
    };
    if ret != CR_SUCCESS || size == 0 {
        return Err(anyhow!("{instance_id} publishes no hub interface"));
    }

    let mut buffer = vec![0u16; size as usize];
    // SAFETY: the buffer is sized by the call above.
    let ret = unsafe {
        CM_Get_Device_Interface_ListW(
            &GUID_DEVINTERFACE_USB_HUB,
            PCWSTR(id.as_mut_ptr()),
            &mut buffer,
            CM_GET_DEVICE_INTERFACE_LIST_PRESENT,
        )
    };
    if ret != CR_SUCCESS {
        return Err(anyhow!("could not read the hub interface: {ret:?}"));
    }

    // A multi-string; the first entry is the one wanted.
    let end = buffer.iter().position(|&c| c == 0).unwrap_or(buffer.len());
    if end == 0 {
        return Err(anyhow!("{instance_id} has an empty hub interface list"));
    }
    buffer.truncate(end + 1);
    Ok(buffer)
}

fn open(path: &[u16]) -> Result<std::fs::File> {
    use std::os::windows::ffi::OsStringExt;
    let text = std::ffi::OsString::from_wide(&path[..path.len() - 1]);
    // Read/write: the connection IOCTLs need write access on the handle even
    // though nothing about the hub is changed.
    std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&text)
        .with_context(|| format!("could not open {}", text.to_string_lossy()))
}

fn port_count(handle: &std::fs::File) -> Result<u32> {
    let mut info = USB_NODE_INFORMATION::default();
    let size = std::mem::size_of::<USB_NODE_INFORMATION>() as u32;
    let mut returned = 0u32;
    // SAFETY: one struct in, the same struct out, both sized by `size`.
    unsafe {
        DeviceIoControl(
            HANDLE(handle.as_raw_handle()),
            IOCTL_USB_GET_NODE_INFORMATION,
            Some(&mut info as *mut _ as *mut _),
            size,
            Some(&mut info as *mut _ as *mut _),
            size,
            Some(&mut returned),
            None,
        )
    }
    .context("the hub did not answer IOCTL_USB_GET_NODE_INFORMATION")?;

    // SAFETY: the union holds hub information whenever the node is a hub, which
    // it is: the handle came from GUID_DEVINTERFACE_USB_HUB.
    let ports = unsafe { info.u.HubInformation.HubDescriptor.bNumberOfPorts };
    Ok(ports as u32)
}

fn connection(handle: &std::fs::File, port: u32) -> Result<HubPort> {
    let mut info = USB_NODE_CONNECTION_INFORMATION_EX {
        ConnectionIndex: port,
        ..Default::default()
    };
    let size = std::mem::size_of::<USB_NODE_CONNECTION_INFORMATION_EX>() as u32;
    let mut returned = 0u32;
    // SAFETY: ConnectionIndex is set on the way in; the rest is filled in.
    unsafe {
        DeviceIoControl(
            HANDLE(handle.as_raw_handle()),
            IOCTL_USB_GET_NODE_CONNECTION_INFORMATION_EX,
            Some(&mut info as *mut _ as *mut _),
            size,
            Some(&mut info as *mut _ as *mut _),
            size,
            Some(&mut returned),
            None,
        )
    }
    .with_context(|| format!("the hub did not answer for port {port}"))?;

    let status = info.ConnectionStatus.0 as u32;
    let connected = status == 1;
    // The descriptor sits in a packed struct, so the fields are copied out
    // rather than referenced.
    let (vid, pid) = {
        let d = info.DeviceDescriptor;
        (d.idVendor, d.idProduct)
    };
    Ok(HubPort {
        port,
        status,
        status_name: status_name(status),
        connected,
        vid_pid: connected.then(|| format!("{vid:04x}:{pid:04x}")),
    })
}

/// `USB_CONNECTION_STATUS`, which the crate exposes as a bare integer.
fn status_name(status: u32) -> &'static str {
    match status {
        0 => "NoDeviceConnected",
        1 => "DeviceConnected",
        2 => "DeviceFailedEnumeration",
        3 => "DeviceGeneralFailure",
        4 => "DeviceCausedOvercurrent",
        5 => "DeviceNotEnoughPower",
        6 => "DeviceNotEnoughBandwidth",
        7 => "DeviceHubNestedTooDeeply",
        8 => "DeviceInLegacyHub",
        9 => "DeviceEnumerating",
        10 => "DeviceReset",
        _ => "unknown",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn connection_statuses_are_named() {
        assert_eq!(status_name(0), "NoDeviceConnected");
        assert_eq!(status_name(1), "DeviceConnected");
        assert_eq!(status_name(4), "DeviceCausedOvercurrent");
        assert_eq!(status_name(99), "unknown");
    }

    /// Anything that is not a hub answers `None` rather than failing, so a
    /// caller can offer it every node and keep what comes back.
    #[test]
    fn a_node_that_is_not_a_hub_is_not_a_hub() {
        assert!(ports("USB\\VID_DEAD&PID_BEEF\\nothing-like-this").is_none());
    }
}
