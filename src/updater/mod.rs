use std::net::IpAddr;
use std::rc::Rc;

use anyhow::Result;
use anyhow::{anyhow, Context};
use futures::future::join_all;
use futures::join;
use parking_lot::RwLock;
use tracing::{error, info, instrument};

use crate::cloudflare::record::DnsRecord;
use crate::cloudflare::CloudFlare;
use crate::config::ZoneRecord;
use crate::lookup::{LookupProvider, Provider};
use crate::updater::cache::IdCache;
use crate::AppContext;

mod cache;

pub struct Updater<'a> {
    app: &'a AppContext,
    lookup: Provider,
    cf: CloudFlare,
    cache: RwLock<IdCache>,
}

// FIXME: investigate a explicit way to save cache
impl<'a> Drop for Updater<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.cache.read().save() {
            error!("Failed to save id cache: {e}");
        }
    }
}

impl AppContext {
    pub fn new_updater(&self) -> Result<Updater<'_>> {
        let lookup =
            Provider::new(&self.config.lookup).context("unable to initialize lookup provider")?;
        let cf = CloudFlare::new(&self.config.token)?;
        let cache = IdCache::load().unwrap_or_default();
        Ok(Updater {
            app: self,
            lookup,
            cf,
            cache: RwLock::new(cache),
        })
    }
    pub async fn update(&self, ns: Option<&str>) -> Result<()> {
        let mut updater = self.new_updater()?;
        updater.update(ns).await;
        Ok(())
    }
}

impl<'a> Updater<'a> {
    // SAFETY: require &mut self because we have interior mutability of the cache
    #[instrument(skip(self))]
    pub async fn update(&mut self, ns: Option<&str>) {
        let mut records = self.app.config.zone_records();
        if let Some(ns) = ns {
            records.retain(|rec| rec.ns == ns)
        }

        info!("Updating records: {records:?}");

        join!(self.update_v4(&records), self.update_v6(&records));
    }

    #[instrument(skip_all)]
    async fn update_v4(&self, records: &[ZoneRecord<'_>]) {
        if !&self.app.config.v4 {
            info!("Skipped IPv4 since it is disabled by config.");
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

    #[instrument(skip_all)]
    async fn update_v6(&self, records: &[ZoneRecord<'_>]) {
        if !&self.app.config.v6 {
            info!("Skipped IPv6 since it is disabled by config.");
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
    async fn update_record(&self, rec: &ZoneRecord<'_>, addr: IpAddr) {
        let rec_type = match addr {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        info!("Updating {rec_type} record for '{}'", rec.ns);
        let zone_id = match self.zone_id(rec.zone).await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to get the identifier of zone '{}': {e}", rec.zone);
                return;
            }
        };
        let rec_id = self.record_id(&zone_id, rec.ns, &addr).await;
        let rec_id = match &rec_id {
            Ok(r) => r,
            Err(e) => {
                error!(
                    "Failed to gather information for {rec_type} record '{}': {e}",
                    rec.ns
                );
                return;
            }
        };
        match rec_id {
            Some(rec_id) => {
                info!(
                    "Found existing {rec_type} record [{}] for '{}', updating...",
                    rec_id, rec.ns
                );
                let resp = self.cf.update_record(&zone_id, rec_id, rec.ns, addr).await;
                match resp {
                    Ok(record) => {
                        info!(
                            "Updated {rec_type} record '{}' to {:?}",
                            record.name, record.content
                        )
                    }
                    Err(e) => {
                        error!("Failed to update {rec_type} record for '{}': {e}", rec.ns);
                    }
                }
            }
            None => {
                info!("Creating {rec_type} record for '{}'", rec.ns);
                let resp = self.cf.create_record(&zone_id, rec.ns, addr).await;
                match resp {
                    Ok(record) => self.update_cache(rec.ns, record),
                    Err(e) => {
                        error!("Failed to create {rec_type} record '{}': {e}", rec.ns);
                    }
                }
            }
        }
    }
    async fn zone_id(&self, zone: &str) -> Result<Rc<str>> {
        if self.cache.read().get_zone(zone).is_none() {
            self.cache_zones().await?;
        }
        self.cache
            .read()
            .get_zone(zone)
            .ok_or_else(|| anyhow!("Cannot find zone: {zone}"))
    }

    async fn record_id(&self, zone_id: &str, ns: &str, addr: &IpAddr) -> Result<Option<Rc<str>>> {
        if self.cache.read().get_record(ns, addr).is_none() {
            self.cache_records(zone_id, ns).await?;
        }
        Ok(self.cache.read().get_record(ns, addr))
    }

    fn update_cache(&self, ns: &str, record: DnsRecord) {
        self.cache.write().update_record(ns, record);
    }

    async fn cache_zones(&self) -> Result<()> {
        self.cf
            .list_zones()
            .await?
            .into_iter()
            .for_each(|zone| self.cache.write().save_zone(zone.name, zone.id));
        Ok(())
    }
    async fn cache_records(&self, zone_id: &str, ns: &str) -> Result<()> {
        self.cf
            .list_records(zone_id, ns)
            .await?
            .into_iter()
            .for_each(|rec| self.update_cache(ns, rec));
        Ok(())
    }
}
