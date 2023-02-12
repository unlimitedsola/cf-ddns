use std::sync::{Arc, Mutex};

use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;

use crate::cache::IdCache;
use crate::cloudflare::Client;
use crate::cmd::Cli;
use crate::cmd::Commands::{Service, Update};
use crate::config::Config;
use crate::lookup::Provider;

mod cache;
mod cloudflare;
mod cmd;
mod config;
mod lookup;
mod service;
mod updater;

pub struct AppContext {
    pub lookup: Provider,
    pub client: Client,
    pub id_cache: Arc<Mutex<IdCache>>,
    pub config: Config,
}

impl AppContext {
    pub fn new(config: Config) -> Result<Self> {
        let lookup = Provider::new(&config);
        let client = Client::new(&config.token)?;
        let id_cache = Arc::new(Mutex::new(IdCache::load().unwrap_or_default()));
        Ok(AppContext {
            lookup,
            client,
            id_cache,
            config,
        })
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = Config::load().with_context(|| "Unable to load config.")?;
    let ctx = Arc::new(AppContext::new(config)?);

    let cli: Cli = Cli::parse();

    match cli.command {
        None => ctx.clone().update(None).await?,
        Some(cmd) => match cmd {
            Update { ns } => ctx.clone().update(ns).await?,
            Service { command } => match command {
                _ => {}
            },
        },
    };

    ctx.id_cache.lock().unwrap().save()?;
    Ok(())
}
