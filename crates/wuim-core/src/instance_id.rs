//! Parsing of Windows Device Instance IDs.
//!
//! Splits `USB\VID_1A86&PID_7523\8&7CC2A31&0&1` into VID, PID and a third element.
//!
//! The third element is the serial number when the device reports one, and a
//! Windows-generated `<parent hub hash>&0&<port>` otherwise (docs/research-findings.ja.md,
//! finding F3). The latter identifies *the port*, not *the device*, so the two are
//! kept apart by the type system rather than by convention.

use serde::{Deserialize, Serialize};

/// What the third element of an instance ID actually is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", content = "value", rename_all = "snake_case")]
pub enum UnitId {
    /// A serial number reported by the device. Usable to identify the unit.
    Serial(String),
    /// An ID Windows derived from the port. Not usable to identify the unit.
    PortGenerated(String),
}

impl UnitId {
    /// Classifies the third element of an instance ID.
    ///
    /// Windows always puts `&` in the IDs it generates, and USB serial strings
    /// never contain one, so the presence of `&` separates the two.
    pub fn classify(raw: &str) -> Self {
        if raw.contains('&') {
            Self::PortGenerated(raw.to_owned())
        } else {
            Self::Serial(raw.to_owned())
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            Self::Serial(s) | Self::PortGenerated(s) => s,
        }
    }

    /// Returns the serial number, if this is one.
    pub fn serial(&self) -> Option<&str> {
        match self {
            Self::Serial(s) => Some(s),
            Self::PortGenerated(_) => None,
        }
    }

    pub fn has_serial(&self) -> bool {
        matches!(self, Self::Serial(_))
    }
}

/// A parsed Device Instance ID.
///
/// `raw` is the only correct key for joining our device model with `usbipd`
/// (finding F4, requirement R5.1).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InstanceId {
    /// The original string. Casing differs between Windows and usbipd, so
    /// compare with [`InstanceId::matches`] rather than `==`.
    pub raw: String,
    /// First element. `USB` for USB devices.
    pub enumerator: String,
    /// Parsed from `VID_xxxx`. `None` for root hubs and similar nodes.
    pub vid: Option<u16>,
    /// Parsed from `PID_xxxx`.
    pub pid: Option<u16>,
    /// Third element.
    pub unit: UnitId,
}

impl InstanceId {
    pub fn parse(raw: &str) -> Self {
        let mut parts = raw.splitn(3, '\\');
        let enumerator = parts.next().unwrap_or_default().to_owned();
        let device = parts.next().unwrap_or_default();
        let unit_raw = parts.next().unwrap_or_default();

        // usbipd spells its stub IDs `Vid_80EE&Pid_CAFE`, so prefix matching
        // has to ignore case.
        let mut vid = None;
        let mut pid = None;
        for field in device.split('&') {
            let upper = field.to_ascii_uppercase();
            if let Some(hex) = upper.strip_prefix("VID_") {
                vid = u16::from_str_radix(hex, 16).ok();
            } else if let Some(hex) = upper.strip_prefix("PID_") {
                pid = u16::from_str_radix(hex, 16).ok();
            }
        }

        Self {
            raw: raw.to_owned(),
            enumerator,
            vid,
            pid,
            unit: UnitId::classify(unit_raw),
        }
    }

    /// Case-insensitive comparison against an instance ID from usbipd.
    pub fn matches(&self, other: &str) -> bool {
        self.raw.eq_ignore_ascii_case(other)
    }

    pub fn vid_pid(&self) -> Option<(u16, u16)> {
        Some((self.vid?, self.pid?))
    }

    /// `1a86:7523` form, or `None` when there is no VID/PID.
    pub fn vid_pid_string(&self) -> Option<String> {
        let (vid, pid) = self.vid_pid()?;
        Some(format!("{vid:04x}:{pid:04x}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ch340_without_serial_is_port_generated() {
        let id = InstanceId::parse(r"USB\VID_1A86&PID_7523\8&7CC2A31&0&1");
        assert_eq!(id.vid_pid(), Some((0x1a86, 0x7523)));
        assert_eq!(id.unit, UnitId::PortGenerated("8&7CC2A31&0&1".into()));
        assert!(id.unit.serial().is_none());
    }

    #[test]
    fn ch343_with_serial_is_serial() {
        let id = InstanceId::parse(r"USB\VID_1A86&PID_55D3\5B5F090816");
        assert_eq!(id.unit.serial(), Some("5B5F090816"));
        assert_eq!(id.vid_pid_string().as_deref(), Some("1a86:55d3"));
    }

    #[test]
    fn esp32s3_mac_with_colons_is_serial() {
        let id = InstanceId::parse(r"USB\VID_303A&PID_1001\70:04:1D:DA:86:F0");
        assert_eq!(id.unit.serial(), Some("70:04:1D:DA:86:F0"));
    }

    #[test]
    fn root_hub_has_no_vid_pid() {
        let id = InstanceId::parse(r"USB\ROOT_HUB30\4&1D2E4F8A&0&0");
        assert_eq!(id.vid_pid(), None);
        assert!(!id.unit.has_serial());
    }

    #[test]
    fn attached_stub_keeps_the_unit_id() {
        // While attached, the device is re-enumerated as a VID_80EE&PID_CAFE stub,
        // but the third element carries over (finding F4).
        let id = InstanceId::parse(r"USB\Vid_80EE&Pid_CAFE\5B5F090816");
        assert_eq!(id.vid_pid(), Some((0x80ee, 0xcafe)));
        assert_eq!(id.unit.serial(), Some("5B5F090816"));
    }

    #[test]
    fn matching_ignores_case() {
        let id = InstanceId::parse(r"USB\VID_1A86&PID_7523\8&7CC2A31&0&1");
        assert!(id.matches(r"usb\vid_1a86&pid_7523\8&7cc2a31&0&1"));
    }
}
