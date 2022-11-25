use std::sync::Arc;

use anyhow::{Context, Result};
use clap::Parser;
use env_logger::Env;

use crate::cmd::Cli;
use crate::cmd::Commands::Update;
use crate::config::Config;
use crate::context::AppContext;

mod cache;
mod cloudflare;
mod cmd;
mod config;
mod context;
mod lookup;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
    let config = Config::load().with_context(|| "Unable to load config.")?;
    let ctx = Arc::new(AppContext::new(config)?);

    let cli: Cli = Cli::parse();

    match cli.command {
        None => ctx.clone().update(None),
        Some(cmd) => match cmd {
            Update { ns } => ctx.clone().update(ns),
        },
    }
    .await?;

    ctx.id_cache.lock().unwrap().save()?;
    Ok(())
}
