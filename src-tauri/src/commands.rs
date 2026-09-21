//! The commands the frontend can call.
//!
//! Everything privileged lives behind one of these rather than behind a Tauri
//! plugin, so the capability file stays at the core permission set and the list
//! of things the UI can reach is exactly this file.

use std::sync::OnceLock;
use std::time::Instant;

use wuim_core::UsbIds;
use wuim_core::autostart;
use wuim_core::instance_id::InstanceId;
use wuim_core::ppps;
use wuim_core::shell_open;
use wuim_core::snapshot::{DeviceRow, Snapshot};
use wuim_core::usbipd::{self, Availability, Operation};
use wuim_core::webview2;
use wuim_probe::TargetIdentity;

use crate::logging;
use crate::state;
use crate::tray::{self, TrayView};
use crate::view::{DeviceView, HubView, Identity, LastKnown, PortView, SettingsView, TopologyView};

/// anyhow's chain, flattened for the frontend and written to the log on the way
/// past. Tauri needs a `Serialize` error and `{:#}` keeps the causes that make a
/// failure diagnosable.
fn to_message(e: anyhow::Error) -> String {
    let message = format!("{e:#}");
    logging::error(&message);
    message
}

/// Runs blocking work off the main thread.
///
/// Tauri runs a synchronous command on the main thread, which also serves the
/// IPC channel — so a command that shells out to usbipd and waits several
/// seconds stops the frontend from being told anything at all, including that
/// the operation started. Everything here that waits on a process or on device
/// enumeration goes through this.
async fn off_thread<T, F>(what: &str, work: F) -> Result<T, String>
where
    F: FnOnce() -> Result<T, String> + Send + 'static,
    T: Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work)
        .await
        .map_err(|e| {
            let message = format!("{what} did not finish: {e}");
            logging::error(&message);
            message
        })?
}

/// Puts the frontend's labels and counts into the tray menu.
///
/// The translations live in the frontend (R10.5), so the text arrives from
/// there rather than being built here.
#[tauri::command]
pub fn set_tray(view: TrayView) -> Result<(), String> {
    tray::apply(view).map_err(to_message)
}

/// Shows the window, for when something needs to be asked of the user.
#[tauri::command]
pub fn show_window(app: tauri::AppHandle) {
    tray::show(&app);
}

/// Hides the window, leaving the application in the tray.
#[tauri::command]
pub fn hide_window(window: tauri::Window) -> Result<(), String> {
    window.hide().map_err(|e| to_message(e.into()))
}

/// Whether usbipd is installed and answering (requirement R13.1).
///
/// Runs two processes, so it is kept off the main thread and called only at
/// startup and after a listing fails — not on the two-second timer.
#[tauri::command]
pub async fn check_usbipd() -> Result<Availability, String> {
    off_thread("the usbipd check", || Ok(usbipd::availability())).await
}

/// Somewhere this application can point Windows at.
///
/// An enum rather than a path, so nothing the frontend holds decides what gets
/// opened: every value here resolves to a folder this application writes to or
/// to a URL compiled into it.
#[derive(Debug, Clone, Copy, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenTarget {
    /// The folder holding `wuim.log`.
    LogFolder,
    /// The folder holding the settings file.
    SettingsFolder,
    UsbipdReleases,
    Webview2Download,
    /// Where this application comes from: releases, the documentation and
    /// wherever a problem gets reported.
    ProjectHome,
    /// The page `vhfilter` is downloaded from, for anyone who would rather
    /// fetch it themselves than run the script.
    VhfilterHome,
}

const USBIPD_RELEASES: &str = "https://github.com/dorssel/usbipd-win/releases/latest";
const PROJECT_HOME: &str = "https://github.com/tanakamasayuki/wsl-usb-identity-manager";
const VHFILTER_HOME: &str = "https://www.virtualhere.com/node/4352";

#[tauri::command]
pub fn open_target(target: OpenTarget) -> Result<(), String> {
    let result = match target {
        OpenTarget::LogFolder => open_parent_of(&logging::path()),
        OpenTarget::SettingsFolder => open_parent_of(&state::path()),
        OpenTarget::UsbipdReleases => shell_open::url(USBIPD_RELEASES),
        OpenTarget::Webview2Download => shell_open::url(webview2::DOWNLOAD_URL),
        OpenTarget::ProjectHome => shell_open::url(PROJECT_HOME),
        OpenTarget::VhfilterHome => shell_open::url(VHFILTER_HOME),
    };
    result.map_err(to_message)
}

