//! The settings and identity file.
//!
//! JSON, because a person has to be able to open it and see what the
//! application thinks (requirement R7.3), and because a portable install should
//! be able to carry it around.
//!
//! Nothing that names a *position* is stored as an identity: no bus id, no
//! Linux device node, and not usbipd's `PersistedGuid` — for a device with no
//! serial number that GUID is tied to the port, so persisting it would quietly
//! reintroduce the problem this application exists to solve (requirement R7.1).
//! The port is kept, in [`Hints`], but only as a hint.

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
    #[serde(default)]
    pub devices: Vec<StoredDevice>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            settings: Settings::default(),
            devices: Vec::new(),
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
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // On, gated by the exclusion list rather than by an allow list.
            auto_identify: true,
            auto_exclude: Vec::new(),
        }
    }
}

/// What was learned about one physical device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StoredDevice {
    /// The identity key from §4.5, and the key everything else hangs off.
    pub identity_key: String,
    pub device_type: String,
    pub device_id: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub hardware_revision: Option<String>,
    /// The transport's serial number, when it has one. Route 1 of §4.1.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub usb_serial: Option<String>,
    pub vid: u16,
    pub pid: u16,
    #[serde(default)]
    pub hints: Hints,
}

/// Where the device was last seen. Updated as it moves; never an identity.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct Hints {
    /// `DEVPKEY_Device_LocationPaths`. Route 2 of §4.1 matches on this.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_location_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_instance_id: Option<String>,
    /// For display only (R7.1).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_com_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_seen_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probe_result_at: Option<String>,
}

impl Store {
    /// Inserts or replaces the record for an identity key.
    pub fn remember(&mut self, device: StoredDevice) {
        match self
            .devices
            .iter_mut()
            .find(|d| d.identity_key == device.identity_key)
        {
            Some(existing) => *existing = device,
            None => self.devices.push(device),
        }
    }

    pub fn find(&self, identity_key: &str) -> Option<&StoredDevice> {
        self.devices.iter().find(|d| d.identity_key == identity_key)
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

/// Now, as an RFC 3339 timestamp for the `last_seen_at` style fields.
pub fn timestamp() -> String {
    Zoned::now().strftime("%FT%T%:z").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn device(key: &str) -> StoredDevice {
        StoredDevice {
            identity_key: key.to_owned(),
            device_type: "esp32-s3".into(),
            device_id: "34:85:18:8f:6d:7c".into(),
            hardware_revision: Some("v0.1".into()),
            usb_serial: None,
            vid: 0x1a86,
            pid: 0x7523,
            hints: Hints {
                last_location_path: Some("PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)".into()),
                ..Hints::default()
            },
        }
    }

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
        store.remember(device("esp32-s3-3485188f6d7c"));
        save(&path, &store).unwrap();

        let Loaded::Ok(read) = load(&path) else {
            panic!("did not read back");
        };
        assert_eq!(read.schema_version, SCHEMA_VERSION);
        assert_eq!(read.settings.auto_exclude, vec!["1a86:7523".to_string()]);
        assert_eq!(read.devices, store.devices);
    }

    #[test]
    fn remembering_the_same_key_replaces_rather_than_duplicates() {
        let mut store = Store::default();
        store.remember(device("esp32-s3-3485188f6d7c"));
        let mut moved = device("esp32-s3-3485188f6d7c");
        moved.hints.last_com_port = Some("COM9".into());
        store.remember(moved);

        assert_eq!(store.devices.len(), 1);
        assert_eq!(
            store
                .find("esp32-s3-3485188f6d7c")
                .unwrap()
                .hints
                .last_com_port,
            Some("COM9".into())
        );
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
    fn settings_default_to_automatic_identification_with_nothing_excluded() {
        let settings = Settings::default();
        assert!(settings.auto_identify);
        assert!(settings.auto_exclude.is_empty());
    }

    #[test]
    fn an_older_file_without_the_newer_fields_still_reads() {
        let path = temp("minimal.json");
        fs::write(&path, r#"{"schema_version": 1}"#).unwrap();
        let Loaded::Ok(store) = load(&path) else {
            panic!("a minimal file should read");
        };
        assert!(store.devices.is_empty());
        assert!(store.settings.auto_identify);
    }

    #[test]
    fn saving_leaves_no_temporary_file_behind() {
        let path = temp("atomic.json");
        save(&path, &Store::default()).unwrap();
        assert!(!path.with_extension("json.tmp").exists());
    }
}
