#![cfg(windows)]

use anyhow::Result;

#[allow(unused_imports)] // bug
pub use main::run_as_service;
#[allow(unused_imports)] // bug
pub use sys::is_in_windows_service;

use crate::AppContext;
use crate::cli::ServiceCommand;
use crate::service::windows::install::{install, uninstall};

mod install;
mod main;
mod sys;

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
