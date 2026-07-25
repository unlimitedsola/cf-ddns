use std::fs;
use std::path::Path;

use anyhow::{bail, Context, Result};
use const_format::concatcp;

use crate::current_exe_str;
use crate::service::exec::exec;
use crate::service::linux::{SERVICE_DESCRIPTION, SERVICE_NAME};

const UNIT_FILE: &str = concatcp!("/etc/systemd/system/", SERVICE_NAME, ".service");

pub fn install() -> Result<()> {
    let unit_path = Path::new(UNIT_FILE);
    if unit_path.exists() {
        bail!(
            "service '{SERVICE_NAME}' is already installed at {}",
            unit_path.display()
        );
    }

    let unit_def = gen_unit_def(current_exe_str());
    fs::write(UNIT_FILE, unit_def.as_bytes()).with_context(|| {
        format!("unable to write systemd unit file at {UNIT_FILE} (did you forget 'sudo'?)")
    })?;
    exec(SYSTEMCTL, &["daemon-reload"])
        .with_context(|| "unable to reload systemd daemon (did you forget 'sudo'?)")?;
    exec(SYSTEMCTL, &["enable", "--now", SERVICE_NAME])
        .with_context(|| "unable to enable systemd service (did you forget 'sudo'?)")?;
    Ok(())
}

fn ensure_installed() -> Result<()> {
    let unit_path = Path::new(UNIT_FILE);
    if !unit_path.exists() {
        bail!("service '{SERVICE_NAME}' is not installed");
    }
    Ok(())
}

pub fn uninstall() -> Result<()> {
    ensure_installed()?;

    exec(SYSTEMCTL, &["disable", "--now", SERVICE_NAME])
        .with_context(|| "unable to disable systemd service (did you forget 'sudo'?)")?;
    fs::remove_file(UNIT_FILE).with_context(|| {
        format!("unable to remove systemd unit file at {UNIT_FILE} (did you forget 'sudo'?)")
    })?;
    exec(SYSTEMCTL, &["daemon-reload"])
        .with_context(|| "unable to reload systemd daemon (did you forget 'sudo'?)")?;
    Ok(())
}

pub fn start() -> Result<()> {
    ensure_installed()?;
    exec(SYSTEMCTL, &["start", SERVICE_NAME]).with_context(|| {
        "unable to start systemd service (did you forget 'sudo'?)"
    })
}

pub fn stop() -> Result<()> {
    ensure_installed()?;
    exec(SYSTEMCTL, &["stop", SERVICE_NAME]).with_context(|| {
        "unable to stop systemd service (did you forget 'sudo'?)"
    })
}

pub fn status() -> Result<()> {
    ensure_installed()?;
    exec(SYSTEMCTL, &["status", SERVICE_NAME]).with_context(|| {
        "unable to query systemd service status (did you forget 'sudo'?)"
    })
}

pub fn log(follow: bool, lines: usize) -> Result<()> {
    let lines_str = lines.to_string();
    if follow {
        exec("journalctl", &["-u", SERVICE_NAME, "-n", &lines_str, "-f"])
    } else {
        exec("journalctl", &["-u", SERVICE_NAME, "-n", &lines_str])
    }
}

fn gen_unit_def(exec: &str) -> String {
    format!(
        include_str!("systemd.service"),
        desc = SERVICE_DESCRIPTION,
        exec = exec
    )
}

const SYSTEMCTL: &str = "systemctl";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_def() {
        let exec = "/usr/local/bin/cf-ddns";
        assert_eq!(
            gen_unit_def(exec),
            r"[Unit]
Description=Updates Cloudflare DNS records with the current public IP address.
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/cf-ddns service run
Restart=on-failure

[Install]
WantedBy=multi-user.target
",
        );
    }
}
