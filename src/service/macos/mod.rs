use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Subcommand;
use tokio::signal::ctrl_c;

use crate::AppContext;
use crate::service::macos::install::{install, uninstall};

mod install;

const SERVICE_NAME: &str = "cf-ddns";

#[derive(Debug, Subcommand, Clone)]
pub enum ServiceCommand {
    Install {
        /// Optional user account to run the daemon as (e.g. --user nobody).
        /// If omitted, runs as root (default).
        #[arg(long)]
        user: Option<String>,

        /// Optional path to the zone/record ID cache file for the service.
        #[arg(long)]
        id_cache: Option<PathBuf>,
    },
    Uninstall,
    Run,
}

impl AppContext {
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        match command {
            ServiceCommand::Install { user, id_cache } => {
                install(user.as_deref(), id_cache.as_deref())
            }
            ServiceCommand::Uninstall => uninstall(),
            ServiceCommand::Run => self.run_service(ctrl_c()).await,
        }
        .with_context(|| format!("unable to run service command: {command:?}"))
    }
}
