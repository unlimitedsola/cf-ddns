use std::collections::HashMap;
use std::fs::File;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, OnceLock, RwLock, RwLockReadGuard, RwLockWriteGuard};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::cloudflare::record::DnsContent::{A, AAAA};
use crate::cloudflare::record::DnsRecord;
use crate::current_exe;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IdCache {
    /// Zone name to Zone Id
    zones: HashMap<String, Arc<str>>,
    /// Record name to Record Ids (v4, v6)
    records: HashMap<String, RecordIdCache>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RecordIdCache {
    pub v4: Option<Arc<str>>,
    pub v6: Option<Arc<str>>,
}

impl RecordIdCache {
    pub fn get_for(&self, addr: &IpAddr) -> Option<Arc<str>> {
        match addr {
            IpAddr::V4(_) => self.v4.as_ref().cloned(),
            IpAddr::V6(_) => self.v6.as_ref().cloned(),
        }
    }

    pub fn update(&mut self, record: DnsRecord) {
        match record.content {
            A { .. } => self.v4 = Some(record.id.into()),
            AAAA { .. } => self.v6 = Some(record.id.into()),
            _ => {}
        }
    }
}

impl IdCache {
    fn cache_path() -> &'static Path {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| current_exe().with_file_name("id_cache.json"))
    }

    fn load() -> Result<IdCache> {
        let file = File::open(Self::cache_path())?;
        serde_json::from_reader(file).context("failed to read cache file")
    }

    // to avoid file contention, we require exclusive access to the cache to save it
    pub fn save(&mut self) -> Result<()> {
        let file = File::create(Self::cache_path())?;
        serde_json::to_writer(file, self).context("failed to write cache file")
    }

    fn get() -> &'static RwLock<IdCache> {
        static CACHE: OnceLock<RwLock<IdCache>> = OnceLock::new();
        CACHE.get_or_init(|| {
            RwLock::new(IdCache::load().unwrap_or_else(|e| {
                warn!("Failed to load cache: {e}");
                IdCache::default()
            }))
        })
    }

    pub fn read() -> RwLockReadGuard<'static, IdCache> {
        Self::get().read().unwrap()
    }

    pub fn write() -> RwLockWriteGuard<'static, IdCache> {
        Self::get().write().unwrap()
    }
}

impl IdCache {
    pub fn get_zone(&self, zone: &str) -> Option<Arc<str>> {
        self.zones.get(zone).cloned()
    }

    pub fn get_record(&self, name: &str, addr: &IpAddr) -> Option<Arc<str>> {
        self.records.get(name).and_then(|r| r.get_for(addr))
    }

    pub fn save_zone(&mut self, zone: String, id: String) {
        self.zones.insert(zone, id.into());
    }

    pub fn update_record(&mut self, name: &str, record: DnsRecord) {
        if let Some(cache) = self.records.get_mut(name) {
            cache.update(record)
        } else {
            let mut cache = RecordIdCache::default();
            cache.update(record);
            self.records.insert(name.to_owned(), cache);
        }
    }
}
