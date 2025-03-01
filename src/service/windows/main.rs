use std::sync::Arc;

use anyhow::Result;
use clap::Parser;
use futures::channel::oneshot::Receiver;
use tracing::info;
use tracing_appender::rolling::RollingFileAppender;

use crate::cli::Cli;
use crate::service::windows::SERVICE_NAME;
use crate::service::windows::sys::run;
use crate::{AppContext, current_exe};

pub fn run_as_service() -> Result<()> {
    let file_appender = RollingFileAppender::builder()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .filename_prefix("cf-ddns")
        .filename_suffix("log")
        .max_log_files(10)
        .build(current_exe().parent().unwrap())?;
    tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(file_appender)
        .init();
    info!("Starting as a Windows service...");
    run(SERVICE_NAME, service_main)
}

fn service_main(args: Vec<String>, cancel: Receiver<()>) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async { service_main_async(args, cancel).await })
}

async fn service_main_async(args: Vec<String>, cancel: Receiver<()>) -> Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let app = Arc::new(AppContext::new(cli)?);
    app.run_service(cancel).await
}
