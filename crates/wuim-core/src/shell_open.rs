//! Handing a folder or a web page to Windows to open.
//!
//! The log is the first thing to look at when something goes wrong, and a
//! release build has no console to print its path to, so there has to be a way
//! to reach it from the window (requirement R13.8).
//!
//! Neither function takes anything the user typed. Every caller passes a value
//! this application produced — a path it writes to, or a URL compiled into it —
//! because both of these hand a string to the shell to act on.

use std::path::Path;

use anyhow::{Result, anyhow, bail};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows::core::PCWSTR;

/// Opens a folder in Explorer.
///
/// The folder rather than the file: `.log` and `.json` have no association on a
/// clean Windows, where opening the file itself asks the user to choose a
/// program before they can read anything.
pub fn folder(path: &Path) -> Result<()> {
    if !path.is_dir() {
        bail!("{} is not a folder", path.display());
    }
    execute(&path.to_string_lossy())
}

/// Opens a URL in the default browser.
pub fn url(address: &str) -> Result<()> {
    // Nothing here builds a URL from input, and the shell would happily run a
    // `file:` or a local path handed to it as one.
    if !address.starts_with("https://") {
        bail!("refusing to open {address}: only https is opened");
    }
    execute(address)
}

fn execute(target: &str) -> Result<()> {
    let verb = wide("open");
    let file = wide(target);
    let result = unsafe {
        ShellExecuteW(
            None,
            PCWSTR(verb.as_ptr()),
            PCWSTR(file.as_ptr()),
            PCWSTR::null(),
            PCWSTR::null(),
            SW_SHOWNORMAL,
        )
    };
    // ShellExecuteW reports success as an HINSTANCE above 32, and an error code
    // below it. It is the one Win32 function in this codebase that does not use
    // the usual convention.
    if result.0 as isize > 32 {
        Ok(())
    } else {
        Err(anyhow!(
            "could not open {target} (code {})",
            result.0 as isize
        ))
    }
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn refuses_anything_that_is_not_https() {
        for address in [
            "http://example.com",
            "file:///C:/Windows/System32/cmd.exe",
            r"C:\Windows\System32\cmd.exe",
            "javascript:alert(1)",
        ] {
            assert!(url(address).is_err(), "opened {address}");
        }
    }

    #[test]
    fn refuses_a_path_that_is_not_a_folder() {
        assert!(folder(Path::new(r"C:\no such folder 7f3a1c")).is_err());
    }
}
