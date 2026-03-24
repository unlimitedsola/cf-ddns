use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::Result;

mod exec;
mod icanhazip;
pub use exec::ExecLookup;
pub use icanhazip::ICanHazIp;

pub trait LookupSpec {
    async fn lookup_v4(&self) -> Result<Ipv4Addr>;
    async fn lookup_v6(&self) -> Result<Ipv6Addr>;
}

pub enum Lookup {
    ICanHazIp(ICanHazIp),
    Exec(ExecLookup),
}

impl LookupSpec for Lookup {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        match self {
            Lookup::ICanHazIp(i) => i.lookup_v4().await,
            Lookup::Exec(e) => e.lookup_v4().await,
        }
    }
    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        match self {
            Lookup::ICanHazIp(i) => i.lookup_v6().await,
            Lookup::Exec(e) => e.lookup_v6().await,
        }
    }
}
