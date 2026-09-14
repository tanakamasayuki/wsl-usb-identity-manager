//! Matching what is plugged in now against what was identified before.
//!
//! This is what makes the stored file worth having: without it, restarting the
//! application means every board has to be probed again, and every probe resets
//! a board. The routes are the ones in requirements §4.1 — a serial number when
//! there is one, the port position when there is not — and the outcome carries
//! the confidence level of §4.2 so the UI can say how sure it is (R4.3).
//!
//! A guess is never presented as a fact. Where the evidence does not single out
//! one device, the answer is [`Confidence::Ambiguous`] and nothing is claimed.

use std::collections::HashMap;

use serde::Serialize;

use crate::snapshot::DeviceRow;
use crate::store::{Store, StoredDevice};

/// How much the match is worth, from requirements §4.2.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    /// The device reports a serial number and it matches what was stored, or it
    /// was just probed. Safe to act on automatically.
    Confirmed,
    /// No serial number, but the port and the VID/PID match a single stored
    /// device and nothing else competes for it.
    Probable,
    /// More than one stored device fits, or more than one connected device
    /// fits the same record. **Never act on this automatically** (R4.4).
    Ambiguous,
}

/// One connected device matched to a stored one.
#[derive(Debug, Clone)]
pub struct Recalled<'a> {
    pub device: &'a StoredDevice,
    pub confidence: Confidence,
}

