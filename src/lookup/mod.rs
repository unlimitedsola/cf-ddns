use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::Result;

mod exec;
mod icanhazip;
pub(crate) mod interface;
pub use exec::ExecLookup;
pub use icanhazip::ICanHazIp;
pub use interface::InterfaceLookup;

pub trait LookupSpec {
    async fn lookup_v4(&self) -> Result<Ipv4Addr>;
    async fn lookup_v6(&self) -> Result<Ipv6Addr>;
}

/// Holds per-protocol lookup providers.
pub struct Lookup {
    pub v4: Provider,
    pub v6: Provider,
}

/// Lookup provider for a single protocol.
pub enum Provider {
    ICanHazIp(ICanHazIp),
    Exec(ExecLookup),
    Interface(InterfaceLookup),
}

impl LookupSpec for Lookup {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        match &self.v4 {
            Provider::ICanHazIp(i) => i.lookup_v4().await,
            Provider::Exec(e) => e.lookup_v4().await,
            Provider::Interface(i) => i.lookup_v4().await,
        }
    }
    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        match &self.v6 {
            Provider::ICanHazIp(i) => i.lookup_v6().await,
            Provider::Exec(e) => e.lookup_v6().await,
            Provider::Interface(i) => i.lookup_v6().await,
        }
    }
}
