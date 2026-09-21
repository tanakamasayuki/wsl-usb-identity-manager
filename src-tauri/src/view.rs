//! The shape the frontend receives.
//!
//! Deliberately a separate type from [`wuim_core::DeviceRow`]: the wire format
//! is something the UI depends on, and pinning it here means the core model can
//! change without silently reshaping what Svelte is reading.

use serde::{Deserialize, Serialize};
use wuim_core::UsbIds;
use wuim_core::autoattach::{self, Candidate, Rule, RuleKind};
use wuim_core::snapshot::DeviceRow;
use wuim_core::store::{self, Settings};
use wuim_probe::TargetIdentity;
use wuim_probe::notes;

/// One row of the device list.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceView {
    /// The join key, and what every command takes to name a device.
    pub instance_id: String,
    pub name: String,
    /// `1a86:7523`, absent for hubs.
    pub vid_pid: Option<String>,
    /// Vendor name from the USB ID Repository. Absent when the vendor never
    /// submitted an entry, which is common enough to be unremarkable.
    pub vendor: Option<String>,
    /// Product name from the same source. Rarer than the vendor name.
    pub usb_product: Option<String>,

    // Runtime connection. None of this is an identity (requirement R7.1); it is
    // here to be displayed and nothing else.
    pub bus_id: Option<String>,
    pub com_port: Option<String>,
    /// `DEVPKEY_Device_LocationPaths`, the stable identifier of the physical
    /// port (finding F2). Displayed, never persisted as an identity (R7.1).
    pub location_path: Option<String>,
    /// The same path as a short hub-port chain, `1-3-3`.
    pub port_chain: Option<String>,
    /// The hub this device is plugged into, and the port number on it. Read
    /// from the device tree, so a `usbipd` bind does not take it away.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_address: Option<u32>,

    // usbipd state, pre-computed so the table does not have to derive it.
    pub state: &'static str,
    /// usbipd still reports a bus id. True even while the device is attached.
    pub present: bool,
    /// Windows currently exposes a device node we could talk to.
    ///
    /// Distinct from `present`: while a device is attached, usbipd keeps
    /// reporting its bus id but Windows has handed the device to the client, so
    /// nothing here can reach it. This is the signal for "arrived", not
    /// `present`, which never goes false across an attach/detach cycle.
    pub reachable: bool,
    pub shared: bool,
    pub attached: bool,
    /// The client holding the device while attached.
    pub client_address: Option<String>,

    // Identity.
    /// The serial number the device reports, when it reports one. Shown as-is:
    /// "has a serial" tells the user nothing, the value tells them which unit.
    pub serial: Option<String>,
    /// `serial` or `port only`.
    pub identity_basis: &'static str,
    /// Whether the descriptors leave the unit undetermined (finding F3).
    pub needs_probe: bool,

    pub driver: Option<String>,
    pub driver_version: Option<String>,
    /// Device revision from the descriptor, `2.64`.
    pub revision: Option<String>,
    /// Windows' problem code, present only when there is one.
    pub problem_code: Option<u32>,
    /// Probes that could be run, with the side effects to show first (R4.7).
    pub probes: Vec<ProbeOption>,
    /// Which usbipd operations make sense for the device as it stands.
    pub actions: Actions,
    /// What an automatic-attach rule could name this device by, and what names
    /// it now (§9).
    pub auto_attach: AutoAttach,
    /// What a probe found this session, if one has run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<Identity>,
    /// What the device was called the last time Windows could describe it, when
    /// that is not the name being shown now. Reference only: shown apart from
    /// `name` rather than in place of it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_name: Option<String>,
    /// The last identification, when there is no current one. Reference only —
    /// after an unplug the board on the end of the cable need not be the one
    /// this names, so it is never a rule candidate.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_identity: Option<Identity>,
    /// When `last_identity` was read, as `2026-09-18 10:22:31`. It survives a
    /// restart, so how old it is decides how much it is worth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_identified_at: Option<String>,
}