/// Matches every row against the store, keyed by instance id.
///
/// Rows with no match are simply absent from the result: an unknown device is
/// not a failure, it is a device that has not been identified yet.
pub fn recall<'a>(store: &'a Store, rows: &[DeviceRow]) -> HashMap<String, Recalled<'a>> {
    let mut matched: HashMap<String, Recalled<'a>> = HashMap::new();
    // Which rows laid claim to each stored device, so a record wanted by two
    // rows makes both ambiguous rather than arbitrarily belonging to the first.
    let mut claims: HashMap<&str, Vec<String>> = HashMap::new();

    for row in rows {
        let Some((vid, pid)) = row.instance_id.vid_pid() else {
            continue;
        };

        // Route 1: the device says who it is.
        if let Some(serial) = row.instance_id.unit.serial() {
            if let Some(device) = store
                .devices
                .iter()
                .find(|d| d.vid == vid && d.pid == pid && d.usb_serial.as_deref() == Some(serial))
            {
                matched.insert(
                    row.instance_id.raw.clone(),
                    Recalled {
                        device,
                        confidence: Confidence::Confirmed,
                    },
                );
                claims
                    .entry(device.identity_key.as_str())
                    .or_default()
                    .push(row.instance_id.raw.clone());
            }
            continue;
        }

        // Route 2: no serial, so the port is the only handle. It has to single
        // out exactly one stored device to be worth anything.
        let Some(location) = row.location_path() else {
            continue;
        };
        let candidates: Vec<&StoredDevice> = store
            .devices
            .iter()
            .filter(|d| {
                d.vid == vid
                    && d.pid == pid
                    && d.usb_serial.is_none()
                    && d.hints.last_location_path.as_deref() == Some(location)
            })
            .collect();

        let (device, confidence) = match candidates.as_slice() {
            [] => continue,
            [only] => (*only, Confidence::Probable),
            // The store disagrees with itself about this port; say so rather
            // than picking one.
            many => (many[0], Confidence::Ambiguous),
        };
        matched.insert(row.instance_id.raw.clone(), Recalled { device, confidence });
        claims
            .entry(device.identity_key.as_str())
            .or_default()
            .push(row.instance_id.raw.clone());
    }

    // Two devices cannot be the same board. If a record was claimed twice,
    // neither claim is trustworthy.
    for (_, instance_ids) in claims.iter().filter(|(_, ids)| ids.len() > 1) {
        for instance_id in instance_ids {
            if let Some(entry) = matched.get_mut(instance_id) {
                entry.confidence = Confidence::Ambiguous;
            }
        }
    }

    matched
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::snapshot::Snapshot;
    use crate::store::{Hints, StoredDevice};
    use crate::usbipd;
    use crate::windevice::WinUsbDevice;

    const CH340: (u16, u16) = (0x1a86, 0x7523);
    const PORT_A: &str = "PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(1)";
    const PORT_B: &str = "PCIROOT(0)#PCI(1400)#USBROOT(0)#USB(1)#USB(3)";

    fn stored(key: &str, serial: Option<&str>, location: Option<&str>) -> StoredDevice {
        StoredDevice {
            identity_key: key.to_owned(),
            device_type: "esp32-s3".into(),
            device_id: key.rsplit('-').next().unwrap().to_owned(),
            hardware_revision: None,
            usb_serial: serial.map(str::to_owned),
            vid: CH340.0,
            pid: CH340.1,
            hints: Hints {
                last_location_path: location.map(str::to_owned),
                ..Hints::default()
            },
        }
    }

    /// Builds one row with a Windows node at the given location.
    fn row(instance_id: &str, location: &str) -> DeviceRow {
        let json = format!(
            r#"{{"Devices":[{{"BusId":"1-1","ClientIPAddress":null,"Description":"test",
            "InstanceId":{},"IsForced":false,"PersistedGuid":null,"StubInstanceId":null}}]}}"#,
            serde_json::to_string(instance_id).unwrap()
        );
        let entry = usbipd::parse(&json).unwrap().pop().unwrap();
        let windows = WinUsbDevice {
            instance_id: crate::InstanceId::parse(instance_id),
            device_desc: None,
            friendly_name: None,
            bus_reported_device_desc: None,
            manufacturer: None,
            service: None,
            container_id: None,
            location_paths: vec![location.to_owned()],
            com_port: None,
            revision: None,
            driver_version: None,
            problem_code: None,
        };
        let mut snapshot = Snapshot::join(vec![windows], vec![entry]);
        snapshot.devices.pop().unwrap()
    }

    #[test]
    fn a_serial_match_is_confirmed() {
        let mut store = Store::default();
        let mut device = stored("esp32-s3-aaaaaaaaaaaa", Some("5B5F090816"), None);
        device.pid = 0x55d3;
        store.remember(device);

        let rows = vec![row(r"USB\VID_1A86&PID_55D3\5B5F090816", PORT_A)];
        let found = recall(&store, &rows);
        let hit = &found[r"USB\VID_1A86&PID_55D3\5B5F090816"];
        assert_eq!(hit.confidence, Confidence::Confirmed);
        assert_eq!(hit.device.identity_key, "esp32-s3-aaaaaaaaaaaa");
    }

    #[test]
    fn a_port_match_with_no_competition_is_probable() {
        let mut store = Store::default();
        store.remember(stored("esp32-s3-111111111111", None, Some(PORT_A)));

        let rows = vec![row(r"USB\VID_1A86&PID_7523\8&A&0&1", PORT_A)];
        let found = recall(&store, &rows);
        assert_eq!(
            found[r"USB\VID_1A86&PID_7523\8&A&0&1"].confidence,
            Confidence::Probable
        );
    }

    #[test]
    fn a_device_moved_to_another_port_is_not_recalled() {
        let mut store = Store::default();
        store.remember(stored("esp32-s3-111111111111", None, Some(PORT_A)));

        // Same board, different socket: nothing ties the two together without
        // asking the board again.
        let rows = vec![row(r"USB\VID_1A86&PID_7523\8&A&0&1", PORT_B)];
        assert!(recall(&store, &rows).is_empty());
    }

    #[test]
    fn two_stored_devices_on_one_port_are_ambiguous() {
        let mut store = Store::default();
        store.remember(stored("esp32-s3-111111111111", None, Some(PORT_A)));
        store.remember(stored("esp32-s3-222222222222", None, Some(PORT_A)));

        let rows = vec![row(r"USB\VID_1A86&PID_7523\8&A&0&1", PORT_A)];
        let found = recall(&store, &rows);
        assert_eq!(
            found[r"USB\VID_1A86&PID_7523\8&A&0&1"].confidence,
            Confidence::Ambiguous
        );
    }

    #[test]
    fn each_port_recalls_its_own_board() {
        let mut store = Store::default();
        store.remember(stored("esp32-s3-111111111111", None, Some(PORT_A)));
        store.remember(stored("esp32-s3-222222222222", None, Some(PORT_B)));

        let rows = vec![
            row(r"USB\VID_1A86&PID_7523\8&A&0&1", PORT_A),
            row(r"USB\VID_1A86&PID_7523\8&A&0&3", PORT_B),
        ];
        let found = recall(&store, &rows);
        assert_eq!(
            found[r"USB\VID_1A86&PID_7523\8&A&0&1"].device.identity_key,
            "esp32-s3-111111111111"
        );
        assert_eq!(
            found[r"USB\VID_1A86&PID_7523\8&A&0&3"].device.identity_key,
            "esp32-s3-222222222222"
        );
    }

    #[test]
    fn nothing_is_recalled_from_an_empty_store() {
        let rows = vec![row(r"USB\VID_1A86&PID_7523\8&A&0&1", PORT_A)];
        assert!(recall(&Store::default(), &rows).is_empty());
    }
}
