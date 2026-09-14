// Release builds start without a console window; debug builds keep one so
// panics and logs are visible while developing.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commands;
mod logging;
mod state;
mod view;

fn main() {
    logging::init();

    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            commands::list_devices,
            commands::probe_device,
            commands::run_operation,
            commands::log_message,
            commands::read_settings,
            commands::write_settings
        ])
        .run(tauri::generate_context!())
        .expect("failed to start the application");
}
