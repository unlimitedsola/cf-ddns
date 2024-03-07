use anyhow::Result;
use const_format::concatcp;
use std::fs;

use crate::current_exe_str;
use crate::service::exec::exec;
use crate::service::linux::{SERVICE_DESCRIPTION, SERVICE_NAME};

const UNIT_FILE: &str = concatcp!("/etc/systemd/system/", SERVICE_NAME, ".service");

pub fn install() -> Result<()> {
    let unit_def = gen_unit_def(current_exe_str());
    fs::write(UNIT_FILE, unit_def.as_bytes())?;
    exec(SYSTEMCTL, &["daemon-reload"])?;
    exec(SYSTEMCTL, &["enable", "--now", SERVICE_NAME])?;
    Ok(())
}

pub fn uninstall() -> Result<()> {
    exec(SYSTEMCTL, &["disable", "--now", SERVICE_NAME])?;
    fs::remove_file(UNIT_FILE)?;
    exec(SYSTEMCTL, &["daemon-reload"])?;
    Ok(())
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
