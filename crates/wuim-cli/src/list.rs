//! `wuim list` — what the core sees, without touching anything.

use anyhow::Result;
use wuim_core::snapshot::Snapshot;

struct Options {
    all: bool,
    nodes: bool,
    json: bool,
}

pub fn run(args: &[String]) -> Result<()> {
    let mut opts = Options {
        all: false,
        nodes: false,
        json: false,
    };
    for arg in args {
        match arg.as_str() {
            "--all" | "-a" => opts.all = true,
            "--nodes" => opts.nodes = true,
            "--json" => opts.json = true,
            other => {
                eprintln!("unknown argument: {other}");
                std::process::exit(2);
            }
        }
    }

    let snapshot = Snapshot::capture()?;

    if opts.json {
        println!("{}", serde_json::to_string_pretty(&snapshot)?);
        return Ok(());
    }

    print_devices(&snapshot, &opts);
    print_ambiguous(&snapshot);
    if opts.nodes {
        print_other_nodes(&snapshot);
    }
    Ok(())
}

fn print_devices(snapshot: &Snapshot, opts: &Options) {
    let rows: Vec<_> = if opts.all {
        snapshot.devices.iter().collect()
    } else {
        snapshot.connected().collect()
    };

    println!(
        "{} device(s) shown — {} known to usbipd, {} connected",
        rows.len(),
        snapshot.devices.len(),
        snapshot.connected().count()
    );
    println!();

    for row in rows {
        println!(
            "  [{:^10}] {:<6} {:<6} {}",
            row.sharing_state().label(),
            row.bus_id().unwrap_or("-"),
            row.com_port().unwrap_or("-"),
            row.name
        );
        println!(
            "               {:<10} {:<10} {}",
            row.instance_id.vid_pid_string().as_deref().unwrap_or("-"),
            row.identity_basis.label(),
            row.instance_id.unit.as_str()
        );
        if let Some(path) = row.location_path() {
            println!("               {path}");
        }
        if let Some(win) = &row.windows {
            let container = match &win.container_id {
                Some(c) if c.is_stable() => format!("{} (v{}, device)", c.uuid, c.version),
                Some(c) => format!("{} (v{}, port)", c.uuid, c.version),
                None => "-".into(),
            };
            println!(
                "               driver={}  container={container}",
                win.service.as_deref().unwrap_or("-")
            );
        }
        if let Some(stub) = &row.stub {
            println!("               attach stub: {}", stub.instance_id.raw);
        }
        println!();
    }
}

fn print_ambiguous(snapshot: &Snapshot) {
    let groups = snapshot.ambiguous_groups();
    if groups.is_empty() {
        return;
    }
    println!("-- indistinguishable from USB descriptors alone --");
    println!("   (a probe is needed to tell these apart)");
    println!();
    for (vid_pid, rows) in groups {
        println!("  {vid_pid}  x{}", rows.len());
        for row in rows {
            println!(
                "    {:<6} {:<6} {:<52} {}",
                row.bus_id().unwrap_or("-"),
                row.com_port().unwrap_or("-"),
                row.location_path().unwrap_or("-"),
                row.name
            );
        }
        println!();
    }
}

fn print_other_nodes(snapshot: &Snapshot) {
    println!(
        "-- {} node(s) Windows sees but usbipd does not track --",
        snapshot.other_nodes.len()
    );
    for node in &snapshot.other_nodes {
        println!(
            "  {:<10} {}",
            node.service.as_deref().unwrap_or("-"),
            node.instance_id.raw
        );
    }
    println!();
}
