use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use clap::Subcommand;
use tokio::signal::ctrl_c;

use crate::AppContext;
use crate::service::linux::install::{install, log, start, status, stop, uninstall};

mod install;

const SERVICE_NAME: &str = "cf-ddns";

const SERVICE_DESCRIPTION: &str =
    "Updates Cloudflare DNS records with the current public IP address.";

#[derive(Debug, Subcommand, Clone)]
pub enum ServiceCommand {
    Install {
        /// Optional path to the configuration file for the service.
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Optional path to the zone/record ID cache file for the service.
        #[arg(long)]
        id_cache: Option<PathBuf>,
    },
    Uninstall,
    Start,
    Stop,
    Status,
    Log {
        /// Stream log output continuously (follow log).
        #[arg(short, long)]
        follow: bool,

        /// Number of lines to output (default 1000).
        #[arg(short = 'n', long, default_value_t = 1000)]
        lines: usize,
    },
    Run,
}

impl AppContext {
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        match command {
            ServiceCommand::Install { config, id_cache } => {
                install(config.as_deref(), id_cache.as_deref())
            }
            ServiceCommand::Uninstall => uninstall(),
            ServiceCommand::Start => start(),
            ServiceCommand::Stop => stop(),
            ServiceCommand::Status => status(),
            ServiceCommand::Log { follow, lines } => log(*follow, *lines),
            ServiceCommand::Run => self.run_service(ctrl_c()).await,
        }
        .with_context(|| format!("unable to run service command: {command:?}"))
    }
}
