#![cfg(windows)]

use std::env::current_exe;

use anyhow::{Context, Result};
use windows::Win32::System::Services::{
    SC_MANAGER_ALL_ACCESS, SERVICE_ALL_ACCESS, SERVICE_AUTO_START, SERVICE_ERROR_CRITICAL,
    SERVICE_WIN32_OWN_PROCESS,
};

pub use entry::service_entry;
pub use sys::is_in_windows_service;

use crate::service::windows::sys::{ServiceConfig, ServiceManager};

mod entry;
mod sys;

fn install() -> Result<()> {
    let mgr = ServiceManager::local_computer(SC_MANAGER_ALL_ACCESS)?;
    mgr.create_service(
        ServiceConfig {
            name: "cf-ddns",
            display_name: "Cloudflare DDNS",
            service_type: SERVICE_WIN32_OWN_PROCESS,
            start_type: SERVICE_AUTO_START,
            error_control: SERVICE_ERROR_CRITICAL,
            command: current_exe()?.to_str().context("Invalid path")?,
        },
        SERVICE_ALL_ACCESS,
    )?;
    Ok(())
}
