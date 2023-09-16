#![cfg(windows)]

use std::env::current_exe;

use anyhow::{Context, Result};
use windows::Win32::System::Services::SERVICE_AUTO_START;

#[allow(unused_imports)] // bug
pub use entry::run_as_service;
#[allow(unused_imports)] // bug
pub use sys::is_in_windows_service;

use crate::cli::ServiceCommand;
use crate::service::windows::sys::{ServiceCreateConfig, ServiceManager};
use crate::service::{SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME, SERVICE_NAME};
use crate::AppContext;

mod entry;
mod sys;

pub fn install() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    mgr.create_service(ServiceCreateConfig {
        name: SERVICE_NAME,
        display_name: SERVICE_DISPLAY_NAME,
        description: SERVICE_DESCRIPTION,
        start_type: SERVICE_AUTO_START,
        command: current_exe()?
            .to_str()
            .context("unable to get executable path")?,
    })?;
    Ok(())
}

pub fn uninstall() -> Result<()> {
    let mgr = ServiceManager::local_computer()?;
    mgr.open_service(SERVICE_NAME)?.delete()
}

impl AppContext {
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        match command {
            ServiceCommand::Install => install()?,
            ServiceCommand::Uninstall => uninstall()?,
            _ => {}
        }
        Ok(())
    }
}
