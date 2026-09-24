// Release builds start without a console window; debug builds keep one so
// panics and logs are visible while developing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use tauri::Emitter;
use tauri::Manager;
use wuim_core::{autostart, shell_open, single_instance, webview2};

mod commands;
mod logging;
mod preflight;
mod state;
mod tray;
mod view;

fn main() {
    logging::init();

    let started_hidden = std::env::args().any(|arg| arg == autostart::HIDDEN_ARG);

    // Before anything builds a window: the window is a WebView2 control, so
    // without the runtime there is nothing to draw the interface in and nothing
    // on screen to say why (requirement R11.2). This is the one check that has
    // to stop startup rather than disable a feature (R13.2).
    if !webview2::is_installed() {
        logging::error("the WebView2 runtime is not installed; cannot open a window");
        if preflight::ask_to_install_webview2() {
            let _ = shell_open::url(webview2::DOWNLOAD_URL);
        }
        return;
    }

    // The title comes from the same config the window is built from, so the
    // lookup below cannot drift away from what it is looking for.
    let context = tauri::generate_context!();
    let title = context
        .config()
        .app
        .windows
        .first()
        .map(|window| window.title.clone())
        .unwrap_or_default();

    // One copy per sign-in session. A second one polls usbipd alongside the
    // first and acts on the same automatic rules, so two attaches race for one
    // device and one of them fails for no reason the user can see.
    let _single = match single_instance::acquire("wsl-usb-identity-manager") {
        Ok(single_instance::Instance::Only(lock)) => Some(lock),
        Ok(single_instance::Instance::Second) => {
            logging::info("already running; bringing the existing window to the front");
            if !single_instance::focus(&title) {
                logging::error("could not find the running window to raise");
            }
            return;
        }
        // Worth logging, not worth refusing to start over: the worst case is
        // the very state this guard exists to avoid, and the user asked for a
        // window.
        Err(e) => {
            logging::error(&format!("{e:#}; starting anyway"));
            None
        }
    };

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::probe_device,
            commands::run_operation,
            commands::log_message,
            commands::read_settings,
            commands::write_settings,
            commands::check_usbipd,
            commands::open_target,
            commands::clear_remembered,
            commands::read_topology,
            commands::switch_port,
            commands::switch_hub,
            commands::write_vhfilter_setup,
            commands::set_tray,
            commands::hide_window,
            commands::show_window
        ])
        .setup(move |app| {
            // Not fatal: without a tray the window still works, and refusing to
            // start would be a worse answer than starting without an icon.
            // Relaunching brings a hidden window back either way (R10.19).
            if let Err(e) = tray::create(app.handle()) {
                logging::error(&format!("could not create the tray icon: {e:#}"));
            }

            // Reinstalling removes the startup entry (see
            // `autostart::restore_if_missing`), so the setting is re-applied
            // here. Not from a debug build: that would point the entry at a
            // copy under `target`.
            let wanted = state::with(|store| store.settings.start_with_windows);
            if wanted && !cfg!(debug_assertions) {
                match autostart::restore_if_missing() {
                    Ok(true) => logging::info("startup entry was missing; restored"),
                    Ok(false) => {}
                    Err(e) => {
                        logging::error(&format!("could not restore the startup entry: {e:#}"))
                    }
                }
            }

            // Sized before it is shown, so a restored size does not arrive as a
            // visible jump. The position is not restored — see
            // `Settings::window_width` for why.
            let size =
                state::with(|store| (store.settings.window_width, store.settings.window_height));
            if let (Some(width), Some(height)) = size
                && let Some(window) = app.get_webview_window("main")
                && let Err(e) =
                    window.set_size(tauri::PhysicalSize::new(width.max(320), height.max(240)))
            {
                logging::error(&format!("could not restore the window size: {e}"));
            }
            // Started by the `Run` entry, so the window stays in the tray:
            // signing in is not a request to be interrupted. The window itself
            // is created hidden (tauri.conf.json), so nothing flashes either
            // way.
            if started_hidden {
                logging::info("started hidden; the window is in the tray");
            } else {
                tray::show(app.handle());
            }
            Ok(())
        })
        // Closing the window does not stop the application: automatic attach
        // and automatic identification only run while it is alive (R10.14).
        // The frontend decides what to say about that, so it is asked rather
        // than the window simply being hidden here.
        .on_window_event(|window, event| {
            if let tauri::WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(e) = window.emit("close-requested", ()) {
                    logging::error(&format!("could not ask the window to hide: {e}"));
                    let _ = window.hide();
                }
            }
        })
        .run(context)
        .expect("failed to start the application");
}
