//! The commands the frontend can call.
//!
//! Everything privileged lives behind one of these rather than behind a Tauri
//! plugin, so the capability file stays at the core permission set and the list
//! of things the UI can reach is exactly this file.

use std::sync::OnceLock;
use std::time::Instant;

use wuim_core::UsbIds;
use wuim_core::recall;
use wuim_core::snapshot::{DeviceRow, Snapshot};
use wuim_core::store::{Hints, Settings, StoredDevice};
use wuim_core::usbipd::{self, Operation};
use wuim_probe::TargetIdentity;

use crate::logging;
use crate::state;
use crate::view::{DeviceView, Identity};

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
    // Recalling costs one pass over the stored file and saves a probe — which
    // is a board reset — for every device it recognises.
    let recalled = state::with(|store| {
        recall::recall(store, &snapshot.devices)
            .iter()
            .map(|(instance_id, hit)| (instance_id.clone(), Identity::from_recalled(hit)))
            .collect::<std::collections::HashMap<_, _>>()
    });

    snapshot
        .devices
        .iter()
        .map(|row| {
            let identity = recalled.get(&row.instance_id.raw).cloned();
            DeviceView::from_row(row, ids, identity)
        })
        .collect()
}

/// Hands the stored settings to the frontend at startup.
#[tauri::command]
pub fn read_settings() -> StoredSettings {
    StoredSettings {
        settings: state::with(|store| store.settings.clone()),
        writable: state::is_writable(),
        path: state::path().display().to_string(),
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct StoredSettings {
    pub settings: Settings,
    /// False when the file on disk was refused; the panel says so.
    pub writable: bool,
    pub path: String,
}

/// Saves the settings the user changed.
#[tauri::command]
pub fn write_settings(settings: Settings) {
    logging::info(&format!(
        "settings: auto_identify={} exclude={:?}",
        settings.auto_identify, settings.auto_exclude
    ));
    state::update(|store| store.settings = settings);
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

    // Remembered so a restart does not mean probing — and resetting — every
    // board again. The port is kept as a hint so a device with no serial can be
    // recognised where it sits (§4.1 route 2), never as an identity (R7.1).
    let now = wuim_core::store::timestamp();
    state::update(|store| {
        store.remember(StoredDevice {
            identity_key: identity.identity_key.clone(),
            device_type: identity.device_type.clone(),
            device_id: identity.device_id.clone(),
            hardware_revision: identity.hardware_revision.clone(),
            usb_serial: row.instance_id.unit.serial().map(str::to_owned),
            vid: row.instance_id.vid.unwrap_or(0),
            pid: row.instance_id.pid.unwrap_or(0),
            hints: Hints {
                last_location_path: row.location_path().map(str::to_owned),
                last_instance_id: Some(row.instance_id.raw.clone()),
                last_com_port: row.com_port().map(str::to_owned),
                last_seen_at: Some(now.clone()),
                probe_result_at: Some(now.clone()),
            },
        });
    });

    Ok(identity)
}

fn find<'a>(snapshot: &'a Snapshot, instance_id: &str) -> Option<&'a DeviceRow> {
    snapshot
        .devices
        .iter()
        .find(|row| row.instance_id.matches(instance_id))
}
