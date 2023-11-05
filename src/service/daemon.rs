use std::future::Future;
use std::time::Duration;

use anyhow::Result;
use futures::StreamExt;
use tokio::time::interval;
use tokio_stream::wrappers::IntervalStream;

use crate::AppContext;

impl AppContext {
    pub async fn run_service<Fut>(&self, cancel: Fut) -> Result<()>
        where Fut: Future
    {
        let updater = self.new_updater()?;
        IntervalStream::new(interval(Duration::from_secs(self.config.interval)))
            .take_until(cancel)
            .fold(updater, |mut updater, _| async move {
                updater.update(None).await;
                updater
            })
            .await;
        Ok(())
    }
}