/// The hub tree, and what this application has asked of its ports.
///
/// Sent apart from the device list rather than folded into it: a hub is not
/// something to bind or attach, and its **ports** are the point — a port with
/// nothing in it has no device row to carry it.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TopologyView {
    pub hubs: Vec<HubView>,
    /// Where `vhfilter.exe` was found, if it was. `None` means per-port power is
    /// unavailable and the interface says where to put the file.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vhfilter: Option<String>,
    /// `configured` or `searched`, so the interface can say that nothing had to
    /// be set up rather than only where the file is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vhfilter_how: Option<&'static str>,
    /// The directories searched, so the interface can name them rather than
    /// only report a failure.
    pub vhfilter_search_path: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HubView {
    pub instance_id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location_path: Option<String>,
    /// The node this hub hangs off, and the port number on it.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_instance_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_address: Option<u32>,
    /// `1a86:8094`, absent on a root hub.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vid_pid: Option<String>,
    /// From the USB ID Repository, the same source the device rows use.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub vendor: Option<String>,
    /// From the same source; rarer than the vendor name.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub usb_product: Option<String>,
    /// What Windows records as the maker, which is sometimes all there is.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manufacturer: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub driver_version: Option<String>,
    /// True when `vhfilter` lists this hub as able to switch port power. It
    /// means the hub **advertises** the feature and nothing more: hubs that
    /// claim it and do nothing are common.
    pub ppps: bool,
    pub ports: Vec<PortView>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PortView {
    pub port: u32,
    pub connected: bool,
    pub status: &'static str,
    /// What this application asked of the port this session, if anything.
    ///
    /// **Not an observation.** Nothing on Windows reports port power, so absent
    /// means "never switched by us" and never "on". See `wuim_core::ppps`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub switched: Option<&'static str>,
}

/// What was last known about a device, for [`DeviceView::from_row`] to fall
/// back on where the live value has gone.
///
/// Its own type rather than `state::LastSeen` so this module keeps deciding the
/// shape of the wire format on its own, and stays testable without a process
/// holding a settings file.
#[derive(Debug, Clone, Default)]
pub struct LastKnown {
    pub name: Option<String>,
    pub identity: Option<Identity>,
    pub identified_at: Option<String>,
}

impl From<&store::LastSeen> for LastKnown {
    fn from(seen: &store::LastSeen) -> Self {
        Self {
            name: seen.name.clone(),
            identity: seen.identity.as_ref().map(Identity::from),
            identified_at: seen.identified_at.clone(),
        }
    }
}

impl From<&store::SeenIdentity> for Identity {
    fn from(seen: &store::SeenIdentity) -> Self {
        Self {
            identity_key: seen.identity_key.clone(),
            device_type: seen.device_type.clone(),
            device_id: seen.device_id.clone(),
            hardware_revision: seen.hardware_revision.clone(),
            id_source: seen.id_source.clone(),
        }
    }
}

/// Settings as the frontend sees them.
///
/// A separate type from [`Settings`] for the same reason [`DeviceView`] is
/// separate from `DeviceRow`: the stored file is snake_case throughout so it
/// reads consistently when opened by hand, while the frontend works in
/// camelCase. Converting here keeps each side in its own convention instead of
/// one leaking into the other — which is exactly what went wrong when the two
/// shared a struct and the field names silently failed to line up.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SettingsView {
    pub auto_identify: bool,
    pub auto_exclude: Vec<String>,
    pub confirm_before_identify: bool,
    pub start_with_windows: bool,
    pub auto_attach: bool,
    pub auto_attach_rules: Vec<Rule>,
    pub vhfilter_path: String,
    pub told_about_tray: bool,
}

impl From<&Settings> for SettingsView {
    fn from(settings: &Settings) -> Self {
        Self {
            auto_identify: settings.auto_identify,
            auto_exclude: settings.auto_exclude.clone(),
            confirm_before_identify: settings.confirm_before_identify,
            start_with_windows: settings.start_with_windows,
            auto_attach: settings.auto_attach,
            auto_attach_rules: settings.auto_attach_rules.clone(),
            vhfilter_path: settings.vhfilter_path.clone(),
            told_about_tray: settings.told_about_tray,
        }
    }
}

