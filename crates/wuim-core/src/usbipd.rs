//! Reading `usbipd` state.
//!
//! State comes from `usbipd state` (JSON) only. The text output of `usbipd list`
//! is never parsed (requirement R5.2).

use std::path::PathBuf;
use std::process::Command;

use anyhow::{Context, Result, anyhow, bail};
use serde::{Deserialize, Serialize};

use crate::elevate::run_elevated;

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

/// An operation usbipd can perform on a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Operation {
    /// Make the device shareable. Changes system state, so it needs admin.
    Bind,
    /// Stop sharing it.
    Unbind,
    /// Hand the device to WSL.
    Attach,
    /// Take it back.
    Detach,
}

impl Operation {
    fn verb(self) -> &'static str {
        match self {
            Self::Bind => "bind",
            Self::Unbind => "unbind",
            Self::Attach => "attach",
            Self::Detach => "detach",
        }
    }

    /// Arguments beyond `--busid`.
    ///
    /// usbipd refuses `attach` without a client to hand the device to. `--wsl`
    /// with no distribution named uses the default one, which is what usbipd
    /// 5.x asks for — per-device distribution selection (requirement R6.2)
    /// needs somewhere to store the choice and is not built yet.
    fn extra_args(self) -> &'static [&'static str] {
        match self {
            Self::Attach => &["--wsl"],
            _ => &[],
        }
    }

    /// Whether the operation has to run elevated (requirements §5.2).
    pub fn needs_admin(self) -> bool {
        matches!(self, Self::Bind | Self::Unbind)
    }
}

/// What a usbipd invocation actually did, for the caller to log.
#[derive(Debug, Clone)]
pub struct Executed {
    /// The command line as it was issued, including the bus id that was
    /// resolved for it.
    pub command_line: String,
    pub elevated: bool,
    pub stdout: String,
    pub stderr: String,
}

/// Runs an operation against the device with this instance id.
///
/// The bus id is read from `usbipd state` immediately before the command is
/// issued and never taken from anything the caller is holding (requirement
/// R5.4): hub numbering shifts, so a bus id noted even seconds ago may now name
/// a different device — and this operation would then hand *that* device to
/// WSL.
pub fn run(operation: Operation, instance_id: &str) -> Result<Executed> {
    let devices = query()?;
    let device = devices
        .iter()
        .find(|d| d.instance_id.eq_ignore_ascii_case(instance_id))
        .ok_or_else(|| anyhow!("usbipd no longer knows about {instance_id}"))?;

    let selector = selector_for(operation, device)?;

    let exe = locate()?;
    let extra = operation.extra_args();
    let arguments = std::iter::once(operation.verb())
        .chain([selector.flag, selector.value.as_str()])
        .chain(extra.iter().copied())
        .collect::<Vec<_>>()
        .join(" ");

    let command_line = format!("{} {arguments}", exe.display());

    if operation.needs_admin() {
        let code = run_elevated(&exe, &arguments)?;
        if code != 0 {
            bail!("`{command_line}` (elevated) exited with {code}");
        }
        // An elevated child writes to its own console, so there is nothing to
        // capture beyond the exit code.
        return Ok(Executed {
            command_line,
            elevated: true,
            stdout: String::new(),
            stderr: String::new(),
        });
    }

    let output = command(&exe)
        .args([operation.verb(), selector.flag, selector.value.as_str()])
        .args(extra)
        .output()
        .with_context(|| format!("failed to run {}", exe.display()))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_owned();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_owned();
    if !output.status.success() {
        let detail = if stderr.is_empty() { &stdout } else { &stderr };
        bail!("`{command_line}` failed: {detail}");
    }
    Ok(Executed {
        command_line,
        elevated: false,
        stdout,
        stderr,
    })
}

/// How a device is named on the usbipd command line.
struct Selector {
    flag: &'static str,
    value: String,
}

