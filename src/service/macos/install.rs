use std::env::current_exe;
use std::fs::remove_file;
use std::process::{Command, Stdio};

use anyhow::{bail, Context, Result};
use const_format::formatcp;
use serde::Serialize;

use crate::service::SERVICE_NAME;

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct LaunchdConfig<'a> {
    label: &'a str,
    program_arguments: Box<[&'a str]>,
    keep_alive: bool,
    run_at_load: bool,
    standard_out_path: Option<&'a str>,
    standard_error_path: Option<&'a str>,
}

impl Default for LaunchdConfig<'_> {
    fn default() -> Self {
        LaunchdConfig {
            label: SERVICE_NAME,
            program_arguments: Box::new([]),
            keep_alive: true,
            run_at_load: true,
            standard_out_path: None,
            standard_error_path: None,
        }
    }
}

const PLIST_PATH: &str = formatcp!("/Library/LaunchDaemons/{SERVICE_NAME}.plist");

pub fn install() -> Result<()> {
    let current_exe = current_exe().context("unable to get executable path")?;
    let log_path = current_exe.with_file_name(formatcp!("{SERVICE_NAME}.log"));

    let cfg = LaunchdConfig {
        program_arguments: Box::new([
            current_exe
                .to_str()
                .context("unable to get executable path")?,
            "service",
            "run",
        ]),
        standard_out_path: log_path.to_str(),
        standard_error_path: log_path.to_str(),
        ..Default::default()
    };
    plist::to_file_xml(PLIST_PATH, &cfg).context("unable to write service file")?;
    launchctl(&["load", "-w", PLIST_PATH])
}

pub fn uninstall() -> Result<()> {
    launchctl(&["unload", PLIST_PATH])?;
    remove_file(PLIST_PATH).context("unable to remove service file")
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
    fn plist_deserialize() {
        let cfg = LaunchdConfig {
            program_arguments: Box::new(["test", "test"]),
            ..Default::default()
        };
        let mut buf = vec![];
        plist::to_writer_xml(&mut buf, &cfg).unwrap();
        let xml = String::from_utf8(buf).unwrap();
        assert_eq!(
            xml,
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>{SERVICE_NAME}</string>
	<key>ProgramArguments</key>
	<array>
		<string>test</string>
		<string>test</string>
	</array>
	<key>KeepAlive</key>
	<true/>
	<key>RunAtLoad</key>
	<true/>
</dict>
</plist>"#
            )
        );
    }

    #[test]
    fn plist_path() {
        assert_eq!(
            PLIST_PATH,
            format!("/Library/LaunchDaemons/{SERVICE_NAME}.plist")
        )
    }
}
