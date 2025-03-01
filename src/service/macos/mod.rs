use anyhow::{Context, Result};
use clap::Subcommand;
use tokio::signal::ctrl_c;

use crate::AppContext;
use crate::service::macos::install::{install, uninstall};

mod install;

const SERVICE_NAME: &str = "cf-ddns";

#[derive(Debug, Subcommand, Clone)]
pub enum ServiceCommand {
    Install,
    Uninstall,
    Run,
}

impl AppContext {
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        match command {
            ServiceCommand::Install => install(),
            ServiceCommand::Uninstall => uninstall(),
            ServiceCommand::Run => self.run_service(ctrl_c()).await,
        }
        .with_context(|| format!("unable to run service command: {:?}", command))
    }
}