/// Decides which name to use for this device, and refuses if there is none.
///
/// A connected device is named by its bus id. A device that is not connected
/// has no bus id at all, which is where every operation but one stops. Unbind
/// is the exception: the bind record outlives the device, and removing that
/// record is exactly what is being asked for when a row reads "absent". usbipd
/// names a record with no device behind it by its persisted GUID.
fn selector_for(operation: Operation, device: &UsbipdDevice) -> Result<Selector> {
    if let Some(bus_id) = device.bus_id.as_deref() {
        return Ok(Selector {
            flag: "--busid",
            value: checked_bus_id(bus_id)?.to_owned(),
        });
    }

    if operation == Operation::Unbind
        && let Some(guid) = device.persisted_guid.as_deref()
    {
        return Ok(Selector {
            flag: "--guid",
            value: checked_guid(guid)?.to_owned(),
        });
    }

    bail!("{} is not connected", device.description)
}

/// Rejects anything that is not a plain `<hub>-<port>`.
///
/// The elevated path passes the bus id inside a command line rather than as an
/// argument vector, so it is checked here rather than trusted because it came
/// from usbipd a moment ago.
fn checked_bus_id(bus_id: &str) -> Result<&str> {
    let valid = bus_id.split_once('-').is_some_and(|(hub, port)| {
        !hub.is_empty()
            && !port.is_empty()
            && hub.bytes().all(|b| b.is_ascii_digit())
            && port.bytes().all(|b| b.is_ascii_digit())
    });
    if valid {
        Ok(bus_id)
    } else {
        Err(anyhow!("usbipd reported an unusable bus id: {bus_id:?}"))
    }
}

/// Rejects anything that is not a plain GUID, for the same reason as
/// [`checked_bus_id`]: unbind runs elevated, through a command line.
fn checked_guid(guid: &str) -> Result<&str> {
    const DASHES: [usize; 4] = [8, 13, 18, 23];
    let valid = guid.len() == 36
        && guid.bytes().enumerate().all(|(i, b)| {
            if DASHES.contains(&i) {
                b == b'-'
            } else {
                b.is_ascii_hexdigit()
            }
        });
    if valid {
        Ok(guid)
    } else {
        Err(anyhow!(
            "usbipd reported an unusable persisted GUID: {guid:?}"
        ))
    }
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StateDocument {
    devices: Vec<UsbipdDevice>,
}

/// The oldest `usbipd` whose `state` output this build has been written
/// against. Older versions report a different JSON shape (R13.1).
pub const MINIMUM_MAJOR_VERSION: u32 = 5;

/// Whether usbipd is installed and answering.
///
/// Checked so that a missing usbipd reads as "install usbipd-win" rather than
/// as a failure on every operation the user tries (requirements R13.1, R13.2).
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum Availability {
    /// Found and answering. `supported` is false for a version older than
    /// [`MINIMUM_MAJOR_VERSION`], which still works but may not be understood.
    Ok { version: String, supported: bool },
    /// No `usbipd.exe`, either at the install path or on `PATH`.
    NotInstalled,
    /// The executable is there, but it did not answer. Usually the service.
    NotAnswering { detail: String },
}

/// Asks usbipd what it is and whether it is running.
pub fn availability() -> Availability {
    let Ok(exe) = locate() else {
        return Availability::NotInstalled;
    };

    // `--version` answers without the service, so it separates "not installed"
    // from "installed but not running".
    let version = match command(&exe).arg("--version").output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_owned()
        }
        Ok(output) => {
            return Availability::NotAnswering {
                detail: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
            };
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Availability::NotInstalled,
        Err(e) => {
            return Availability::NotAnswering {
                detail: e.to_string(),
            };
        }
    };

    if let Err(e) = query() {
        return Availability::NotAnswering {
            detail: format!("{e:#}"),
        };
    }

    Availability::Ok {
        supported: major_version(&version).is_some_and(|major| major >= MINIMUM_MAJOR_VERSION),
        version,
    }
}

