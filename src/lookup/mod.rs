use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::Result;
use async_trait::async_trait;

use crate::config::{Config, LookupConfig};
use crate::lookup::icanhazip::ICanHazIp;
use crate::lookup::Provider::ICANHAZIP;

mod icanhazip;

#[async_trait]
pub trait LookupProvider {
    async fn lookup_v4(&self) -> Result<Ipv4Addr>;
    async fn lookup_v6(&self) -> Result<Ipv6Addr>;
}

pub enum Provider {
    ICANHAZIP(ICanHazIp),
}

impl Provider {
    pub fn new(cfg: &Config) -> Self {
        match cfg.lookup.as_ref() {
            None | Some(LookupConfig::ICANHAZIP) => ICANHAZIP(ICanHazIp),
        }
    }
}
#[async_trait]
impl LookupProvider for Provider {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        match self {
            ICANHAZIP(i) => i.lookup_v4(),
        }
        .await
    }
    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        match self {
            ICANHAZIP(i) => i.lookup_v6(),
        }
        .await
    }
}
