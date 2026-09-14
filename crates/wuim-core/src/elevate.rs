//! Running one command with administrator rights.
//!
//! The application itself starts unelevated (requirement R5.6). Only the two
//! usbipd operations that genuinely need administrator rights — bind and unbind
//! — are raised, and only at the moment they run, so the UAC prompt the user
//! sees is tied to the action they just asked for rather than to opening the
//! application.

use std::path::Path;

use anyhow::{Result, anyhow, bail};
use windows::Win32::Foundation::{CloseHandle, ERROR_CANCELLED, HANDLE, WAIT_OBJECT_0};
use windows::Win32::System::Threading::{GetExitCodeProcess, INFINITE, WaitForSingleObject};
use windows::Win32::UI::Shell::{SEE_MASK_NOCLOSEPROCESS, SHELLEXECUTEINFOW, ShellExecuteExW};
use windows::Win32::UI::WindowsAndMessaging::SW_HIDE;
use windows::core::PCWSTR;

/// Runs `exe` elevated and waits for it, returning its exit code.
///
/// `arguments` is passed as a single command line, so anything interpolated
/// into it must be checked by the caller first.
pub fn run_elevated(exe: &Path, arguments: &str) -> Result<u32> {
    let verb = wide("runas");
    let file = wide(&exe.to_string_lossy());
    let params = wide(arguments);

    let mut info = SHELLEXECUTEINFOW {
        cbSize: size_of::<SHELLEXECUTEINFOW>() as u32,
        // Without this the process handle is not returned and there is no way
        // to tell whether the command succeeded.
        fMask: SEE_MASK_NOCLOSEPROCESS,
        lpVerb: PCWSTR(verb.as_ptr()),
        lpFile: PCWSTR(file.as_ptr()),
        lpParameters: PCWSTR(params.as_ptr()),
        nShow: SW_HIDE.0,
        ..Default::default()
    };

    unsafe { ShellExecuteExW(&mut info) }.map_err(|e| {
        if e.code().0 as u32 & 0xFFFF == ERROR_CANCELLED.0 {
            anyhow!("the administrator prompt was dismissed")
        } else {
            anyhow!("could not start {} elevated: {e}", exe.display())
        }
    })?;

    let process = info.hProcess;
    if process.is_invalid() {
        bail!("{} started but returned no process handle", exe.display());
    }

    let exit_code = wait_for(process);
    let _ = unsafe { CloseHandle(process) };
    exit_code
}

fn wait_for(process: HANDLE) -> Result<u32> {
    if unsafe { WaitForSingleObject(process, INFINITE) } != WAIT_OBJECT_0 {
        bail!("waiting for the elevated command failed");
    }
    let mut code = 0u32;
    unsafe { GetExitCodeProcess(process, &mut code) }
        .map_err(|e| anyhow!("could not read the exit code of the elevated command: {e}"))?;
    Ok(code)
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
