//! Core of WSL USB Identity Manager.
//!
//! The design rests on keeping "where a device is plugged in" apart from "what
//! device it is" (requirements §1.3). This crate provides the foundation for that:
//!
//! - [`windevice`] — the USB devices Windows currently sees
//! - [`usbipd`] — the output of `usbipd state`
//! - [`snapshot`] — the two joined together
//!
//! **Probing is deliberately absent.** A probe has side effects — reading an
//! ESP32's eFuse MAC restarts its firmware, attaching to a WCH-Link halts the
//! target core — so it may only run on the two triggers in requirement R4.5.
//! Keeping it out of this crate means no enumeration path can reach it by accident.

pub mod autoattach;
pub mod autostart;
pub mod elevate;
pub mod instance_id;
pub mod registry;
pub mod shell_open;
pub mod single_instance;
pub mod snapshot;
pub mod store;
pub mod usb_ids;
pub mod usbipd;
pub mod webview2;
pub mod windevice;

pub use autoattach::{Candidate, Rule, RuleKind};
pub use instance_id::{InstanceId, UnitId};
pub use snapshot::{DeviceRow, IdentityBasis, Snapshot};
pub use store::Settings;
pub use usb_ids::UsbIds;
pub use usbipd::{Operation, SharingState, UsbipdDevice};
