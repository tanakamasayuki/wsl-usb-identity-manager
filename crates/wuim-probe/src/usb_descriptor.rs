//! Naming a board from its USB descriptors, without touching it.
//!
//! Not a [`TargetProbe`](crate::TargetProbe), and deliberately so. Everything
//! else in this crate costs the target something — an ESP32 restarts, a
//! WCH-Link halts the core — which is why probing is gated behind requirement
//! R4.5. This route reads what Windows already enumerated and sends nothing, so
//! there is no cost to gate and nothing to warn about: it runs on every refresh,
//! for every device, including ones handed to WSL (the instance id carries the
//! VID, PID and serial number, and outlives an attach — finding F4).
//!
//! It answers only where the descriptors settle both halves of the question:
//!
//! * **which model**, from a VID/PID pair the vendor programmed for that board,
//!   and
//! * **which unit**, from the serial number the device reports.
//!
//! A stock USB-UART bridge answers neither. A CH340 is a CH340 whether it is
//! soldered to an ESP32 or wired to a bare AVR, so its pair names the cable and
//! its serial — when it has one — belongs to the cable too. Those pairs are
//! refused here (see [`is_generic_bridge`]) even if the table ever claims one,
//! which board definitions do occasionally do: the Sony Spresense claims the
//! stock CP2102 `10c4:ea60`.

use std::collections::BTreeMap;

use wuim_core::instance_id::InstanceId;

use crate::board_ids::{BOARD_IDS, BoardId};
use crate::identity::identity_key;
use crate::{TargetIdentity, id_sources};

/// Stock USB-UART bridge pairs, as they ship. Never a board identity.
///
/// Kept here rather than in the generated table because it is not the board
/// definitions' to say: the pairs below describe a chip that sits in front of
/// boards of every family.
const GENERIC_BRIDGE_IDS: &[(u16, u16)] = &[
    (0x0403, 0x6001), // FT232R
    (0x0403, 0x6010), // FT2232
    (0x0403, 0x6011), // FT4232
    (0x0403, 0x6014), // FT232H
    (0x0403, 0x6015), // FT230X / FT231X
    (0x067B, 0x2303), // PL2303
    (0x067B, 0x23A3), // PL2303GC
    (0x067B, 0x23B3), // PL2303GB
    (0x067B, 0x23C3), // PL2303GT
    (0x067B, 0x23D3), // PL2303GL
    (0x067B, 0x23E3), // PL2303GE
    (0x067B, 0x23F3), // PL2303GS
    (0x10C4, 0xEA60), // CP2102 / CP2109
    (0x10C4, 0xEA61), // CP2101
    (0x10C4, 0xEA63), // CP2102N
    (0x10C4, 0xEA70), // CP2105
    (0x10C4, 0xEA71), // CP2108
    (0x1A86, 0x5523), // CH341 in serial mode
    (0x1A86, 0x55D2), // CH9102
    (0x1A86, 0x55D3), // CH343
    (0x1A86, 0x55D4), // CH9102F
    (0x1A86, 0x55D5), // CH344
    (0x1A86, 0x7522), // CH340
    (0x1A86, 0x7523), // CH340
    (0x1A86, 0x7584), // CH340S
    (0x4348, 0x5523), // CH341
];

/// Whether the pair is a stock bridge rather than a board.
pub fn is_generic_bridge(vid: u16, pid: u16) -> bool {
    GENERIC_BRIDGE_IDS.contains(&(vid, pid))
}

/// What the board definitions say claims this pair, if anything.
pub fn board_for_usb_id(vid: u16, pid: u16) -> Option<&'static BoardId> {
    if is_generic_bridge(vid, pid) {
        return None;
    }
    let key = (vid as u32) << 16 | pid as u32;
    let at = BOARD_IDS
        .binary_search_by_key(&key, |entry| entry.key)
        .ok()?;
    BOARD_IDS.get(at)
}

/// The board an instance id names, where its descriptors name one.
///
/// `None` is the normal answer: most devices are bridges, hubs, or boards no
/// definition claims. It means "nothing to say here", never "not a board".
pub fn identify(instance_id: &InstanceId) -> Option<TargetIdentity> {
    let (vid, pid) = instance_id.vid_pid()?;
    let board = board_for_usb_id(vid, pid)?;
    // A pair several boards share names the family but not the board, and a
    // name every unit of a model shares is not a name.
    let variant = board.variant?;
    let serial = instance_id.unit.serial()?;

    // A serial too short to be an identity is no better than none. The same
    // rule the probes are held to (§4.5), applied to the same value.
    let identity_key = identity_key(variant, serial).ok()?;

    let mut details = BTreeMap::new();
    details.insert("family".into(), board.family.to_owned());
    Some(TargetIdentity {
        // The board's own family, which is what the table knows and what a
        // reader wants: `renesas`, not the name of the lookup that found it.
        family: board.family,
        identity_key,
        device_id: serial.to_owned(),
        device_type: variant.to_owned(),
        hardware_revision: None,
        id_source: id_sources::USB_SERIAL,
        details,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn identity_for(raw: &str) -> Option<TargetIdentity> {
        identify(&InstanceId::parse(raw))
    }

    #[test]
    fn the_table_is_sorted_so_the_search_works() {
        assert!(
            BOARD_IDS.windows(2).all(|pair| pair[0].key < pair[1].key),
            "board_ids.rs must be sorted and free of repeats"
        );
    }

    #[test]
    fn a_board_with_its_own_pair_and_a_serial_is_named() {
        // Arduino Nano ESP32 is out (Espressif); this is a UNO R4 Minima.
        let found = identity_for("USB\\VID_2341&PID_0069\\34B7DA65B1C8").expect("named");
        assert_eq!(found.device_type, "arduino-uno-r4-minima");
        assert_eq!(found.device_id, "34B7DA65B1C8");
        assert_eq!(found.identity_key, "arduino-uno-r4-minima-34b7da65b1c8");
        assert_eq!(found.id_source, id_sources::USB_SERIAL);
    }

    #[test]
    fn a_stock_bridge_is_never_a_board() {
        // A CH340 with a serial number is still a CH340: the serial belongs to
        // the cable, and what is on the far end of it is not in the descriptors.
        assert!(identity_for("USB\\VID_1A86&PID_7523\\5B5F090816").is_none());
        assert!(is_generic_bridge(0x10C4, 0xEA60));
        assert!(board_for_usb_id(0x10C4, 0xEA60).is_none());
    }

    #[test]
    fn a_pair_several_boards_share_names_no_board() {
        // 0483:3748 is an ST-LINK, claimed by the whole stm32 platform.
        let board = board_for_usb_id(0x0483, 0x3748).expect("in the table");
        assert_eq!(board.family, "stm32");
        assert!(board.variant.is_none());
        assert!(identity_for("USB\\VID_0483&PID_3748\\066BFF3837334D4643133213").is_none());
    }

    #[test]
    fn without_a_serial_number_the_model_is_known_but_the_unit_is_not() {
        // Port-derived third element: no serial, so nothing pins the unit down.
        assert!(identity_for("USB\\VID_2341&PID_0069\\7&19033BE6&0&1").is_none());
    }

    #[test]
    fn a_serial_too_short_to_identify_is_refused() {
        // Five usable characters, under the floor the identity keys set.
        assert!(identity_for("USB\\VID_2341&PID_0069\\AB12").is_none());
    }

    #[test]
    fn espressif_pairs_were_left_out_of_the_table() {
        // 303a:1001 is the ESP32-S3 native USB. The eFuse probe owns these.
        assert!(board_for_usb_id(0x303A, 0x1001).is_none());
    }
}
