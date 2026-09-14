//! A log file, so a misbehaving run can be read rather than photographed.
//!
//! Everything that touches a device or shells out to usbipd is recorded here
//! with the exact command line and what came back. The frontend writes to the
//! same file through [`crate::commands::log_message`], so one file carries the
//! whole story rather than half of it living in a devtools console nobody has
//! open.

use std::fs::{File, OpenOptions, create_dir_all};
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;

use jiff::Zoned;

/// Opened once and held, so a burst of writes does not reopen the file each time.
static FILE: Mutex<Option<File>> = Mutex::new(None);

/// Where the log lives.
///
/// Beside the executable when `portable.txt` is next to it, matching where the
/// settings will go (requirement R7.2); under `%LOCALAPPDATA%` otherwise.
pub fn path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && dir.join("portable.txt").is_file()
    {
        return dir.join("wuim.log");
    }
    let base = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("wsl-usb-identity-manager").join("wuim.log")
}

/// Opens the log and records that the application started.
pub fn init() {
    let path = path();
    if let Some(dir) = path.parent() {
        let _ = create_dir_all(dir);
    }
    match OpenOptions::new().create(true).append(true).open(&path) {
        Ok(file) => {
            *FILE.lock().unwrap() = Some(file);
            write("info", &format!("started, logging to {}", path.display()));
        }
        Err(e) => {
            // Nothing else to do: the application is still usable without a log.
            eprintln!("could not open {}: {e}", path.display());
        }
    }
    // Also on stdout, so `tauri dev` shows where to look.
    println!("log file: {}", path.display());
}

/// Appends one line. Never panics and never fails the caller: a log that takes
/// the application down with it is worse than no log.
pub fn write(level: &str, message: &str) {
    let line = format!(
        "{} {:<5} {message}\n",
        Zoned::now().strftime("%F %T%.3f"),
        level
    );
    let Ok(mut guard) = FILE.lock() else {
        return;
    };
    if let Some(file) = guard.as_mut() {
        let _ = file.write_all(line.as_bytes());
        let _ = file.flush();
    }
}

pub fn info(message: &str) {
    write("info", message);
}

pub fn error(message: &str) {
    write("error", message);
}
