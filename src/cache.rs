use std::collections::HashMap;
use std::env::current_exe;
use std::fs::File;
use std::net::IpAddr;
use std::path::PathBuf;

use anyhow::{Error, Result};
use serde::{Deserialize, Serialize};

use crate::AppContext;
use crate::cloudflare::dns::DnsContent::{A, AAAA};
use crate::cloudflare::dns::DnsRecord;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct IdCache {
    zones: HashMap<String, String>,
    records: HashMap<String, RecordIdCache>,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct RecordIdCache {
    pub v4: Option<String>,
    pub v6: Option<String>,
}

impl IdCache {
    fn save_zone(&mut self, zone: String, id: String) {
        self.zones.insert(zone, id);
    }

    fn save_record(&mut self, ns: String, record: RecordIdCache) {
        self.records.insert(ns, record);
    }

    fn update_record(&mut self, ns: &str, record: &DnsRecord) {
        if let Some(cache) = self.records.get_mut(ns) {
            match &record.content {
                A { .. } => cache.v4 = Some(record.id.clone()),
                AAAA { .. } => cache.v6 = Some(record.id.clone()),
                _ => {}
            }
        }
    }

    fn get_zone(&self, zone: &str) -> Option<&String> {
        self.zones.get(zone)
    }

    fn get_record(&self, ns: &str) -> Option<&RecordIdCache> {
        self.records.get(ns)
    }
}

impl RecordIdCache {
    pub fn get_for(&self, addr: &IpAddr) -> Option<&String> {
        match addr {
            IpAddr::V4(_) => self.v4.as_ref(),
            IpAddr::V6(_) => self.v6.as_ref(),
        }
    }
}

impl IdCache {
    pub fn load() -> Result<IdCache> {
        let file = File::open(Self::cache_path()?)?;
        Ok(serde_json::from_reader(file)?)
    }
    pub fn save(&self) -> Result<()> {
        let file = File::create(Self::cache_path()?)?;
        Ok(serde_json::to_writer(file, self)?)
    }

    fn cache_path() -> Result<PathBuf> {
        Ok(current_exe()?.with_file_name("id_cache.json"))
    }
}

impl AppContext {
    pub async fn zone_id(&self, zone: &str) -> Result<String> {
        if self.id_cache.lock().unwrap().get_zone(zone).is_none() {
            self.cache_zones().await?;
        }
        self.id_cache
            .lock()
            .unwrap()
            .get_zone(zone)
            .cloned()
            .ok_or_else(|| Error::msg(format!("Cannot find zone: {zone}")))
    }

    pub async fn record_id(&self, zone_id: &str, ns: &str) -> Result<RecordIdCache> {
        if self.id_cache.lock().unwrap().get_record(ns).is_none() {
            self.cache_record(zone_id, ns).await?;
        }
        self.id_cache
            .lock()
            .unwrap()
            .get_record(ns)
            .cloned()
            .ok_or_else(|| Error::msg(format!("Cannot find record: {ns}")))
    }

    pub fn update_cache(&self, ns: &str, record: &DnsRecord) {
        let mut cache = self.id_cache.lock().unwrap();
        cache.update_record(ns, record);
    }

    async fn cache_zones(&self) -> Result<()> {
        let res = self.client.list_zones().await?.into_iter();
        let mut cache = self.id_cache.lock().unwrap();
        res.for_each(|zone| cache.save_zone(zone.name, zone.id));
        Ok(())
    }
    async fn cache_record(&self, zone_id: &str, ns: &str) -> Result<()> {
        let mut cache = RecordIdCache::default();
        self.client
            .list_records(zone_id, ns)
            .await?
            .into_iter()
            .for_each(|rec| match rec.content {
                A { .. } => cache.v4 = Some(rec.id),
                AAAA { .. } => cache.v6 = Some(rec.id),
                _ => {}
            });
        self.id_cache
            .lock()
            .unwrap()
            .save_record(ns.to_owned(), cache);
        Ok(())
    }
}
