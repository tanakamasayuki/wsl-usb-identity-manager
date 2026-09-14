//! ESP32 family, identified through its ROM bootloader.
//!
//! The eFuse MAC is burned at the factory and is unique per chip, which makes it
//! the one thing that tells two boards behind identical CH340s apart.
//!
//! The protocol work is espflash's, not ours (docs/platform-evaluation.ja.md §5).

use std::collections::BTreeMap;
use std::time::Duration;

use anyhow::{Context, Result, anyhow, bail};
use espflash::connection::{Connection, ResetAfterOperation, ResetBeforeOperation};
use espflash::flasher::Flasher;
use serialport::{FlowControl, UsbPortInfo};
use wuim_core::windevice::WinUsbDevice;

use crate::identity::{esp_variant, identity_key};
use crate::{Applicability, TargetIdentity, TargetProbe};

/// The ROM loader answers at 115200 baud; there is no reason to go faster for a
/// handful of register reads.
const PROBE_BAUD: u32 = 115_200;

/// Long enough for a board that resets slowly, short enough that a device which
/// is not an ESP32 does not hold the UI.
const PROBE_TIMEOUT: Duration = Duration::from_secs(3);

pub struct Esp32Probe;

impl TargetProbe for Esp32Probe {
    fn family(&self) -> &'static str {
        "esp32"
    }

    fn side_effect(&self) -> &'static str {
        "Resets the board into its ROM bootloader and then back, so the running firmware restarts."
    }

    fn applicability(&self, device: &WinUsbDevice) -> Applicability {
        if device.com_port.is_none() {
            // Either the board is behind something that is not a serial port, or
            // its driver is not loaded. Either way there is nothing to talk to.
            return Applicability::NotApplicable("no COM port");
        }
        Applicability::Supported
    }

    fn probe(&self, device: &WinUsbDevice) -> Result<TargetIdentity> {
        let port_name = device
            .com_port
            .as_deref()
            .ok_or_else(|| anyhow!("device has no COM port"))?;

        let info = read_device_info(port_name, device)?;

        let mac = info
            .mac_address
            .ok_or_else(|| anyhow!("the chip did not report a MAC address"))?;
        let variant = esp_variant(&info.chip.to_string());
        let identity_key = identity_key(&variant, &mac)
            .map_err(|e| anyhow!("could not build an identity key from {variant}/{mac}: {e}"))?;

        let mut details = BTreeMap::new();
        details.insert("crystal".into(), info.crystal_frequency.to_string());
        details.insert("flash_size".into(), info.flash_size.to_string());
        if !info.features.is_empty() {
            details.insert("features".into(), info.features.join(", "));
        }
        details.insert("probed_via".into(), port_name.to_owned());

        Ok(TargetIdentity {
            family: self.family(),
            identity_key,
            device_id: mac,
            device_type: variant,
            hardware_revision: info
                .revision
                .map(|(major, minor)| format!("v{major}.{minor}")),
            details,
        })
    }
}

/// Opens the port, talks to the ROM loader, and puts the board back into a
/// normal boot before letting go.
fn read_device_info(
    port_name: &str,
    device: &WinUsbDevice,
) -> Result<espflash::flasher::DeviceInfo> {
    let serial = serialport::new(port_name, PROBE_BAUD)
        .flow_control(FlowControl::None)
        .timeout(PROBE_TIMEOUT)
        .open_native()
        .with_context(|| format!("could not open {port_name}"))?;

    let (vid, pid) = device.instance_id.vid_pid().unwrap_or((0, 0));
    let port_info = UsbPortInfo {
        // espflash picks its reset sequence from the PID: a board with native
        // USB needs the USB-JTAG-Serial sequence, not the DTR/RTS one.
        vid,
        pid,
        serial_number: device.instance_id.unit.serial().map(str::to_owned),
        manufacturer: device.manufacturer.clone(),
        product: device.bus_reported_device_desc.clone(),
        // The USB topology and interface index are espflash's to ignore; the
        // reset sequence only looks at the PID.
        location: None,
        interface: None,
    };

    let connection = Connection::new(
        serial,
        port_info,
        ResetAfterOperation::HardReset,
        ResetBeforeOperation::DefaultReset,
        PROBE_BAUD,
    );

    // No stub: the eFuse MAC is readable straight from the ROM loader, and
    // skipping the upload keeps the disturbance down to the single reset that
    // getting into the loader already costs.
    let mut flasher = Flasher::connect(connection, false, false, false, None, None)
        .map_err(|e| anyhow!("{port_name} did not answer as an ESP32: {e}"))?;

    let info = flasher.device_info();

    // Reset back into the application regardless of how the read went, so a
    // failed probe does not leave the board sitting in its bootloader.
    let chip = flasher.chip();
    if let Err(e) = flasher.connection().reset_after(false, chip)
        && info.is_ok()
    {
        bail!("read the chip but could not reset it back into the application: {e}");
    }

    info.map_err(|e| anyhow!("could not read the chip: {e}"))
}
