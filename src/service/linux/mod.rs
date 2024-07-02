use anyhow::Context;
use anyhow::Result;
use clap::Subcommand;
use tokio::signal::ctrl_c;

use crate::service::linux::install::{install, uninstall};
use crate::AppContext;

mod install;

const SERVICE_NAME: &str = "cf-ddns";

const SERVICE_DESCRIPTION: &str =
    "Updates Cloudflare DNS records with the current public IP address.";

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
