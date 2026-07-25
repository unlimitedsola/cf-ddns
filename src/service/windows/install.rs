use std::fs::File;
use std::io::{BufRead, BufReader, Seek, SeekFrom};
use std::path::PathBuf;
use std::thread::sleep;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use windows::Win32::System::Services::{
    SERVICE_AUTO_START, SERVICE_CONTINUE_PENDING, SERVICE_PAUSED, SERVICE_PAUSE_PENDING,
    SERVICE_RUNNING, SERVICE_START_PENDING, SERVICE_STOPPED, SERVICE_STOP_PENDING,
};

use crate::service::windows::sys::{ServiceCreateConfig, ServiceManager};
use crate::service::windows::{SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME, SERVICE_NAME};
use crate::{current_exe, current_exe_str};

pub fn install() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    let svc = mgr.create_service(ServiceCreateConfig {
        name: SERVICE_NAME,
        display_name: SERVICE_DISPLAY_NAME,
        description: SERVICE_DESCRIPTION,
        start_type: SERVICE_AUTO_START,
        command: current_exe_str(),
    })?;
    svc.start()
}

pub fn uninstall() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    mgr.open_service(SERVICE_NAME)?.delete()
}

pub fn start() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    mgr.open_service(SERVICE_NAME)?.start()
}

pub fn stop() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    mgr.open_service(SERVICE_NAME)?.stop()
}

#[expect(clippy::print_stderr, reason = "CLI user status notification")]
pub fn status() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    let svc = mgr.open_service(SERVICE_NAME)?;
    let status = svc.query_status()?;
    let state_str = match status.dwCurrentState {
        SERVICE_STOPPED => "stopped",
        SERVICE_START_PENDING => "start pending",
        SERVICE_STOP_PENDING => "stop pending",
        SERVICE_RUNNING => "running",
        SERVICE_CONTINUE_PENDING => "continue pending",
        SERVICE_PAUSE_PENDING => "pause pending",
        SERVICE_PAUSED => "paused",
        _ => "unknown",
    };
    eprintln!("service '{SERVICE_NAME}' is {state_str}");
    Ok(())
}

fn default_log_path() -> PathBuf {
    current_exe().with_file_name("cf-ddns.log")
}

#[expect(clippy::print_stderr, reason = "CLI log output")]
pub fn log(follow: bool, lines: usize) -> Result<()> {
    let log_path = default_log_path();
    if !log_path.exists() {
        bail!("log file '{}' does not exist yet", log_path.display());
    }

    let content = std::fs::read_to_string(&log_path)
        .with_context(|| format!("unable to read log file {}", log_path.display()))?;
    let all_lines: Vec<&str> = content.lines().collect();
    let start = all_lines.len().saturating_sub(lines);
    for line in &all_lines[start..] {
        eprintln!("{line}");
    }

    if !follow {
        return Ok(());
    }

    let file = File::open(&log_path)
        .with_context(|| format!("unable to open log file {}", log_path.display()))?;
    let mut reader = BufReader::new(file);
    reader.seek(SeekFrom::End(0))?;
    let mut line = String::new();
    loop {
        line.clear();
        match reader.read_line(&mut line) {
            Ok(0) => sleep(Duration::from_millis(250)),
            Ok(_) => {
                eprint!("{line}");
            }
            Err(e) => return Err(e.into()),
        }
    }
}
