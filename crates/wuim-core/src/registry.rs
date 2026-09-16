//! Reading string values out of the registry.
//!
//! Only reading, and only strings: the two things this application needs from
//! the registry are what it registered under `Run` ([`crate::autostart`]) and
//! whether the WebView2 runtime is installed ([`crate::webview2`]). Writing
//! stays with the module that owns the value.

use anyhow::{Result, anyhow};
use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
use windows::Win32::System::Registry::{
    HKEY, KEY_READ, REG_SZ, RegCloseKey, RegOpenKeyExW, RegQueryValueExW,
};
use windows::core::PCWSTR;

/// Reads one `REG_SZ` value, or `None` when the key or the value is absent.
///
/// A missing key is not an error: "the WebView2 runtime is not installed" and
/// "this application is not registered to start with Windows" are both answers,
/// and both arrive as a key that is not there.
pub fn read_string(root: HKEY, key: &str, value: &str) -> Result<Option<String>> {
    let Some(handle) = open(root, key)? else {
        return Ok(None);
    };
    let result = read_from(handle, value);
    let _ = unsafe { RegCloseKey(handle) };
    result
}

fn open(root: HKEY, key: &str) -> Result<Option<HKEY>> {
    let wide_key = wide(key);
    let mut handle = HKEY::default();
    let opened =
        unsafe { RegOpenKeyExW(root, PCWSTR(wide_key.as_ptr()), None, KEY_READ, &mut handle) };
    if opened == ERROR_FILE_NOT_FOUND {
        return Ok(None);
    }
    opened
        .ok()
        .map_err(|e| anyhow!("could not open the registry key {key}: {e}"))?;
    Ok(Some(handle))
}

fn read_from(handle: HKEY, value: &str) -> Result<Option<String>> {
    let name = wide(value);
    let mut size = 0u32;
    let mut kind = REG_SZ;

    // Asked for the size first, so the buffer is exactly what the value needs.
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
        return Ok(None);
    }
    found
        .ok()
        .map_err(|e| anyhow!("could not measure the registry value {value}: {e}"))?;

    let mut buffer = vec![0u8; size as usize];
    unsafe {
        RegQueryValueExW(
            handle,
            PCWSTR(name.as_ptr()),
            None,
            Some(&mut kind),
            Some(buffer.as_mut_ptr()),
            Some(&mut size),
        )
    }
    .ok()
    .map_err(|e| anyhow!("could not read the registry value {value}: {e}"))?;

    buffer.truncate(size as usize);
    Ok(Some(from_utf16_bytes(&buffer)))
}

/// A `REG_SZ` comes back as bytes; it is UTF-16 with a terminating NUL.
fn from_utf16_bytes(bytes: &[u8]) -> String {
    let units: Vec<u16> = bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|pair| u16::from_le_bytes([pair[0], pair[1]]))
        .collect();
    String::from_utf16_lossy(&units)
        .trim_end_matches('\0')
        .to_owned()
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use windows::Win32::System::Registry::HKEY_LOCAL_MACHINE;

    #[test]
    fn reads_a_value_windows_always_has() {
        let value = read_string(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "ProductName",
        )
        .unwrap();
        assert!(
            value.is_some_and(|name| name.contains("Windows")),
            "the OS product name should be readable"
        );
    }

    #[test]
    fn a_missing_key_is_none_rather_than_an_error() {
        let value = read_string(HKEY_LOCAL_MACHINE, r"SOFTWARE\no such key 7f3a1c", "pv").unwrap();
        assert_eq!(value, None);
    }

    #[test]
    fn a_missing_value_under_a_real_key_is_also_none() {
        let value = read_string(
            HKEY_LOCAL_MACHINE,
            r"SOFTWARE\Microsoft\Windows NT\CurrentVersion",
            "no such value 7f3a1c",
        )
        .unwrap();
        assert_eq!(value, None);
    }
}
