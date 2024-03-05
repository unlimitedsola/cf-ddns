use std::path::{Path, PathBuf};
use std::sync::OnceLock;

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

struct AppContext {
    cli: Cli,
    config: Config,
}

impl AppContext {
    fn new(cli: Cli) -> Result<Self> {
        let config = Config::load()?;
        Ok(AppContext { cli, config })
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    #[cfg(all(windows, feature = "service"))]
    if service::is_in_windows_service()? {
        return service::run_as_service();
    }

    tracing_subscriber::fmt().init();

    let cli: Cli = Cli::parse();
    let app = AppContext::new(cli)?;
    app.run().await?;
    Ok(())
}

fn current_exe() -> &'static Path {
    static EXE: OnceLock<PathBuf> = OnceLock::new();
    EXE.get_or_init(|| {
        std::env::current_exe()
            .context("unable to get current executable path")
            .unwrap()
    })
}

fn current_exe_str() -> &'static str {
    static STR: OnceLock<&'static str> = OnceLock::new();
    STR.get_or_init(|| {
        current_exe()
            .to_str()
            .context("unable to convert current executable path to string")
            .unwrap()
    })
}
