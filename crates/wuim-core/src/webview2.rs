//! Whether the WebView2 runtime is installed.
//!
//! The window is a WebView2 control, so without the runtime there is nothing to
//! draw the interface in. Checked before the window is created, because after
//! that the failure is a window that never appears — with nothing on screen to
//! say why (requirement R11.2).
//!
//! Windows 11 ships the runtime, and Microsoft describes it as present on the
//! large majority of Windows 10 machines. What is left is Windows Server, LTSC
//! and images cut off from Windows Update: rare, and worth one registry read to
//! turn into a sentence rather than a silent failure.

use windows::Win32::System::Registry::{HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};

use crate::registry;

/// The runtime's own product code under `EdgeUpdate\Clients`.
const CLIENT: &str = "{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";

pub const DOWNLOAD_URL: &str = "https://developer.microsoft.com/microsoft-edge/webview2/";

/// Whether a usable runtime is registered.
///
/// A per-machine install and a per-user install both count, and EdgeUpdate
/// writes the version under a different root for each. The version itself is
/// not compared against a minimum: `0.0.0.0` is EdgeUpdate's way of saying the
/// entry exists but nothing is installed, and anything else is a real runtime.
pub fn is_installed() -> bool {
    locations()
        .into_iter()
        .filter_map(|(root, key)| registry::read_string(root, &key, "pv").ok().flatten())
        .any(|version| !version.trim().is_empty() && version.trim() != "0.0.0.0")
}

fn locations() -> Vec<(HKEY, String)> {
    vec![
        // Where a 64-bit Windows records a per-machine install.
        (
            HKEY_LOCAL_MACHINE,
            format!(r"SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{CLIENT}"),
        ),
        (
            HKEY_LOCAL_MACHINE,
            format!(r"SOFTWARE\Microsoft\EdgeUpdate\Clients\{CLIENT}"),
        ),
        (
            HKEY_CURRENT_USER,
            format!(r"Software\Microsoft\EdgeUpdate\Clients\{CLIENT}"),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    /// This test runs in a WebView2-capable environment by construction: the
    /// application it belongs to could not have been developed without one.
    #[test]
    fn finds_the_runtime_on_a_machine_that_has_it() {
        assert!(is_installed());
    }

    #[test]
    fn every_location_is_a_clients_key_for_the_runtime() {
        for (_, key) in locations() {
            assert!(key.ends_with(CLIENT), "{key}");
            assert!(key.contains(r"EdgeUpdate\Clients"), "{key}");
        }
    }
}
