//! The settings file.
//!
//! JSON, because a person has to be able to open it and see what the
//! application thinks (requirement R7.3), and because a portable install should
//! be able to carry it around.
//!
//! **No identities are stored.** An earlier version cached probe results and
//! matched a device back to one by its port, or by the serial number of the
//! adapter in front of it. Neither says anything about the board on the other
//! end of the cable: a USB-serial adapter can be moved to a different board
//! without one byte changing on the USB side, so the match could be confidently
//! wrong, which is worse than knowing nothing. Identities now come from asking
//! the board, and last only while it stays plugged in.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use jiff::Zoned;
use serde::{Deserialize, Serialize};

/// Bumped only alongside a migration. See [`load`].
pub const SCHEMA_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub schema_version: u32,
    #[serde(default)]
    pub settings: Settings,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            settings: Settings::default(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct Settings {
    /// Identify a device as soon as it is plugged in.
    pub auto_identify: bool,
    /// `vid:pid` never probed automatically.
    pub auto_exclude: Vec<String>,
    /// Ask before probing, stating what it does to the board (R4.7).
    ///
    /// Turning this off is a deliberate choice by someone who already knows a
    /// probe restarts the board and does not need telling every time.
    pub confirm_before_identify: bool,
    /// Start with Windows, through the per-user `Run` key.
    ///
    /// The registry is what Windows acts on, so it is the authority; this field
    /// records what the user asked for and is applied to the registry when it
    /// changes. See [`crate::autostart`].
    pub start_with_windows: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // On, gated by the exclusion list rather than by an allow list.
            auto_identify: true,
            auto_exclude: Vec::new(),
            confirm_before_identify: true,
            start_with_windows: false,
        }
    }
}

/// What came back from reading the file.
#[derive(Debug)]
pub enum Loaded {
    /// No file yet. Start empty; writing is safe.
    Fresh,
    /// Read successfully. Writing is safe.
    Ok(Store),
    /// Left alone deliberately. **Do not write over it.**
    Refused { reason: String },
}

/// Reads the file.
///
/// A file from a newer schema is refused rather than parsed (requirement R7.5):
/// a future version will store things this one knows nothing about, and writing
/// over it would throw them away. Refusing is also why [`Loaded`] is an enum
/// instead of an `Option` — the caller cannot ignore the difference between
/// "nothing there yet" and "something there that must not be touched".
pub fn load(path: &Path) -> Loaded {
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Loaded::Fresh,
        Err(e) => {
            return Loaded::Refused {
                reason: format!("could not read {}: {e}", path.display()),
            };
        }
    };

    // The version is read before anything else, so a newer file is never parsed
    // against this version's expectations.
    let version = match serde_json::from_str::<VersionOnly>(&text) {
        Ok(v) => v.schema_version,
        Err(e) => {
            return Loaded::Refused {
                reason: format!("{} is not readable as JSON: {e}", path.display()),
            };
        }
    };

    if version > SCHEMA_VERSION {
        return Loaded::Refused {
            reason: format!(
                "{} was written by a newer version (schema {version}, this build reads {SCHEMA_VERSION})",
                path.display()
            ),
        };
    }

    // Only version 1 exists. When a second one appears, migrate here and move
    // the original aside first, as requirement R7.5 describes.
    match serde_json::from_str::<Store>(&text) {
        Ok(store) => Loaded::Ok(store),
        Err(e) => Loaded::Refused {
            reason: format!("{} could not be parsed: {e}", path.display()),
        },
    }
}

#[derive(Deserialize)]
struct VersionOnly {
    schema_version: u32,
}

/// Writes the file so that an interrupted write cannot destroy the old one
/// (requirement R7.6): the new content lands in a temporary file first and only
/// then replaces the target, which is a single atomic step.
pub fn save(path: &Path, store: &Store) -> Result<()> {
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).with_context(|| format!("could not create {}", dir.display()))?;
    }
    let json = serde_json::to_string_pretty(store).context("could not serialise the store")?;

    let temporary = path.with_extension("json.tmp");
    fs::write(&temporary, json.as_bytes())
        .with_context(|| format!("could not write {}", temporary.display()))?;
    fs::rename(&temporary, path)
        .with_context(|| format!("could not replace {}", path.display()))?;
    Ok(())
}

