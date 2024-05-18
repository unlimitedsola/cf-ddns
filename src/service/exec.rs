#![cfg(not(windows))]

use std::process::Command;

use anyhow::{bail, Result};

pub fn exec(cmd: &str, args: &[&str]) -> Result<()> {
    let status = Command::new(cmd).args(args).status()?;
    if !status.success() {
        bail!("`{} {:?}` failed with status: {}", cmd, args, status);
    }
    Ok(())
}
