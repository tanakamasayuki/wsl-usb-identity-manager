//! Vendor and product names from the USB ID Repository's `usb.ids`.
//!
//! The file is read from the usbipd-win installation rather than shipped with
//! this application. usbipd-win 5.x is already a requirement and installs a copy
//! next to `usbipd.exe`, so reading it costs nothing and keeps `usb.ids` — which
//! is dual-licensed GPLv2-or-later / BSD-3-Clause — out of this repository
//! entirely.
//!
//! Everything here fails soft. A missing or unreadable file means devices show
//! no vendor name, which is how they looked before; it is never a reason to fail
//! an enumeration.

use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

/// Vendor and product names, indexed for lookup.
#[derive(Debug, Default)]
pub struct UsbIds {
    vendors: HashMap<u16, String>,
    products: HashMap<(u16, u16), String>,
}

impl UsbIds {
    /// Reads and indexes `usb.ids`. An empty index is returned rather than an
    /// error when the file is not there.
    pub fn load(path: &Path) -> Self {
        let Ok(file) = File::open(path) else {
            return Self::default();
        };
        Self::parse(BufReader::new(file))
    }

    pub fn parse<R: BufRead>(reader: R) -> Self {
        let mut ids = Self::default();
        let mut vendor: Option<u16> = None;

        for line in reader.lines().map_while(Result::ok) {
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            // Two tabs in means an interface, which nothing here uses. The
            // class/HID/language sections further down the file start with a
            // letter, so the hex check below rejects them.
            if line.starts_with("\t\t") {
                continue;
            }

            if let Some(rest) = line.strip_prefix('\t') {
                let Some(current) = vendor else { continue };
                if let Some((id, name)) = split_entry(rest) {
                    ids.products.insert((current, id), name.to_owned());
                }
                continue;
            }

            match split_entry(&line) {
                Some((id, name)) => {
                    vendor = Some(id);
                    ids.vendors.insert(id, name.to_owned());
                }
                // A section heading such as `C 00  ...`; nothing below it is a
                // product of the last vendor.
                None => vendor = None,
            }
        }
        ids
    }

    pub fn vendor(&self, vid: u16) -> Option<&str> {
        self.vendors.get(&vid).map(String::as_str)
    }

    pub fn product(&self, vid: u16, pid: u16) -> Option<&str> {
        self.products.get(&(vid, pid)).map(String::as_str)
    }

    pub fn is_empty(&self) -> bool {
        self.vendors.is_empty()
    }
}

/// Splits `1a86  QinHeng Electronics` into its id and name.
///
/// Returns `None` unless the id is exactly four hex digits, which is what keeps
/// the class and language sections at the end of the file out of the index.
fn split_entry(line: &str) -> Option<(u16, &str)> {
    let (id, name) = line.split_once("  ")?;
    if id.len() != 4 || !id.bytes().all(|b| b.is_ascii_hexdigit()) {
        return None;
    }
    Some((u16::from_str_radix(id, 16).ok()?, name.trim()))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Shaped like the real file, including the sections that follow the
    /// vendor list.
    const SAMPLE: &str = "\
# Comment
#
1a86  QinHeng Electronics
\t5523  CH341 in serial mode, usb to serial port converter
\t7523  CH340 serial converter
\t\t0001  an interface, ignored
0b95  ASIX Electronics Corp.
\t1790  AX88179 Gigabit Ethernet

# List of known device classes
C 00  (Defined at Interface level)
\t01  something that is not a product
";

    fn sample() -> UsbIds {
        UsbIds::parse(SAMPLE.as_bytes())
    }

    #[test]
    fn finds_vendors_and_products() {
        let ids = sample();
        assert_eq!(ids.vendor(0x1a86), Some("QinHeng Electronics"));
        assert_eq!(ids.product(0x1a86, 0x7523), Some("CH340 serial converter"));
        assert_eq!(ids.vendor(0x0b95), Some("ASIX Electronics Corp."));
        assert_eq!(
            ids.product(0x0b95, 0x1790),
            Some("AX88179 Gigabit Ethernet")
        );
    }

    #[test]
    fn does_not_confuse_vendors_with_each_other() {
        let ids = sample();
        assert_eq!(ids.product(0x0b95, 0x7523), None);
    }

    #[test]
    fn ignores_interfaces_and_the_class_sections() {
        let ids = sample();
        assert_eq!(ids.vendors.len(), 2);
        // `C 00` is not a vendor, and the line indented under it is not one of
        // ASIX's products.
        assert_eq!(ids.products.len(), 3);
    }

    /// Reads the copy usbipd-win installs, when it is there. Skipped elsewhere
    /// so the suite still passes on a machine without usbipd.
    #[test]
    fn reads_the_copy_usbipd_installs() {
        let Some(path) = crate::usbipd::usb_ids_path() else {
            return;
        };
        let ids = UsbIds::load(&path);
        assert!(!ids.is_empty(), "{} parsed to nothing", path.display());
        assert_eq!(ids.vendor(0x1a86), Some("QinHeng Electronics"));
        assert_eq!(ids.vendor(0x0b95), Some("ASIX Electronics Corp."));
        // The repository is not exhaustive: Espressif's 303a is registered with
        // the USB-IF but was never submitted to linux-usb.org, so a device on it
        // has no vendor name to show. Pinned here so the gap stays a known fact
        // rather than looking like a parsing failure.
        assert_eq!(ids.vendor(0x303a), None);
    }

    #[test]
    fn a_missing_file_is_not_an_error() {
        let ids = UsbIds::load(Path::new("no-such-usb.ids"));
        assert!(ids.is_empty());
        assert_eq!(ids.vendor(0x1a86), None);
    }
}
