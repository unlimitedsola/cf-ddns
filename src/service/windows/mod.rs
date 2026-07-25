use anyhow::Result;
use clap::Subcommand;

pub use main::run_as_service;
pub use sys::is_in_windows_service;

use crate::AppContext;
use crate::service::windows::install::{install, log, start, status, stop, uninstall};

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
}

impl AppContext {
    #[expect(
        clippy::unused_async,
        reason = "matching cross-platform async signature required by cli caller"
    )]
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        match command {
            ServiceCommand::Install => install()?,
            ServiceCommand::Uninstall => uninstall()?,
            ServiceCommand::Start => start()?,
            ServiceCommand::Stop => stop()?,
            ServiceCommand::Status => status()?,
            ServiceCommand::Log { follow, lines } => log(*follow, *lines)?,
        }
        Ok(())
    }
}
