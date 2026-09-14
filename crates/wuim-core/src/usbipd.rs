//! Reading `usbipd` state.
//!
//! State comes from `usbipd state` (JSON) only. The text output of `usbipd list`
//! is never parsed (requirement R5.2).

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};

/// Default install location, used when usbipd is not on PATH.
const DEFAULT_INSTALL_PATH: &str = r"C:\Program Files\usbipd-win\usbipd.exe";

/// One entry from `usbipd state`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct UsbipdDevice {
    /// Only meaningful while connected. Never persist it as an identifier
    /// (finding F1, requirement R7.1): the same bus ID comes to mean a
    /// different device after hubs are re-enumerated.
    pub bus_id: Option<String>,
    #[serde(rename = "ClientIPAddress")]
    pub client_ip_address: Option<String>,
    pub description: String,
    /// Matches the Windows PnP Device Instance ID exactly, and is the only
    /// correct key for joining the two (requirement R5.1).
    pub instance_id: String,
    pub is_forced: bool,
    /// Issued by usbipd on bind and kept while the device is away. It is an
    /// internal usbipd identifier and must not be persisted (R7.1).
    pub persisted_guid: Option<String>,
    /// While attached: `Vid_80EE&Pid_CAFE\<third element>` (requirement R5.3).
    pub stub_instance_id: Option<String>,
}

impl UsbipdDevice {
    /// Whether the device is physically connected to Windows right now.
    pub fn is_connected(&self) -> bool {
        self.bus_id.is_some()
    }

    /// Whether the device is bound, i.e. shareable with WSL.
    pub fn is_shared(&self) -> bool {
        self.persisted_guid.is_some()
    }

    /// Whether some client currently holds the device.
    pub fn is_attached(&self) -> bool {
        self.stub_instance_id.is_some() || self.client_ip_address.is_some()
    }

    pub fn sharing_state(&self) -> SharingState {
        if self.is_attached() {
            SharingState::Attached
        } else if !self.is_connected() {
            SharingState::Absent
        } else if self.is_shared() {
            SharingState::Shared
        } else {
            SharingState::NotShared
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SharingState {
    /// Connected but not bound. Attaching requires a bind first, which needs
    /// administrator rights.
    NotShared,
    /// Bound and connected, so ready to attach.
    Shared,
    /// Attached to a client. Windows cannot reach the device itself, which
    /// also means it cannot be probed.
    Attached,
    /// Only a bind record remains; the device is not connected.
    Absent,
}

impl SharingState {
    pub fn label(self) -> &'static str {
        match self {
            Self::NotShared => "not shared",
            Self::Shared => "shared",
            Self::Attached => "attached",
            Self::Absent => "absent",
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StateDocument {
    devices: Vec<UsbipdDevice>,
}

/// Runs `usbipd state` and returns what it reports.
pub fn query() -> Result<Vec<UsbipdDevice>> {
    let exe = locate()?;
    let output = command(&exe)
        .arg("state")
        .output()
        .with_context(|| format!("failed to run {}", exe.display()))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!(
            "usbipd state exited with {}: {}",
            output.status.code().unwrap_or(-1),
            stderr.trim()
        );
    }

    parse(&String::from_utf8_lossy(&output.stdout))
}

/// Parses the JSON produced by `usbipd state`.
pub fn parse(json: &str) -> Result<Vec<UsbipdDevice>> {
    let doc: StateDocument =
        serde_json::from_str(json).context("could not parse the JSON from usbipd state")?;
    Ok(doc.devices)
}

/// Decides which usbipd.exe to run: the default install location if it exists,
/// otherwise whatever PATH resolves.
pub fn locate() -> Result<PathBuf> {
    let default = PathBuf::from(DEFAULT_INSTALL_PATH);
    if default.is_file() {
        return Ok(default);
    }
    Ok(PathBuf::from("usbipd.exe"))
}

/// Builds a command that starts without flashing a console window, so the GUI
/// can call it without a black box appearing on screen.
fn command(exe: &PathBuf) -> Command {
    let mut cmd = Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Excerpted from the `usbipd state` output of a real machine.
    const SAMPLE: &str = r#"{
      "Devices": [
        {
          "BusId": "8-4",
          "ClientIPAddress": null,
          "Description": "Microsoft USB IntelliMouse Optical",
          "InstanceId": "USB\\VID_045E&PID_0039\\7&19033BE6&0&4",
          "IsForced": false,
          "PersistedGuid": null,
          "StubInstanceId": null
        },
        {
          "BusId": null,
          "ClientIPAddress": null,
          "Description": "USB-SERIAL CH340 (COM8)",
          "InstanceId": "USB\\VID_1A86&PID_7523\\7&19033BE6&0&2",
          "IsForced": false,
          "PersistedGuid": "05d75b37-ae96-4b2a-9d0d-5cded25f52eb",
          "StubInstanceId": null
        },
        {
          "BusId": "16-1",
          "ClientIPAddress": "172.22.176.1",
          "Description": "USB-Enhanced-SERIAL CH343 (COM19)",
          "InstanceId": "USB\\VID_1A86&PID_55D3\\58FA041019",
          "IsForced": false,
          "PersistedGuid": "a4cc7bfb-f9c5-4211-b4c0-f84aaa11109a",
          "StubInstanceId": "USB\\Vid_80EE&Pid_CAFE\\58FA041019"
        }
      ]
    }"#;

    #[test]
    fn parses_real_state_output() {
        let devices = parse(SAMPLE).unwrap();
        assert_eq!(devices.len(), 3);
        assert_eq!(devices[0].bus_id.as_deref(), Some("8-4"));
        assert_eq!(
            devices[0].instance_id,
            r"USB\VID_045E&PID_0039\7&19033BE6&0&4"
        );
    }

    #[test]
    fn classifies_sharing_state() {
        let devices = parse(SAMPLE).unwrap();
        // Connected, never bound.
        assert_eq!(devices[0].sharing_state(), SharingState::NotShared);
        // Only the bind record is left; the device is gone.
        assert_eq!(devices[1].sharing_state(), SharingState::Absent);
        // Held by a client.
        assert_eq!(devices[2].sharing_state(), SharingState::Attached);
    }

    #[test]
    fn missing_devices_array_is_an_error() {
        assert!(parse("{}").is_err());
    }
}
