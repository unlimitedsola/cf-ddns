use std::fs;
use std::fs::remove_file;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use const_format::concatcp;

use crate::service::macos::SERVICE_NAME;
use crate::{current_exe, current_exe_str};

const PLIST_PATH: &str = concatcp!("/Library/LaunchDaemons/", SERVICE_NAME, ".plist");

pub fn install() -> Result<()> {
    let log_path = current_exe().with_file_name(concatcp!(SERVICE_NAME, ".log"));

    let plist = gen_plist(current_exe_str(), log_path.to_str().unwrap());

    fs::write(PLIST_PATH, plist).context("unable to write service file")?;
    exec(LAUNCHCTL, &["load", "-w", PLIST_PATH])
}

pub fn uninstall() -> Result<()> {
    exec(LAUNCHCTL, &["unload", "-w", PLIST_PATH])?;
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
