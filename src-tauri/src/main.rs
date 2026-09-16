// Release builds start without a console window; debug builds keep one so
// panics and logs are visible while developing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use wuim_core::single_instance;

mod commands;
mod logging;
mod state;
mod view;

fn main() {
    logging::init();

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
            commands::write_settings
        ])
        .run(context)
        .expect("failed to start the application");
}
