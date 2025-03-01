use anyhow::Result;
use clap::Subcommand;

pub use main::run_as_service;
pub use sys::is_in_windows_service;

use crate::AppContext;
use crate::service::windows::install::{install, uninstall};

mod install;
mod main;
mod sys;

const SERVICE_NAME: &str = "cf-ddns";
const SERVICE_DISPLAY_NAME: &str = "Cloudflare DDNS";

const SERVICE_DESCRIPTION: &str =
    "Updates Cloudflare DNS records with the current public IP address.";

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