fn open_parent_of(file: &std::path::Path) -> anyhow::Result<()> {
    let folder = file
        .parent()
        .ok_or_else(|| anyhow::anyhow!("{} has no folder to open", file.display()))?;
    shell_open::folder(folder)
}

/// Lets the frontend write to the same log, so one file has the whole story.
#[tauri::command]
pub fn log_message(level: String, message: String) {
    logging::write(if level == "error" { "error" } else { "info" }, &message);
}

/// Reads the current state. Probes nothing (requirement R4.6), so this is safe
/// to call on a timer or on every device-change event.
#[tauri::command]
pub async fn list_devices() -> Result<Vec<DeviceView>, String> {
    off_thread("listing devices", || {
        let snapshot = Snapshot::capture().map_err(to_message)?;
        Ok(to_views(&snapshot))
    })
    .await
}

/// Read once: the file is three quarters of a megabyte and never changes while
/// the application runs.
fn usb_ids() -> &'static UsbIds {
    static IDS: OnceLock<UsbIds> = OnceLock::new();
    IDS.get_or_init(|| {
        let Some(path) = usbipd::usb_ids_path() else {
            logging::info("usb.ids not found next to usbipd; vendor names unavailable");
            return UsbIds::default();
        };
        let ids = UsbIds::load(&path);
        logging::info(&format!(
            "loaded vendor names from {}{}",
            path.display(),
            if ids.is_empty() { " (empty)" } else { "" }
        ));
        ids
    })
}

fn to_views(snapshot: &Snapshot) -> Vec<DeviceView> {
    let ids = usb_ids();
    // Read once for the whole list: every row is measured against the same
    // rules, and reading them per device would take the lock as many times.
    let rules = state::with(|store| store.settings.auto_attach_rules.clone());

    // An identity belongs to a device that is still plugged in. Anything that
    // has gone is forgotten here rather than lingering to be matched back to
    // whatever appears in its place.
    let present: Vec<String> = snapshot
        .devices
        .iter()
        .filter(|row| row.usbipd.as_ref().is_some_and(|d| d.is_connected()))
        .map(|row| row.instance_id.raw.clone())
        .collect();
    state::forget_absent(&present);
    remember_names(snapshot);

    snapshot
        .devices
        .iter()
        .map(|row| {
            let identity = state::identity(&row.instance_id.raw)
                .as_ref()
                .map(Identity::from)
                // Nothing probed it, but the descriptors may name it anyway.
                // Second, not first: a probe read the silicon, which outlives a
                // reflash changing what the descriptors say. Derived fresh each
                // time rather than remembered, because the instance id it comes
                // from is the key everything here is already filed under — and
                // that survives an attach, where a probe cannot reach at all.
                .or_else(|| {
                    wuim_probe::usb_descriptor::identify(&row.instance_id)
                        .as_ref()
                        .map(Identity::from)
                });
            let last = state::last_seen(&row.instance_id.raw)
                .as_ref()
                .map(LastKnown::from)
                .unwrap_or_default();
            DeviceView::from_row(row, ids, identity, last, &rules)
        })
        .collect()
}

/// Records the name of every device Windows can currently describe.
///
/// Once a device is attached the only name left is the description `usbipd`
/// cached, and a device bound with `--force` has had its driver swapped for the
/// stub before that cache was even taken — so neither is guaranteed to still say
/// what the device is (finding F4). Taken before the views are built, so a row
/// showing for the first time already has its own name recorded and does not
/// appear to have been renamed.
fn remember_names(snapshot: &Snapshot) {
    let seen: Vec<(String, String)> = snapshot
        .devices
        .iter()
        .filter(|row| row.windows.is_some())
        .map(|row| (row.instance_id.raw.clone(), row.name.clone()))
        .collect();
    state::remember_names(&seen);
}

