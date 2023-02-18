use std::path::PathBuf;

use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::cli::Commands::{Service, Update};
use crate::AppContext;

#[derive(Debug, Parser)]
#[command(name = "cf-ddns")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Sets a custom config file
    #[arg(short, long, value_name = "PATH")]
    pub config: Option<PathBuf>,
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Update {
        ns: Option<String>,
    },
    Service {
        #[command(subcommand)]
        command: ServiceCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ServiceCommands {
    Install,
    Remove,
    Start,
    Stop,
    Run,
    Debug,
}

impl AppContext {
    pub async fn run(&self) -> Result<()> {
        match self.cli.command.as_ref() {
            None => self.update(None).await,
            Some(cmd) => match cmd {
                Update { ns } => self.update(ns.as_ref()).await,
                Service { command } => match command {
                    _ => Ok(()),
                },
            },
        }
    }
}
