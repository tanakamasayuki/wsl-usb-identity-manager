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
    /// Where each connected device was last seen plugged in, as (hub, port).
    ///
    /// Not an identity and not persisted: it fills the gap while Windows
    /// re-enumerates a device, which is the one moment the device has no node
    /// to read a port from while still being connected. Dropped as soon as the
    /// device stops being connected, so it can never outlive the socket it
    /// describes.
    ports: HashMap<String, (String, u32)>,
    /// Identities from probes, keyed by device instance id. Never written out.
    ///
    /// The confirmed ones: what a board answered while it was plugged in. What
    /// each device *last* looked like is [`store::LastSeen`], which is written
    /// out, and the two must not be confused — see the store's module docs.
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
            ports: HashMap::new(),
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
    update_when(|store| {
        f(store);
        true
    });
}

/// Changes the store, and writes it out only when `f` reports a change.
///
/// The remembered names are offered every device, every refresh, and all but a
/// handful of those are already what the file says. Writing on each would mean
/// rewriting the file twice a second to change nothing.
pub fn update_when(f: impl FnOnce(&mut Store) -> bool) {
    let mut guard = held().lock().unwrap();
    if !f(&mut guard.store) {
        return;
    }
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

/// Records where a device is plugged in, and answers for it while Windows has
/// nothing to say.
///
/// `live` is what the enumeration knows right now. When it has an answer that
/// answer is kept and returned; when it does not — the moment between a detach
/// and the device node coming back — the last one stands in. The device has not
/// moved in that moment; only Windows' account of it has gone briefly missing.
pub fn port_of(instance_id: &str, live: Option<(String, u32)>) -> Option<(String, u32)> {
    let mut guard = held().lock().unwrap();
    match live {
        Some(port) => {
            guard.ports.insert(instance_id.to_owned(), port.clone());
            Some(port)
        }
        None => guard.ports.get(instance_id).cloned(),
    }
}

/// Forgets where devices were plugged in once they stop being connected.
///
/// Unlike the name and the identity, this is not kept as a reminder: a port is
/// only worth reporting for something that is actually on it.
pub fn forget_ports(connected: &[String]) {
    let mut guard = held().lock().unwrap();
    guard
        .ports
        .retain(|id, _| connected.iter().any(|c| c == id));
}

/// Records what a probe found, as this session's answer and as the reminder
/// that outlives it.
pub fn remember_identity(instance_id: String, identity: TargetIdentity) {
    let seen = store::SeenIdentity {
        identity_key: identity.identity_key.clone(),
        device_type: identity.device_type.clone(),
        device_id: identity.device_id.clone(),
        hardware_revision: identity.hardware_revision.clone(),
        id_source: Some(identity.id_source.to_owned()),
    };
    held()
        .lock()
        .unwrap()
        .identities
        .insert(instance_id.clone(), identity);
    update_when(|store| store.remember_identity(&instance_id, seen, store::stamp()));
}

/// Records the names seen in one pass of the device list.
///
/// Takes the whole pass rather than one device at a time so the file is written
/// once, or not at all. The last name wins rather than the first, so a device
/// that genuinely renames itself is followed.
pub fn remember_names(seen: &[(String, String)]) {
    update_when(|store| {
        let mut changed = false;
        for (instance_id, name) in seen {
            changed |= store.remember_name(instance_id, name);
        }
        changed
    });
}

pub fn last_seen(instance_id: &str) -> Option<store::LastSeen> {
    with(|store| store.last_seen(instance_id).cloned())
}

/// Drops every remembered device, and says how many there were.
pub fn forget_everything_seen() -> usize {
    let mut dropped = 0;
    update(|store| dropped = store.forget_everything_seen());
    logging::info(&format!("forgot {dropped} remembered device(s) on request"));
    dropped
}

/// How many devices are remembered, for the settings screen to offer clearing.
pub fn remembered_count() -> usize {
    with(|store| store.last_seen.len())
}

pub fn identity(instance_id: &str) -> Option<TargetIdentity> {
    held().lock().unwrap().identities.get(instance_id).cloned()
}

/// Forgets identities for devices that are no longer plugged in.
///
/// Unplugging is the one event after which what is on the end of the cable can
/// have changed without anything on the USB side saying so. Keeping the identity
/// across it would be a guess wearing the clothes of a fact.
///
/// [`store::LastSeen`] is deliberately left alone: it is already labelled as the
/// past, so it has nothing to lose by being out of date.
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
