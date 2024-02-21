use std::env::current_exe;
use std::fs;
use std::fs::remove_file;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use const_format::concatcp;

use crate::service::macos::SERVICE_NAME;

const PLIST_PATH: &str = concatcp!("/Library/LaunchDaemons/", SERVICE_NAME, ".plist");

pub fn install() -> Result<()> {
    let current_exe = current_exe().context("unable to get executable path")?;
    let log_path = current_exe.with_file_name(concatcp!(SERVICE_NAME, ".log"));

    let plist = gen_plist(
        current_exe.to_str().context("path is not valid utf-8")?,
        log_path.to_str().context("path is not valid utf-8")?,
    );

    fs::write(PLIST_PATH, plist).context("unable to write service file")?;
    launchctl(&["load", "-w", PLIST_PATH])
}

pub fn uninstall() -> Result<()> {
    launchctl(&["unload", PLIST_PATH])?;
    remove_file(PLIST_PATH).context("unable to remove service file")
}

fn gen_plist(exec: &str, log: &str) -> String {
    format!(
        include_str!("launchd.plist"),
        label = SERVICE_NAME,
        exec = exec,
        log = log
    )
}

const LAUNCHCTL: &str = "launchctl";

fn launchctl(args: &[&str]) -> Result<()> {
    let output = Command::new(LAUNCHCTL)
        .args(args)
        .stdin(Stdio::null())
        .stderr(Stdio::piped())
        .stdout(Stdio::piped())
        .output()?;
    if output.status.success() {
        Ok(())
    } else {
        let msg = String::from_utf8(output.stderr)
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                String::from_utf8(output.stdout)
                    .ok()
                    .filter(|s| !s.trim().is_empty())
            })
            .unwrap_or_else(|| format!("Failed to execute: {LAUNCHCTL} {args:?}"));

        bail!(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_gen() {
        let plist = gen_plist("/usr/local/bin/cf-ddns", "/var/log/cf-ddns.log");
        assert_eq!(
            plist,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>cf-ddns</string>

	<key>ProgramArguments</key>
	<array>
		<string>/usr/local/bin/cf-ddns</string>
		<string>service</string>
		<string>run</string>
	</array>

    <key>KeepAlive</key>
    <dict>
      <key>NetworkState</key>
      <true/>
    </dict>

	<key>RunAtLoad</key>
	<true/>

    <key>StandardOutPath</key>
    <string>/var/log/cf-ddns.log</string>
    <key>StandardErrorPath</key>
    <string>/var/log/cf-ddns.log</string>
</dict>
</plist>
"#
        );
    }
}
