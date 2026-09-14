//! `wuim probe` — ask a board what it is.
//!
//! This is trigger 1 of requirement R4.5: an explicit request from the user.
//! Nothing else in this CLI probes, and the confirmation below is requirement
//! R4.7 — the side effects are stated before anything happens.

use std::io::{self, Write};

use anyhow::{Result, anyhow, bail};
use wuim_core::snapshot::{DeviceRow, Snapshot};
use wuim_probe::{Applicability, TargetProbe};

pub fn run(args: &[String]) -> Result<()> {
    let mut target = None;
    let mut assume_yes = false;
    for arg in args {
        match arg.as_str() {
            "--yes" | "-y" => assume_yes = true,
            other if other.starts_with('-') => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
            other => target = Some(other.to_owned()),
        }
    }
    let Some(target) = target else {
        bail!("name a device to probe, by COM port (COM32) or bus id (8-1)");
    };

    let snapshot = Snapshot::capture()?;
    let row = find(&snapshot, &target)?;
    let device = row
        .windows
        .as_ref()
        .ok_or_else(|| match row.sharing_state() {
            wuim_core::SharingState::Attached => anyhow!(
                "{target} is attached to WSL; Windows cannot reach the device, so it cannot be probed"
            ),
            _ => anyhow!("{target} is not currently present on this machine"),
        })?;

    println!("{}", row.name);
    println!("  instance id  {}", row.instance_id.raw);
    println!("  port         {}", row.com_port().unwrap_or("-"));
    println!("  identity now {}", row.identity_basis.label());
    println!();

    let candidates: Vec<Box<dyn TargetProbe>> = wuim_probe::applicable(device)
        .into_iter()
        .filter_map(|(probe, verdict)| match verdict {
            Applicability::Supported => Some(probe),
            Applicability::NotApplicable(reason) => {
                println!("  skipping {}: {reason}", probe.family());
                None
            }
            Applicability::Blocked(reason) => {
                println!("  {} is blocked: {reason}", probe.family());
                None
            }
        })
        .collect();

    if candidates.is_empty() {
        bail!("no probe applies to this device; it can be identified down to the transport only");
    }

    let mut declined = 0usize;
    let mut attempted = 0usize;
    for probe in candidates {
        println!("probe: {}", probe.family());
        println!("  {}", probe.side_effect());
        if !assume_yes && !confirm()? {
            println!("  cancelled");
            declined += 1;
            continue;
        }
        attempted += 1;

        match probe.probe(device) {
            Ok(identity) => {
                println!();
                println!("  identity key      {}", identity.identity_key);
                println!("  device type       {}", identity.device_type);
                println!("  device id         {}", identity.device_id);
                if let Some(rev) = &identity.hardware_revision {
                    println!("  hardware revision {rev}");
                }
                for (key, value) in &identity.details {
                    println!("  {key:<17} {value}");
                }
                return Ok(());
            }
            Err(e) => {
                println!("  {} did not identify this device: {e:#}", probe.family());
            }
        }
    }

    // Declining is a choice, not a failure; only report one when a probe
    // actually ran and came back empty.
    if attempted == 0 && declined > 0 {
        return Ok(());
    }
    bail!("no probe identified this device")
}

/// Matches a device by COM port or bus id, both case-insensitively.
fn find<'a>(snapshot: &'a Snapshot, target: &str) -> Result<&'a DeviceRow> {
    let matches: Vec<&DeviceRow> = snapshot
        .devices
        .iter()
        .filter(|row| {
            row.com_port()
                .is_some_and(|p| p.eq_ignore_ascii_case(target))
                || row.bus_id().is_some_and(|b| b.eq_ignore_ascii_case(target))
        })
        .collect();

    match matches.as_slice() {
        [] => Err(anyhow!("no device matches {target:?}; try `wuim list`")),
        [row] => Ok(row),
        // A COM number can be reused by a device that is not connected, so an
        // ambiguous match is reported rather than guessed at.
        many => Err(anyhow!(
            "{} devices match {target:?}; identify it by bus id instead",
            many.len()
        )),
    }
}

fn confirm() -> Result<bool> {
    print!("  run it? [y/N] ");
    io::stdout().flush()?;
    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;
    // A piped answer is not echoed, so end the prompt line ourselves.
    println!();
    Ok(matches!(answer.trim(), "y" | "Y" | "yes"))
}
