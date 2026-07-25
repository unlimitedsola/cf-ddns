#![cfg(not(windows))]

use std::process::Command;

use anyhow::{Result, bail};

pub fn exec(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        let full_cmd = if args.is_empty() {
            cmd.to_owned()
        } else {
            format!("{cmd} {}", args.join(" "))
        };
        bail!("command `{full_cmd}` failed: {status}");
    }
    Ok(())
}
