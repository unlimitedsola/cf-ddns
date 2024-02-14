#![cfg(windows)]

use anyhow::Result;
use clap::Subcommand;

pub use main::run_as_service;
pub use sys::is_in_windows_service;

use crate::service::windows::install::{install, uninstall};
use crate::AppContext;

mod install;
mod main;
mod sys;

#[derive(Debug, Subcommand, Clone)]
pub enum ServiceCommand {
    Install,
    Uninstall,
}

impl AppContext {
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        match command {
            ServiceCommand::Install => install()?,
            ServiceCommand::Uninstall => uninstall()?,
        }
        Ok(())
    }
}
