//! The stored file, held for the lifetime of the application.
//!
//! One place owns it so that a write from a probe and a write from the settings
//! panel cannot interleave into a half-updated file, and so that a file this
//! build refused to read is never written over (requirement R7.5).

use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

use wuim_core::store::{self, Loaded, Store};

use crate::logging;

struct Held {
    path: PathBuf,
    store: Store,
    /// False when the file on disk was refused. Everything still works for this
    /// session; nothing is saved, because saving would destroy it.
    writable: bool,
}

static HELD: OnceLock<Mutex<Held>> = OnceLock::new();

fn held() -> &'static Mutex<Held> {
    HELD.get_or_init(|| {
        let path = store::store_path();
        let (store, writable) = match store::load(&path) {
            Loaded::Ok(store) => {
                logging::info(&format!(
                    "loaded {} device(s) from {}",
                    store.devices.len(),
                    path.display()
                ));
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
