#![cfg(target_os = "macos")]

use anyhow::{Context, Result};
use tokio::signal::ctrl_c;

use crate::AppContext;
use crate::cli::ServiceCommand;
use crate::service::macos::install::{install, uninstall};

mod install;

impl AppContext {
    pub async fn run_service_command(&self, command: &ServiceCommand) -> Result<()> {
        match command {
            ServiceCommand::Install => install(),
            ServiceCommand::Uninstall => uninstall(),
            ServiceCommand::Run => self.run_service(ctrl_c()).await,
            _ => Ok(())
        }.with_context(|| format!("unable to run service command: {:?}", command))
    }
}
