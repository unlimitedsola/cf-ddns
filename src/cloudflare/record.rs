//! Partial implementation, only contains fields that we'll use

use std::net::IpAddr::{V4, V6};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

use reqwest::{IntoUrl, Method};
use serde::{Deserialize, Serialize};

use crate::cloudflare::client::{ApiRequest, BASE_URL};
use crate::cloudflare::record::DnsContent::{A, AAAA};

#[derive(Deserialize, Clone, Debug)]
pub struct DnsRecord {
    /// DNS record identifier tag
    pub id: String,
    /// DNS record name
    pub name: String,
    /// Type of the DNS record that also holds the record value
    #[serde(flatten)]
    pub content: DnsContent,
}

/// Type of the DNS record, along with the associated value.
/// We only care about A and AAAA records for now.
#[derive(Deserialize, Serialize, Clone, Debug)]
#[serde(tag = "type")]
#[allow(clippy::upper_case_acronyms)]
pub enum DnsContent {
    A { content: Ipv4Addr },
    AAAA { content: Ipv6Addr },
}

impl From<IpAddr> for DnsContent {
    fn from(ip: IpAddr) -> Self {
        match ip {
            V4(content) => A { content },
            V6(content) => AAAA { content },
        }
    }
}

/// [List DNS Records](https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-list-dns-records)
#[derive(Debug)]
pub struct ListDnsRecords<'a> {
    pub zone_identifier: &'a str,
    pub params: ListDnsRecordsParams<'a>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug, Default)]
pub struct ListDnsRecordsParams<'a> {
    #[serde(rename = "type")]
    pub record_type: Option<&'a str>,
    pub name: Option<&'a str>,
}

impl<'a> ApiRequest for ListDnsRecords<'a> {
    type Request = ();
    type Query = ListDnsRecordsParams<'a>;
    type Response = Vec<DnsRecord>;
    fn url(&self) -> impl IntoUrl {
        format!("{}/zones/{}/dns_records", BASE_URL, self.zone_identifier)
    }
    fn query(&self) -> Option<&Self::Query> {
        Some(&self.params)
    }
}

/// [Create DNS Record](https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-create-dns-record)
#[derive(Debug)]
pub struct CreateDnsRecord<'a> {
    pub zone_identifier: &'a str,
    pub params: CreateDnsRecordParams<'a>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug)]
pub struct CreateDnsRecordParams<'a> {
    /// Time to live for DNS record. Value of 1 is 'automatic'
    pub ttl: Option<u32>,
    /// Whether the record is receiving the performance and security benefits of Cloudflare
    pub proxied: Option<bool>,
    /// DNS record name
    pub name: &'a str,
    /// Type of the DNS record that also holds the record value
    #[serde(flatten)]
    pub content: DnsContent,
}

impl<'a> ApiRequest for CreateDnsRecord<'a> {
    type Request = CreateDnsRecordParams<'a>;
    type Query = ();
    type Response = DnsRecord;

    fn method(&self) -> Method {
        Method::POST
    }
    fn url(&self) -> impl IntoUrl {
        format!("{}/zones/{}/dns_records", BASE_URL, self.zone_identifier)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.params)
    }
}

/// [Update DNS Record](https://developers.cloudflare.com/api/operations/dns-records-for-a-zone-patch-dns-record)
#[derive(Debug)]
pub struct UpdateDnsRecord<'a> {
    pub zone_identifier: &'a str,
    pub identifier: &'a str,
    pub params: UpdateDnsRecordParams<'a>,
}

#[serde_with::skip_serializing_none]
#[derive(Serialize, Clone, Debug)]
pub struct UpdateDnsRecordParams<'a> {
    /// DNS record name
    pub name: &'a str,
    /// Type of the DNS record that also holds the record value
    #[serde(flatten)]
    pub content: DnsContent,
}

impl<'a> ApiRequest for UpdateDnsRecord<'a> {
    type Request = UpdateDnsRecordParams<'a>;
    type Query = ();
    type Response = DnsRecord;

    // We use PATCH here to allow user to preserve some modifications
    // in case they are not satisfied with default TTL, proxied, etc.
    fn method(&self) -> Method {
        Method::PATCH
    }
    fn url(&self) -> impl IntoUrl {
        format!(
            "{}/zones/{}/dns_records/{}",
            BASE_URL, self.zone_identifier, self.identifier
        )
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.params)
    }
}
