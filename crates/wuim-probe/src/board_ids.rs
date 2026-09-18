//! VID/PID pairs that name a board on their own.
//!
//! **Generated. Do not edit by hand** — see `scripts/generate_board_ids.py`.
//!
//! Source: board-identify's `arduino_ids.py` (825 pairs), itself merged
//! from a published dump of `arduino-cli board details`. Copied rather than
//! re-derived so that both tools give one board one name.
//!
//! Espressif pairs are left out: those boards are identified from the eFuse MAC
//! instead, which comes from the silicon rather than from a descriptor a
//! reflash can change.

/// One VID/PID pair, and what the board definitions say claims it.
pub struct BoardId {
    /// `vid << 16 | pid`, which is what the table is sorted and searched on.
    pub key: u32,
    /// The family, e.g. `rp2040`. Known even where the board is not.
    pub family: &'static str,
    /// `None` when several boards report this pair, so no one name is right.
    pub variant: Option<&'static str>,
}

/// Sorted by `key`, for [`slice::binary_search_by_key`].
pub static BOARD_IDS: &[BoardId] = &[
    BoardId {
        key: 0x03EB2145,
        family: "avr",
        variant: Some("arduino-uno-wifi-rev2"),
    },
    BoardId {
        key: 0x04833744,
        family: "stm32",
        variant: None,
    },
    BoardId {
        key: 0x04833748,
        family: "stm32",
        variant: None,
    },
    BoardId {
        key: 0x0483374B,
        family: "stm32",
        variant: None,
    },
    BoardId {
        key: 0x0483374E,
        family: "stm32",
        variant: None,
    },
    BoardId {
        key: 0x0483374F,
        family: "stm32",
        variant: None,
    },
    BoardId {
        key: 0x04833752,
        family: "stm32",
        variant: None,
    },
    BoardId {
        key: 0x04833753,
        family: "stm32",
        variant: None,
    },
    BoardId {
        key: 0x04835740,
        family: "stm32",
        variant: None,
    },
    BoardId {
        key: 0x0D280204,
        family: "stm32",
        variant: Some("steami-board"),
    },
    BoardId {
        key: 0x12092008,
        family: "rp2040",
        variant: Some("picolume-transceiver"),
    },
    BoardId {
        key: 0x12092108,
        family: "rp2040",
        variant: Some("picolume-transceiver"),
    },
    BoardId {
        key: 0x12096008,
        family: "rp2040",
        variant: Some("picolume-transceiver"),
    },
    BoardId {
        key: 0x12096108,
        family: "rp2040",
        variant: Some("picolume-transceiver"),
    },
    BoardId {
        key: 0x1209A008,
        family: "rp2040",
        variant: Some("picolume-transceiver"),
    },
    BoardId {
        key: 0x1209A108,
        family: "rp2040",
        variant: Some("picolume-transceiver"),
    },
    BoardId {
        key: 0x1209A182,
        family: "rp2040",
        variant: Some("solder-party-rp2040-stamp"),
    },
    BoardId {
        key: 0x1209A183,
        family: "rp2040",
        variant: Some("solder-party-rp2350-stamp"),
    },
    BoardId {
        key: 0x1209A184,
        family: "rp2040",
        variant: Some("solder-party-rp2350-stamp-xl"),
    },
    BoardId {
        key: 0x1209CA01,
        family: "zephyr",
        variant: Some("arduino-ventuno-q"),
    },
    BoardId {
        key: 0x1209CB74,
        family: "rp2040",
        variant: Some("0xcb-helios"),
    },
    BoardId {
        key: 0x1209E008,
        family: "rp2040",
        variant: Some("picolume-transceiver"),
    },
    BoardId {
        key: 0x1209E108,
        family: "rp2040",
        variant: Some("picolume-transceiver"),
    },
    BoardId {
        key: 0x1209E182,
        family: "rp2040",
        variant: Some("solder-party-rp2040-stamp"),
    },
    BoardId {
        key: 0x1209E183,
        family: "rp2040",
        variant: Some("solder-party-rp2350-stamp"),
    },
    BoardId {
        key: 0x1209E184,
        family: "rp2040",
        variant: Some("solder-party-rp2350-stamp-xl"),
    },
    BoardId {
        key: 0x1209F502,
        family: "rp2040",
        variant: Some("silicognition-rp2040-shim"),
    },
    BoardId {
        key: 0x15BA0026,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x15BA0126,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x15BA4026,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x15BA4126,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x15BA8026,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x15BA8126,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x15BAC026,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x15BAC126,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4F0026,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4F0038,
        family: "rp2040",
        variant: Some("sparkfun-thing-plus-rp2350"),
    },
    BoardId {
        key: 0x1B4F0044,
        family: "rp2040",
        variant: Some("sparkfun-iot-node-lorawan"),
    },
    BoardId {
        key: 0x1B4F0045,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller-beta"),
    },
    BoardId {
        key: 0x1B4F0046,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller"),
    },
    BoardId {
        key: 0x1B4F0047,
        family: "rp2040",
        variant: Some("sparkfun-iot-redboard-rp2350"),
    },
    BoardId {
        key: 0x1B4F0126,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4F0138,
        family: "rp2040",
        variant: Some("sparkfun-thing-plus-rp2350"),
    },
    BoardId {
        key: 0x1B4F0144,
        family: "rp2040",
        variant: Some("sparkfun-iot-node-lorawan"),
    },
    BoardId {
        key: 0x1B4F0145,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller-beta"),
    },
    BoardId {
        key: 0x1B4F0146,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller"),
    },
    BoardId {
        key: 0x1B4F0147,
        family: "rp2040",
        variant: Some("sparkfun-iot-redboard-rp2350"),
    },
    BoardId {
        key: 0x1B4F4026,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4F4038,
        family: "rp2040",
        variant: Some("sparkfun-thing-plus-rp2350"),
    },
    BoardId {
        key: 0x1B4F4044,
        family: "rp2040",
        variant: Some("sparkfun-iot-node-lorawan"),
    },
    BoardId {
        key: 0x1B4F4045,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller-beta"),
    },
    BoardId {
        key: 0x1B4F4046,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller"),
    },
    BoardId {
        key: 0x1B4F4047,
        family: "rp2040",
        variant: Some("sparkfun-iot-redboard-rp2350"),
    },
    BoardId {
        key: 0x1B4F4126,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4F4138,
        family: "rp2040",
        variant: Some("sparkfun-thing-plus-rp2350"),
    },
    BoardId {
        key: 0x1B4F4144,
        family: "rp2040",
        variant: Some("sparkfun-iot-node-lorawan"),
    },
    BoardId {
        key: 0x1B4F4145,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller-beta"),
    },
    BoardId {
        key: 0x1B4F4146,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller"),
    },
    BoardId {
        key: 0x1B4F4147,
        family: "rp2040",
        variant: Some("sparkfun-iot-redboard-rp2350"),
    },
    BoardId {
        key: 0x1B4F8026,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4F8038,
        family: "rp2040",
        variant: Some("sparkfun-thing-plus-rp2350"),
    },
    BoardId {
        key: 0x1B4F8044,
        family: "rp2040",
        variant: Some("sparkfun-iot-node-lorawan"),
    },
    BoardId {
        key: 0x1B4F8045,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller-beta"),
    },
    BoardId {
        key: 0x1B4F8046,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller"),
    },
    BoardId {
        key: 0x1B4F8047,
        family: "rp2040",
        variant: Some("sparkfun-iot-redboard-rp2350"),
    },
    BoardId {
        key: 0x1B4F8126,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4F8138,
        family: "rp2040",
        variant: Some("sparkfun-thing-plus-rp2350"),
    },
    BoardId {
        key: 0x1B4F8144,
        family: "rp2040",
        variant: Some("sparkfun-iot-node-lorawan"),
    },
    BoardId {
        key: 0x1B4F8145,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller-beta"),
    },
    BoardId {
        key: 0x1B4F8146,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller"),
    },
    BoardId {
        key: 0x1B4F8147,
        family: "rp2040",
        variant: Some("sparkfun-iot-redboard-rp2350"),
    },
    BoardId {
        key: 0x1B4F9207,
        family: "avr",
        variant: Some("lilypad-arduino-usb"),
    },
    BoardId {
        key: 0x1B4F9208,
        family: "avr",
        variant: Some("lilypad-arduino-usb"),
    },
    BoardId {
        key: 0x1B4FC026,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4FC038,
        family: "rp2040",
        variant: Some("sparkfun-thing-plus-rp2350"),
    },
    BoardId {
        key: 0x1B4FC044,
        family: "rp2040",
        variant: Some("sparkfun-iot-node-lorawan"),
    },
    BoardId {
        key: 0x1B4FC045,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller-beta"),
    },
    BoardId {
        key: 0x1B4FC046,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller"),
    },
    BoardId {
        key: 0x1B4FC047,
        family: "rp2040",
        variant: Some("sparkfun-iot-redboard-rp2350"),
    },
    BoardId {
        key: 0x1B4FC126,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1B4FC138,
        family: "rp2040",
        variant: Some("sparkfun-thing-plus-rp2350"),
    },
    BoardId {
        key: 0x1B4FC144,
        family: "rp2040",
        variant: Some("sparkfun-iot-node-lorawan"),
    },
    BoardId {
        key: 0x1B4FC145,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller-beta"),
    },
    BoardId {
        key: 0x1B4FC146,
        family: "rp2040",
        variant: Some("sparkfun-xrp-controller"),
    },
    BoardId {
        key: 0x1B4FC147,
        family: "rp2040",
        variant: Some("sparkfun-iot-redboard-rp2350"),
    },
    BoardId {
        key: 0x1D50ACAB,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1D50ADAB,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1D50ECAB,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x1D50EDAB,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x23410001,
        family: "avr",
        variant: Some("arduino-uno"),
    },
    BoardId {
        key: 0x23410010,
        family: "avr",
        variant: Some("arduino-mega-or-mega-2560"),
    },
    BoardId {
        key: 0x23410036,
        family: "avr",
        variant: Some("arduino-leonardo"),
    },
    BoardId {
        key: 0x23410037,
        family: "avr",
        variant: Some("arduino-micro"),
    },
    BoardId {
        key: 0x23410038,
        family: "avr",
        variant: Some("arduino-robot-control"),
    },
    BoardId {
        key: 0x23410039,
        family: "avr",
        variant: Some("arduino-robot-motor"),
    },
    BoardId {
        key: 0x2341003C,
        family: "avr",
        variant: Some("arduino-esplora"),
    },
    BoardId {
        key: 0x2341003F,
        family: "avr",
        variant: Some("arduino-mega-adk"),
    },
    BoardId {
        key: 0x23410041,
        family: "avr",
        variant: Some("arduino-yun"),
    },
    BoardId {
        key: 0x23410042,
        family: "avr",
        variant: Some("arduino-mega-or-mega-2560"),
    },
    BoardId {
        key: 0x23410043,
        family: "avr",
        variant: Some("arduino-uno"),
    },
    BoardId {
        key: 0x23410044,
        family: "avr",
        variant: Some("arduino-mega-adk"),
    },
    BoardId {
        key: 0x23410058,
        family: "avr",
        variant: Some("arduino-nano-every"),
    },
    BoardId {
        key: 0x2341005E,
        family: "rp2040",
        variant: Some("arduino-nano-rp2040-connect"),
    },
    BoardId {
        key: 0x2341005F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x23410062,
        family: "avr",
        variant: Some("arduino-uno-mini"),
    },
    BoardId {
        key: 0x23410069,
        family: "renesas",
        variant: Some("arduino-uno-r4-minima"),
    },
    BoardId {
        key: 0x2341006A,
        family: "avr",
        variant: Some("arduino-uno"),
    },
    BoardId {
        key: 0x2341006D,
        family: "renesas",
        variant: Some("arduino-uno-r4-wifi"),
    },
    BoardId {
        key: 0x23410074,
        family: "renesas",
        variant: Some("arduino-nano-r4"),
    },
    BoardId {
        key: 0x23410078,
        family: "zephyr",
        variant: Some("arduino-uno-q"),
    },
    BoardId {
        key: 0x2341007A,
        family: "zephyr",
        variant: Some("arduino-ventuno-q"),
    },
    BoardId {
        key: 0x2341015E,
        family: "rp2040",
        variant: Some("arduino-nano-rp2040-connect"),
    },
    BoardId {
        key: 0x2341015F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x23410210,
        family: "avr",
        variant: Some("arduino-mega-or-mega-2560"),
    },
    BoardId {
        key: 0x23410237,
        family: "avr",
        variant: Some("arduino-micro"),
    },
    BoardId {
        key: 0x23410242,
        family: "avr",
        variant: Some("arduino-mega-or-mega-2560"),
    },
    BoardId {
        key: 0x23410243,
        family: "avr",
        variant: Some("arduino-uno"),
    },
    BoardId {
        key: 0x2341025E,
        family: "rp2040",
        variant: Some("arduino-nano-rp2040-connect"),
    },
    BoardId {
        key: 0x2341025F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x23410369,
        family: "renesas",
        variant: Some("arduino-uno-r4-minima"),
    },
    BoardId {
        key: 0x23410374,
        family: "renesas",
        variant: Some("arduino-nano-r4"),
    },
    BoardId {
        key: 0x23410C9F,
        family: "avr",
        variant: Some("arduino-gemma"),
    },
    BoardId {
        key: 0x23411002,
        family: "renesas",
        variant: Some("arduino-uno-r4-wifi"),
    },
    BoardId {
        key: 0x23418036,
        family: "avr",
        variant: Some("arduino-leonardo"),
    },
    BoardId {
        key: 0x23418037,
        family: "avr",
        variant: Some("arduino-micro"),
    },
    BoardId {
        key: 0x23418038,
        family: "avr",
        variant: Some("arduino-robot-control"),
    },
    BoardId {
        key: 0x23418039,
        family: "avr",
        variant: Some("arduino-robot-motor"),
    },
    BoardId {
        key: 0x2341803C,
        family: "avr",
        variant: Some("arduino-esplora"),
    },
    BoardId {
        key: 0x23418041,
        family: "avr",
        variant: Some("arduino-yun"),
    },
    BoardId {
        key: 0x2341805E,
        family: "rp2040",
        variant: Some("arduino-nano-rp2040-connect"),
    },
    BoardId {
        key: 0x2341805F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x23418237,
        family: "avr",
        variant: Some("arduino-micro"),
    },
    BoardId {
        key: 0x239A8011,
        family: "avr",
        variant: Some("adafruit-circuit-playground"),
    },
    BoardId {
        key: 0x239A80E3,
        family: "rp2040",
        variant: Some("adafruit-stemma-friend-rp2040"),
    },
    BoardId {
        key: 0x239A80F1,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040"),
    },
    BoardId {
        key: 0x239A80F7,
        family: "rp2040",
        variant: Some("adafruit-qt-py-rp2040"),
    },
    BoardId {
        key: 0x239A80FD,
        family: "rp2040",
        variant: Some("adafruit-itsybitsy-rp2040"),
    },
    BoardId {
        key: 0x239A8105,
        family: "rp2040",
        variant: Some("adafruit-kb2040"),
    },
    BoardId {
        key: 0x239A8107,
        family: "rp2040",
        variant: Some("adafruit-macropad-rp2040"),
    },
    BoardId {
        key: 0x239A8109,
        family: "rp2040",
        variant: Some("adafruit-trinkey-rp2040-qt"),
    },
    BoardId {
        key: 0x239A8121,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-scorpio"),
    },
    BoardId {
        key: 0x239A8127,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-dvi"),
    },
    BoardId {
        key: 0x239A8129,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-usb-host"),
    },
    BoardId {
        key: 0x239A812B,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-thinkink"),
    },
    BoardId {
        key: 0x239A812D,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-rfm"),
    },
    BoardId {
        key: 0x239A812F,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-can"),
    },
    BoardId {
        key: 0x239A8131,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-prop-maker"),
    },
    BoardId {
        key: 0x239A813D,
        family: "rp2040",
        variant: Some("adafruit-metro-rp2040"),
    },
    BoardId {
        key: 0x239A814D,
        family: "rp2040",
        variant: Some("adafruit-metro-rp2350"),
    },
    BoardId {
        key: 0x239A814F,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2350-hstx"),
    },
    BoardId {
        key: 0x239A8151,
        family: "rp2040",
        variant: Some("adafruit-floppsy"),
    },
    BoardId {
        key: 0x239A815D,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-adalogger"),
    },
    BoardId {
        key: 0x239A816B,
        family: "rp2040",
        variant: Some("adafruit-fruit-jam-rp2350"),
    },
    BoardId {
        key: 0x239A816D,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2350-adalogger"),
    },
    BoardId {
        key: 0x239A81E3,
        family: "rp2040",
        variant: Some("adafruit-stemma-friend-rp2040"),
    },
    BoardId {
        key: 0x239A81F1,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040"),
    },
    BoardId {
        key: 0x239A81F7,
        family: "rp2040",
        variant: Some("adafruit-qt-py-rp2040"),
    },
    BoardId {
        key: 0x239A81FD,
        family: "rp2040",
        variant: Some("adafruit-itsybitsy-rp2040"),
    },
    BoardId {
        key: 0x239AC0E3,
        family: "rp2040",
        variant: Some("adafruit-stemma-friend-rp2040"),
    },
    BoardId {
        key: 0x239AC0F1,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040"),
    },
    BoardId {
        key: 0x239AC0F7,
        family: "rp2040",
        variant: Some("adafruit-qt-py-rp2040"),
    },
    BoardId {
        key: 0x239AC0FD,
        family: "rp2040",
        variant: Some("adafruit-itsybitsy-rp2040"),
    },
    BoardId {
        key: 0x239AC105,
        family: "rp2040",
        variant: Some("adafruit-kb2040"),
    },
    BoardId {
        key: 0x239AC107,
        family: "rp2040",
        variant: Some("adafruit-macropad-rp2040"),
    },
    BoardId {
        key: 0x239AC109,
        family: "rp2040",
        variant: Some("adafruit-trinkey-rp2040-qt"),
    },
    BoardId {
        key: 0x239AC121,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-scorpio"),
    },
    BoardId {
        key: 0x239AC127,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-dvi"),
    },
    BoardId {
        key: 0x239AC129,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-usb-host"),
    },
    BoardId {
        key: 0x239AC12B,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-thinkink"),
    },
    BoardId {
        key: 0x239AC12D,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-rfm"),
    },
    BoardId {
        key: 0x239AC12F,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-can"),
    },
    BoardId {
        key: 0x239AC131,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-prop-maker"),
    },
    BoardId {
        key: 0x239AC13D,
        family: "rp2040",
        variant: Some("adafruit-metro-rp2040"),
    },
    BoardId {
        key: 0x239AC14D,
        family: "rp2040",
        variant: Some("adafruit-metro-rp2350"),
    },
    BoardId {
        key: 0x239AC14F,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2350-hstx"),
    },
    BoardId {
        key: 0x239AC151,
        family: "rp2040",
        variant: Some("adafruit-floppsy"),
    },
    BoardId {
        key: 0x239AC15D,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040-adalogger"),
    },
    BoardId {
        key: 0x239AC16B,
        family: "rp2040",
        variant: Some("adafruit-fruit-jam-rp2350"),
    },
    BoardId {
        key: 0x239AC16D,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2350-adalogger"),
    },
    BoardId {
        key: 0x239AC1E3,
        family: "rp2040",
        variant: Some("adafruit-stemma-friend-rp2040"),
    },
    BoardId {
        key: 0x239AC1F1,
        family: "rp2040",
        variant: Some("adafruit-feather-rp2040"),
    },
    BoardId {
        key: 0x239AC1F7,
        family: "rp2040",
        variant: Some("adafruit-qt-py-rp2040"),
    },
    BoardId {
        key: 0x239AC1FD,
        family: "rp2040",
        variant: Some("adafruit-itsybitsy-rp2040"),
    },
    BoardId {
        key: 0x27707303,
        family: "rp2040",
        variant: Some("amken-bunnyboard"),
    },
    BoardId {
        key: 0x27707304,
        family: "rp2040",
        variant: Some("amken-revelop"),
    },
    BoardId {
        key: 0x27707305,
        family: "rp2040",
        variant: Some("amken-revelop-plus"),
    },
    BoardId {
        key: 0x27707306,
        family: "rp2040",
        variant: Some("amken-revelop-es"),
    },
    BoardId {
        key: 0x28860000,
        family: "samd",
        variant: Some("seeeduino-grove-ui-wireles-samd51"),
    },
    BoardId {
        key: 0x2886000B,
        family: "samd",
        variant: Some("seeeduino-zero"),
    },
    BoardId {
        key: 0x2886000C,
        family: "samd",
        variant: Some("seeeduino-lorawan"),
    },
    BoardId {
        key: 0x2886000E,
        family: "samd",
        variant: Some("seeeduino-wio-gps-board"),
    },
    BoardId {
        key: 0x2886002A,
        family: "samd",
        variant: Some("seeeduino-wio-lite-mg126"),
    },
    BoardId {
        key: 0x2886002B,
        family: "imxrt",
        variant: Some("seeeduino-arch-mix"),
    },
    BoardId {
        key: 0x2886002C,
        family: "samd",
        variant: Some("seeeduino-femto"),
    },
    BoardId {
        key: 0x2886002D,
        family: "samd",
        variant: Some("seeeduino-wio-terminal"),
    },
    BoardId {
        key: 0x2886002F,
        family: "samd",
        variant: Some("seeeduino-xiao"),
    },
    BoardId {
        key: 0x28860031,
        family: "samd",
        variant: Some("wio-lte-cat-1"),
    },
    BoardId {
        key: 0x2886003F,
        family: "samd",
        variant: Some("seeed-xiao-m0-plus"),
    },
    BoardId {
        key: 0x28860044,
        family: "nrf52",
        variant: Some("seeed-xiao-nrf52840"),
    },
    BoardId {
        key: 0x28860045,
        family: "nrf52",
        variant: None,
    },
    BoardId {
        key: 0x28860049,
        family: "renesas",
        variant: Some("xiao-ra4m1"),
    },
    BoardId {
        key: 0x28860050,
        family: "rp2040",
        variant: Some("seeed-indicator-rp2040"),
    },
    BoardId {
        key: 0x28860055,
        family: "nrf52",
        variant: Some("seeed-wio-tracker-1110"),
    },
    BoardId {
        key: 0x28860057,
        family: "nrf52",
        variant: Some("seeed-tracker-t1000-e-lorawan"),
    },
    BoardId {
        key: 0x28860058,
        family: "rp2040",
        variant: Some("seeed-xiao-rp2350"),
    },
    BoardId {
        key: 0x28860064,
        family: "nrf52",
        variant: None,
    },
    BoardId {
        key: 0x28860065,
        family: "nrf52",
        variant: None,
    },
    BoardId {
        key: 0x28860145,
        family: "mbed",
        variant: None,
    },
    BoardId {
        key: 0x28860150,
        family: "rp2040",
        variant: Some("seeed-indicator-rp2040"),
    },
    BoardId {
        key: 0x28860158,
        family: "rp2040",
        variant: Some("seeed-xiao-rp2350"),
    },
    BoardId {
        key: 0x28860164,
        family: "mbed",
        variant: Some("xiao-nrf52840-plus-no-updates"),
    },
    BoardId {
        key: 0x28860165,
        family: "mbed",
        variant: Some("xiao-nrf52840-sense-plus-no-updates"),
    },
    BoardId {
        key: 0x28864050,
        family: "rp2040",
        variant: Some("seeed-indicator-rp2040"),
    },
    BoardId {
        key: 0x28864058,
        family: "rp2040",
        variant: Some("seeed-xiao-rp2350"),
    },
    BoardId {
        key: 0x28864150,
        family: "rp2040",
        variant: Some("seeed-indicator-rp2040"),
    },
    BoardId {
        key: 0x28864158,
        family: "rp2040",
        variant: Some("seeed-xiao-rp2350"),
    },
    BoardId {
        key: 0x2886800B,
        family: "samd",
        variant: Some("seeeduino-zero"),
    },
    BoardId {
        key: 0x2886800C,
        family: "samd",
        variant: Some("seeeduino-lorawan"),
    },
    BoardId {
        key: 0x2886800E,
        family: "samd",
        variant: Some("seeeduino-wio-gps-board"),
    },
    BoardId {
        key: 0x2886802A,
        family: "samd",
        variant: Some("seeeduino-wio-lite-mg126"),
    },
    BoardId {
        key: 0x2886802B,
        family: "imxrt",
        variant: Some("seeeduino-arch-mix"),
    },
    BoardId {
        key: 0x2886802C,
        family: "samd",
        variant: Some("seeeduino-femto"),
    },
    BoardId {
        key: 0x2886802D,
        family: "samd",
        variant: Some("seeeduino-wio-terminal"),
    },
    BoardId {
        key: 0x2886802F,
        family: "samd",
        variant: Some("seeeduino-xiao"),
    },
    BoardId {
        key: 0x28868031,
        family: "samd",
        variant: Some("wio-lte-cat-1"),
    },
    BoardId {
        key: 0x2886803F,
        family: "samd",
        variant: Some("seeed-xiao-m0-plus"),
    },
    BoardId {
        key: 0x28868044,
        family: "nrf52",
        variant: Some("seeed-xiao-nrf52840"),
    },
    BoardId {
        key: 0x28868045,
        family: "nrf52",
        variant: None,
    },
    BoardId {
        key: 0x28868049,
        family: "renesas",
        variant: Some("xiao-ra4m1"),
    },
    BoardId {
        key: 0x28868050,
        family: "rp2040",
        variant: Some("seeed-indicator-rp2040"),
    },
    BoardId {
        key: 0x28868055,
        family: "nrf52",
        variant: Some("seeed-wio-tracker-1110"),
    },
    BoardId {
        key: 0x28868057,
        family: "nrf52",
        variant: Some("seeed-tracker-t1000-e-lorawan"),
    },
    BoardId {
        key: 0x28868058,
        family: "rp2040",
        variant: Some("seeed-xiao-rp2350"),
    },
    BoardId {
        key: 0x28868064,
        family: "nrf52",
        variant: None,
    },
    BoardId {
        key: 0x28868065,
        family: "nrf52",
        variant: None,
    },
    BoardId {
        key: 0x28868150,
        family: "rp2040",
        variant: Some("seeed-indicator-rp2040"),
    },
    BoardId {
        key: 0x28868158,
        family: "rp2040",
        variant: Some("seeed-xiao-rp2350"),
    },
    BoardId {
        key: 0x2886C050,
        family: "rp2040",
        variant: Some("seeed-indicator-rp2040"),
    },
    BoardId {
        key: 0x2886C058,
        family: "rp2040",
        variant: Some("seeed-xiao-rp2350"),
    },
    BoardId {
        key: 0x2886C150,
        family: "rp2040",
        variant: Some("seeed-indicator-rp2040"),
    },
    BoardId {
        key: 0x2886C158,
        family: "rp2040",
        variant: Some("seeed-xiao-rp2350"),
    },
    BoardId {
        key: 0x2A030001,
        family: "avr",
        variant: Some("linino-one"),
    },
    BoardId {
        key: 0x2A030010,
        family: "avr",
        variant: Some("arduino-mega-or-mega-2560"),
    },
    BoardId {
        key: 0x2A030036,
        family: "avr",
        variant: Some("arduino-leonardo"),
    },
    BoardId {
        key: 0x2A030037,
        family: "avr",
        variant: Some("arduino-micro"),
    },
    BoardId {
        key: 0x2A030038,
        family: "avr",
        variant: Some("arduino-robot-control"),
    },
    BoardId {
        key: 0x2A030039,
        family: "avr",
        variant: Some("arduino-robot-motor"),
    },
    BoardId {
        key: 0x2A03003C,
        family: "avr",
        variant: Some("arduino-esplora"),
    },
    BoardId {
        key: 0x2A03003F,
        family: "avr",
        variant: Some("arduino-mega-adk"),
    },
    BoardId {
        key: 0x2A030040,
        family: "avr",
        variant: Some("arduino-leonardo-eth"),
    },
    BoardId {
        key: 0x2A030041,
        family: "avr",
        variant: Some("arduino-yun"),
    },
    BoardId {
        key: 0x2A030042,
        family: "avr",
        variant: Some("arduino-mega-or-mega-2560"),
    },
    BoardId {
        key: 0x2A030043,
        family: "avr",
        variant: Some("arduino-uno"),
    },
    BoardId {
        key: 0x2A030044,
        family: "avr",
        variant: Some("arduino-mega-adk"),
    },
    BoardId {
        key: 0x2A030050,
        family: "avr",
        variant: Some("arduino-yun-mini"),
    },
    BoardId {
        key: 0x2A030056,
        family: "avr",
        variant: Some("arduino-industrial-101"),
    },
    BoardId {
        key: 0x2A030057,
        family: "avr",
        variant: Some("arduino-uno-wifi"),
    },
    BoardId {
        key: 0x2A038001,
        family: "avr",
        variant: Some("linino-one"),
    },
    BoardId {
        key: 0x2A038036,
        family: "avr",
        variant: Some("arduino-leonardo"),
    },
    BoardId {
        key: 0x2A038037,
        family: "avr",
        variant: Some("arduino-micro"),
    },
    BoardId {
        key: 0x2A038038,
        family: "avr",
        variant: Some("arduino-robot-control"),
    },
    BoardId {
        key: 0x2A038039,
        family: "avr",
        variant: Some("arduino-robot-motor"),
    },
    BoardId {
        key: 0x2A03803C,
        family: "avr",
        variant: Some("arduino-esplora"),
    },
    BoardId {
        key: 0x2A038040,
        family: "avr",
        variant: Some("arduino-leonardo-eth"),
    },
    BoardId {
        key: 0x2A038041,
        family: "avr",
        variant: Some("arduino-yun"),
    },
    BoardId {
        key: 0x2A038050,
        family: "avr",
        variant: Some("arduino-yun-mini"),
    },
    BoardId {
        key: 0x2A038056,
        family: "avr",
        variant: Some("arduino-industrial-101"),
    },
    BoardId {
        key: 0x2E8A0003,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A000A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A000F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A008A,
        family: "rp2040",
        variant: Some("deruilab-flyboard2040core"),
    },
    BoardId {
        key: 0x2E8A00C0,
        family: "rp2040",
        variant: Some("rakwireless-rak11300"),
    },
    BoardId {
        key: 0x2E8A0103,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A010A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A010F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A018A,
        family: "rp2040",
        variant: Some("deruilab-flyboard2040core"),
    },
    BoardId {
        key: 0x2E8A01C0,
        family: "rp2040",
        variant: Some("rakwireless-rak11300"),
    },
    BoardId {
        key: 0x2E8A1000,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A1005,
        family: "rp2040",
        variant: Some("melopero-shake-rp2040"),
    },
    BoardId {
        key: 0x2E8A1006,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A1007,
        family: "rp2040",
        variant: Some("upesy-rp2040-devkit"),
    },
    BoardId {
        key: 0x2E8A1008,
        family: "rp2040",
        variant: Some("pimoroni-pga2040"),
    },
    BoardId {
        key: 0x2E8A100A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A100B,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A100D,
        family: "rp2040",
        variant: Some("ilabs-challenger-nb-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A100F,
        family: "rp2040",
        variant: Some("cytron-maker-nano-rp2040"),
    },
    BoardId {
        key: 0x2E8A1010,
        family: "rp2040",
        variant: Some("ilabs-rpico32"),
    },
    BoardId {
        key: 0x2E8A1011,
        family: "rp2040",
        variant: Some("melopero-cookie-rp2040"),
    },
    BoardId {
        key: 0x2E8A1018,
        family: "rp2040",
        variant: Some("pimoroni-pga2350"),
    },
    BoardId {
        key: 0x2E8A1020,
        family: "rp2040",
        variant: Some("waveshare-rp2040-plus"),
    },
    BoardId {
        key: 0x2E8A1021,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A1023,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-lora"),
    },
    BoardId {
        key: 0x2E8A1027,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A1028,
        family: "rp2040",
        variant: Some("wiznet-wizfi360-evb-pico"),
    },
    BoardId {
        key: 0x2E8A1029,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A102C,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A102D,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-sd-rtc"),
    },
    BoardId {
        key: 0x2E8A1032,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-subghz"),
    },
    BoardId {
        key: 0x2E8A1036,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-nfc"),
    },
    BoardId {
        key: 0x2E8A1037,
        family: "rp2040",
        variant: Some("electroniccats-huntercat-nfc-rp2040"),
    },
    BoardId {
        key: 0x2E8A1039,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-1-28"),
    },
    BoardId {
        key: 0x2E8A103A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A1041,
        family: "rp2040",
        variant: Some("bridgetek-idm2040-7a"),
    },
    BoardId {
        key: 0x2E8A1043,
        family: "rp2040",
        variant: Some("newsan-archi"),
    },
    BoardId {
        key: 0x2E8A1052,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-uwb"),
    },
    BoardId {
        key: 0x2E8A105E,
        family: "rp2040",
        variant: Some("breadstick-raspberry"),
    },
    BoardId {
        key: 0x2E8A105F,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi6-ble"),
    },
    BoardId {
        key: 0x2E8A106F,
        family: "rp2040",
        variant: Some("l-atelier-d-arnoz-dudescab"),
    },
    BoardId {
        key: 0x2E8A1071,
        family: "rp2040",
        variant: Some("cytron-maker-uno-rp2040"),
    },
    BoardId {
        key: 0x2E8A107B,
        family: "rp2040",
        variant: Some("ilabs-connectivity-2040-lte-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A1093,
        family: "rp2040",
        variant: Some("cytron-iriv-io-controller"),
    },
    BoardId {
        key: 0x2E8A1096,
        family: "rp2040",
        variant: Some("cytron-motion-2350-pro"),
    },
    BoardId {
        key: 0x2E8A109A,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A109B,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-bconnect"),
    },
    BoardId {
        key: 0x2E8A10A5,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A10AE,
        family: "rp2040",
        variant: Some("datanoisetv-picoadk-v2"),
    },
    BoardId {
        key: 0x2E8A10B0,
        family: "rp2040",
        variant: Some("waveshare-rp2350-zero"),
    },
    BoardId {
        key: 0x2E8A10B1,
        family: "rp2040",
        variant: Some("waveshare-rp2350-plus"),
    },
    BoardId {
        key: 0x2E8A10B7,
        family: "rp2040",
        variant: Some("waveshare-rp2350-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A10C0,
        family: "rp2040",
        variant: Some("pimoroni-explorer"),
    },
    BoardId {
        key: 0x2E8A10EC,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-rp2350"),
    },
    BoardId {
        key: 0x2E8A1100,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A1105,
        family: "rp2040",
        variant: Some("melopero-shake-rp2040"),
    },
    BoardId {
        key: 0x2E8A1106,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A1107,
        family: "rp2040",
        variant: Some("upesy-rp2040-devkit"),
    },
    BoardId {
        key: 0x2E8A1108,
        family: "rp2040",
        variant: Some("pimoroni-pga2040"),
    },
    BoardId {
        key: 0x2E8A110A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A110B,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A110D,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A110F,
        family: "rp2040",
        variant: Some("cytron-maker-nano-rp2040"),
    },
    BoardId {
        key: 0x2E8A1110,
        family: "rp2040",
        variant: Some("ilabs-rpico32"),
    },
    BoardId {
        key: 0x2E8A1111,
        family: "rp2040",
        variant: Some("melopero-cookie-rp2040"),
    },
    BoardId {
        key: 0x2E8A1118,
        family: "rp2040",
        variant: Some("pimoroni-pga2350"),
    },
    BoardId {
        key: 0x2E8A1120,
        family: "rp2040",
        variant: Some("waveshare-rp2040-plus"),
    },
    BoardId {
        key: 0x2E8A1121,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A1122,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-ethernet-w55rp20"),
    },
    BoardId {
        key: 0x2E8A1123,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A1127,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A1128,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A1129,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A112C,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A112D,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-sd-rtc"),
    },
    BoardId {
        key: 0x2E8A1132,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-subghz"),
    },
    BoardId {
        key: 0x2E8A1136,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-nfc"),
    },
    BoardId {
        key: 0x2E8A1137,
        family: "rp2040",
        variant: Some("electroniccats-huntercat-nfc-rp2040"),
    },
    BoardId {
        key: 0x2E8A1139,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-1-28"),
    },
    BoardId {
        key: 0x2E8A113A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A113F,
        family: "rp2040",
        variant: Some("ilabs-cpico-2350"),
    },
    BoardId {
        key: 0x2E8A1141,
        family: "rp2040",
        variant: Some("bridgetek-idm2040-7a"),
    },
    BoardId {
        key: 0x2E8A1143,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A1152,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-uwb"),
    },
    BoardId {
        key: 0x2E8A115E,
        family: "rp2040",
        variant: Some("breadstick-raspberry"),
    },
    BoardId {
        key: 0x2E8A115F,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi6-ble"),
    },
    BoardId {
        key: 0x2E8A116F,
        family: "rp2040",
        variant: Some("l-atelier-d-arnoz-dudescab"),
    },
    BoardId {
        key: 0x2E8A1171,
        family: "rp2040",
        variant: Some("cytron-maker-uno-rp2040"),
    },
    BoardId {
        key: 0x2E8A117B,
        family: "rp2040",
        variant: Some("ilabs-connectivity-2040-lte-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A1193,
        family: "rp2040",
        variant: Some("cytron-iriv-io-controller"),
    },
    BoardId {
        key: 0x2E8A1196,
        family: "rp2040",
        variant: Some("cytron-motion-2350-pro"),
    },
    BoardId {
        key: 0x2E8A119A,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A119B,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-bconnect"),
    },
    BoardId {
        key: 0x2E8A11A5,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A11AE,
        family: "rp2040",
        variant: Some("datanoisetv-picoadk-v2"),
    },
    BoardId {
        key: 0x2E8A11B0,
        family: "rp2040",
        variant: Some("waveshare-rp2350-zero"),
    },
    BoardId {
        key: 0x2E8A11B1,
        family: "rp2040",
        variant: Some("waveshare-rp2350-plus"),
    },
    BoardId {
        key: 0x2E8A11B7,
        family: "rp2040",
        variant: Some("waveshare-rp2350-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A11C0,
        family: "rp2040",
        variant: Some("pimoroni-explorer"),
    },
    BoardId {
        key: 0x2E8A11EC,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-rp2350"),
    },
    BoardId {
        key: 0x2E8A3001,
        family: "rp2040",
        variant: Some("mete-hoca-akana-r1"),
    },
    BoardId {
        key: 0x2E8A3101,
        family: "rp2040",
        variant: Some("mete-hoca-akana-r1"),
    },
    BoardId {
        key: 0x2E8A4003,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A400A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A400F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A408A,
        family: "rp2040",
        variant: Some("deruilab-flyboard2040core"),
    },
    BoardId {
        key: 0x2E8A40C0,
        family: "rp2040",
        variant: Some("rakwireless-rak11300"),
    },
    BoardId {
        key: 0x2E8A4103,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A410A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A410F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A418A,
        family: "rp2040",
        variant: Some("deruilab-flyboard2040core"),
    },
    BoardId {
        key: 0x2E8A41C0,
        family: "rp2040",
        variant: Some("rakwireless-rak11300"),
    },
    BoardId {
        key: 0x2E8A5000,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A5005,
        family: "rp2040",
        variant: Some("melopero-shake-rp2040"),
    },
    BoardId {
        key: 0x2E8A5006,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A5007,
        family: "rp2040",
        variant: Some("upesy-rp2040-devkit"),
    },
    BoardId {
        key: 0x2E8A5008,
        family: "rp2040",
        variant: Some("pimoroni-pga2040"),
    },
    BoardId {
        key: 0x2E8A500A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A500B,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A500D,
        family: "rp2040",
        variant: Some("ilabs-challenger-nb-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A500F,
        family: "rp2040",
        variant: Some("cytron-maker-nano-rp2040"),
    },
    BoardId {
        key: 0x2E8A5010,
        family: "rp2040",
        variant: Some("ilabs-rpico32"),
    },
    BoardId {
        key: 0x2E8A5011,
        family: "rp2040",
        variant: Some("melopero-cookie-rp2040"),
    },
    BoardId {
        key: 0x2E8A5018,
        family: "rp2040",
        variant: Some("pimoroni-pga2350"),
    },
    BoardId {
        key: 0x2E8A5020,
        family: "rp2040",
        variant: Some("waveshare-rp2040-plus"),
    },
    BoardId {
        key: 0x2E8A5021,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A5023,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-lora"),
    },
    BoardId {
        key: 0x2E8A5027,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A5028,
        family: "rp2040",
        variant: Some("wiznet-wizfi360-evb-pico"),
    },
    BoardId {
        key: 0x2E8A5029,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A502C,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A502D,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-sd-rtc"),
    },
    BoardId {
        key: 0x2E8A5032,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-subghz"),
    },
    BoardId {
        key: 0x2E8A5036,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-nfc"),
    },
    BoardId {
        key: 0x2E8A5037,
        family: "rp2040",
        variant: Some("electroniccats-huntercat-nfc-rp2040"),
    },
    BoardId {
        key: 0x2E8A5039,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-1-28"),
    },
    BoardId {
        key: 0x2E8A503A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A5041,
        family: "rp2040",
        variant: Some("bridgetek-idm2040-7a"),
    },
    BoardId {
        key: 0x2E8A5043,
        family: "rp2040",
        variant: Some("newsan-archi"),
    },
    BoardId {
        key: 0x2E8A5052,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-uwb"),
    },
    BoardId {
        key: 0x2E8A505E,
        family: "rp2040",
        variant: Some("breadstick-raspberry"),
    },
    BoardId {
        key: 0x2E8A505F,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi6-ble"),
    },
    BoardId {
        key: 0x2E8A506F,
        family: "rp2040",
        variant: Some("l-atelier-d-arnoz-dudescab"),
    },
    BoardId {
        key: 0x2E8A5071,
        family: "rp2040",
        variant: Some("cytron-maker-uno-rp2040"),
    },
    BoardId {
        key: 0x2E8A507B,
        family: "rp2040",
        variant: Some("ilabs-connectivity-2040-lte-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A5093,
        family: "rp2040",
        variant: Some("cytron-iriv-io-controller"),
    },
    BoardId {
        key: 0x2E8A5096,
        family: "rp2040",
        variant: Some("cytron-motion-2350-pro"),
    },
    BoardId {
        key: 0x2E8A509A,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A509B,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-bconnect"),
    },
    BoardId {
        key: 0x2E8A50A5,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A50AE,
        family: "rp2040",
        variant: Some("datanoisetv-picoadk-v2"),
    },
    BoardId {
        key: 0x2E8A50B0,
        family: "rp2040",
        variant: Some("waveshare-rp2350-zero"),
    },
    BoardId {
        key: 0x2E8A50B1,
        family: "rp2040",
        variant: Some("waveshare-rp2350-plus"),
    },
    BoardId {
        key: 0x2E8A50B7,
        family: "rp2040",
        variant: Some("waveshare-rp2350-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A50C0,
        family: "rp2040",
        variant: Some("pimoroni-explorer"),
    },
    BoardId {
        key: 0x2E8A50EC,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-rp2350"),
    },
    BoardId {
        key: 0x2E8A5100,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A5105,
        family: "rp2040",
        variant: Some("melopero-shake-rp2040"),
    },
    BoardId {
        key: 0x2E8A5106,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A5107,
        family: "rp2040",
        variant: Some("upesy-rp2040-devkit"),
    },
    BoardId {
        key: 0x2E8A5108,
        family: "rp2040",
        variant: Some("pimoroni-pga2040"),
    },
    BoardId {
        key: 0x2E8A510A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A510B,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A510D,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A510F,
        family: "rp2040",
        variant: Some("cytron-maker-nano-rp2040"),
    },
    BoardId {
        key: 0x2E8A5110,
        family: "rp2040",
        variant: Some("ilabs-rpico32"),
    },
    BoardId {
        key: 0x2E8A5111,
        family: "rp2040",
        variant: Some("melopero-cookie-rp2040"),
    },
    BoardId {
        key: 0x2E8A5118,
        family: "rp2040",
        variant: Some("pimoroni-pga2350"),
    },
    BoardId {
        key: 0x2E8A5120,
        family: "rp2040",
        variant: Some("waveshare-rp2040-plus"),
    },
    BoardId {
        key: 0x2E8A5121,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A5122,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-ethernet-w55rp20"),
    },
    BoardId {
        key: 0x2E8A5123,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A5127,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A5128,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A5129,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A512C,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A512D,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-sd-rtc"),
    },
    BoardId {
        key: 0x2E8A5132,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-subghz"),
    },
    BoardId {
        key: 0x2E8A5136,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-nfc"),
    },
    BoardId {
        key: 0x2E8A5137,
        family: "rp2040",
        variant: Some("electroniccats-huntercat-nfc-rp2040"),
    },
    BoardId {
        key: 0x2E8A5139,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-1-28"),
    },
    BoardId {
        key: 0x2E8A513A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A513F,
        family: "rp2040",
        variant: Some("ilabs-cpico-2350"),
    },
    BoardId {
        key: 0x2E8A5141,
        family: "rp2040",
        variant: Some("bridgetek-idm2040-7a"),
    },
    BoardId {
        key: 0x2E8A5143,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A5152,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-uwb"),
    },
    BoardId {
        key: 0x2E8A515E,
        family: "rp2040",
        variant: Some("breadstick-raspberry"),
    },
    BoardId {
        key: 0x2E8A515F,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi6-ble"),
    },
    BoardId {
        key: 0x2E8A516F,
        family: "rp2040",
        variant: Some("l-atelier-d-arnoz-dudescab"),
    },
    BoardId {
        key: 0x2E8A5171,
        family: "rp2040",
        variant: Some("cytron-maker-uno-rp2040"),
    },
    BoardId {
        key: 0x2E8A517B,
        family: "rp2040",
        variant: Some("ilabs-connectivity-2040-lte-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A5193,
        family: "rp2040",
        variant: Some("cytron-iriv-io-controller"),
    },
    BoardId {
        key: 0x2E8A5196,
        family: "rp2040",
        variant: Some("cytron-motion-2350-pro"),
    },
    BoardId {
        key: 0x2E8A519A,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A519B,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-bconnect"),
    },
    BoardId {
        key: 0x2E8A51A5,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A51AE,
        family: "rp2040",
        variant: Some("datanoisetv-picoadk-v2"),
    },
    BoardId {
        key: 0x2E8A51B0,
        family: "rp2040",
        variant: Some("waveshare-rp2350-zero"),
    },
    BoardId {
        key: 0x2E8A51B1,
        family: "rp2040",
        variant: Some("waveshare-rp2350-plus"),
    },
    BoardId {
        key: 0x2E8A51B7,
        family: "rp2040",
        variant: Some("waveshare-rp2350-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A51C0,
        family: "rp2040",
        variant: Some("pimoroni-explorer"),
    },
    BoardId {
        key: 0x2E8A51EC,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-rp2350"),
    },
    BoardId {
        key: 0x2E8A6E61,
        family: "rp2040",
        variant: Some("nullbits-bit-c-pro"),
    },
    BoardId {
        key: 0x2E8A6F61,
        family: "rp2040",
        variant: Some("nullbits-bit-c-pro"),
    },
    BoardId {
        key: 0x2E8A7001,
        family: "rp2040",
        variant: Some("mete-hoca-akana-r1"),
    },
    BoardId {
        key: 0x2E8A7101,
        family: "rp2040",
        variant: Some("mete-hoca-akana-r1"),
    },
    BoardId {
        key: 0x2E8A8003,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A800A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A800F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A808A,
        family: "rp2040",
        variant: Some("deruilab-flyboard2040core"),
    },
    BoardId {
        key: 0x2E8A80C0,
        family: "rp2040",
        variant: Some("rakwireless-rak11300"),
    },
    BoardId {
        key: 0x2E8A8103,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A810A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A810F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A818A,
        family: "rp2040",
        variant: Some("deruilab-flyboard2040core"),
    },
    BoardId {
        key: 0x2E8A81C0,
        family: "rp2040",
        variant: Some("rakwireless-rak11300"),
    },
    BoardId {
        key: 0x2E8A9000,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A9005,
        family: "rp2040",
        variant: Some("melopero-shake-rp2040"),
    },
    BoardId {
        key: 0x2E8A9006,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A9007,
        family: "rp2040",
        variant: Some("upesy-rp2040-devkit"),
    },
    BoardId {
        key: 0x2E8A9008,
        family: "rp2040",
        variant: Some("pimoroni-pga2040"),
    },
    BoardId {
        key: 0x2E8A900A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A900B,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A900D,
        family: "rp2040",
        variant: Some("ilabs-challenger-nb-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A900F,
        family: "rp2040",
        variant: Some("cytron-maker-nano-rp2040"),
    },
    BoardId {
        key: 0x2E8A9010,
        family: "rp2040",
        variant: Some("ilabs-rpico32"),
    },
    BoardId {
        key: 0x2E8A9011,
        family: "rp2040",
        variant: Some("melopero-cookie-rp2040"),
    },
    BoardId {
        key: 0x2E8A9018,
        family: "rp2040",
        variant: Some("pimoroni-pga2350"),
    },
    BoardId {
        key: 0x2E8A9020,
        family: "rp2040",
        variant: Some("waveshare-rp2040-plus"),
    },
    BoardId {
        key: 0x2E8A9021,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A9023,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-lora"),
    },
    BoardId {
        key: 0x2E8A9027,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A9028,
        family: "rp2040",
        variant: Some("wiznet-wizfi360-evb-pico"),
    },
    BoardId {
        key: 0x2E8A9029,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A902C,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A902D,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-sd-rtc"),
    },
    BoardId {
        key: 0x2E8A9032,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-subghz"),
    },
    BoardId {
        key: 0x2E8A9036,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-nfc"),
    },
    BoardId {
        key: 0x2E8A9037,
        family: "rp2040",
        variant: Some("electroniccats-huntercat-nfc-rp2040"),
    },
    BoardId {
        key: 0x2E8A9039,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-1-28"),
    },
    BoardId {
        key: 0x2E8A903A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A9041,
        family: "rp2040",
        variant: Some("bridgetek-idm2040-7a"),
    },
    BoardId {
        key: 0x2E8A9043,
        family: "rp2040",
        variant: Some("newsan-archi"),
    },
    BoardId {
        key: 0x2E8A9052,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-uwb"),
    },
    BoardId {
        key: 0x2E8A905E,
        family: "rp2040",
        variant: Some("breadstick-raspberry"),
    },
    BoardId {
        key: 0x2E8A905F,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi6-ble"),
    },
    BoardId {
        key: 0x2E8A906F,
        family: "rp2040",
        variant: Some("l-atelier-d-arnoz-dudescab"),
    },
    BoardId {
        key: 0x2E8A9071,
        family: "rp2040",
        variant: Some("cytron-maker-uno-rp2040"),
    },
    BoardId {
        key: 0x2E8A907B,
        family: "rp2040",
        variant: Some("ilabs-connectivity-2040-lte-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A9093,
        family: "rp2040",
        variant: Some("cytron-iriv-io-controller"),
    },
    BoardId {
        key: 0x2E8A9096,
        family: "rp2040",
        variant: Some("cytron-motion-2350-pro"),
    },
    BoardId {
        key: 0x2E8A909A,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A909B,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-bconnect"),
    },
    BoardId {
        key: 0x2E8A90A5,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A90AE,
        family: "rp2040",
        variant: Some("datanoisetv-picoadk-v2"),
    },
    BoardId {
        key: 0x2E8A90B0,
        family: "rp2040",
        variant: Some("waveshare-rp2350-zero"),
    },
    BoardId {
        key: 0x2E8A90B1,
        family: "rp2040",
        variant: Some("waveshare-rp2350-plus"),
    },
    BoardId {
        key: 0x2E8A90B7,
        family: "rp2040",
        variant: Some("waveshare-rp2350-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A90C0,
        family: "rp2040",
        variant: Some("pimoroni-explorer"),
    },
    BoardId {
        key: 0x2E8A90EC,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-rp2350"),
    },
    BoardId {
        key: 0x2E8A9100,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A9101,
        family: "rp2040",
        variant: Some("pintronix-pinmax"),
    },
    BoardId {
        key: 0x2E8A9105,
        family: "rp2040",
        variant: Some("melopero-shake-rp2040"),
    },
    BoardId {
        key: 0x2E8A9106,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi"),
    },
    BoardId {
        key: 0x2E8A9107,
        family: "rp2040",
        variant: Some("upesy-rp2040-devkit"),
    },
    BoardId {
        key: 0x2E8A9108,
        family: "rp2040",
        variant: Some("pimoroni-pga2040"),
    },
    BoardId {
        key: 0x2E8A910A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A910B,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A910D,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A910F,
        family: "rp2040",
        variant: Some("cytron-maker-nano-rp2040"),
    },
    BoardId {
        key: 0x2E8A9110,
        family: "rp2040",
        variant: Some("ilabs-rpico32"),
    },
    BoardId {
        key: 0x2E8A9111,
        family: "rp2040",
        variant: Some("melopero-cookie-rp2040"),
    },
    BoardId {
        key: 0x2E8A9118,
        family: "rp2040",
        variant: Some("pimoroni-pga2350"),
    },
    BoardId {
        key: 0x2E8A9120,
        family: "rp2040",
        variant: Some("waveshare-rp2040-plus"),
    },
    BoardId {
        key: 0x2E8A9121,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A9122,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-ethernet-w55rp20"),
    },
    BoardId {
        key: 0x2E8A9123,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A9127,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A9128,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A9129,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A912C,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A912D,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-sd-rtc"),
    },
    BoardId {
        key: 0x2E8A9132,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-subghz"),
    },
    BoardId {
        key: 0x2E8A9136,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-nfc"),
    },
    BoardId {
        key: 0x2E8A9137,
        family: "rp2040",
        variant: Some("electroniccats-huntercat-nfc-rp2040"),
    },
    BoardId {
        key: 0x2E8A9139,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-1-28"),
    },
    BoardId {
        key: 0x2E8A913A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A913F,
        family: "rp2040",
        variant: Some("ilabs-cpico-2350"),
    },
    BoardId {
        key: 0x2E8A9141,
        family: "rp2040",
        variant: Some("bridgetek-idm2040-7a"),
    },
    BoardId {
        key: 0x2E8A9143,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A9152,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-uwb"),
    },
    BoardId {
        key: 0x2E8A915E,
        family: "rp2040",
        variant: Some("breadstick-raspberry"),
    },
    BoardId {
        key: 0x2E8A915F,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi6-ble"),
    },
    BoardId {
        key: 0x2E8A916F,
        family: "rp2040",
        variant: Some("l-atelier-d-arnoz-dudescab"),
    },
    BoardId {
        key: 0x2E8A9171,
        family: "rp2040",
        variant: Some("cytron-maker-uno-rp2040"),
    },
    BoardId {
        key: 0x2E8A917B,
        family: "rp2040",
        variant: Some("ilabs-connectivity-2040-lte-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A9193,
        family: "rp2040",
        variant: Some("cytron-iriv-io-controller"),
    },
    BoardId {
        key: 0x2E8A9196,
        family: "rp2040",
        variant: Some("cytron-motion-2350-pro"),
    },
    BoardId {
        key: 0x2E8A919A,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-wifi-ble"),
    },
    BoardId {
        key: 0x2E8A919B,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-bconnect"),
    },
    BoardId {
        key: 0x2E8A91A5,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8A91AE,
        family: "rp2040",
        variant: Some("datanoisetv-picoadk-v2"),
    },
    BoardId {
        key: 0x2E8A91B0,
        family: "rp2040",
        variant: Some("waveshare-rp2350-zero"),
    },
    BoardId {
        key: 0x2E8A91B1,
        family: "rp2040",
        variant: Some("waveshare-rp2350-plus"),
    },
    BoardId {
        key: 0x2E8A91B7,
        family: "rp2040",
        variant: Some("waveshare-rp2350-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8A91C0,
        family: "rp2040",
        variant: Some("pimoroni-explorer"),
    },
    BoardId {
        key: 0x2E8A91EC,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-rp2350"),
    },
    BoardId {
        key: 0x2E8AB001,
        family: "rp2040",
        variant: Some("mete-hoca-akana-r1"),
    },
    BoardId {
        key: 0x2E8AB101,
        family: "rp2040",
        variant: Some("mete-hoca-akana-r1"),
    },
    BoardId {
        key: 0x2E8AC003,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AC00A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AC00F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AC08A,
        family: "rp2040",
        variant: Some("deruilab-flyboard2040core"),
    },
    BoardId {
        key: 0x2E8AC0C0,
        family: "rp2040",
        variant: Some("rakwireless-rak11300"),
    },
    BoardId {
        key: 0x2E8AC103,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AC10A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AC10F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AC18A,
        family: "rp2040",
        variant: Some("deruilab-flyboard2040core"),
    },
    BoardId {
        key: 0x2E8AC1C0,
        family: "rp2040",
        variant: Some("rakwireless-rak11300"),
    },
    BoardId {
        key: 0x2E8AD000,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD005,
        family: "rp2040",
        variant: Some("melopero-shake-rp2040"),
    },
    BoardId {
        key: 0x2E8AD006,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi"),
    },
    BoardId {
        key: 0x2E8AD007,
        family: "rp2040",
        variant: Some("upesy-rp2040-devkit"),
    },
    BoardId {
        key: 0x2E8AD008,
        family: "rp2040",
        variant: Some("pimoroni-pga2040"),
    },
    BoardId {
        key: 0x2E8AD00A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD00B,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD00D,
        family: "rp2040",
        variant: Some("ilabs-challenger-nb-2040-wifi"),
    },
    BoardId {
        key: 0x2E8AD00F,
        family: "rp2040",
        variant: Some("cytron-maker-nano-rp2040"),
    },
    BoardId {
        key: 0x2E8AD010,
        family: "rp2040",
        variant: Some("ilabs-rpico32"),
    },
    BoardId {
        key: 0x2E8AD011,
        family: "rp2040",
        variant: Some("melopero-cookie-rp2040"),
    },
    BoardId {
        key: 0x2E8AD018,
        family: "rp2040",
        variant: Some("pimoroni-pga2350"),
    },
    BoardId {
        key: 0x2E8AD020,
        family: "rp2040",
        variant: Some("waveshare-rp2040-plus"),
    },
    BoardId {
        key: 0x2E8AD021,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8AD023,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-lora"),
    },
    BoardId {
        key: 0x2E8AD027,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD028,
        family: "rp2040",
        variant: Some("wiznet-wizfi360-evb-pico"),
    },
    BoardId {
        key: 0x2E8AD029,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD02C,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi-ble"),
    },
    BoardId {
        key: 0x2E8AD02D,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-sd-rtc"),
    },
    BoardId {
        key: 0x2E8AD032,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-subghz"),
    },
    BoardId {
        key: 0x2E8AD036,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-nfc"),
    },
    BoardId {
        key: 0x2E8AD037,
        family: "rp2040",
        variant: Some("electroniccats-huntercat-nfc-rp2040"),
    },
    BoardId {
        key: 0x2E8AD039,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-1-28"),
    },
    BoardId {
        key: 0x2E8AD03A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD041,
        family: "rp2040",
        variant: Some("bridgetek-idm2040-7a"),
    },
    BoardId {
        key: 0x2E8AD043,
        family: "rp2040",
        variant: Some("newsan-archi"),
    },
    BoardId {
        key: 0x2E8AD052,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-uwb"),
    },
    BoardId {
        key: 0x2E8AD05E,
        family: "rp2040",
        variant: Some("breadstick-raspberry"),
    },
    BoardId {
        key: 0x2E8AD05F,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi6-ble"),
    },
    BoardId {
        key: 0x2E8AD06F,
        family: "rp2040",
        variant: Some("l-atelier-d-arnoz-dudescab"),
    },
    BoardId {
        key: 0x2E8AD071,
        family: "rp2040",
        variant: Some("cytron-maker-uno-rp2040"),
    },
    BoardId {
        key: 0x2E8AD07B,
        family: "rp2040",
        variant: Some("ilabs-connectivity-2040-lte-wifi-ble"),
    },
    BoardId {
        key: 0x2E8AD093,
        family: "rp2040",
        variant: Some("cytron-iriv-io-controller"),
    },
    BoardId {
        key: 0x2E8AD096,
        family: "rp2040",
        variant: Some("cytron-motion-2350-pro"),
    },
    BoardId {
        key: 0x2E8AD09A,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-wifi-ble"),
    },
    BoardId {
        key: 0x2E8AD09B,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-bconnect"),
    },
    BoardId {
        key: 0x2E8AD0A5,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD0AE,
        family: "rp2040",
        variant: Some("datanoisetv-picoadk-v2"),
    },
    BoardId {
        key: 0x2E8AD0B0,
        family: "rp2040",
        variant: Some("waveshare-rp2350-zero"),
    },
    BoardId {
        key: 0x2E8AD0B1,
        family: "rp2040",
        variant: Some("waveshare-rp2350-plus"),
    },
    BoardId {
        key: 0x2E8AD0B7,
        family: "rp2040",
        variant: Some("waveshare-rp2350-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8AD0C0,
        family: "rp2040",
        variant: Some("pimoroni-explorer"),
    },
    BoardId {
        key: 0x2E8AD0EC,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-rp2350"),
    },
    BoardId {
        key: 0x2E8AD100,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD101,
        family: "rp2040",
        variant: Some("pintronix-pinmax"),
    },
    BoardId {
        key: 0x2E8AD105,
        family: "rp2040",
        variant: Some("melopero-shake-rp2040"),
    },
    BoardId {
        key: 0x2E8AD106,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi"),
    },
    BoardId {
        key: 0x2E8AD107,
        family: "rp2040",
        variant: Some("upesy-rp2040-devkit"),
    },
    BoardId {
        key: 0x2E8AD108,
        family: "rp2040",
        variant: Some("pimoroni-pga2040"),
    },
    BoardId {
        key: 0x2E8AD10A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD10B,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD10D,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD10F,
        family: "rp2040",
        variant: Some("cytron-maker-nano-rp2040"),
    },
    BoardId {
        key: 0x2E8AD110,
        family: "rp2040",
        variant: Some("ilabs-rpico32"),
    },
    BoardId {
        key: 0x2E8AD111,
        family: "rp2040",
        variant: Some("melopero-cookie-rp2040"),
    },
    BoardId {
        key: 0x2E8AD118,
        family: "rp2040",
        variant: Some("pimoroni-pga2350"),
    },
    BoardId {
        key: 0x2E8AD120,
        family: "rp2040",
        variant: Some("waveshare-rp2040-plus"),
    },
    BoardId {
        key: 0x2E8AD121,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8AD122,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-ethernet-w55rp20"),
    },
    BoardId {
        key: 0x2E8AD123,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD127,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD128,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD129,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD12C,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi-ble"),
    },
    BoardId {
        key: 0x2E8AD12D,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-sd-rtc"),
    },
    BoardId {
        key: 0x2E8AD132,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-subghz"),
    },
    BoardId {
        key: 0x2E8AD136,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-nfc"),
    },
    BoardId {
        key: 0x2E8AD137,
        family: "rp2040",
        variant: Some("electroniccats-huntercat-nfc-rp2040"),
    },
    BoardId {
        key: 0x2E8AD139,
        family: "rp2040",
        variant: Some("waveshare-rp2040-lcd-1-28"),
    },
    BoardId {
        key: 0x2E8AD13A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD13F,
        family: "rp2040",
        variant: Some("ilabs-cpico-2350"),
    },
    BoardId {
        key: 0x2E8AD141,
        family: "rp2040",
        variant: Some("bridgetek-idm2040-7a"),
    },
    BoardId {
        key: 0x2E8AD143,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD152,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-uwb"),
    },
    BoardId {
        key: 0x2E8AD15E,
        family: "rp2040",
        variant: Some("breadstick-raspberry"),
    },
    BoardId {
        key: 0x2E8AD15F,
        family: "rp2040",
        variant: Some("ilabs-challenger-2040-wifi6-ble"),
    },
    BoardId {
        key: 0x2E8AD16F,
        family: "rp2040",
        variant: Some("l-atelier-d-arnoz-dudescab"),
    },
    BoardId {
        key: 0x2E8AD171,
        family: "rp2040",
        variant: Some("cytron-maker-uno-rp2040"),
    },
    BoardId {
        key: 0x2E8AD17B,
        family: "rp2040",
        variant: Some("ilabs-connectivity-2040-lte-wifi-ble"),
    },
    BoardId {
        key: 0x2E8AD193,
        family: "rp2040",
        variant: Some("cytron-iriv-io-controller"),
    },
    BoardId {
        key: 0x2E8AD196,
        family: "rp2040",
        variant: Some("cytron-motion-2350-pro"),
    },
    BoardId {
        key: 0x2E8AD19A,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-wifi-ble"),
    },
    BoardId {
        key: 0x2E8AD19B,
        family: "rp2040",
        variant: Some("ilabs-challenger-2350-bconnect"),
    },
    BoardId {
        key: 0x2E8AD1A5,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AD1AE,
        family: "rp2040",
        variant: Some("datanoisetv-picoadk-v2"),
    },
    BoardId {
        key: 0x2E8AD1B0,
        family: "rp2040",
        variant: Some("waveshare-rp2350-zero"),
    },
    BoardId {
        key: 0x2E8AD1B1,
        family: "rp2040",
        variant: Some("waveshare-rp2350-plus"),
    },
    BoardId {
        key: 0x2E8AD1B7,
        family: "rp2040",
        variant: Some("waveshare-rp2350-lcd-0-96"),
    },
    BoardId {
        key: 0x2E8AD1C0,
        family: "rp2040",
        variant: Some("pimoroni-explorer"),
    },
    BoardId {
        key: 0x2E8AD1EC,
        family: "rp2040",
        variant: Some("soldered-electronics-nula-rp2350"),
    },
    BoardId {
        key: 0x2E8AEE20,
        family: "rp2040",
        variant: Some("extremeelectronics-rc2040"),
    },
    BoardId {
        key: 0x2E8AEE61,
        family: "rp2040",
        variant: Some("nullbits-bit-c-pro"),
    },
    BoardId {
        key: 0x2E8AEF20,
        family: "rp2040",
        variant: Some("extremeelectronics-rc2040"),
    },
    BoardId {
        key: 0x2E8AEF61,
        family: "rp2040",
        variant: Some("nullbits-bit-c-pro"),
    },
    BoardId {
        key: 0x2E8AF001,
        family: "rp2040",
        variant: Some("mete-hoca-akana-r1"),
    },
    BoardId {
        key: 0x2E8AF00A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AF00F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AF101,
        family: "rp2040",
        variant: Some("mete-hoca-akana-r1"),
    },
    BoardId {
        key: 0x2E8AF10A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8AF10F,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8BF00A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x2E8BF10A,
        family: "rp2040",
        variant: None,
    },
    BoardId {
        key: 0x33434253,
        family: "rp2040",
        variant: Some("dfrobot-beetle-rp2040"),
    },
    BoardId {
        key: 0x33434353,
        family: "rp2040",
        variant: Some("dfrobot-beetle-rp2040"),
    },
    BoardId {
        key: 0x3343C253,
        family: "rp2040",
        variant: Some("dfrobot-beetle-rp2040"),
    },
    BoardId {
        key: 0x3343C353,
        family: "rp2040",
        variant: Some("dfrobot-beetle-rp2040"),
    },
];
