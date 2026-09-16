//! One copy of the application per sign-in session.
//!
//! Two copies are worse than redundant. Both poll `usbipd`, both act on the
//! automatic rules, and two automatic attaches racing for the same device
//! produce a failure the user never asked for. The second copy therefore hands
//! the window to the first and leaves.

use anyhow::{Result, anyhow};
use windows::Win32::Foundation::{CloseHandle, ERROR_ALREADY_EXISTS, GetLastError, HANDLE};
use windows::Win32::System::Threading::CreateMutexW;
use windows::Win32::UI::WindowsAndMessaging::{
    FindWindowW, IsIconic, SW_RESTORE, SetForegroundWindow, ShowWindow,
};
use windows::core::PCWSTR;

/// Holds the name for as long as it is alive.
///
/// Dropping it lets another copy start, so the caller has to keep it for the
/// life of the process.
#[derive(Debug)]
pub struct Lock(HANDLE);

impl Drop for Lock {
    fn drop(&mut self) {
        // Windows would release the handle at exit anyway; closing it here
        // keeps the lifetime visible in the code rather than implied by it.
        let _ = unsafe { CloseHandle(self.0) };
    }
}

#[derive(Debug)]
pub enum Instance {
    /// Nothing else holds the name.
    Only(Lock),
    /// Another copy is already running.
    Second,
}

/// Claims the name for this process.
///
/// The name is scoped to the sign-in session, which is the right scope: the
/// settings file, the `Run` entry and the window all belong to one user, so a
/// second user signing in gets their own copy rather than being told the
/// application is already running.
pub fn acquire(name: &str) -> Result<Instance> {
    let name = wide(&format!("Local\\{name}"));
    // Ownership is not taken: nothing here waits on the mutex, and an owned
    // mutex left behind by a crash is reported as abandoned to whoever waits
    // next. Only the name matters.
    let handle = unsafe { CreateMutexW(None, false, PCWSTR(name.as_ptr())) }
        .map_err(|e| anyhow!("could not take the single-instance lock: {e}"))?;

    // The call succeeds either way. What separates the two cases is the last
    // error, so it is read before anything else can overwrite it.
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        let _ = unsafe { CloseHandle(handle) };
        return Ok(Instance::Second);
    }
    Ok(Instance::Only(Lock(handle)))
}

/// Brings the running copy's window to the front.
///
/// Best effort, and reported as such: Windows refuses `SetForegroundWindow`
/// from a process that has not just been given the foreground, and failing to
/// raise a window is not a reason to start a second copy.
pub fn focus(window_title: &str) -> bool {
    let title = wide(window_title);
    let Ok(window) = (unsafe { FindWindowW(PCWSTR::null(), PCWSTR(title.as_ptr())) }) else {
        return false;
    };
    if window.is_invalid() {
        return false;
    }
    // A minimised window accepts the foreground without coming back into view,
    // which looks exactly like nothing happening.
    if unsafe { IsIconic(window) }.as_bool() {
        let _ = unsafe { ShowWindow(window, SW_RESTORE) };
    }
    unsafe { SetForegroundWindow(window) }.as_bool()
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_second_claim_on_a_name_is_refused() {
        let name = format!("wuim-test-{}", std::process::id());
        let first = acquire(&name).unwrap();
        assert!(matches!(first, Instance::Only(_)));

        // Same process or another, the name is what decides.
        assert!(matches!(acquire(&name).unwrap(), Instance::Second));

        drop(first);
        assert!(
            matches!(acquire(&name).unwrap(), Instance::Only(_)),
            "the name should be free once the lock is dropped"
        );
    }

    #[test]
    fn focusing_a_window_that_is_not_there_says_so() {
        assert!(!focus("no window has this title, 7f3a1c"));
    }
}
