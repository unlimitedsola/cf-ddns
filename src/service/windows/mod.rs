#![cfg(windows)]

use std::env::current_exe;

use anyhow::{Context, Result};
use windows::Win32::System::Services::SERVICE_AUTO_START;

pub use entry::run_as_service;
pub use sys::is_in_windows_service;

use crate::service::windows::sys::{ServiceCreateConfig, ServiceManager};
use crate::service::{SERVICE_DISPLAY_NAME, SERVICE_NAME};

mod entry;
mod sys;

pub fn install() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    mgr.create_service(ServiceCreateConfig {
        name: SERVICE_NAME,
        display_name: SERVICE_DISPLAY_NAME,
        start_type: SERVICE_AUTO_START,
        command: current_exe()?.to_str().context("Invalid path")?,
    })?;
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    mgr.open_service(SERVICE_NAME)?.delete()
}