impl From<SettingsView> for Settings {
    fn from(view: SettingsView) -> Self {
        Self {
            auto_identify: view.auto_identify,
            auto_exclude: view.auto_exclude,
            confirm_before_identify: view.confirm_before_identify,
            start_with_windows: view.start_with_windows,
            auto_attach: view.auto_attach,
            auto_attach_rules: view.auto_attach_rules,
            vhfilter_path: view.vhfilter_path,
            told_about_tray: view.told_about_tray,
        }
        // Whatever the frontend sent, the stored list holds no blanks and no
        // repeats.
        .sanitised()
    }
}

/// What an automatic-attach rule could name a device by.
///
/// Computed here rather than in the UI so the rule semantics live in one place
/// and can be tested. Nothing in this touches a device: every candidate comes
/// from the enumeration that has already happened, which is what keeps
/// requirement R9.4 — no probing to decide an attach — true by construction.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AutoAttach {
    /// Most specific first. Only what the device actually offers: no serial
    /// number means no serial candidate, and no probe has run means no identity
    /// candidate.
    pub candidates: Vec<Candidate>,
    /// The kind of rule matching it right now, if any.
    pub matched: Option<RuleKind>,
}

/// What a probe found. Present only while the device it came from stays
/// plugged in; see `state::forget_absent`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identity {
    pub identity_key: String,
    pub device_type: String,
    pub device_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hardware_revision: Option<String>,
    /// How the unit was pinned down: read from the silicon, or read from the
    /// board's own USB descriptors. Shown, never branched on.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id_source: Option<String>,
}

impl From<&TargetIdentity> for Identity {
    fn from(identity: &TargetIdentity) -> Self {
        Self {
            identity_key: identity.identity_key.clone(),
            device_type: identity.device_type.clone(),
            device_id: identity.device_id.clone(),
            hardware_revision: identity.hardware_revision.clone(),
            id_source: Some(identity.id_source.to_owned()),
        }
    }
}

/// The usbipd operations offered for a device.
///
/// Decided here rather than in the UI so the rule lives next to the state it
/// reads. The backend re-checks anyway — this only decides what to show.
#[derive(Debug, Clone, Copy, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Actions {
    /// Share it with usbipd. Needs administrator rights.
    pub bind: bool,
    /// Stop sharing it. Needs administrator rights.
    pub unbind: bool,
    /// Hand it to WSL. Only possible once bound.
    pub attach: bool,
    /// Take it back from WSL.
    pub detach: bool,
}

/// A probe the user could choose to run against this device.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProbeOption {
    pub family: &'static str,
    /// Translation key for the side effects to show before running (R4.7).
    pub side_effect: &'static str,
    pub available: bool,
    /// Translation key for why it is unavailable, when it is.
    pub reason: Option<&'static str>,
}

impl DeviceView {
    pub fn from_row(
        row: &DeviceRow,
        ids: &UsbIds,
        identity: Option<Identity>,
        last: LastKnown,
        rules: &[Rule],
    ) -> Self {
        let usbipd = row.usbipd.as_ref();
        let last_name = worth_showing_name(last.name, &row.name);
        let last_identity = worth_showing_identity(last.identity, identity.as_ref());
        // The date belongs to the value it dates; without one it is noise.
        let last_identified_at = last.identified_at.filter(|_| last_identity.is_some());
        let (vendor, usb_product) = match row.instance_id.vid_pid() {
            Some((vid, pid)) => (
                ids.vendor(vid).map(str::to_owned),
                ids.product(vid, pid).map(str::to_owned),
            ),
            None => (None, None),
        };
        Self {
            instance_id: row.instance_id.raw.clone(),
            name: row.name.clone(),
            vid_pid: row.instance_id.vid_pid_string(),
            vendor,
            usb_product,

            bus_id: row.bus_id().map(str::to_owned),
            com_port: row.com_port().map(str::to_owned),
            location_path: row.location_path().map(str::to_owned),
            port_chain: row.windows.as_ref().and_then(|w| w.port_chain()),
            parent_instance_id: row.parent_instance_id().map(str::to_owned),
            port_address: row.port_address(),

            state: row.sharing_state().label(),
            present: usbipd.is_some_and(|d| d.is_connected()),
            reachable: row.windows.is_some(),
            shared: usbipd.is_some_and(|d| d.is_shared()),
            attached: usbipd.is_some_and(|d| d.is_attached()),
            client_address: usbipd.and_then(|d| d.client_ip_address.clone()),

            serial: row.instance_id.unit.serial().map(str::to_owned),
            identity_basis: row.identity_basis.label(),
            needs_probe: row.identity_basis.needs_probe(),

            driver: row.windows.as_ref().and_then(|w| w.service.clone()),
            driver_version: row.windows.as_ref().and_then(|w| w.driver_version.clone()),
            revision: row.windows.as_ref().and_then(|w| w.revision.clone()),
            problem_code: row.windows.as_ref().and_then(|w| w.problem_code),
            probes: probe_options(row),
            actions: actions(row),
            // Only the confirmed identity is a candidate. `last_identity` is
            // not offered here, and must not be: a rule that fired on it would
            // hand a board to WSL on the strength of what used to be plugged
            // into that port.
            auto_attach: auto_attach(row, identity.as_ref(), rules),
            identity,
            last_name,
            last_identity,
            last_identified_at,
        }
    }
}

