//! The commands the frontend can call.
//!
//! Everything privileged lives behind one of these rather than behind a Tauri
//! plugin, so the capability file stays at the core permission set and the list
//! of things the UI can reach is exactly this file.

use std::sync::OnceLock;
use std::time::Instant;

use wuim_core::UsbIds;
use wuim_core::autostart;
use wuim_core::shell_open;
use wuim_core::snapshot::{DeviceRow, Snapshot};
use wuim_core::usbipd::{self, Availability, Operation};
use wuim_core::webview2;
use wuim_probe::TargetIdentity;

use crate::logging;
use crate::state;
use crate::tray::{self, TrayView};
use crate::view::{DeviceView, Identity, SettingsView};

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
}

const USBIPD_RELEASES: &str = "https://github.com/dorssel/usbipd-win/releases/latest";

#[tauri::command]
pub fn open_target(target: OpenTarget) -> Result<(), String> {
    let result = match target {
        OpenTarget::LogFolder => open_parent_of(&logging::path()),
        OpenTarget::SettingsFolder => open_parent_of(&state::path()),
        OpenTarget::UsbipdReleases => shell_open::url(USBIPD_RELEASES),
        OpenTarget::Webview2Download => shell_open::url(webview2::DOWNLOAD_URL),
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

    snapshot
        .devices
        .iter()
        .map(|row| {
            let identity = state::identity(&row.instance_id.raw)
                .as_ref()
                .map(Identity::from);
            DeviceView::from_row(row, ids, identity, &rules)
        })
        .collect()
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
    }
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
}

/// Saves the settings the user changed.
#[tauri::command]
pub fn write_settings(settings: SettingsView) -> Result<(), String> {
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