/// Hands the stored settings to the frontend at startup.
#[tauri::command]
pub fn read_settings() -> StoredSettings {
    let mut settings = state::with(|store| SettingsView::from(&store.settings));
    // The registry is what Windows acts on, so it decides. An entry removed by
    // hand, or left behind by a copy that has since moved, is reported as it
    // actually is rather than as the file remembers it.
    settings.start_with_windows = autostart::is_enabled();

    StoredSettings {
        settings,
        writable: state::is_writable(),
        path: state::path().display().to_string(),
        log_path: logging::path().display().to_string(),
        remembered: state::remembered_count(),
    }
}

/// The hub tree, with what has been asked of each port.
///
/// Read on its own rather than with the device list: it costs an IOCTL per hub,
/// and the flat list — which is what most people leave the window on — has no
/// use for it.
#[tauri::command]
pub async fn read_topology() -> Result<TopologyView, String> {
    off_thread("reading the hub topology", || {
        let snapshot = Snapshot::capture().map_err(to_message)?;

        // A hub that has gone takes its switching record with it: unplugging one
        // powers its ports back up, so keeping the record would leave us
        // asserting "off" about a port that is on.
        let present: std::collections::HashSet<String> = snapshot
            .hubs
            .iter()
            .map(|h| h.instance_id.clone())
            .collect();
        let dropped = ppps::forget_absent(&present);
        if dropped > 0 {
            logging::info(&format!(
                "forgot the switched ports of {dropped} hub(s) that went away"
            ));
        }

        let configured = state::with(|store| store.settings.vhfilter_path.clone());
        let located = ppps::found(Some(&configured));
        let vhfilter = located.as_ref().map(|(path, _)| path.clone());
        let present_ids: Vec<String> = present.iter().cloned().collect();
        let ppps_hubs: Vec<String> = vhfilter
            .as_deref()
            .and_then(|path| ppps::hubs(path, &present_ids).ok())
            .unwrap_or_default()
            .into_iter()
            .map(|h| h.instance_id)
            .collect();

        let ids = usb_ids();
        let hubs = snapshot
            .hubs
            .iter()
            .map(|h| {
                // The same lookup a device row gets. A hub is a USB device like
                // any other, and `1a86:8094` on its own says less than "WCH".
                let (vendor, usb_product) = match InstanceId::parse(&h.instance_id).vid_pid() {
                    Some((vid, pid)) => (
                        ids.vendor(vid).map(str::to_owned),
                        ids.product(vid, pid).map(str::to_owned),
                    ),
                    None => (None, None),
                };
                HubView {
                    ppps: ppps_hubs
                        .iter()
                        .any(|id| id.eq_ignore_ascii_case(&h.instance_id)),
                    instance_id: h.instance_id.clone(),
                    name: h.name.clone(),
                    location_path: h.location_path.clone(),
                    parent_instance_id: h.parent_instance_id.clone(),
                    port_address: h.address,
                    vid_pid: h.vid_pid.clone(),
                    vendor,
                    usb_product,
                    manufacturer: h.manufacturer.clone(),
                    driver_version: h.driver_version.clone(),
                    ports: h
                        .ports
                        .iter()
                        .map(|p| PortView {
                            port: p.port,
                            connected: p.connected,
                            status: p.status_name,
                            switched: ppps::recorded(&h.instance_id, p.port).map(
                                |state| match state {
                                    ppps::PortPower::SwitchedOff => "off",
                                    ppps::PortPower::SwitchedOn => "on",
                                },
                            ),
                        })
                        .collect(),
                }
            })
            .collect();

        Ok(TopologyView {
            hubs,
            vhfilter_how: located.as_ref().map(|(_, how)| match how {
                ppps::FoundHow::Configured => "configured",
                ppps::FoundHow::Searched => "searched",
            }),
            vhfilter: vhfilter.map(|p| p.display().to_string()),
            vhfilter_search_path: ppps::search_path()
                .iter()
                .map(|p| p.display().to_string())
                .collect(),
        })
    })
    .await
}

/// Switches a hub port's power.
///
/// Runs unelevated: `vhfilter` needs administrator rights to install its filter
/// driver, once, and none to use it afterwards (measured). What the switch did
/// cannot be read back, so the result recorded here is what was asked for.
#[tauri::command]
pub async fn switch_port(hub: String, port: u32, on: bool) -> Result<(), String> {
    off_thread("switching a hub port", move || {
        let configured = state::with(|store| store.settings.vhfilter_path.clone());
        let vhfilter = ppps::locate(Some(&configured))
            .ok_or_else(|| "vhfilter.exe was not found".to_owned())?;
        logging::info(&format!(
            "switching port {port} of {hub} {}",
            if on { "on" } else { "off" }
        ));
        ppps::switch(&vhfilter, &hub, port, on).map_err(to_message)
    })
    .await
}

