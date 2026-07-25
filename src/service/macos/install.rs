use std::ffi::CString;
use std::fs;
use std::fs::remove_file;
use std::os::unix::fs::MetadataExt;
use std::path::Path;

use anyhow::{Context, Result};
use const_format::concatcp;
use tracing::warn;

use crate::service::exec::exec;
use crate::service::macos::SERVICE_NAME;
use crate::{current_exe, current_exe_str};

const PLIST_PATH: &str = concatcp!("/Library/LaunchDaemons/", SERVICE_NAME, ".plist");

pub fn install(user: Option<&str>) -> Result<()> {
    let log_path = current_exe().with_file_name(concatcp!(SERVICE_NAME, ".log"));

    if let Some(u) = user {
        check_writable_for_user(&log_path, u);
        let id_cache_path = Path::new("/tmp/cf-ddns.json");
        check_writable_for_user(id_cache_path, u);
    }

    let plist = gen_plist(
        current_exe_str(),
        log_path.to_str().expect("path should be valid UTF-8"),
        user,
    );

    fs::write(PLIST_PATH, plist).context("unable to write service file")?;
    exec(LAUNCHCTL, &["bootstrap", "system", PLIST_PATH])
}

pub fn uninstall() -> Result<()> {
    exec(LAUNCHCTL, &["bootout", "system", PLIST_PATH])?;
    remove_file(PLIST_PATH).context("unable to remove service file")
}

fn gen_plist(exec: &str, log: &str, user: Option<&str>) -> String {
    let user_section = match user {
        Some(u) => format!("\n\n\t<key>UserName</key>\n\t<string>{u}</string>"),
        None => String::new(),
    };
    let extra_args = match user {
        Some(_) => {
            "\n\t\t<string>--id-cache</string>\n\t\t<string>/tmp/cf-ddns.json</string>".to_owned()
        }
        None => String::new(),
    };

    format!(
        include_str!("launchd.plist"),
        label = SERVICE_NAME,
        user_section = user_section,
        exec = exec,
        extra_args = extra_args,
        log = log
    )
}

#[expect(
    clippy::similar_names,
    reason = "target_uid and target_gid represent the target user's UID and GID"
)]
fn check_writable_for_user(path: &Path, user: &str) {
    let Ok(c_user) = CString::new(user) else {
        warn!("invalid user name specified: '{user}'");
        return;
    };

    let pwd = unsafe { libc::getpwnam(c_user.as_ptr()) };
    if pwd.is_null() {
        warn!("user '{user}' was not found on this system");
        return;
    }

    let (target_uid, target_gid) = unsafe { ((*pwd).pw_uid, (*pwd).pw_gid) };

    let target = path
        .ancestors()
        .find(|p| p.exists())
        .unwrap_or_else(|| Path::new("/"));

    let Ok(meta) = target.metadata() else {
        warn!("unable to query metadata for path '{}'", target.display());
        return;
    };

    let mode = meta.mode();
    let uid = meta.uid();
    let gid = meta.gid();

    let is_writable = if uid == target_uid {
        mode & 0o200 != 0
    } else if gid == target_gid {
        mode & 0o020 != 0
    } else {
        mode & 0o002 != 0
    };

    if !is_writable {
        warn!(
            "location '{}' (checked target '{}') may not be writable by user '{user}'",
            path.display(),
            target.display()
        );
    }
}

const LAUNCHCTL: &str = "launchctl";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plist_gen_default_rootful() {
        let plist = gen_plist("/usr/local/bin/cf-ddns", "/var/log/cf-ddns.log", None);
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

    #[test]
    fn plist_gen_with_user() {
        let plist = gen_plist("/usr/local/bin/cf-ddns", "/var/log/cf-ddns.log", Some("nobody"));
        assert_eq!(
            plist,
            r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>Label</key>
	<string>cf-ddns</string>

	<key>UserName</key>
	<string>nobody</string>

	<key>ProgramArguments</key>
	<array>
		<string>/usr/local/bin/cf-ddns</string>
		<string>service</string>
		<string>run</string>
		<string>--id-cache</string>
		<string>/tmp/cf-ddns.json</string>
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

    #[test]
    fn check_user_permission_warning() {
        // Verify check_writable_for_user executes safely without panicking for invalid, missing, and nobody users
        check_writable_for_user(Path::new("/tmp/cf-ddns.json"), "nobody");
        check_writable_for_user(Path::new("/var/log/cf-ddns.log"), "nonexistent_user_12345");
    }
}
