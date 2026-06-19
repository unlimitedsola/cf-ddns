#![cfg(not(windows))]

use std::process::Command;

use anyhow::{Result, bail};

pub fn exec(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        bail!("`{cmd} {args:?}` failed with status: {status}");
    }
    Ok(())
}
