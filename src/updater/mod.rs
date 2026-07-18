use std::cell::RefCell;
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use std::rc::Rc;
use std::time::Duration;

use anyhow::Result;
use anyhow::{Context, anyhow};
use futures::future::join_all;
use futures::join;
use tracing::{error, info, warn};

use crate::AppContext;
use crate::cloudflare::CloudFlare;
use crate::cloudflare::record::DnsRecord;
use crate::config::{LookupConfig, ProviderConfig, Records, RetryConfig, ZoneRecord};

use crate::lookup::{LookupSpec, Provider};
use crate::updater::id_cache::IdCache;
use crate::updater::lookup_cache::{LookupCache, UpdateResult};

mod id_cache;
mod lookup_cache;

pub struct Updater {
    lookup_config: LookupConfig,
    /// All lookup providers keyed by their config — global (v4/v6 defaults) and
    /// per-record overrides live in the same map so both code paths are identical.
    providers: HashMap<ProviderConfig, Provider>,
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
    retry: RetryConfig,
    interval: Duration,
}

impl AppContext {
    pub fn new_updater(&self) -> Result<Updater> {
        let lookup_config = self.config.lookup.clone();
        let mut providers = HashMap::new();
        // Global providers: fail fast if they can't be initialized.
        for cfg in [&lookup_config.v4, &lookup_config.v6] {
            if !providers.contains_key(cfg) {
                providers.insert(
                    cfg.clone(),
                    cfg.to_provider()
                        .context("unable to initialize lookup provider")?,
                );
            }
        }
        // Per-record overrides: warn and skip on failure so other records still update.
        for rec in self.config.records.v4.iter().chain(&self.config.records.v6) {
            if let Some(cfg) = &rec.lookup
                && !providers.contains_key(cfg)
            {
                match cfg.to_provider() {
                    Ok(provider) => {
                        providers.insert(cfg.clone(), provider);
                    }
                    Err(e) => warn!("Failed to initialize custom lookup provider: {e}"),
                }
            }
        }
        let cf = CloudFlare::new(&self.config.token)?;
        let id_cache = RefCell::new(IdCache::load().unwrap_or_else(|e| {
            warn!("Failed to load cache: {e}");
            IdCache::default()
        }));
        let lookup_cache = RefCell::new(LookupCache::default());
        Ok(Updater {
            lookup_config,
            providers,
            cf,
            id_cache,
            lookup_cache,
            retry: self.config.retry,
            interval: self.config.interval,
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

        // Group records by their effective lookup config (per-record override or global default).
        let mut groups: HashMap<&ProviderConfig, Vec<&ZoneRecord>> = HashMap::new();
        for rec in records {
            let cfg = rec.lookup.as_ref().unwrap_or(&self.lookup_config.v4);
            groups.entry(cfg).or_default().push(rec);
        }

        let futs: Vec<_> = groups
            .iter()
            .filter_map(|(cfg, recs)| {
                if let Some(provider) = self.providers.get(*cfg) {
                    Some(self.update_v4_with_provider(provider, cfg, recs))
                } else {
                    error!("Lookup provider unexpectedly missing for {cfg:?}");
                    None
                }
            })
            .collect();
        join_all(futs).await;
    }

    async fn update_v4_with_provider(
        &self,
        provider: &Provider,
        cache_key: &ProviderConfig,
        records: &[&ZoneRecord],
    ) {
        let mut staged: Option<Ipv4Addr> = None;
        let mut attempt: u32 = 0;

        loop {
            attempt = attempt.saturating_add(1);

            // Perform lookup only when we don't have a staged IP yet.
            if staged.is_none() {
                match provider.lookup_v4().await {
                    Ok(addr) => {
                        match self.lookup_cache.borrow_mut().update_v4(cache_key, addr) {
                            UpdateResult::Initialized => info!("Current IPv4: {addr}"),
                            UpdateResult::Updated(old) => {
                                info!("Current IPv4: {addr} (was {old})");
                            }
                            UpdateResult::Unchanged => {
                                info!("Current IPv4: {addr} (unchanged, skipping update)");
                                return;
                            }
                        }
                        staged = Some(addr);
                    }
                    Err(e) => {
                        error!("Failed to lookup current IPv4 address: {e}");
                    }
                }
            }

            // DNS update with the staged IP (no re-lookup on retry).
            if let Some(addr) = staged {
                let results = join_all(
                    records
                        .iter()
                        .map(|rec| self.update_record_print(rec, addr.into())),
                )
                .await;

                if results.iter().all(|&s| s) {
                    return;
                }
            }

            // Retry only if within budget and the next delay fits within the current interval.
            let delay = backoff_delay(
                attempt,
                self.retry.base_delay,
                self.retry.backoff_multiplier,
            );
            if attempt < self.retry.max_attempts && delay < self.interval {
                warn!("Retrying IPv4 in {:.1}s...", delay.as_secs_f64());
                tokio::time::sleep(delay).await;
            } else {
                break;
            }
        }

        if staged.is_some() {
            error!("IPv4 DNS update failed, giving up until next interval");
        } else {
            error!("IPv4 lookup failed, giving up until next interval");
        }
    }

    async fn update_v6(&self, records: &[ZoneRecord]) {
        if records.is_empty() {
            return;
        }

        let mut groups: HashMap<&ProviderConfig, Vec<&ZoneRecord>> = HashMap::new();
        for rec in records {
            let cfg = rec.lookup.as_ref().unwrap_or(&self.lookup_config.v6);
            groups.entry(cfg).or_default().push(rec);
        }

        let futs: Vec<_> = groups
            .iter()
            .filter_map(|(cfg, recs)| {
                if let Some(provider) = self.providers.get(*cfg) {
                    Some(self.update_v6_with_provider(provider, cfg, recs))
                } else {
                    error!("Lookup provider unexpectedly missing for {cfg:?}");
                    None
                }
            })
            .collect();
        join_all(futs).await;
    }

    async fn update_v6_with_provider(
        &self,
        provider: &Provider,
        cache_key: &ProviderConfig,
        records: &[&ZoneRecord],
    ) {
        let mut staged: Option<Ipv6Addr> = None;
        let mut attempt: u32 = 0;

        loop {
            attempt = attempt.saturating_add(1);

            // Perform lookup only when we don't have a staged IP yet.
            if staged.is_none() {
                match provider.lookup_v6().await {
                    Ok(addr) => {
                        match self.lookup_cache.borrow_mut().update_v6(cache_key, addr) {
                            UpdateResult::Initialized => info!("Current IPv6: {addr}"),
                            UpdateResult::Updated(old) => {
                                info!("Current IPv6: {addr} (was {old})");
                            }
                            UpdateResult::Unchanged => {
                                info!("Current IPv6: {addr} (unchanged, skipping update)");
                                return;
                            }
                        }
                        staged = Some(addr);
                    }
                    Err(e) => {
                        error!("Failed to lookup current IPv6 address: {e}");
                    }
                }
            }

            // DNS update with the staged IP (no re-lookup on retry).
            if let Some(addr) = staged {
                let results = join_all(
                    records
                        .iter()
                        .map(|rec| self.update_record_print(rec, addr.into())),
                )
                .await;

                if results.iter().all(|&s| s) {
                    return;
                }
            }

            // Retry only if within budget and the next delay fits within the current interval.
            let delay = backoff_delay(
                attempt,
                self.retry.base_delay,
                self.retry.backoff_multiplier,
            );
            if attempt < self.retry.max_attempts && delay < self.interval {
                warn!("Retrying IPv6 in {:.1}s...", delay.as_secs_f64());
                tokio::time::sleep(delay).await;
            } else {
                break;
            }
        }

        if staged.is_some() {
            error!("IPv6 DNS update failed, giving up until next interval");
        } else {
            error!("IPv6 lookup failed, giving up until next interval");
        }
    }

    async fn update_record_print(&self, rec: &ZoneRecord, addr: IpAddr) -> bool {
        let rec_type = match addr {
            IpAddr::V4(_) => "A",
            IpAddr::V6(_) => "AAAA",
        };
        info!("Updating {rec_type} record '{}'", rec.name);
        if let Err(e) = self.update_record(rec, addr).await {
            error!("Failed to update {rec_type} record '{}': {e}", rec.name);
            false
        } else {
            info!("Updated {rec_type} record '{}'", rec.name);
            true
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
        let record = if let Some(rec_id) = rec_id {
            self.cf
                .update_record(&zone_id, &rec_id, &rec.name, addr)
                .await
                .context("Failed to update the record")?
        } else {
            let record = self
                .cf
                .create_record(&zone_id, &rec.name, addr)
                .await
                .context("Failed to create the record")?;
            self.update_cache(&rec.name, &record)?;
            record
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

    fn update_cache(&self, name: &str, record: &DnsRecord) -> Result<()> {
        let mut cache = self.id_cache.borrow_mut();
        cache.update_record(name, record);
        cache.save()
    }

    async fn cache_zones(&self) -> Result<()> {
        let zones = self.cf.list_zones().await?;
        let mut cache = self.id_cache.borrow_mut();
        for zone in zones {
            cache.save_zone(zone.name, zone.id);
        }
        cache.save()
    }

    async fn cache_records(&self, zone_id: &str, name: &str) -> Result<()> {
        let records = self.cf.list_records(zone_id, name).await?;
        let mut cache = self.id_cache.borrow_mut();
        for rec in &records {
            cache.update_record(name, rec);
        }
        cache.save()
    }
}

/// Calculates the backoff delay for a given attempt number.
///
/// Formula: `base_delay * multiplier^(attempt - 1)`.
/// Attempt 1 returns `base_delay` unchanged.
/// Returns `Duration::MAX` on overflow (caller interprets this as "give up").
pub(crate) fn backoff_delay(attempt: u32, retry_interval: Duration, multiplier: f64) -> Duration {
    let exp = attempt.saturating_sub(1).min(i32::MAX as u32).cast_signed();
    let delay = retry_interval.as_secs_f64() * multiplier.powi(exp);
    if delay.is_finite() {
        Duration::from_secs_f64(delay)
    } else {
        Duration::MAX
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn backoff_first_attempt_is_base_delay() {
        let interval = Duration::from_secs(5);
        assert_eq!(backoff_delay(1, interval, 2.0), Duration::from_secs(5));
    }

    #[test]
    fn backoff_doubles_each_attempt() {
        let interval = Duration::from_secs(5);
        assert_eq!(backoff_delay(1, interval, 2.0), Duration::from_secs(5));
        assert_eq!(backoff_delay(2, interval, 2.0), Duration::from_secs(10));
        assert_eq!(backoff_delay(3, interval, 2.0), Duration::from_secs(20));
        assert_eq!(backoff_delay(4, interval, 2.0), Duration::from_secs(40));
        assert_eq!(backoff_delay(5, interval, 2.0), Duration::from_secs(80));
    }

    #[test]
    fn backoff_respects_multiplier() {
        let interval = Duration::from_secs(10);
        assert_eq!(backoff_delay(1, interval, 3.0), Duration::from_secs(10));
        assert_eq!(backoff_delay(2, interval, 3.0), Duration::from_secs(30));
        assert_eq!(backoff_delay(3, interval, 3.0), Duration::from_secs(90));
    }

    #[test]
    fn backoff_overflow_returns_max() {
        // Very large attempt count overflows f64 to infinity → Duration::MAX
        assert_eq!(
            backoff_delay(u32::MAX, Duration::from_secs(5), 2.0),
            Duration::MAX
        );
    }

    #[test]
    fn custom_provider_initialization_failure_skipped() -> Result<()> {
        let config = crate::config::Config::from_toml(
            r#"
                token = "test_token"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v4 = { lookup = { provider = "interface", interface = "" } }
            "#,
        )?;

        let ctx = AppContext {
            cli: crate::cli::Cli { command: None },
            config,
        };

        // This should not crash, it should log a warning and return the Updater successfully,
        // but with the failed provider missing from the providers map.
        let updater = ctx.new_updater()?;

        // The global providers should be initialized (default is ICanHazIp).
        assert!(updater.providers.contains_key(&ProviderConfig::ICanHazIp));

        // The failed custom provider should not be present.
        let custom_cfg = ProviderConfig::Interface {
            interface: String::new(),
            matchers: crate::config::MatcherConfig::default(),
        };
        assert!(!updater.providers.contains_key(&custom_cfg));

        Ok(())
    }
}
