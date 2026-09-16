//! The one message this application shows without a window.
//!
//! Everything else the user reads is translated in the frontend. When the
//! WebView2 runtime is missing there is no frontend to translate anything, so
//! the two sentences live here instead, picked from the language Windows is
//! set to.

use windows::Win32::Globalization::GetUserDefaultUILanguage;
use windows::Win32::UI::WindowsAndMessaging::{IDOK, MB_ICONERROR, MB_OKCANCEL, MessageBoxW};
use windows::core::PCWSTR;

/// Primary language id for Japanese, the low 10 bits of a LANGID.
const LANG_JAPANESE: u16 = 0x11;

/// Tells the user the runtime is missing, and asks whether to open the
/// download page. True if they said yes.
pub fn ask_to_install_webview2() -> bool {
    let (title, body) = if is_japanese() {
        (
            "WSL USB Identity Manager",
            "このアプリケーションの画面は WebView2 ランタイムを使います。\n\
             WebView2 ランタイムが見つからないため、起動できません。\n\n\
             ダウンロードページを開きますか？",
        )
    } else {
        (
            "WSL USB Identity Manager",
            "This application draws its window with the WebView2 runtime.\n\
             The runtime is not installed, so there is nothing to start.\n\n\
             Open the download page?",
        )
    };

    let title = wide(title);
    let body = wide(body);
    let answer = unsafe {
        MessageBoxW(
            None,
            PCWSTR(body.as_ptr()),
            PCWSTR(title.as_ptr()),
            MB_OKCANCEL | MB_ICONERROR,
        )
    };
    answer == IDOK
}

fn is_japanese() -> bool {
    // The high bits are the sub-language (regional variant), which does not
    // change which of the two messages to show.
    let language = unsafe { GetUserDefaultUILanguage() };
    (language & 0x3ff) == LANG_JAPANESE
}

fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(std::iter::once(0)).collect()
}
