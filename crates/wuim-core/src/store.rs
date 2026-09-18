//! The settings file.
//!
//! JSON, because a person has to be able to open it and see what the
//! application thinks (requirement R7.3), and because a portable install should
//! be able to carry it around.
//!
//! **No identity is stored as a fact.** A probe result cannot be matched back to
//! a device by its port, or by the serial number of the adapter in front of it:
//! neither says anything about the board on the other end of the cable, which
//! can be swapped without one byte changing on the USB side. A match made that
//! way is confidently wrong, which is worse than knowing nothing. A confirmed
//! identity therefore comes from asking the board, and lasts only while it
//! stays plugged in.
//!
//! What is stored is [`LastSeen`]: the name and identification a device last
//! had, as a reminder for the person reading the list. It survives a restart
//! because the alternative is worse — a device already attached to WSL cannot be
//! probed at all (finding F4), so without this a machine that starts with its
//! boards already forwarded can never say which is which. Read back, it is
//! marked as the past (R4.21) and is never a candidate for a rule (R4.22). The
//! guarantee above is about what the application acts on, and that is unchanged.

use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use jiff::Zoned;
use serde::{Deserialize, Serialize};

use crate::autoattach::{self, Rule};

/// Bumped only alongside a migration. See [`load`].
pub const SCHEMA_VERSION: u32 = 1;

/// How many devices are remembered. Least recently seen dropped first.
///
/// A bench accumulates ports faster than it accumulates boards, and every port
/// a device has ever been plugged into earns an entry. The cap keeps the file
/// readable by hand (R7.3) without the user having to think about it.
pub const LAST_SEEN_LIMIT: usize = 200;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Store {
    pub schema_version: u32,
    #[serde(default)]
    pub settings: Settings,
    /// What each device last looked like. Least recently seen first, so the end
    /// of the list is the most recent and the front is what the cap drops.
    #[serde(default)]
    pub last_seen: Vec<LastSeen>,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            schema_version: SCHEMA_VERSION,
            settings: Settings::default(),
            last_seen: Vec::new(),
        }
    }
}

/// What a device looked like the last time anything could describe it.
///
/// A reminder, not a record of what is plugged in now. See the module docs for
/// why it is kept and what it may not be used for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LastSeen {
    /// The device instance id it was seen on. For a device with no serial
    /// number this derives from the port, so the entry says as much about the
    /// socket as about the device — which is exactly why it stays a reminder.
    pub instance_id: String,
    /// The name Windows reported while it could still see the device.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity: Option<SeenIdentity>,
    /// When [`Self::identity`] was read, as `2026-09-18 10:22:31`.
    ///
    /// A display string rather than an instant: nothing computes with it, and
    /// its whole job is to let someone judge how old the answer is — in the
    /// file as readily as in the window.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identified_at: Option<String>,
}

/// An identification as it is written down.
///
/// Its own type rather than `wuim_probe::TargetIdentity`, which this crate
/// could not name anyway: this one is a file format, and a field added to a
/// probe result has no business changing what is on disk.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeenIdentity {
    pub identity_key: String,
    pub device_type: String,
    pub device_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub hardware_revision: Option<String>,
    /// Where the id came from, as `wuim_probe::id_sources` spells it. Absent in
    /// files written before it was recorded, which is why it is optional rather
    /// than defaulted to a guess.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id_source: Option<String>,
}

/// The current time in the form [`LastSeen::identified_at`] holds.
pub fn stamp() -> String {
    Zoned::now().strftime("%F %T").to_string()
}

impl Store {
    pub fn last_seen(&self, instance_id: &str) -> Option<&LastSeen> {
        self.last_seen
            .iter()
            .find(|seen| seen.instance_id == instance_id)
    }

    /// Records the name a device is going by, returning whether that changed
    /// anything.
    ///
    /// The caller writes the file only when it did. This runs against every row
    /// of every refresh, and saving each time would rewrite the file twice a
    /// second for no reason.
    pub fn remember_name(&mut self, instance_id: &str, name: &str) -> bool {
        let entry = self.entry(instance_id);
        if entry.name.as_deref() == Some(name) {
            return false;
        }
        entry.name = Some(name.to_owned());
        self.cap();
        true
    }

    /// Records what a probe found, with the time it found it.
    pub fn remember_identity(
        &mut self,
        instance_id: &str,
        identity: SeenIdentity,
        at: String,
    ) -> bool {
        let entry = self.entry(instance_id);
        if entry.identity.as_ref() == Some(&identity) {
            return false;
        }
        entry.identity = Some(identity);
        entry.identified_at = Some(at);
        self.cap();
        true
    }

    /// Drops every remembered device.
    ///
    /// Offered to the user because this is the one part of the file that can go
    /// wrong without anything having gone wrong: boards move between ports, and
    /// a list of what used to be where eventually stops helping.
    pub fn forget_everything_seen(&mut self) -> usize {
        let dropped = self.last_seen.len();
        self.last_seen.clear();
        dropped
    }

    /// Finds or creates the entry, moving it to the end so the list stays in
    /// least-recently-seen order.
    fn entry(&mut self, instance_id: &str) -> &mut LastSeen {
        let found = self
            .last_seen
            .iter()
            .position(|seen| seen.instance_id == instance_id);
        let moved = match found {
            Some(at) => self.last_seen.remove(at),
            None => LastSeen {
                instance_id: instance_id.to_owned(),
                name: None,
                identity: None,
                identified_at: None,
            },
        };
        self.last_seen.push(moved);
        self.last_seen.last_mut().expect("just pushed")
    }

