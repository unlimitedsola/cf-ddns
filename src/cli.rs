use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::AppContext;
#[cfg(feature = "service")]
use crate::service::ServiceCommand;

#[derive(Debug, Parser)]
#[command(name = "cf-ddns")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    Update {
        name: Option<String>,
    },
    #[cfg(feature = "service")]
    #[command(subcommand)]
    Service(ServiceCommand),
}

impl AppContext {
    pub async fn run(&self) -> Result<()> {
        match self.cli.command.clone() {
            None => self.update(None).await?,
            Some(cmd) => match cmd {
                Command::Update { name } => self.update(name.as_deref()).await?,
                #[cfg(feature = "service")]
                Command::Service(command) => self.run_service_command(&command).await?,
            },
        }
        Ok(())
    }
}
