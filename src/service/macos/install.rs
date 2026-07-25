use std::ffi::CString;
use std::fs;
use std::fs::remove_file;
use std::io::Write;
use std::os::unix::fs::MetadataExt;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use const_format::concatcp;
use serde::Serialize;
use tracing::warn;

use crate::service::exec::exec;
use crate::service::macos::SERVICE_NAME;
use crate::{current_exe, current_exe_str};

const PLIST_PATH: &str = concatcp!("/Library/LaunchDaemons/", SERVICE_NAME, ".plist");

pub fn install(
    user: Option<&str>,
    id_cache: Option<&Path>,
    log_file: Option<&Path>,
) -> Result<()> {
    let plist_path = Path::new(PLIST_PATH);
    if plist_path.exists() {
        bail!(
            "service '{SERVICE_NAME}' is already installed at {}",
            plist_path.display()
        );
    }

    let log_path = log_file.map_or_else(default_log_path, Path::to_path_buf);
    let id_cache_path = id_cache.map_or_else(default_id_cache_path, Path::to_path_buf);

    if let Some(u) = user {
        check_writable_for_user(&log_path, u);
        check_writable_for_user(&id_cache_path, u);
    }

    let file = fs::File::create(PLIST_PATH).with_context(|| {
        format!("unable to create service file at {PLIST_PATH} (did you forget 'sudo'?)")
    })?;
    write_plist(
        file,
        current_exe_str(),
        log_path.to_str().expect("path should be valid UTF-8"),
        user,
        id_cache.and_then(Path::to_str),
    )?;

    exec(LAUNCHCTL, &["bootstrap", "system", PLIST_PATH]).with_context(|| {
        "unable to register service with launchctl (did you forget 'sudo'?)"
    })
}

fn default_log_path() -> PathBuf {
    current_exe().with_file_name(concatcp!(SERVICE_NAME, ".log"))
}

fn default_id_cache_path() -> PathBuf {
    current_exe().with_file_name("id_cache.json")
}

pub fn uninstall() -> Result<()> {
    let plist_path = Path::new(PLIST_PATH);
    if !plist_path.exists() {
        bail!("service '{SERVICE_NAME}' is not installed");
    }

    exec(LAUNCHCTL, &["bootout", "system", PLIST_PATH]).with_context(|| {
        "unable to unregister service with launchctl (did you forget 'sudo'?)"
    })?;
    remove_file(PLIST_PATH).with_context(|| {
        format!("unable to remove service file at {PLIST_PATH} (did you forget 'sudo'?)")
    })
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct KeepAlive {
    network_state: bool,
}

#[derive(Serialize)]
#[serde(rename_all = "PascalCase")]
struct LaunchdPlist<'a> {
    label: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_name: Option<&'a str>,
    program_arguments: Vec<&'a str>,
    keep_alive: KeepAlive,
    run_at_load: bool,
    standard_out_path: &'a str,
    standard_error_path: &'a str,
}

fn write_plist<W: Write>(
    writer: W,
    exec: &str,
    log: &str,
    user: Option<&str>,
    id_cache: Option<&str>,
) -> Result<()> {
    let mut program_arguments = vec![exec, "service", "run"];
    if let Some(cache_path) = id_cache {
        program_arguments.push("--id-cache");
        program_arguments.push(cache_path);
    }

    let plist = LaunchdPlist {
        label: SERVICE_NAME,
        user_name: user,
        program_arguments,
        keep_alive: KeepAlive { network_state: true },
        run_at_load: true,
        standard_out_path: log,
        standard_error_path: log,
    };

    plist::to_writer_xml(writer, &plist).context("unable to serialize launchd plist XML")
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
    fn plist_gen_default_rootful() -> Result<()> {
        let mut buf = Vec::new();
        write_plist(&mut buf, "/usr/local/bin/cf-ddns", "/var/log/cf-ddns.log", None, None)?;
        let val: plist::Value = plist::from_bytes(&buf).context("parse plist")?;
        let dict = val.as_dictionary().context("dict missing")?;

        assert_eq!(
            dict.get("Label").and_then(|v| v.as_string()),
            Some("cf-ddns")
        );
        assert_eq!(dict.get("UserName"), None);
        assert_eq!(
            dict.get("RunAtLoad").and_then(|v| v.as_boolean()),
            Some(true)
        );
        assert_eq!(
            dict.get("StandardOutPath").and_then(|v| v.as_string()),
            Some("/var/log/cf-ddns.log")
        );

        let args: Vec<&str> = dict
            .get("ProgramArguments")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
            .unwrap_or_default();

        assert_eq!(args, vec!["/usr/local/bin/cf-ddns", "service", "run"]);
        Ok(())
    }

    #[test]
    fn plist_gen_with_user() -> Result<()> {
        let mut buf = Vec::new();
        write_plist(&mut buf, "/usr/local/bin/cf-ddns", "/var/log/cf-ddns.log", Some("nobody"), None)?;
        let val: plist::Value = plist::from_bytes(&buf).context("parse plist")?;
        let dict = val.as_dictionary().context("dict missing")?;

        assert_eq!(
            dict.get("Label").and_then(|v| v.as_string()),
            Some("cf-ddns")
        );
        assert_eq!(
            dict.get("UserName").and_then(|v| v.as_string()),
            Some("nobody")
        );

        let args: Vec<&str> = dict
            .get("ProgramArguments")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
            .unwrap_or_default();

        assert_eq!(args, vec!["/usr/local/bin/cf-ddns", "service", "run"]);
        Ok(())
    }

    #[test]
    fn plist_gen_with_user_and_id_cache() -> Result<()> {
        let mut buf = Vec::new();
        write_plist(
            &mut buf,
            "/usr/local/bin/cf-ddns",
            "/var/log/custom.log",
            Some("nobody"),
            Some("/tmp/custom_cache.json"),
        )?;
        let val: plist::Value = plist::from_bytes(&buf).context("parse plist")?;
        let dict = val.as_dictionary().context("dict missing")?;

        assert_eq!(
            dict.get("UserName").and_then(|v| v.as_string()),
            Some("nobody")
        );
        assert_eq!(
            dict.get("StandardOutPath").and_then(|v| v.as_string()),
            Some("/var/log/custom.log")
        );
        assert_eq!(
            dict.get("StandardErrorPath").and_then(|v| v.as_string()),
            Some("/var/log/custom.log")
        );

        let args: Vec<&str> = dict
            .get("ProgramArguments")
            .and_then(|v| v.as_array())
            .map(|arr| arr.iter().filter_map(|v| v.as_string()).collect())
            .unwrap_or_default();

        assert_eq!(
            args,
            vec![
                "/usr/local/bin/cf-ddns",
                "service",
                "run",
                "--id-cache",
                "/tmp/custom_cache.json"
            ]
        );
        Ok(())
    }

    #[test]
    fn check_user_permission_warning() {
        // Verify check_writable_for_user executes safely without panicking for invalid, missing, and nobody users
        check_writable_for_user(Path::new("/tmp/cf-ddns.json"), "nobody");
        check_writable_for_user(Path::new("/var/log/cf-ddns.log"), "nonexistent_user_12345");
    }
}