/// The leading number of `5.3.0-54+Branch.master.Sha.aa3db8b`.
fn major_version(version: &str) -> Option<u32> {
    version.split(['.', '-', '+']).next()?.trim().parse().ok()
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

/// Where usbipd-win keeps its copy of `usb.ids`, next to the executable.
///
/// Only meaningful when usbipd was found at a real path; a bare `usbipd.exe`
/// resolved through PATH gives no directory to look in.
pub fn usb_ids_path() -> Option<PathBuf> {
    let default = PathBuf::from(DEFAULT_INSTALL_PATH);
    let candidate = default.parent()?.join("usb.ids");
    candidate.is_file().then_some(candidate)
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

    #[test]
    fn reads_the_major_version_out_of_what_usbipd_prints() {
        // As measured: usbipd 5.3.0 prints its build metadata too.
        assert_eq!(
            major_version("5.3.0-54+Branch.master.Sha.aa3db8b82c4cb5071fd31bc54211606c70886912"),
            Some(5)
        );
        assert_eq!(major_version("5.3.0"), Some(5));
        assert_eq!(major_version("4.2.0"), Some(4));
        assert_eq!(major_version(""), None);
        assert_eq!(major_version("not a version"), None);
    }

    #[test]
    fn only_bind_and_unbind_need_admin() {
        assert!(Operation::Bind.needs_admin());
        assert!(Operation::Unbind.needs_admin());
        assert!(!Operation::Attach.needs_admin());
        assert!(!Operation::Detach.needs_admin());
    }

    #[test]
    fn attach_names_a_client_to_hand_the_device_to() {
        // usbipd refuses an attach with no client.
        assert_eq!(Operation::Attach.extra_args(), &["--wsl"]);
        assert!(Operation::Detach.extra_args().is_empty());
        assert!(Operation::Bind.extra_args().is_empty());
    }

    #[test]
    fn accepts_real_bus_ids() {
        assert_eq!(checked_bus_id("10-1").unwrap(), "10-1");
        assert_eq!(checked_bus_id("2-10").unwrap(), "2-10");
    }

    #[test]
    fn an_absent_device_is_unbound_by_its_persisted_guid() {
        // The row the user sees says "absent" and offers to stop sharing. There
        // is no bus id to name it with, so usbipd takes the GUID instead.
        let absent = &parse(SAMPLE).unwrap()[1];
        assert_eq!(absent.sharing_state(), SharingState::Absent);

        let selector = selector_for(Operation::Unbind, absent).unwrap();
        assert_eq!(selector.flag, "--guid");
        assert_eq!(selector.value, "05d75b37-ae96-4b2a-9d0d-5cded25f52eb");
    }

    #[test]
    fn nothing_else_can_run_against_an_absent_device() {
        let absent = &parse(SAMPLE).unwrap()[1];
        for operation in [Operation::Bind, Operation::Attach, Operation::Detach] {
            assert!(
                selector_for(operation, absent).is_err(),
                "{operation:?} should need a connected device"
            );
        }
    }

    #[test]
    fn a_connected_device_is_named_by_its_bus_id() {
        let connected = &parse(SAMPLE).unwrap()[0];
        let selector = selector_for(Operation::Bind, connected).unwrap();
        assert_eq!(selector.flag, "--busid");
        assert_eq!(selector.value, "8-4");
    }

    #[test]
    fn rejects_anything_that_is_not_a_guid() {
        assert!(checked_guid("05d75b37-ae96-4b2a-9d0d-5cded25f52eb").is_ok());
        for bad in [
            "",
            "05d75b37ae964b2a9d0d5cded25f52eb",
            "05d75b37-ae96-4b2a-9d0d-5cded25f52e",
            "05d75b37-ae96-4b2a-9d0d-5cded25f52eb ",
            "05d75b37-ae96-4b2a-9d0d-5cded25f52e&",
            "zzzzzzzz-ae96-4b2a-9d0d-5cded25f52eb",
        ] {
            assert!(checked_guid(bad).is_err(), "accepted {bad:?}");
        }
    }

    #[test]
    fn rejects_anything_that_is_not_a_bus_id() {
        for bad in ["", "10", "10-", "-1", "10-1 && calc", "a-1", "10-1-2"] {
            assert!(checked_bus_id(bad).is_err(), "accepted {bad:?}");
        }
    }
}