/// The remembered name, where it still tells the user something.
///
/// Suppressed when it matches the live name, which it does whenever Windows can
/// describe the device: the same string twice in one cell reads as two devices,
/// not as one device with a history.
fn worth_showing_name(last: Option<String>, current: &str) -> Option<String> {
    last.filter(|name| name != current)
}

/// The remembered identification, where there is no current one to prefer.
///
/// A confirmed identity is the answer, and showing the old one beside it would
/// only invite the question of which to believe.
fn worth_showing_identity(last: Option<Identity>, current: Option<&Identity>) -> Option<Identity> {
    match current {
        Some(_) => None,
        None => last,
    }
}

fn auto_attach(row: &DeviceRow, identity: Option<&Identity>, rules: &[Rule]) -> AutoAttach {
    let mut candidates = Vec::new();
    // In RuleKind::ALL order, most specific first, which is the order
    // `matching` reports a winner in.
    if let Some(identity) = identity {
        candidates.push(Candidate::new(RuleKind::Identity, &identity.identity_key));
    }
    if let Some(serial) = row.instance_id.unit.serial() {
        candidates.push(Candidate::new(RuleKind::Serial, serial));
    }
    if let Some(vid_pid) = row.instance_id.vid_pid_string() {
        candidates.push(Candidate::new(RuleKind::VidPid, vid_pid));
    }
    if let Some(bus_id) = row.bus_id() {
        candidates.push(Candidate::new(RuleKind::BusId, bus_id));
    }

    let matched = autoattach::matching(rules, &candidates);
    AutoAttach {
        candidates,
        matched,
    }
}

fn actions(row: &DeviceRow) -> Actions {
    let usbipd = row.usbipd.as_ref();
    let connected = usbipd.is_some_and(|d| d.is_connected());
    let shared = usbipd.is_some_and(|d| d.is_shared());
    let attached = usbipd.is_some_and(|d| d.is_attached());

    Actions {
        // Binding an absent device is not something usbipd can do: the bus id
        // it would need does not exist.
        bind: connected && !shared,
        unbind: shared && !attached,
        attach: connected && shared && !attached,
        detach: attached,
    }
}

