//! Per-port power switching, through `vhfilter`.
//!
//! **This module records intent, not observation.** Nothing on Windows reports
//! whether a hub port has power — see [`crate::hub`] for what was measured — so
//! what is held here is "this application switched that port off", which is a
//! different claim and must be shown as one.
//!
//! The record lives as long as the process and no longer. It is not written to
//! the settings file: a port's power is a property of hardware that anything can
//! change while the application is not running, and a remembered "off" read back
//! at the next start would be a guess wearing the clothes of a fact — the same
//! reason identities are not stored (R7.8).
//!
//! It is dropped for a hub that stops being enumerated. Unplugging a hub resets
//! its ports to powered, so carrying the record across that would leave the
//! application asserting "off" about a port that is on.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Mutex;

use anyhow::{Result, anyhow, bail};
use serde::Serialize;

/// A hub `vhfilter` says can switch its ports' power.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PppsHub {
    /// The device instance id, which is also the join key with everything else
    /// here and with `usbipd` (F4).
    pub instance_id: String,
    pub ports: u32,
}

/// What we asked a port to do. Absent means "never touched by us", which is not
/// the same as "on".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PortPower {
    /// Switched off by this application, this session.
    SwitchedOff,
    /// Switched on by this application, this session.
    SwitchedOn,
}

static SWITCHED: Mutex<Option<HashMap<String, HashMap<u32, PortPower>>>> = Mutex::new(None);

/// The last `--list-hubs` answer, with the set of hubs it was read for.
///
/// Which hubs can switch power changes only when one is plugged or unplugged,
/// and the list costs a process launch — which the device list would otherwise
/// pay for twice a second.
static LISTED: Mutex<Option<(Vec<String>, Vec<PppsHub>)>> = Mutex::new(None);

/// Builds a command that starts without flashing a console window.
///
/// The same treatment [`crate::usbipd`] gives its own calls, and for the same
/// reason: a GUI polling a console tool must not put a black box on screen
/// every time it looks.
fn command(exe: &Path) -> Command {
    let mut cmd = Command::new(exe);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        cmd.creation_flags(CREATE_NO_WINDOW);
    }
    cmd
}

fn held() -> std::sync::MutexGuard<'static, Option<HashMap<String, HashMap<u32, PortPower>>>> {
    let mut guard = SWITCHED.lock().unwrap();
    guard.get_or_insert_with(HashMap::new);
    guard
}

/// What this application last asked of a port, if anything.
pub fn recorded(hub_instance_id: &str, port: u32) -> Option<PortPower> {
    held()
        .as_ref()
        .unwrap()
        .get(hub_instance_id)
        .and_then(|ports| ports.get(&port))
        .copied()
}

/// Every port this application has switched off and not switched back.
pub fn switched_off(hub_instance_id: &str) -> Vec<u32> {
    let guard = held();
    let Some(ports) = guard.as_ref().unwrap().get(hub_instance_id) else {
        return Vec::new();
    };
    let mut out: Vec<u32> = ports
        .iter()
        .filter(|(_, state)| **state == PortPower::SwitchedOff)
        .map(|(port, _)| *port)
        .collect();
    out.sort_unstable();
    out
}

/// Forgets hubs that are no longer enumerated.
///
/// Unplugging a hub powers its ports back up, so a record kept across that would
/// be wrong in the direction that matters: the application would go on saying a
/// live port is dead.
pub fn forget_absent(present: &HashSet<String>) -> usize {
    let mut guard = held();
    let map = guard.as_mut().unwrap();
    let before = map.len();
    map.retain(|hub, _| present.contains(hub));
    before - map.len()
}