    /// Enforces [`LAST_SEEN_LIMIT`]. Applied on the way in as well, so a file
    /// edited by hand cannot grow without bound.
    pub fn cap(&mut self) {
        if self.last_seen.len() > LAST_SEEN_LIMIT {
            let excess = self.last_seen.len() - LAST_SEEN_LIMIT;
            self.last_seen.drain(..excess);
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
    /// Attach matching devices to WSL without being asked (§9).
    ///
    /// Off by default. Attaching takes a device away from Windows, so it starts
    /// only when the user says so — and the switch sits on the main window
    /// rather than in here, because it is the one setting they will want to
    /// flip in the middle of working.
    pub auto_attach: bool,
    /// What to attach automatically. An empty list attaches nothing, whatever
    /// [`Self::auto_attach`] says.
    pub auto_attach_rules: Vec<Rule>,
    /// Whether the user has been told that closing the window leaves the
    /// application running in the tray.
    ///
    /// Asked once. "I closed it and it is still running" is the one thing about
    /// a tray application that has to be said out loud, and saying it every
    /// time would be worse than not saying it at all.
    pub told_about_tray: bool,
}

impl Settings {
    /// Cleans the rule list. Called on the way in from the frontend and on the
    /// way out to the file, so neither can hold a blank or a repeat.
    pub fn sanitised(mut self) -> Self {
        self.auto_attach_rules = autoattach::sanitise(self.auto_attach_rules);
        self
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            // On, gated by the exclusion list rather than by an allow list.
            auto_identify: true,
            auto_exclude: Vec::new(),
            confirm_before_identify: true,
            start_with_windows: false,
            // Opt-in: handing a device to WSL takes it away from Windows, which
            // is not something to start doing on a fresh install.
            auto_attach: false,
            auto_attach_rules: Vec::new(),
            told_about_tray: false,
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
        Ok(mut store) => {
            store.cap();
            Loaded::Ok(store)
        }
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

    fn seen(key: &str) -> SeenIdentity {
        SeenIdentity {
            identity_key: key.to_owned(),
            device_type: "esp32-s3".to_owned(),
            device_id: key.to_owned(),
            hardware_revision: None,
            id_source: Some("target-mac".to_owned()),
        }
    }

    #[test]
    fn a_name_already_recorded_is_not_written_again() {
        let mut store = Store::default();
        assert!(store.remember_name("USB\\a", "CH340"));
        // Every refresh offers the same name; only a change is worth a write.
        assert!(!store.remember_name("USB\\a", "CH340"));
        assert!(store.remember_name("USB\\a", "CH343"));
        assert_eq!(
            store.last_seen("USB\\a").unwrap().name.as_deref(),
            Some("CH343")
        );
        assert_eq!(store.last_seen.len(), 1);
    }

    #[test]
    fn an_identification_is_dated_and_survives_a_round_trip() {
        let mut store = Store::default();
        store.remember_identity(
            "USB\\a",
            seen("esp32-s3-abc"),
            "2026-09-18 10:22:31".to_owned(),
        );

        let path = temp("remembered.json");
        save(&path, &store).unwrap();
        let Loaded::Ok(read) = load(&path) else {
            panic!("what was just written should read back");
        };

        let entry = read.last_seen("USB\\a").unwrap();
        assert_eq!(
            entry.identity.as_ref().unwrap().identity_key,
            "esp32-s3-abc"
        );
        assert_eq!(entry.identified_at.as_deref(), Some("2026-09-18 10:22:31"));
    }

    #[test]
    fn a_file_without_the_field_still_reads() {
        // Written before anything was remembered. The settings are the point of
        // the file, and an older one must not be refused over an addition.
        let path = temp("no-last-seen.json");
        fs::write(
            &path,
            r#"{"schema_version": 1, "settings": {"auto_identify": false}}"#,
        )
        .unwrap();

        let Loaded::Ok(store) = load(&path) else {
            panic!("a file written before this field should still read");
        };
        assert!(store.last_seen.is_empty());
    }

    #[test]
    fn the_least_recently_seen_is_dropped_first() {
        let mut store = Store::default();
        for n in 0..LAST_SEEN_LIMIT + 10 {
            store.remember_name(&format!("USB\\{n}"), "CH340");
        }
        assert_eq!(store.last_seen.len(), LAST_SEEN_LIMIT);
        assert!(
            store.last_seen("USB\\0").is_none(),
            "the oldest should have gone"
        );
        assert!(
            store
                .last_seen(&format!("USB\\{}", LAST_SEEN_LIMIT + 9))
                .is_some()
        );
    }

    #[test]
    fn seeing_a_device_again_moves_it_out_of_the_firing_line() {
        let mut store = Store::default();
        store.remember_name("USB\\old", "CH340");
        for n in 0..LAST_SEEN_LIMIT - 1 {
            store.remember_name(&format!("USB\\{n}"), "CH340");
        }
        // Still plugged in, so still worth remembering: touching it moves it to
        // the recent end and the next arrival evicts something else.
        store.remember_name("USB\\old", "CH340 (COM9)");
        store.remember_name("USB\\new", "CH343");

        assert_eq!(store.last_seen.len(), LAST_SEEN_LIMIT);
        assert!(store.last_seen("USB\\old").is_some());
        assert!(store.last_seen("USB\\0").is_none());
    }

    #[test]
    fn clearing_leaves_the_settings_alone() {
        let mut store = Store::default();
        store.settings.auto_attach = true;
        store.remember_name("USB\\a", "CH340");

        assert_eq!(store.forget_everything_seen(), 1);
        assert!(store.last_seen.is_empty());
        assert!(store.settings.auto_attach);
    }
}
