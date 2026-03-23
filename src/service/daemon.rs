use std::future::Future;

use anyhow::Result;
use futures::StreamExt;
use tokio::time::{MissedTickBehavior, interval};
use tokio_stream::wrappers::IntervalStream;

use crate::AppContext;

impl AppContext {
    pub async fn run_service<Fut>(&self, cancel: Fut) -> Result<()>
    where
        Fut: Future,
    {
        let updater = self.new_updater()?;
        let mut interval = interval(self.config.interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);
        IntervalStream::new(interval)
            .take_until(cancel)
            .fold(updater, |updater, _| async move {
                updater.update(&self.config.records).await;
                updater
            })
            .await;
        Ok(())
    }
}
