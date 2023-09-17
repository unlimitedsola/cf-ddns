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
        IntervalStream::new(interval(Duration::from_secs(60)))
            .take_until(cancel)
            .for_each(|_| self.update(None))
            .await;
        Ok(())
    }
}
