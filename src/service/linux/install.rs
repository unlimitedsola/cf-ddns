use std::fs;

use anyhow::{Context, Result};
use const_format::concatcp;

use crate::current_exe;
use crate::service::linux::{SERVICE_DESCRIPTION, SERVICE_NAME};

const UNIT_FILE: &str = concatcp!("/etc/systemd/system/", SERVICE_NAME, ".service");

pub fn install() -> Result<()> {
    let exec = current_exe()?;
    let unit_def = gen_unit_def(exec.to_str().context("path is not valid utf-8")?);
    fs::write(UNIT_FILE, unit_def.as_bytes())?;
    Ok(())
}

pub fn uninstall() -> Result<()> {
    fs::remove_file(UNIT_FILE)?;
    Ok(())
}

fn gen_unit_def(exec: &str) -> String {
    format!(
        include_str!("systemd.service"),
        desc = SERVICE_DESCRIPTION,
        exec = exec
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unit_def() {
        let exec = "/usr/local/bin/cf-ddns";
        assert_eq!(
            gen_unit_def(exec),
            r#"[Unit]
Description=Updates Cloudflare DNS records with the current public IP address.
After=network-online.target
Wants=network-online.target

[Service]
Type=simple
ExecStart=/usr/local/bin/cf-ddns service run
Restart=on-failure

[Install]
WantedBy=multi-user.target
"#,
        )
    }
}
