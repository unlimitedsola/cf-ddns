use std::cell::RefCell;
use std::net::IpAddr;
use std::rc::Rc;

use anyhow::Result;
use anyhow::{Context, anyhow};
use futures::future::join_all;
use futures::join;
use tracing::{error, info, warn};

use crate::AppContext;
use crate::cloudflare::CloudFlare;
use crate::cloudflare::record::DnsRecord;
use crate::config::{Records, ZoneRecord};
use crate::lookup::{Lookup, Provider};
use crate::updater::id_cache::IdCache;
use crate::updater::lookup_cache::{LookupCache, UpdateResult};

mod id_cache;
mod lookup_cache;

pub struct Updater {
    lookup: Provider,
    cf: CloudFlare,
    // SAFETY: RefCell is used to allow mutable access to the cache across async calls.
    // We ensure that any borrow of the cache won't be held across an await point,
    // so there won't be concurrent borrows and should not cause any panicking.
    // Updater is also not `Sync` because of this, so it can't be shared across threads.
    // We could have used `RwLock` to make it `Sync`, but we are not expecting this to be a
    // bottleneck, so it is better to be more resource-efficient and save the overhead of
    // memory barriers and atomic operations instead.
    id_cache: RefCell<IdCache>,
    lookup_cache: RefCell<LookupCache>,
}

impl AppContext {
    pub fn new_updater(&self) -> Result<Updater> {
        let lookup =
            Provider::new(&self.config.lookup).context("unable to initialize lookup service")?;
        let cf = CloudFlare::new(&self.config.token)?;
        let id_cache = RefCell::new(IdCache::load().unwrap_or_else(|e| {
            warn!("Failed to load cache: {e}");
            IdCache::default()
        }));
        let lookup_cache = RefCell::new(LookupCache::default());
        Ok(Updater {
            lookup,
            cf,
            id_cache,
            lookup_cache,
        })
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
            Ok(res) => res,
            Err(e) => {
                error!("Failed to lookup the current IPv4 address: {e}");
                return;
            }
        };
        match self.lookup_cache.borrow_mut().update_v4(&addr) {
            UpdateResult::Initialized => info!("Current IPv4: {addr}"),
            UpdateResult::Updated(old) => {
                info!("Current IPv4: {addr} (was {old})");
            }
            UpdateResult::Unchanged => {
                info!("Current IPv4: {addr} (unchanged, skipping update)");
                return;
            }
        }
        let addr = addr.into();
        join_all(
            records
                .iter()
                .map(|rec| self.update_record_print(rec, addr)),
        )
        .await;
    }

    async fn update_v6(&self, records: &[ZoneRecord]) {
        if records.is_empty() {
            return;
        }

        let addr = match self.lookup.lookup_v6().await {
            Ok(res) => res,
            Err(e) => {
                error!("Failed to lookup the current IPv6 address: {e}");
                return;
            }
        };
        match self.lookup_cache.borrow_mut().update_v6(&addr) {
            UpdateResult::Initialized => info!("Current IPv6: {addr}"),
            UpdateResult::Updated(old) => {
                info!("Current IPv6: {addr} (was {old})");
            }
            UpdateResult::Unchanged => {
                info!("Current IPv6: {addr} (unchanged, skipping update)");
                return;
            }
        }
        let addr = addr.into();
        join_all(
            records
                .iter()
                .map(|rec| self.update_record_print(rec, addr)),
        )
        .await;
    }

    async fn update_record_print(&self, rec: &ZoneRecord, addr: IpAddr) {
        let rec_type = match addr {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        info!("Updating {rec_type} record '{}'", rec.name);
        match self.update_record(rec, addr).await {
            Err(e) => {
                error!("Failed to update {rec_type} record '{}': {e}", rec.name);
            }
            Ok(_) => {
                info!("Updated {rec_type} record '{}'", rec.name);
            }
        }
    }

    async fn update_record(&self, rec: &ZoneRecord, addr: IpAddr) -> Result<DnsRecord> {
        let zone_id = self
            .zone_id(&rec.zone)
            .await
            .context("Failed to get the zone identifier")?;
        let rec_id = self
            .record_id(&zone_id, &rec.name, &addr)
            .await
            .context("Failed to get the record identifier")?;
        let record = match rec_id {
            Some(rec_id) => self
                .cf
                .update_record(&zone_id, &rec_id, &rec.name, addr)
                .await
                .context("Failed to update the record")?,
            None => {
                let record = self
                    .cf
                    .create_record(&zone_id, &rec.name, addr)
                    .await
                    .context("Failed to create the record")?;
                self.update_cache(&rec.name, record.clone())?;
                record
            }
        };
        Ok(record)
    }
}

impl Updater {
    async fn zone_id(&self, zone: &str) -> Result<Rc<str>> {
        let res = self.id_cache.borrow().get_zone(zone);
        match res {
            Some(id) => return Ok(id),
            None => self.cache_zones().await?,
        }
        self.id_cache
            .borrow()
            .get_zone(zone)
            .ok_or_else(|| anyhow!("Cannot find zone: {zone}"))
    }

    async fn record_id(&self, zone_id: &str, name: &str, addr: &IpAddr) -> Result<Option<Rc<str>>> {
        if self.id_cache.borrow().get_record(name, addr).is_none() {
            self.cache_records(zone_id, name).await?;
        }
        Ok(self.id_cache.borrow().get_record(name, addr))
    }

    fn update_cache(&self, name: &str, record: DnsRecord) -> Result<()> {
        let mut cache = self.id_cache.borrow_mut();
        cache.update_record(name, record);
        cache.save()
    }

    async fn cache_zones(&self) -> Result<()> {
        let zones = self.cf.list_zones().await?;
        let mut cache = self.id_cache.borrow_mut();
        zones
            .into_iter()
            .for_each(|zone| cache.save_zone(zone.name, zone.id));
        cache.save()
    }

    async fn cache_records(&self, zone_id: &str, name: &str) -> Result<()> {
        let records = self.cf.list_records(zone_id, name).await?;
        let mut cache = self.id_cache.borrow_mut();
        records
            .into_iter()
            .for_each(|rec| cache.update_record(name, rec));
        cache.save()
    }
}
