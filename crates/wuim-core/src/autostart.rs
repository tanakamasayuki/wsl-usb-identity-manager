//! Starting with Windows.
//!
//! A value under the per-user `Run` key, which needs no administrator rights
//! and affects only the account that set it. Nothing here touches the
//! machine-wide key: this application has no business starting for other users.

use std::path::PathBuf;

use anyhow::{Context, Result, anyhow};
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{
    HKEY, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_SZ, RegCloseKey, RegDeleteValueW,
    RegOpenKeyExW, RegQueryValueExW, RegSetValueExW,
};
use windows::core::PCWSTR;

const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

/// The value name. Stable, because changing it would strand the old entry.
const VALUE_NAME: &str = "WSL USB Identity Manager";

/// Whether this executable is registered to start with Windows.
///
/// Compares the registered command against the running executable, so an entry
/// left behind by a copy that has since moved reads as "off" — which is what it
/// effectively is.
pub fn is_enabled() -> bool {
    let Ok(expected) = command_line() else {
        return false;
    };
    matches!(read_value(), Ok(Some(found)) if found.eq_ignore_ascii_case(&expected))
}

/// Registers or removes the entry.
pub fn set(enabled: bool) -> Result<()> {
    if enabled {
        write_value(&command_line()?)
    } else {
        delete_value()
    }
}

/// The command Windows will run, quoted because the path contains spaces once
/// the application is installed under Program Files or a user profile.
fn command_line() -> Result<String> {
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
    let handle = open(KEY_READ.0)?;
    let name = wide(VALUE_NAME);
    let mut size = 0u32;
    let mut kind = REG_SZ;

    let found = unsafe {
        RegQueryValueExW(
            handle,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            None,
            Some(&mut size),
        )
    };
    if found == ERROR_FILE_NOT_FOUND {
        let _ = unsafe { RegCloseKey(handle) };
        return Ok(None);
    }

    let mut buffer = vec![0u8; size as usize];
    let read = unsafe {
        RegQueryValueExW(
            handle,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            Some(buffer.as_mut_ptr()),
            Some(&mut size),
        )
    };
    let _ = unsafe { RegCloseKey(handle) };
    read.ok()
        .map_err(|e| anyhow!("could not read the startup entry: {e}"))?;

    buffer.truncate(size as usize);
    let text: Vec<u16> = buffer
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    Ok(Some(
        String::from_utf16_lossy(&text)
            .trim_end_matches('\0')
            .to_owned(),
    ))
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
