use std::net::IpAddr;

use anyhow::{Context, Result};
use clap::Subcommand;

use crate::getifaddrs::{AddressFlags, InterfaceFlags, getifaddrs};
use crate::lookup::interface::{is_public_ipv4, is_public_ipv6};

#[derive(Debug, Subcommand, Clone)]
pub enum DebugCommand {
    /// Print all network interfaces and addresses as cf-ddns sees them.
    ///
    /// Shows link-level flags (UP, LOOPBACK) and per-address notes (temporary, deprecated,
    /// non-public). Useful for diagnosing why an interface or address is not being used.
    Interfaces,
}

impl DebugCommand {
    pub fn run(&self) -> Result<()> {
        match self {
            DebugCommand::Interfaces => print_interfaces(),
        }
    }
}

fn print_interfaces() -> Result<()> {
    let interfaces = getifaddrs().context("failed to enumerate network interfaces")?;

    for iface in &interfaces {
        let mut link_flags: Vec<&str> = Vec::new();
        if iface.flags.contains(InterfaceFlags::LOOPBACK) {
            link_flags.push("LOOPBACK");
        }
        if iface.flags.contains(InterfaceFlags::UP) {
            link_flags.push("UP");
        }
        println!("{} [{}]", iface.name, link_flags.join(", "));

        for addr in &iface.addresses {
            let mut notes: Vec<&str> = Vec::new();
            if addr.flags.contains(AddressFlags::TEMPORARY) {
                notes.push("temporary");
            }
            if addr.flags.contains(AddressFlags::DEPRECATED) {
                notes.push("deprecated");
            }
            let is_public = match addr.address {
                IpAddr::V4(a) => is_public_ipv4(a),
                IpAddr::V6(a) => is_public_ipv6(a),
            };
            if !is_public {
                notes.push("non-public");
            }
            let note = if notes.is_empty() {
                String::new()
            } else {
                format!("  ({})", notes.join(", "))
            };
            println!("  {:<47}{note}", addr.address);
        }
        println!();
    }

    Ok(())
}
