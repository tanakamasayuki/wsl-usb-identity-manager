//! What the application holds while it runs: the settings file, and the
//! identities established this session.
//!
//! The two are kept apart on purpose. Settings are the user's and outlive the
//! process. **Identities do not.** A probe tells us what is on the end of a
//! cable at that moment; nothing in USB tells us the board has not been swapped
//! since, so an identity is dropped the moment its device is unplugged rather
//! than written down and matched back later.

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use wuim_core::store::{self, Loaded, Store};
use wuim_probe::TargetIdentity;

use crate::logging;

struct Held {
    path: PathBuf,
    store: Store,
    /// False when the file on disk was refused. Everything still works for this
    /// session; nothing is saved, because saving would destroy it.
    writable: bool,
    /// Identities from probes, keyed by device instance id. Never written out.
    identities: HashMap<String, TargetIdentity>,
}

static HELD: OnceLock<Mutex<Held>> = OnceLock::new();

fn held() -> &'static Mutex<Held> {
    HELD.get_or_init(|| {
        let path = store::store_path();
        let (store, writable) = match store::load(&path) {
            Loaded::Ok(store) => {
                logging::info(&format!("loaded settings from {}", path.display()));
                (store, true)
            }
            Loaded::Fresh => {
                logging::info(&format!(
                    "no settings file yet; will create {}",
                    path.display()
                ));
                (Store::default(), true)
            }
            Loaded::Refused { reason } => {
                logging::error(&format!("{reason}; running without saving"));
                (Store::default(), false)
            }
        };
        Mutex::new(Held {
            path,
            store,
            writable,
            identities: HashMap::new(),
        })
    })
}

/// Reads from the store.
pub fn with<T>(f: impl FnOnce(&Store) -> T) -> T {
    let guard = held().lock().unwrap();
    f(&guard.store)
}

/// Changes the store and writes it out.
///
/// A failed write is logged and otherwise ignored: the change still applies for
/// this session, and losing the ability to save is not a reason to lose the
/// work the user just did.
pub fn update(f: impl FnOnce(&mut Store)) {
    let mut guard = held().lock().unwrap();
    f(&mut guard.store);
    if !guard.writable {
        return;
    }
    let Held { path, store, .. } = &*guard;
    if let Err(e) = store::save(path, store) {
        logging::error(&format!("could not save {}: {e:#}", path.display()));
    }
}

/// Whether changes are being written. Shown in the settings panel, because a
/// setting that silently does not persist is worse than one that says so.
pub fn is_writable() -> bool {
    held().lock().unwrap().writable
}

pub fn path() -> PathBuf {
    held().lock().unwrap().path.clone()
}

/// Records what a probe found.
pub fn remember_identity(instance_id: String, identity: TargetIdentity) {
    held()
        .lock()
        .unwrap()
        .identities
        .insert(instance_id, identity);
}

pub fn identity(instance_id: &str) -> Option<TargetIdentity> {
    held().lock().unwrap().identities.get(instance_id).cloned()
}

/// Forgets identities for devices that are no longer plugged in.
///
/// Unplugging is the one event after which what is on the end of the cable can
/// have changed without anything on the USB side saying so. Keeping the identity
/// across it would be a guess wearing the clothes of a fact.
pub fn forget_absent(present: &[String]) {
    let mut guard = held().lock().unwrap();
    let before = guard.identities.len();
    guard
        .identities
        .retain(|instance_id, _| present.iter().any(|p| p == instance_id));
    let dropped = before - guard.identities.len();
    if dropped > 0 {
        logging::info(&format!("forgot {dropped} identit(y/ies) after unplug"));
    }
}
