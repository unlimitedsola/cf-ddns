//! Extremely simplified Cloudflare API client for own use.

use std::net::IpAddr;

use anyhow::Result;
use reqwest::ClientBuilder;
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};

use crate::cloudflare::record::{
    CreateDnsRecord, CreateDnsRecordParams, DnsRecord, ListDnsRecords, ListDnsRecordsParams,
    UpdateDnsRecord, UpdateDnsRecordParams,
};
use crate::cloudflare::zone::{ListZones, Zone};

mod client;
pub mod record;
pub mod zone;

pub struct CloudFlare {
    http: reqwest::Client,
}

// Constructors
impl CloudFlare {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = HeaderMap::with_capacity(2);
        headers.insert(
            AUTHORIZATION,
            HeaderValue::try_from(format!("Bearer {token}"))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let http = ClientBuilder::new().default_headers(headers).build()?;
        Ok(CloudFlare { http })
    }
}

// Api wrappers for our actual use cases
impl CloudFlare {
    pub async fn list_zones(&self) -> Result<Vec<Zone>> {
        self.call(&ListZones).await
    }
    pub async fn list_records(&self, zone_id: &str, name: &str) -> Result<Vec<DnsRecord>> {
        let req = ListDnsRecords {
            zone_identifier: zone_id,
            params: ListDnsRecordsParams {
                // we only care about A and AAAA records
                record_type: Some("A,AAAA"),
                name: Some(name),
            },
        };
        self.call(&req).await
    }

    pub async fn create_record(
        &self,
        zone_id: &str,
        name: &str,
        addr: IpAddr,
    ) -> Result<DnsRecord> {
        let req = CreateDnsRecord {
            zone_identifier: zone_id,
            params: CreateDnsRecordParams {
                name,
                content: addr.into(),
                ttl: Some(60),
                proxied: Some(false),
            },
        };
        self.call(&req).await
    }

    pub async fn update_record(
        &self,
        zone_id: &str,
        rec_id: &str,
        name: &str,
        addr: IpAddr,
    ) -> Result<DnsRecord> {
        let req = UpdateDnsRecord {
            zone_identifier: zone_id,
            identifier: rec_id,
            params: UpdateDnsRecordParams {
                name,
                content: addr.into(),
            },
        };
        self.call(&req).await
    }
}
