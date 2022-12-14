use std::net::IpAddr;
use std::sync::Arc;

use anyhow::Result;
use futures::future::{join, join_all};
use log::{error, info};

use crate::AppContext;
use crate::config::ZoneRecord;
use crate::lookup::LookupProvider;

impl AppContext {
    pub async fn update(self: Arc<Self>, ns: Option<String>) -> Result<()> {
        let mut records = self.config.zone_records();
        if let Some(ns) = ns {
            records.retain(|rec| rec.ns == ns)
        }

        join(
            self.clone().update_v4(&records),
            self.clone().update_v6(&records),
        )
            .await;
        Ok(())
    }

    async fn update_v4(self: Arc<Self>, records: &[ZoneRecord<'_>]) {
        if !self.config.v4_enabled() {
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

    async fn update_v6(self: Arc<Self>, records: &[ZoneRecord<'_>]) {
        if !self.config.v6_enabled() {
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
        let rec_id = self.record_id(&zone_id, rec.ns).await;
        let rec_id = match &rec_id {
            Ok(cache) => cache.get_for(&addr),
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
                let resp = self
                    .client
                    .update_record(&zone_id, rec_id, rec.ns, addr)
                    .await;
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
                let resp = self.client.create_record(&zone_id, rec.ns, addr).await;
                match resp {
                    Ok(record) => self.update_cache(rec.ns, &record),
                    Err(e) => {
                        error!("Failed to create {rec_type} record '{}': {e}", rec.ns);
                    }
                }
            }
        }
    }
}
