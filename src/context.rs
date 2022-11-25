use std::sync::{Arc, Mutex};

use crate::cache::IdCache;
use crate::cloudflare::Client;
use crate::config::Config;
use crate::lookup::Provider;

pub struct AppContext {
    pub lookup: Provider,
    pub client: Client,
    pub id_cache: Arc<Mutex<IdCache>>,
    pub config: Config,
}

impl AppContext {
    pub fn new(config: Config) -> anyhow::Result<Self> {
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
