use std::net::IpAddr;
use std::sync::Arc;

use anyhow::Result;
use anyhow::{anyhow, Context};
use futures::future::join_all;
use futures::join;
use tracing::{error, info, instrument};

use crate::cloudflare::record::DnsRecord;
use crate::cloudflare::CloudFlare;
use crate::config::{Records, ZoneRecord};
use crate::lookup::{Lookup, Provider};
use crate::updater::cache::IdCache;
use crate::AppContext;

mod cache;

pub struct Updater {
    lookup: Provider,
    cf: CloudFlare,
}

impl AppContext {
    pub fn new_updater(&self) -> Result<Updater> {
        let lookup =
            Provider::new(&self.config.lookup).context("unable to initialize lookup provider")?;
        let cf = CloudFlare::new(&self.config.token)?;
        Ok(Updater { lookup, cf })
    }
    pub async fn update(&self, name: Option<&str>) -> Result<()> {
        let updater = self.new_updater()?;
        match name {
            Some(name) => {
                let records = self.config.records.filter_name(name);
                updater.update(&records).await;
            }
            None => updater.update(&self.config.records).await,
        }
        Ok(())
    }
}

impl Updater {
    pub async fn update(&self, records: &Records) {
        join!(self.update_v4(&records.v4), self.update_v6(&records.v6));
    }

    async fn update_v4(&self, records: &[ZoneRecord]) {
        if records.is_empty() {
            return;
        }

        let addr = match self.lookup.lookup_v4().await {
            Ok(res) => res.into(),
            Err(e) => {
                error!("Failed to lookup the IPv4 address of the current network: {e}");
                return;
            }
        };
        info!("Current IPv4: {addr}");
        join_all(records.iter().map(|rec| self.update_record(rec, addr))).await;
    }

    async fn update_v6(&self, records: &[ZoneRecord]) {
        if records.is_empty() {
            return;
        }

        let addr = match self.lookup.lookup_v6().await {
            Ok(res) => res.into(),
            Err(e) => {
                error!("Failed to lookup the IPv6 address of the current network: {e}");
                return;
            }
        };
        info!("Current IPv6: {addr}");
        join_all(records.iter().map(|rec| self.update_record(rec, addr))).await;
    }

    #[instrument(skip(self))]
    async fn update_record(&self, rec: &ZoneRecord, addr: IpAddr) {
        let rec_type = match addr {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        info!("Updating {rec_type} record for '{}'", rec.name);
        let zone_id = match self.zone_id(&rec.zone).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to get the identifier of zone '{}': {e}", rec.zone);
                return;
            }
        };
        let rec_id = self.record_id(&zone_id, &rec.name, &addr).await;
        let rec_id = match &rec_id {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Failed to gather information for {rec_type} record '{}': {e}",
                    rec.name
                );
                return;
            }
        };
        match rec_id {
            Some(rec_id) => {
                info!(
                    "Found existing {rec_type} record [{}] for '{}', updating...",
                    rec_id, rec.name
                );
                let resp = self
                    .cf
                    .update_record(&zone_id, rec_id, &rec.name, addr)
                    .await;
                match resp {
                    Ok(record) => {
                        info!(
                            "Updated {rec_type} record '{}' to {:?}",
                            record.name, record.content
                        )
                    }
                    Err(e) => {
                        error!("Failed to update {rec_type} record for '{}': {e}", rec.name);
                    }
                }
            }
            None => {
                info!("Creating {rec_type} record for '{}'", rec.name);
                let resp = self.cf.create_record(&zone_id, &rec.name, addr).await;
                match resp {
                    Ok(record) => self.update_cache(&rec.name, record),
                    Err(e) => {
                        error!("Failed to create {rec_type} record '{}': {e}", rec.name);
                    }
                }
            }
        }
    }
    async fn zone_id(&self, zone: &str) -> Result<Arc<str>> {
        let res = IdCache::read().get_zone(zone);
        match res {
            Some(id) => return Ok(id),
            None => self.cache_zones().await?,
        }
        IdCache::read()
            .get_zone(zone)
            .ok_or_else(|| anyhow!("Cannot find zone: {zone}"))
    }

    async fn record_id(
        &self,
        zone_id: &str,
        name: &str,
        addr: &IpAddr,
    ) -> Result<Option<Arc<str>>> {
        if IdCache::read().get_record(name, addr).is_none() {
            self.cache_records(zone_id, name).await?;
        }
        Ok(IdCache::read().get_record(name, addr))
    }

    fn update_cache(&self, name: &str, record: DnsRecord) {
        let mut cache = IdCache::write();
        cache.update_record(name, record);
        // FIXME: error handling
        cache.save().unwrap();
    }

    async fn cache_zones(&self) -> Result<()> {
        let zones = self.cf.list_zones().await?;
        let mut cache = IdCache::write();
        zones
            .into_iter()
            .for_each(|zone| cache.save_zone(zone.name, zone.id));
        cache.save()
    }
    async fn cache_records(&self, zone_id: &str, name: &str) -> Result<()> {
        let records = self.cf.list_records(zone_id, name).await?;
        let mut cache = IdCache::write();
        records
            .into_iter()
            .for_each(|rec| cache.update_record(name, rec));
        cache.save()
    }
}
