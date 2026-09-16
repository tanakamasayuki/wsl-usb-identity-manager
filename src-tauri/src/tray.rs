//! The tray icon.
//!
//! The application keeps working after its window is closed: automatic attach
//! hands devices to WSL, and automatic identification runs when one is plugged
//! in. Neither happens if the process exits with the window, so closing hides
//! it here and quitting is a deliberate choice from this menu.
//!
//! Every label comes from the frontend through [`apply`]. The translations live
//! there (requirement R10.5), and a second catalogue in Rust would be a second
//! place for them to drift.

use std::sync::{Mutex, OnceLock};

use anyhow::{Result, anyhow};
use serde::Deserialize;
use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{TrayIcon, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Wry};

use crate::logging;
use crate::state;

/// What the frontend puts in the menu, already translated.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayView {
    /// Hover text: what the application is, and what it currently sees.
    pub tooltip: String,
    /// One line of counts, shown as a disabled item at the top of the menu.
    pub status: String,
    pub auto_attach_label: String,
    pub auto_attach_on: bool,
    /// Carries the count, so the menu says how many boards would restart.
    pub identify_all_label: String,
    /// False when there is nothing left to identify.
    pub identify_all_enabled: bool,
    pub open_label: String,
    pub quit_label: String,
}

struct Tray {
    icon: TrayIcon<Wry>,
    status: MenuItem<Wry>,
    auto_attach: CheckMenuItem<Wry>,
    identify_all: MenuItem<Wry>,
    open: MenuItem<Wry>,
    quit: MenuItem<Wry>,
}

static TRAY: OnceLock<Mutex<Tray>> = OnceLock::new();

/// Builds the tray icon. Called once, during setup.
///
/// The labels here are placeholders in English; the frontend replaces them with
/// translated ones as soon as it has read its catalogue.
pub fn create(app: &AppHandle) -> Result<()> {
    let status = MenuItem::with_id(app, "status", "…", false, None::<&str>)?;
    let auto_attach =
        CheckMenuItem::with_id(app, "auto_attach", "Auto-attach", true, false, None::<&str>)?;
    let identify_all = MenuItem::with_id(app, "identify_all", "Identify all", false, None::<&str>)?;
    let open = MenuItem::with_id(app, "open", "Open", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(
        app,
        &[
            &status,
            &PredefinedMenuItem::separator(app)?,
            &auto_attach,
            &identify_all,
            &PredefinedMenuItem::separator(app)?,
            &open,
            &quit,
        ],
    )?;

    let mut builder = TrayIconBuilder::with_id("main")
        .menu(&menu)
        .tooltip("WSL USB Identity Manager")
        // The left click opens the window; the menu is on the right, which is
        // what Windows users expect of a tray icon.
        .show_menu_on_left_click(false)
        .on_menu_event(on_menu_event)
        .on_tray_icon_event(on_icon_event);
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }

    let icon = builder.build(app)?;
    TRAY.set(Mutex::new(Tray {
        icon,
        status,
        auto_attach,
        identify_all,
        open,
        quit,
    }))
    .map_err(|_| anyhow!("the tray icon was built twice"))?;
    Ok(())
}

/// Puts the frontend's labels and counts into the menu.
pub fn apply(view: TrayView) -> Result<()> {
    let Some(tray) = TRAY.get() else {
        return Err(anyhow!("the tray icon does not exist yet"));
    };
    let tray = tray.lock().map_err(|e| anyhow!("{e}"))?;
    tray.icon.set_tooltip(Some(&view.tooltip))?;
    tray.status.set_text(&view.status)?;
    tray.auto_attach.set_text(&view.auto_attach_label)?;
    tray.auto_attach.set_checked(view.auto_attach_on)?;
    tray.identify_all.set_text(&view.identify_all_label)?;
    tray.identify_all.set_enabled(view.identify_all_enabled)?;
    tray.open.set_text(&view.open_label)?;
    tray.quit.set_text(&view.quit_label)?;
    Ok(())
}

fn on_menu_event(app: &AppHandle, event: tauri::menu::MenuEvent) {
    match event.id().as_ref() {
        "open" => show(app),
        "quit" => {
            logging::info("quit from the tray");
            app.exit(0);
        }
        "auto_attach" => toggle_auto_attach(app),
        // Identifying restarts boards, and what it restarts depends on what is
        // connected right now — which the window is the one tracking. It runs
        // there, under the same confirmation setting as the button (R4.7).
        "identify_all" => {
            if let Err(e) = app.emit("identify-all", ()) {
                logging::error(&format!("could not ask the window to identify: {e}"));
            }
        }
        // The status line is disabled and cannot be clicked.
        other => logging::error(&format!("unknown tray menu item {other}")),
    }
}

fn on_icon_event(icon: &TrayIcon<Wry>, event: TrayIconEvent) {
    // Left button up, rather than down: a click that opens on the way down
    // fires while the user is still deciding.
    if let TrayIconEvent::Click {
        button: tauri::tray::MouseButton::Left,
        button_state: tauri::tray::MouseButtonState::Up,
        ..
    } = event
    {
        show(icon.app_handle());
    }
}

/// Brings the window back, from the tray or from a second launch.
pub fn show(app: &AppHandle) {
    let Some(window) = app.get_webview_window("main") else {
        logging::error("there is no main window to show");
        return;
    };
    let _ = window.show();
    let _ = window.unminimize();
    let _ = window.set_focus();
}

/// Flips automatic attach from the menu.
///
/// Written straight to the store rather than asked of the frontend, so the
/// switch works with the window closed. The frontend is then told to re-read,
/// because it is what acts on the rules.
fn toggle_auto_attach(app: &AppHandle) {
    let next = !state::with(|store| store.settings.auto_attach);
    state::update(|store| store.settings.auto_attach = next);
    logging::info(&format!("auto attach {} from the tray", on_off(next)));

    if let Some(tray) = TRAY.get()
        && let Ok(tray) = tray.lock()
    {
        let _ = tray.auto_attach.set_checked(next);
    }
    // The frontend holds its own copy and runs the rules against it.
    if let Err(e) = app.emit("settings-changed", ()) {
        logging::error(&format!("could not tell the window about the change: {e}"));
    }
}

fn on_off(value: bool) -> &'static str {
    if value { "on" } else { "off" }
}