/// Asks every probe what it makes of the device. Touches nothing.
fn probe_options(row: &DeviceRow) -> Vec<ProbeOption> {
    let Some(device) = row.windows.as_ref() else {
        // Attached or absent: there is no device node to talk to.
        let why = if row.sharing_state() == wuim_core::SharingState::Attached {
            notes::ATTACHED
        } else {
            notes::NOT_CONNECTED
        };
        return wuim_probe::probes()
            .iter()
            .map(|probe| ProbeOption {
                family: probe.family(),
                side_effect: probe.side_effect().code,
                available: false,
                reason: Some(why.code),
            })
            .collect();
    };

    wuim_probe::applicable(device)
        .into_iter()
        .map(|(probe, verdict)| ProbeOption {
            family: probe.family(),
            side_effect: probe.side_effect().code,
            available: verdict.is_supported(),
            reason: verdict.note().map(|n| n.code),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The frontend reads these names verbatim from `src/lib/types.ts`.
    ///
    /// Nothing checks the two sides against each other at build time, so a
    /// rename here that is not mirrored there fails silently at runtime — which
    /// is how the settings quietly stopped loading once already. Pinning the
    /// names makes that a failing test instead of a log line nobody reads.
    #[test]
    fn settings_cross_the_boundary_in_camel_case() {
        let json = serde_json::to_value(SettingsView::from(&Settings::default())).unwrap();
        let object = json.as_object().unwrap();
        let keys: Vec<&str> = object.keys().map(String::as_str).collect();

        assert!(
            keys.iter().all(|key| !key.contains('_')),
            "snake_case leaked to the frontend: {keys:?}"
        );
        assert!(object.contains_key("autoIdentify"), "{keys:?}");
        assert!(object.contains_key("autoExclude"), "{keys:?}");
        assert!(object.contains_key("confirmBeforeIdentify"), "{keys:?}");
        assert!(object.contains_key("startWithWindows"), "{keys:?}");
        assert!(object.contains_key("autoAttach"), "{keys:?}");
        assert!(object.contains_key("autoAttachRules"), "{keys:?}");
        assert!(object.contains_key("toldAboutTray"), "{keys:?}");
    }

    /// The rule kinds cross the boundary as strings the frontend switches on
    /// and builds translation keys from, so they are pinned like the field
    /// names above.
    #[test]
    fn rule_kinds_cross_the_boundary_as_snake_case() {
        let spellings: Vec<String> = RuleKind::ALL
            .iter()
            .map(|kind| {
                serde_json::to_value(kind)
                    .unwrap()
                    .as_str()
                    .unwrap()
                    .to_owned()
            })
            .collect();
        assert_eq!(spellings, ["identity", "serial", "vid_pid", "bus_id"]);
    }

    fn identity(key: &str) -> Identity {
        Identity {
            identity_key: key.to_owned(),
            device_type: "esp32-s3".to_owned(),
            device_id: key.to_owned(),
            hardware_revision: None,
            id_source: Some("target-mac".to_owned()),
        }
    }

    #[test]
    fn a_remembered_name_is_shown_only_when_it_differs() {
        // Windows can see the device, so the name it reports is the one that
        // was remembered: nothing to add.
        assert_eq!(
            worth_showing_name(Some("USB-SERIAL CH340".to_owned()), "USB-SERIAL CH340"),
            None
        );
        // Shared to WSL: the live name is whatever is left, and what the device
        // was called is worth keeping on screen.
        assert_eq!(
            worth_showing_name(Some("USB-SERIAL CH340".to_owned()), "USB Input Device"),
            Some("USB-SERIAL CH340".to_owned())
        );
        assert_eq!(worth_showing_name(None, "USB Input Device"), None);
    }

    #[test]
    fn a_remembered_identity_gives_way_to_a_confirmed_one() {
        let known = identity("esp32-s3-3485188f6d7c");
        let older = identity("esp32-s3-d83bda42d640");

        assert!(worth_showing_identity(Some(older.clone()), Some(&known)).is_none());
        assert_eq!(
            worth_showing_identity(Some(older.clone()), None).map(|i| i.identity_key),
            Some(older.identity_key)
        );
        assert!(worth_showing_identity(None, None).is_none());
    }

    #[test]
    fn settings_come_back_from_what_the_frontend_sends() {
        let sent = r#"{"autoIdentify": false, "autoExclude": ["1a86:7523"],
                       "confirmBeforeIdentify": false, "startWithWindows": true,
                       "autoAttach": true, "toldAboutTray": true, "vhfilterPath": "",
                       "autoAttachRules": [{"kind": "vid_pid", "value": "1a86:7523"},
                                           {"kind": "vid_pid", "value": "1A86:7523"}]}"#;
        let view: SettingsView = serde_json::from_str(sent).unwrap();
        let settings: Settings = view.into();

        assert!(!settings.auto_identify);
        assert_eq!(settings.auto_exclude, vec!["1a86:7523".to_string()]);
        assert!(!settings.confirm_before_identify);
        assert!(settings.start_with_windows);
        assert!(settings.auto_attach);
        assert!(settings.told_about_tray);
        // The same rule twice is one rule by the time it is stored.
        assert_eq!(settings.auto_attach_rules.len(), 1);
    }
}