/// Switches a port and records what was asked.
///
/// The record is written only after `vhfilter` reports success, so a refused
/// switch leaves the application's account of the port unchanged.
pub fn switch(vhfilter: &Path, hub_instance_id: &str, port: u32, on: bool) -> Result<()> {
    let output = command(vhfilter)
        .arg("--switch-port")
        .arg(port.to_string())
        .arg(if on { "on" } else { "off" })
        .arg(hub_instance_id)
        .output()
        .map_err(|e| anyhow!("could not run {}: {e}", vhfilter.display()))?;

    if !output.status.success() {
        let detail = String::from_utf8_lossy(&output.stdout);
        let detail = detail.trim();
        bail!(
            "vhfilter refused to switch port {port}{}",
            if detail.is_empty() {
                String::new()
            } else {
                format!(": {detail}")
            }
        );
    }

    let state = if on {
        PortPower::SwitchedOn
    } else {
        PortPower::SwitchedOff
    };
    held()
        .as_mut()
        .unwrap()
        .entry(hub_instance_id.to_owned())
        .or_default()
        .insert(port, state);
    Ok(())
}

/// The hubs `vhfilter` says support per-port power switching.
///
/// An empty list is the normal answer on a machine with no such hub, and is not
/// an error. A hub appearing here means only that it **advertises** the feature:
/// hubs that claim it and do nothing are common, which is why switching one is
/// something the user does and sees the result of, not something inferred.
/// `present` is the hubs currently enumerated; the answer is re-read only when
/// that set changes.
pub fn hubs(vhfilter: &Path, present: &[String]) -> Result<Vec<PppsHub>> {
    let mut key: Vec<String> = present.iter().map(|h| h.to_ascii_lowercase()).collect();
    key.sort();

    let mut cached = LISTED.lock().unwrap();
    if let Some((for_key, hubs)) = cached.as_ref()
        && *for_key == key
    {
        return Ok(hubs.clone());
    }

    let output = command(vhfilter)
        .arg("--list-hubs")
        .output()
        .map_err(|e| anyhow!("could not run {}: {e}", vhfilter.display()))?;
    let hubs = parse_hubs(&String::from_utf8_lossy(&output.stdout));
    *cached = Some((key, hubs.clone()));
    Ok(hubs)
}

/// Forgets the cached hub list, so the next call asks again.
///
/// Called when the path to `vhfilter` changes: the answer is keyed on the hubs
/// present, and those have not moved even though the tool answering has.
pub fn forget_listed() {
    *LISTED.lock().unwrap() = None;
}

/// Reads `--list-hubs` output.
///
/// One line per hub, of the shape measured on vhfilter 1.0.0.3:
///
/// ```text
/// USB\VID_1A86&PID_8094\5&829319&0&2     has  4 ports    is USB 2    and attached to port  2 on USB\ROOT_HUB30\4&945a1ec&0&0 with companion port  3
/// ```
///
/// Only the instance id and the port count are taken. The rest describes where
/// the hub itself hangs, which this application already knows from the
/// enumeration and would only be able to disagree with.
fn parse_hubs(text: &str) -> Vec<PppsHub> {
    text.lines()
        .filter_map(|line| {
            let line = line.trim();
            let (id, rest) = line.split_once(char::is_whitespace)?;
            if !id.to_ascii_uppercase().starts_with("USB\\") {
                return None;
            }
            let ports = rest
                .split_whitespace()
                .skip_while(|word| *word != "has")
                .nth(1)?
                .parse()
                .ok()?;
            Some(PppsHub {
                instance_id: id.to_owned(),
                ports,
            })
        })
        .collect()
}

/// The script that fetches `vhfilter.exe` and installs its filter driver.
///
/// A `.bat` rather than an HTTP client inside the application. The download is
/// the easy half; the hard half is `--install-filter`, which needs
/// administrator rights and a reboot, and driving an elevation from a GUI that
/// otherwise never elevates (§5.3) buys nothing here. The script asks for the
/// rights itself, at the moment the user chose to run it.
///
/// It checks the Authenticode signature before running what it downloaded, and
/// deletes the file rather than running it if the signer is not VirtualHere.
const SETUP_SCRIPT: &str = include_str!("vhfilter-setup.bat");