/// Writes the script that fetches `vhfilter` and installs its filter driver,
/// then opens the folder it went into.
///
/// Written rather than run: it asks for administrator rights and ends in a
/// reboot, and this application does not drive elevations it can hand to the
/// user instead (§5.3).
#[tauri::command]
pub async fn write_vhfilter_setup() -> Result<String, String> {
    off_thread("writing the vhfilter setup script", || {
        // The second search path: under %LOCALAPPDATA%, beside the log, which is
        // writable without asking anyone. Downloading there means a successful
        // run needs nothing configured afterwards.
        let dir = ppps::search_path()
            .into_iter()
            .nth(1)
            .ok_or_else(|| "no writable folder to put it in".to_owned())?;
        let path = ppps::write_setup_script(&dir).map_err(to_message)?;
        logging::info(&format!("wrote {}", path.display()));
        shell_open::folder(&dir).map_err(to_message)?;
        Ok(path.display().to_string())
    })
    .await
}

/// Switches every port of one hub.
///
/// One command rather than a loop in the interface: each switch is a process
/// launch, and doing four of them with a list refresh between each would make
/// "turn this hub off" a visibly staggered thing rather than one action.
#[tauri::command]
pub async fn switch_hub(hub: String, on: bool) -> Result<(), String> {
    off_thread("switching a hub", move || {
        let configured = state::with(|store| store.settings.vhfilter_path.clone());
        let vhfilter = ppps::locate(Some(&configured))
            .ok_or_else(|| "vhfilter.exe was not found".to_owned())?;

        let snapshot = Snapshot::capture().map_err(to_message)?;
        let ports = snapshot
            .hubs
            .iter()
            .find(|h| h.instance_id.eq_ignore_ascii_case(&hub))
            .map(|h| h.ports.len() as u32)
            .ok_or_else(|| format!("{hub} is not connected"))?;

        logging::info(&format!(
            "switching all {ports} ports of {hub} {}",
            if on { "on" } else { "off" }
        ));

        // Every port is attempted even when one refuses: stopping half way
        // through would leave the hub in a state nobody asked for.
        let mut failed = Vec::new();
        for port in 1..=ports {
            if let Err(e) = ppps::switch(&vhfilter, &hub, port, on) {
                failed.push(format!("{port}: {e}"));
            }
        }
        if failed.is_empty() {
            Ok(())
        } else {
            Err(format!("some ports refused — {}", failed.join("; ")))
        }
    })
    .await
}

