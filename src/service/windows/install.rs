use crate::service::windows::sys::{ServiceCreateConfig, ServiceManager};
use crate::service::{SERVICE_DESCRIPTION, SERVICE_DISPLAY_NAME, SERVICE_NAME};
use anyhow::{Context, Result};
use std::env::current_exe;
use windows::Win32::System::Services::SERVICE_AUTO_START;

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
