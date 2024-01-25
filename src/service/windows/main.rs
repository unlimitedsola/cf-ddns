use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use futures::channel::oneshot::Receiver;
use tracing::{info, instrument};

use crate::cli::Cli;
use crate::service::windows::sys::run;
use crate::service::SERVICE_NAME;
use crate::{current_exe, AppContext};

async fn service_main_async(args: Vec<String>, cancel: Receiver<()>) -> Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let app = Arc::new(AppContext::new(cli)?);
    app.run_service(cancel).await
}

#[instrument(skip(cancel), ret, err)]
fn service_main(args: Vec<String>, cancel: Receiver<()>) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async { service_main_async(args, cancel).await })
}

pub fn run_as_service() -> Result<()> {
    let file_appender =
        tracing_appender::rolling::daily(current_exe()?.parent().unwrap(), "cf-ddns.log");
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(file_appender)
        .init();
    info!("Starting as a Windows service...");
    run(SERVICE_NAME, service_main)
}
