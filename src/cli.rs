use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};
use tracing::instrument;

use crate::cli::Command::{Service, Update};
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
    Service {
        #[command(subcommand)]
        command: ServiceCommand,
    },
}

#[derive(Debug, Subcommand, Clone)]
pub enum ServiceCommand {
    Install,
    Uninstall,
    Start,
    Stop,
    Run,
    Debug,
}

impl AppContext {
    #[instrument(skip(self), fields(cli = ? self.cli), ret, err)]
    pub async fn run(&self) -> Result<()> {
        match self.cli.command.clone() {
            None => self.update(None).await?,
            Some(cmd) => match cmd {
                Update { ns } => self.update(ns.as_deref()).await?,
                Service { command } => self.run_service_command(&command).await?,
            },
        }
        Ok(())
    }
}
