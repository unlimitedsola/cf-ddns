use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::instrument;

#[cfg(feature = "service")]
use crate::service::ServiceCommand;
use crate::AppContext;

#[derive(Debug, Parser)]
#[command(name = "cf-ddns")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    Update {
        ns: Option<String>,
    },
    #[cfg(feature = "service")]
    #[command(subcommand)]
    Service(ServiceCommand),
}

impl AppContext {
    #[instrument(skip(self), fields(cli = ? self.cli), ret, err)]
    pub async fn run(&self) -> Result<()> {
        match self.cli.command.clone() {
            None => self.update(None).await?,
            Some(cmd) => match cmd {
                Command::Update { ns } => self.update(ns.as_deref()).await?,
                #[cfg(feature = "service")]
                Command::Service(command) => self.run_service_command(&command).await?,
            },
        }
        Ok(())
    }
}