/// Where the file lives.
///
/// Beside the executable when `portable.txt` sits next to it, so a portable
/// install carries its settings with it; under `%APPDATA%` otherwise
/// (requirement R7.2).
pub fn store_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe()
        && let Some(dir) = exe.parent()
        && dir.join("portable.txt").is_file()
    {
        return dir.join("devices.json");
    }
    let base = std::env::var_os("APPDATA")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    base.join("wsl-usb-identity-manager").join("devices.json")
}

/// Now, as an RFC 3339 timestamp.
pub fn timestamp() -> String {
    Zoned::now().strftime("%FT%T%:z").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join("wuim-store-tests");
        fs::create_dir_all(&dir).unwrap();
        dir.join(name)
    }

    #[test]
    fn a_missing_file_is_fresh_rather_than_an_error() {
        let path = temp("missing.json");
        let _ = fs::remove_file(&path);
        assert!(matches!(load(&path), Loaded::Fresh));
    }

    #[test]
    fn round_trips_through_the_file() {
        let path = temp("round-trip.json");
        let mut store = Store::default();
        store.settings.auto_exclude = vec!["1a86:7523".into()];
        store.settings.confirm_before_identify = false;
        store.settings.start_with_windows = true;
        save(&path, &store).unwrap();

        let Loaded::Ok(read) = load(&path) else {
            panic!("did not read back");
        };
        assert_eq!(read.schema_version, SCHEMA_VERSION);
        assert_eq!(read.settings.auto_exclude, vec!["1a86:7523".to_string()]);
        assert!(!read.settings.confirm_before_identify);
        assert!(read.settings.start_with_windows);
    }

    #[test]
    fn a_newer_schema_is_refused_and_left_alone() {
        let path = temp("newer.json");
        let original = r#"{"schema_version": 99, "somethingNew": true}"#;
        fs::write(&path, original).unwrap();

        let Loaded::Refused { reason } = load(&path) else {
            panic!("a newer file must not be read");
        };
        assert!(reason.contains("newer version"), "{reason}");
        // The caller is told not to write, and nothing here wrote either.
        assert_eq!(fs::read_to_string(&path).unwrap(), original);
    }

    #[test]
    fn damaged_json_is_refused_rather_than_silently_reset() {
        let path = temp("damaged.json");
        fs::write(&path, "{ this is not json").unwrap();
        assert!(matches!(load(&path), Loaded::Refused { .. }));
    }

    #[test]
    fn a_file_from_when_identities_were_cached_still_reads() {
        // Version 1 files written before the cache was removed carry a
        // `devices` array. It is no longer part of the struct, so it is ignored
        // and dropped on the next write rather than blocking the load.
        let path = temp("with-devices.json");
        fs::write(
            &path,
            r#"{"schema_version": 1, "settings": {"auto_identify": false},
                "devices": [{"identity_key": "esp32-s3-abc", "vid": 6790}]}"#,
        )
        .unwrap();

        let Loaded::Ok(store) = load(&path) else {
            panic!("an older version-1 file should still read");
        };
        assert!(!store.settings.auto_identify);
    }

    #[test]
    fn settings_default_to_automatic_identification_with_confirmation() {
        let settings = Settings::default();
        assert!(settings.auto_identify);
        assert!(settings.auto_exclude.is_empty());
        assert!(settings.confirm_before_identify);
        assert!(!settings.start_with_windows);
    }

    #[test]
    fn saving_leaves_no_temporary_file_behind() {
        let path = temp("atomic.json");
        save(&path, &Store::default()).unwrap();
        assert!(!path.with_extension("json.tmp").exists());
    }
}
