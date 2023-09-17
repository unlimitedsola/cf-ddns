//! Extremely simplified Cloudflare API client for own use.

use std::net::IpAddr;

use anyhow::{Error, Result};
use reqwest::{ClientBuilder, Url};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE, HeaderMap, HeaderValue};
use reqwest::Method;
use serde::{Deserialize, Serialize};
use serde::de::DeserializeOwned;

use crate::cloudflare::record::{
    CreateDnsRecord, CreateDnsRecordParams, DnsRecord, ListDnsRecords, ListDnsRecordsParams,
    UpdateDnsRecord, UpdateDnsRecordParams,
};
use crate::cloudflare::zone::{ListZones, Zone};

pub mod record;
pub mod zone;

pub struct Client {
    http: reqwest::Client,
}

// Constructors
impl Client {
    pub fn new(token: &str) -> Result<Self> {
        let mut headers = HeaderMap::new();
        headers.insert(
            AUTHORIZATION,
            HeaderValue::from_str(&format!("Bearer {}", token))?,
        );
        headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
        let http = ClientBuilder::new().default_headers(headers).build()?;
        Ok(Client { http })
    }
}

trait ApiRequest {
    type Request: Serialize;
    type Query: Serialize;
    type Response: DeserializeOwned;
    fn method(&self) -> Method {
        Method::GET
    }
    fn path(&self) -> String;
    fn query(&self) -> Option<&Self::Query> {
        None
    }
    fn body(&self) -> Option<&Self::Request> {
        None
    }
}

const BASE_URL: &str = "https://api.cloudflare.com/client/v4/"; // trailing slash is required!

// Base exchange implementation
impl Client {
    async fn call<Api>(&self, api: &Api) -> Result<Api::Response>
        where
            Api: ApiRequest,
    {
        let mut request = self
            .http
            .request(api.method(), Url::try_from(BASE_URL)?.join(&api.path())?)
            .query(&api.query());

        if let Some(body) = api.body() {
            request = request.json(body);
        }
        extract_response(request.send().await?).await
    }
}

#[derive(Deserialize, Debug)]
struct Response<T> {
    pub result: T,
}

async fn extract_response<T>(resp: reqwest::Response) -> Result<T>
    where
        T: DeserializeOwned,
{
    let status = resp.status();
    if status.is_success() {
        Ok(resp.json::<Response<T>>().await?.result)
    } else {
        Err(Error::msg(format!(
            "Error from Cloudflare API, status: {}, response: {}",
            status,
            resp.text().await?
        )))
    }
}

// Api wrappers for our actual use cases
impl Client {
    pub async fn list_zones(&self) -> Result<Vec<Zone>> {
        self.call(&ListZones).await
    }
    pub async fn list_records(&self, zone_id: &str, ns: &str) -> Result<Vec<DnsRecord>> {
        let req = ListDnsRecords {
            zone_identifier: zone_id,
            params: ListDnsRecordsParams {
                name: Some(ns.to_owned()),
                ..Default::default()
            },
        };
        self.call(&req).await
    }

    pub async fn create_record(&self, zone_id: &str, ns: &str, addr: IpAddr) -> Result<DnsRecord> {
        let req = CreateDnsRecord {
            zone_identifier: zone_id,
            params: CreateDnsRecordParams {
                name: ns,
                content: addr.into(),
                ttl: Some(60),
                proxied: Some(false),
                priority: None,
            },
        };
        self.call(&req).await
    }

    pub async fn update_record(
        &self,
        zone_id: &str,
        rec_id: &str,
        ns: &str,
        addr: IpAddr,
    ) -> Result<DnsRecord> {
        let req = UpdateDnsRecord {
            zone_identifier: zone_id,
            identifier: rec_id,
            params: UpdateDnsRecordParams {
                name: ns,
                content: addr.into(),
                ttl: None,
                proxied: None,
            },
        };
        self.call(&req).await
    }
}
