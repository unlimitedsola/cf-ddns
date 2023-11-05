use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use clap::Parser;

use crate::cache::IdCache;
use crate::cli::Cli;
use crate::cloudflare::Client;
use crate::config::Config;
use crate::lookup::Provider;

mod cache;
mod cli;
mod cloudflare;
mod config;
mod lookup;
mod service;
mod updater;

pub struct AppContext {
    pub cli: Cli,
    pub lookup: Provider,
    pub client: Client,
    pub id_cache: Arc<Mutex<IdCache>>,
    pub config: Config,
}

impl AppContext {
    pub fn new(cli: Cli) -> Result<Self> {
        let config = Config::load(cli.config.as_ref()).context("Unable to load config.")?;
        let lookup = Provider::new(&config).context("Unable to initialize lookup provider")?;
        let client = Client::new(&config.token)?;
        let id_cache = Arc::new(Mutex::new(IdCache::load().unwrap_or_default()));
        Ok(AppContext {
            cli,
            lookup,
            client,
            id_cache,
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
    app.id_cache.lock().unwrap().save()?;
    Ok(())
}
