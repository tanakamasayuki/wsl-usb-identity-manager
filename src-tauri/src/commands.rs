//! The commands the frontend can call.
//!
//! Everything privileged lives behind one of these rather than behind a Tauri
//! plugin, so the capability file stays at the core permission set and the list
//! of things the UI can reach is exactly this file.

use wuim_core::snapshot::{DeviceRow, Snapshot};
use wuim_probe::TargetIdentity;

use crate::view::DeviceView;

/// anyhow's chain, flattened for the frontend. Tauri needs a `Serialize` error
/// and `{:#}` keeps the causes that make a failure diagnosable.
fn to_message(e: anyhow::Error) -> String {
    format!("{e:#}")
}

/// Reads the current state. Probes nothing (requirement R4.6), so this is safe
/// to call on a timer or on every device-change event.
#[tauri::command]
pub fn list_devices() -> Result<Vec<DeviceView>, String> {
    let snapshot = Snapshot::capture().map_err(to_message)?;
    Ok(to_views(&snapshot))
}

fn to_views(snapshot: &Snapshot) -> Vec<DeviceView> {
    snapshot.devices.iter().map(DeviceView::from_row).collect()
}

/// Asks a board what it is.
///
/// **Has side effects.** This is trigger 1 of requirement R4.5 — an explicit
/// request — so it must only ever be reached from a user action that has
/// already shown the side effects the `probes` field describes.
#[tauri::command]
pub fn probe_device(instance_id: String, family: String) -> Result<TargetIdentity, String> {
    // Re-read rather than trusting a snapshot the frontend may have been
    // holding: the device could have moved, been unplugged, or been attached
    // since the list was drawn.
    let snapshot = Snapshot::capture().map_err(to_message)?;
    let row = find(&snapshot, &instance_id)
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

    probe.probe(device).map_err(to_message)
}

fn find<'a>(snapshot: &'a Snapshot, instance_id: &str) -> Option<&'a DeviceRow> {
    snapshot
        .devices
        .iter()
        .find(|row| row.instance_id.matches(instance_id))
}
