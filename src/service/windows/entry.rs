use std::sync::Arc;
use std::time::Duration;

use anyhow::Result;
use clap::Parser;
use futures::channel::oneshot::Receiver;
use futures::StreamExt;
use tokio_stream::wrappers::IntervalStream;

use crate::cli::Cli;
use crate::service::windows::sys::service_main;
use crate::service::SERVICE_NAME;
use crate::AppContext;

async fn service_main_async(args: Vec<String>, cancel: Receiver<()>) -> Result<()> {
    let cli = Cli::try_parse_from(args)?;
    let app = Arc::new(AppContext::new(cli)?);
    IntervalStream::new(tokio::time::interval(Duration::from_secs(60)))
        .take_until(cancel)
        .for_each(|_| {
            let app = Arc::clone(&app);
            async move {
                if let Err(e) = app.run().await {
                    log::error!("Error in service: {}", e);
                }
            }
        })
        .await;
    Ok(())
}

fn service_main(args: Vec<String>, cancel: Receiver<()>) -> Result<()> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?
        .block_on(async { service_main_async(args, cancel).await })
}

pub fn run_as_service() -> Result<()> {
    service_main::run(SERVICE_NAME, service_main)
}