/// Drops every remembered device, returning how many there were.
///
/// Offered because what is remembered can be wrong without anything having gone
/// wrong: a board moved to another port leaves its old entry behind, and after
/// enough of that the reminders stop helping (R7.9).
#[tauri::command]
pub fn clear_remembered() -> usize {
    state::forget_everything_seen()
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSettings {
    pub settings: SettingsView,
    /// False when the file on disk was refused; the panel says so.
    pub writable: bool,
    pub path: String,
    /// Where the log is. A release build has no console to print it to, so this
    /// is the only way the user learns where to look (R13.8).
    pub log_path: String,
    /// How many devices have a remembered name or identification (R4.21).
    pub remembered: usize,
}

/// Saves the settings the user changed.
#[tauri::command]
pub fn write_settings(settings: SettingsView) -> Result<(), String> {
    // The listed hubs are cached against the hubs present, which a new path to
    // vhfilter does not change — so the cache has to be dropped by hand.
    if state::with(|store| store.settings.vhfilter_path != settings.vhfilter_path) {
        ppps::forget_listed();
    }
    logging::info(&format!(
        "settings: auto_identify={} exclude={:?} confirm={} startup={} auto_attach={} rules={:?}",
        settings.auto_identify,
        settings.auto_exclude,
        settings.confirm_before_identify,
        settings.start_with_windows,
        settings.auto_attach,
        settings
            .auto_attach_rules
            .iter()
            .map(|rule| format!("{:?}:{}", rule.kind, rule.value))
            .collect::<Vec<_>>(),
    ));

    // Applied before the file is written: if the registry refuses, the setting
    // has not taken effect and saying otherwise would be a lie the user only
    // discovers at the next login.
    if autostart::is_enabled() != settings.start_with_windows {
        autostart::set(settings.start_with_windows).map_err(to_message)?;
        logging::info(&format!(
            "startup entry {}",
            if settings.start_with_windows {
                "added"
            } else {
                "removed"
            }
        ));
    }

    state::update(|store| store.settings = settings.into());
    Ok(())
}

/// Runs a usbipd operation against a device.
///
/// Bind and unbind raise a UAC prompt at this point and nowhere else: the
/// application itself runs unelevated (requirement R5.6). The bus id is looked
/// up inside `usbipd::run`, immediately before the command is issued, so
/// nothing the frontend was holding decides which device is affected (R5.4).
#[tauri::command]
pub async fn run_operation(
    instance_id: String,
    operation: Operation,
) -> Result<Vec<DeviceView>, String> {
    logging::info(&format!("{operation:?} requested for {instance_id}"));
    let started = Instant::now();
    let executed = off_thread("the usbipd operation", move || {
        usbipd::run(operation, &instance_id).map_err(to_message)
    })
    .await?;
    logging::info(&format!(
        "ran `{}` in {:.1}s{}{}{}",
        executed.command_line,
        started.elapsed().as_secs_f32(),
        if executed.elevated { " (elevated)" } else { "" },
        if executed.stdout.is_empty() {
            String::new()
        } else {
            format!(
                "
  stdout: {}",
                executed.stdout
            )
        },
        if executed.stderr.is_empty() {
            String::new()
        } else {
            format!(
                "
  stderr: {}",
                executed.stderr
            )
        },
    ));
    // Hand back the new state rather than leaving the UI to poll for it: an
    // attach changes several fields at once and the user just asked for it.
    list_devices().await
}

/// Asks a board what it is.
///
/// **Has side effects.** This is trigger 1 of requirement R4.5 — an explicit
/// request — so it must only ever be reached from a user action that has
/// already shown the side effects the `probes` field describes.
#[tauri::command]
pub async fn probe_device(instance_id: String, family: String) -> Result<TargetIdentity, String> {
    off_thread("the probe", move || probe_blocking(&instance_id, &family)).await
}

fn probe_blocking(instance_id: &str, family: &str) -> Result<TargetIdentity, String> {
    // Re-read rather than trusting a snapshot the frontend may have been
    // holding: the device could have moved, been unplugged, or been attached
    // since the list was drawn.
    let snapshot = Snapshot::capture().map_err(to_message)?;
    let row = find(&snapshot, instance_id)
        .ok_or_else(|| format!("{instance_id} is no longer present"))?;

    let device = row
        .windows
        .as_ref()
        .ok_or_else(|| match row.sharing_state() {
            wuim_core::SharingState::Attached => {
                "the device is attached to WSL, so Windows cannot reach it".to_owned()
            }
            _ => "the device is not connected".to_owned(),
        })?;

    let probe = wuim_probe::probes()
        .into_iter()
        .find(|p| p.family() == family)
        .ok_or_else(|| format!("no probe named {family}"))?;

    // Re-check rather than trusting the frontend's copy of the verdict.
    if let Some(note) = probe.applicability(device).note() {
        return Err(format!("{family}: {note}"));
    }

    logging::info(&format!(
        "probing {instance_id} with {family} via {}",
        device.com_port.as_deref().unwrap_or("-")
    ));
    let identity = probe.probe(device).map_err(to_message)?;
    logging::info(&format!(
        "{instance_id} identified as {}",
        identity.identity_key
    ));

    // Held for as long as the device stays plugged in, and no longer. Writing
    // it down would only be useful if it could be matched back afterwards, and
    // nothing in USB can vouch that the same board is still on the cable.
    state::remember_identity(row.instance_id.raw.clone(), identity.clone());

    Ok(identity)
}

fn find<'a>(snapshot: &'a Snapshot, instance_id: &str) -> Option<&'a DeviceRow> {
    snapshot
        .devices
        .iter()
        .find(|row| row.instance_id.matches(instance_id))
}
