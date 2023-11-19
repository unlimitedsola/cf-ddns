use std::path::PathBuf;

use anyhow::{Context, Result};
use clap::Parser;

use crate::cli::Cli;
use crate::config::Config;

mod cli;
mod cloudflare;
mod config;
mod lookup;
mod service;
mod updater;

pub struct AppContext {
    pub cli: Cli,
    pub config: Config,
}

impl AppContext {
    pub fn new(cli: Cli) -> Result<Self> {
        let config = cli.config.as_ref().map_or_else(Config::load, Config::load_from)?;
        Ok(AppContext {
            cli,
            config,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    #[cfg(windows)]
    if service::is_in_windows_service()? {
        return service::run_as_service();
    }

    tracing_subscriber::fmt().init();

    let cli: Cli = Cli::parse();
    let app = AppContext::new(cli)?;
    app.run().await?;
    Ok(())
}

fn current_exe() -> Result<PathBuf> {
    std::env::current_exe().context("unable to get current executable path")
}
