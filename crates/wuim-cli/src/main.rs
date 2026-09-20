//! Development CLI. A scaffold for seeing what the core reads from a real
//! machine, not the product's user interface.
//!
//! ```text
//! wuim list             devices currently connected
//! wuim list --all       include devices that only have a bind record
//! wuim list --nodes     also list nodes usbipd does not track
//! wuim list --json      dump the whole snapshot as JSON
//! wuim probe <target>   ask a board what it is (COM port or bus id)
//! wuim probe <target> --yes   skip the confirmation
//! ```

mod list;
mod probe;

use anyhow::Result;

fn main() -> Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .iter()
        .any(|a| matches!(a.as_str(), "--help" | "-h" | "help"))
    {
        print_help();
        return Ok(());
    }

    let (command, rest) = match args.split_first() {
        None => ("list", &[][..]),
        // Bare flags mean `list`, so `wuim --json` keeps working.
        Some((first, _)) if first.starts_with('-') => ("list", &args[..]),
        Some((first, rest)) => (first.as_str(), rest),
    };

    match command {
        "list" => list::run(rest),
        "probe" => probe::run(rest),
        "hubports" => hubports(rest),
        other => {
            eprintln!("unknown command: {other}");
            print_help();
            std::process::exit(2);
        }
    }
}

fn print_help() {
    println!("{}", env!("CARGO_PKG_DESCRIPTION"));
    println!();
    println!("  wuim list                    devices currently connected");
    println!("  wuim list --all              include devices that only have a bind record");
    println!("  wuim list --nodes            also list nodes usbipd does not track");
    println!("  wuim list --json             dump the whole snapshot as JSON");
    println!();
    println!("  wuim probe <COM32|bus-id>    ask a board what it is");
    println!("  wuim probe <target> --yes    skip the confirmation");
    println!();
    println!("A probe has side effects. It runs only when asked for explicitly.");
}

fn hubports(args: &[String]) -> Result<()> {
    let id = args.first().map(String::as_str).unwrap_or("");
    let Some(ports) = wuim_core::hub::ports(id) else {
        println!("  {id} is not a hub");
        return Ok(());
    };
    for p in ports {
        println!(
            "  port {:<2} {:<24} {}",
            p.port,
            p.status_name,
            p.vid_pid.unwrap_or_default()
        );
    }
    Ok(())
}
