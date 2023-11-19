use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::Result;
use async_trait::async_trait;

use crate::config::LookupConfig;
use crate::lookup::icanhazip::ICanHazIp;

mod icanhazip;

#[async_trait]
pub trait LookupProvider {
    async fn lookup_v4(&self) -> Result<Ipv4Addr>;
    async fn lookup_v6(&self) -> Result<Ipv6Addr>;
}

pub enum Provider {
    ICanHazIp(ICanHazIp),
}

impl Provider {
    pub fn new(cfg: &LookupConfig) -> Result<Self> {
        match cfg {
            LookupConfig::ICanHazIp => Ok(Provider::ICanHazIp(ICanHazIp::new()?)),
        }
    }
}

#[async_trait]
impl LookupProvider for Provider {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        match self {
            Provider::ICanHazIp(i) => i.lookup_v4(),
        }.await
    }
    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        match self {
            Provider::ICanHazIp(i) => i.lookup_v6(),
        }.await
    }
}
