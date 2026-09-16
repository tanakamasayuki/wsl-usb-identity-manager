//! Starting with Windows.
//!
//! A value under the per-user `Run` key, which needs no administrator rights
//! and affects only the account that set it. Nothing here touches the
//! machine-wide key: this application has no business starting for other users.

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_WRITE, REG_SZ, RegCloseKey, RegDeleteValueW, RegOpenKeyExW,
    RegSetValueExW,
};
use windows::core::PCWSTR;

use crate::registry;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// The value name. Stable, because changing it would strand the old entry.
const VALUE_NAME: &str = "WSL USB Identity Manager";

/// The argument that tells a copy started this way to stay in the tray.
///
/// Public because the application checks its own arguments for it.
pub const HIDDEN_ARG: &str = "--hidden";

/// Whether this executable is registered to start with Windows.
///
/// Compares the registered command against the running executable, so an entry
/// left behind by a copy that has since moved reads as "off" — which is what it
/// effectively is.
///
/// An entry without [`HIDDEN_ARG`] counts too. Earlier versions wrote one, and
/// reading it as "off" would leave the user with a switch that says off while
/// the application still starts itself.
pub fn is_enabled() -> bool {
    let Ok(quoted) = quoted_exe() else {
        return false;
    };
    let Ok(Some(found)) = read_value() else {
        return false;
    };
    let found = found.trim();
    found.eq_ignore_ascii_case(&quoted)
        || found.eq_ignore_ascii_case(&format!("{quoted} {HIDDEN_ARG}"))
}

/// Registers or removes the entry.
pub fn set(enabled: bool) -> Result<()> {
    if enabled {
        write_value(&command_line()?)
    } else {
        delete_value()
    }
}

/// The command Windows will run.
///
/// Quoted, because the path contains spaces once the application is installed
/// under a user profile, and with [`HIDDEN_ARG`] so that signing in does not
/// open a window: a copy started for the sake of its automatic rules has no
/// reason to interrupt what the user is doing.
fn command_line() -> Result<String> {
    Ok(format!("{} {HIDDEN_ARG}", quoted_exe()?))
}

fn quoted_exe() -> Result<String> {
    let exe: PathBuf = std::env::current_exe().context("could not find this executable")?;
    Ok(format!("\"{}\"", exe.display()))
}

fn open(access: u32) -> Result<HKEY> {
    let key = wide(RUN_KEY);
    let mut handle = HKEY::default();
    unsafe {
        RegOpenKeyExW(
            HKEY_CURRENT_USER,
            PCWSTR(key.as_ptr()),
            None,
            windows::Win32::System::Registry::REG_SAM_FLAGS(access),
            &mut handle,
        )
    }
    .ok()
    .map_err(|e| anyhow!("could not open HKCU\\{RUN_KEY}: {e}"))?;
    Ok(handle)
}

fn read_value() -> Result<Option<String>> {
    registry::read_string(HKEY_CURRENT_USER, RUN_KEY, VALUE_NAME)
}

fn write_value(command: &str) -> Result<()> {
    let handle = open(KEY_WRITE.0)?;
    let name = wide(VALUE_NAME);
    let value = wide(command);
    // The byte length includes the terminating NUL, as REG_SZ requires.
    let bytes = unsafe {
        std::slice::from_raw_parts(
            value.as_ptr() as *const u8,
            std::mem::size_of_val(&value[..]),
        )
    };
    let result =
        unsafe { RegSetValueExW(handle, PCWSTR(name.as_ptr()), None, REG_SZ, Some(bytes)) };
    let _ = unsafe { RegCloseKey(handle) };
    result
        .ok()
        .map_err(|e| anyhow!("could not write the startup entry: {e}"))
}

fn delete_value() -> Result<()> {
    let handle = open(KEY_WRITE.0)?;
    let name = wide(VALUE_NAME);
    let result = unsafe { RegDeleteValueW(handle, PCWSTR(name.as_ptr())) };
    let _ = unsafe { RegCloseKey(handle) };
    // Already absent is the state that was asked for, not a failure.
    if result == ERROR_FILE_NOT_FOUND {
        return Ok(());
    }
    result
        .ok()
        .map_err(|e| anyhow!("could not remove the startup entry: {e}"))
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
