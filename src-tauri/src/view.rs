//! The shape the frontend receives.
//!
//! Deliberately a separate type from [`wuim_core::DeviceRow`]: the wire format
//! is something the UI depends on, and pinning it here means the core model can
//! change without silently reshaping what Svelte is reading.

use serde::{Deserialize, Serialize};
use wuim_core::UsbIds;
use wuim_core::autoattach::{self, Candidate, Rule, RuleKind};
use wuim_core::snapshot::DeviceRow;
use wuim_core::store::Settings;
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
}

impl From<&TargetIdentity> for Identity {
    fn from(identity: &TargetIdentity) -> Self {
        Self {
            identity_key: identity.identity_key.clone(),
            device_type: identity.device_type.clone(),
            device_id: identity.device_id.clone(),
            hardware_revision: identity.hardware_revision.clone(),
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
        rules: &[Rule],
    ) -> Self {
        let usbipd = row.usbipd.as_ref();
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
            auto_attach: auto_attach(row, identity.as_ref(), rules),
            identity,
        }
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

    #[test]
    fn settings_come_back_from_what_the_frontend_sends() {
        let sent = r#"{"autoIdentify": false, "autoExclude": ["1a86:7523"],
                       "confirmBeforeIdentify": false, "startWithWindows": true,
                       "autoAttach": true, "toldAboutTray": true,
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