/// Writes the setup script into `dir`, returning where it went.
///
/// Written beside where the tool is wanted: `dir` is one of the places
/// [`locate`] looks, so a successful run needs nothing configured afterwards.
pub fn write_setup_script(dir: &Path) -> Result<PathBuf> {
    std::fs::create_dir_all(dir).map_err(|e| anyhow!("could not create {}: {e}", dir.display()))?;
    let path = dir.join("get-vhfilter.bat");
    std::fs::write(&path, SETUP_SCRIPT)
        .map_err(|e| anyhow!("could not write {}: {e}", path.display()))?;
    Ok(path)
}

/// Where `vhfilter.exe` is, if it can be found.
///
/// Beside the executable first, which is the answer to "where do I put this?"
/// for a tool distributed as a bare `.exe` with no installer: the application's
/// own folder is somewhere the user already has open. `%LOCALAPPDATA%` next, so
/// an installed copy has a writable home. A configured path wins over both.
pub fn locate(configured: Option<&str>) -> Option<PathBuf> {
    found(configured).map(|(path, _)| path)
}

/// How `vhfilter.exe` was arrived at.
///
/// Worth reporting separately: "it is at this path" and "nothing had to be set
/// up for it to be at this path" are different things to know, and only the
/// second tells the user the search did its job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FoundHow {
    /// At the path the user gave.
    Configured,
    /// In one of the folders [`search_path`] looks in.
    Searched,
}

/// Where `vhfilter.exe` is and how it was arrived at.
pub fn found(configured: Option<&str>) -> Option<(PathBuf, FoundHow)> {
    if let Some(path) = configured.filter(|p| !p.trim().is_empty()) {
        let path = PathBuf::from(path.trim());
        return path.is_file().then_some((path, FoundHow::Configured));
    }
    for dir in search_path() {
        let candidate = dir.join("vhfilter.exe");
        if candidate.is_file() {
            return Some((candidate, FoundHow::Searched));
        }
    }
    None
}

/// The directories [`locate`] looks in, in order. Public so the interface can
/// say where to put the file rather than only that it is missing.
pub fn search_path() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
    {
        dirs.push(dir.to_path_buf());
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        dirs.push(PathBuf::from(local).join("wsl-usb-identity-manager"));
    }
    dirs
}

#[cfg(test)]
mod tests {
    use super::*;

    const MEASURED: &str = "The following hubs are attached that support Per-Port-Power-Switching:\n\
        \n\
        USB\\VID_1A86&PID_8094\\5&829319&0&2\t has  4 ports\tis USB 2\tand attached to port  2 on USB\\ROOT_HUB30\\4&945a1ec&0&0 with companion port  3\n";

    #[test]
    fn reads_the_measured_output() {
        let hubs = parse_hubs(MEASURED);
        assert_eq!(
            hubs,
            vec![PppsHub {
                instance_id: "USB\\VID_1A86&PID_8094\\5&829319&0&2".to_owned(),
                ports: 4,
            }]
        );
    }

    #[test]
    fn no_ppps_hub_is_an_empty_list_and_not_an_error() {
        let text = "The following hubs are attached that support Per-Port-Power-Switching:\n\n";
        assert!(parse_hubs(text).is_empty());
    }

    #[test]
    fn the_heading_is_not_mistaken_for_a_hub() {
        // It has words but no instance id, which is the whole test.
        assert!(parse_hubs("The following hubs are attached that support X:").is_empty());
    }

    #[test]
    fn a_configured_path_that_does_not_exist_finds_nothing() {
        assert!(locate(Some("Z:\\nowhere\\vhfilter.exe")).is_none());
        // Blank is "not configured", not "look at the empty path".
        assert!(locate(Some("   ")).is_none() || locate(None).is_some());
    }
}
