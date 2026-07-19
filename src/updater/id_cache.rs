use std::collections::HashMap;
use std::fs::File;
use std::net::IpAddr;
use std::path::{Path, PathBuf};
use std::rc::Rc;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::cloudflare::record::DnsContent::{A, AAAA};
use crate::cloudflare::record::DnsRecord;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IdCache {
    #[serde(skip)]
    pub(crate) path: PathBuf,
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
            IpAddr::V4(_) => self.v4.clone(),
            IpAddr::V6(_) => self.v6.clone(),
        }
    }

    pub fn update(&mut self, record: &DnsRecord) {
        match &record.content {
            A { .. } => self.v4 = Some(record.id.clone().into()),
            AAAA { .. } => self.v6 = Some(record.id.clone().into()),
        }
    }
}

impl IdCache {
    pub fn load<P: AsRef<Path>>(path: P) -> Result<IdCache> {
        let file = File::open(path.as_ref())?;
        let mut cache: IdCache = serde_json::from_reader(file).context("failed to load cache")?;
        cache.path = path.as_ref().to_path_buf();
        Ok(cache)
    }

    // to avoid file contention, we require exclusive access to the cache to save it
    pub fn save(&self) -> Result<()> {
        let file = File::create(&self.path)?;
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

    pub fn update_record(&mut self, name: &str, record: &DnsRecord) {
        self.records
            // it is most likely that the record is not present
            // in the cache when this function is called
            .entry(name.to_owned())
            .or_default()
            .update(record);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_cache_path_load_save() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("cf-ddns-test-cache.json");

        let mut cache = IdCache {
            path: temp_file.clone(),
            ..Default::default()
        };
        cache.save_zone("example.com".to_owned(), "zone_id_123".to_owned());
        cache.save()?;

        assert!(temp_file.exists());

        let loaded = IdCache::load(&temp_file)?;
        assert_eq!(loaded.path, temp_file);
        assert_eq!(
            loaded.get_zone("example.com").as_deref(),
            Some("zone_id_123")
        );

        let _ = std::fs::remove_file(temp_file);
        Ok(())
    }
}
