use std::collections::HashMap;
use std::env::current_exe;
use std::fs::File;
use std::net::IpAddr;
use std::path::PathBuf;
use std::rc::Rc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cloudflare::record::DnsContent::{A, AAAA};
use crate::cloudflare::record::DnsRecord;

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

impl IdCache {
    fn cache_path() -> Result<PathBuf> {
        current_exe()
            .map(|p| p.with_file_name("id_cache.json"))
            .context("Failed to determine cache file path.")
    }

    pub fn load() -> Result<IdCache> {
        let file = File::open(Self::cache_path()?)?;
        serde_json::from_reader(file).context("Failed to read cache file.")
    }
    pub fn save(&self) -> Result<()> {
        let file = File::create(Self::cache_path()?)?;
        serde_json::to_writer(file, self).context("Failed to write cache file.")
    }

    pub fn get_zone(&self, zone: &str) -> Option<Rc<str>> {
        self.zones.get(zone).cloned()
    }

    pub fn get_record(&self, ns: &str, addr: &IpAddr) -> Option<Rc<str>> {
        self.records.get(ns).and_then(|r| r.get_for(addr))
    }

    pub fn save_zone(&mut self, zone: String, id: String) {
        self.zones.insert(zone, id.into());
    }


    pub fn update_record(&mut self, ns: &str, record: &DnsRecord) {
        if let Some(cache) = self.records.get_mut(ns) {
            cache.update(record)
        } else {
            let mut cache = RecordIdCache::default();
            cache.update(record);
            self.records.insert(ns.to_owned(), cache);
        }
    }
}

impl RecordIdCache {
    pub fn get_for(&self, addr: &IpAddr) -> Option<Rc<str>> {
        match addr {
            IpAddr::V4(_) => self.v4.as_ref().cloned(),
            IpAddr::V6(_) => self.v6.as_ref().cloned(),
        }
    }

    pub fn update(&mut self, record: &DnsRecord) {
        match record.content {
            A { .. } => self.v4 = Some(record.id.as_str().into()),
            AAAA { .. } => self.v6 = Some(record.id.as_str().into()),
            _ => {}
        }
    }
}
