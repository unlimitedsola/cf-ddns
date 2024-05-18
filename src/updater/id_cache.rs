use std::collections::HashMap;
use std::fs::File;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cloudflare::record::DnsContent::{A, AAAA};
use crate::cloudflare::record::DnsRecord;
use crate::current_exe;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IdCache {
    /// Zone name to Zone Id
    zones: HashMap<String, Rc<str>>,
    /// Record name to Record Ids (v4, v6)
    records: HashMap<String, RecordIdCache>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RecordIdCache {
    pub v4: Option<Rc<str>>,
    pub v6: Option<Rc<str>>,
}

impl RecordIdCache {
    pub fn get_for(&self, addr: &IpAddr) -> Option<Rc<str>> {
        match addr {
            IpAddr::V4(_) => self.v4.as_ref().cloned(),
            IpAddr::V6(_) => self.v6.as_ref().cloned(),
        }
    }

    pub fn update(&mut self, record: DnsRecord) {
        match record.content {
            A { .. } => self.v4 = Some(record.id.into()),
            AAAA { .. } => self.v6 = Some(record.id.into()),
        }
    }
}

impl IdCache {
    fn cache_path() -> &'static Path {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| current_exe().with_file_name("id_cache.json"))
    }

    pub fn load() -> Result<IdCache> {
        let file = File::open(Self::cache_path())?;
        serde_json::from_reader(file).context("failed to load cache")
    }

    // to avoid file contention, we require exclusive access to the cache to save it
    pub fn save(&mut self) -> Result<()> {
        let file = File::create(Self::cache_path())?;
        serde_json::to_writer(file, self).context("failed to write cache")
    }
}

impl IdCache {
    pub fn get_zone(&self, zone: &str) -> Option<Rc<str>> {
        self.zones.get(zone).cloned()
    }

    pub fn get_record(&self, name: &str, addr: &IpAddr) -> Option<Rc<str>> {
        self.records.get(name).and_then(|r| r.get_for(addr))
    }

    pub fn save_zone(&mut self, zone: String, id: String) {
        self.zones.insert(zone, id.into());
    }

    pub fn update_record(&mut self, name: &str, record: DnsRecord) {
        self.records
            // it is most likely that the record is not present
            // in the cache when this function is called
            .entry(name.to_owned())
            .or_default()
            .update(record);
    }
}
