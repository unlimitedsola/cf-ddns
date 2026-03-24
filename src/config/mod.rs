use std::fmt::Debug;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;

use crate::config::raw::RawConfig;
use crate::lookup::{ExecLookup, ICanHazIp, Lookup, Provider};

mod raw;

/// Parsed configuration.
#[derive(Debug)]
pub struct Config {
    pub token: String,
    pub lookup: LookupConfig,
    pub interval: Duration,
    pub retry: RetryConfig,
    pub records: Records,
}

#[derive(Debug, Clone, Copy)]
pub struct RetryConfig {
    pub base_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_attempts: u32,
}

impl Config {
    pub fn load() -> Result<Self> {
        RawConfig::load().map(Into::into)
    }
}

/// Per-protocol lookup provider configuration.
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct LookupConfig {
    pub v4: ProviderConfig,
    pub v6: ProviderConfig,
}

/// Lookup provider configuration for a single protocol.
#[derive(Deserialize, Debug, Default, Clone, Eq, PartialEq)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum ProviderConfig {
    #[default]
    ICanHazIp,
    /// Run a shell command and parse its stdout as an IP address.
    Exec { cmd: String },
}

impl LookupConfig {
    pub fn into_lookup(self) -> Result<Lookup> {
        Ok(Lookup {
            v4: self.v4.into_provider()?,
            v6: self.v6.into_provider()?,
        })
    }
}

impl ProviderConfig {
    fn into_provider(self) -> Result<Provider> {
        match self {
            ProviderConfig::ICanHazIp => Ok(Provider::ICanHazIp(ICanHazIp::new()?)),
            ProviderConfig::Exec { cmd } => Ok(Provider::Exec(ExecLookup::new(cmd))),
        }
    }
}

#[derive(Debug, Default)]
pub struct Records {
    pub v4: Vec<ZoneRecord>,
    pub v6: Vec<ZoneRecord>,
}

impl Records {
    pub fn filter_name(&self, name: &str) -> Records {
        fn filter_records(records: &[ZoneRecord], name: &str) -> Vec<ZoneRecord> {
            records
                .iter()
                .filter(|rec| rec.name == name)
                .cloned()
                .collect()
        }
        Records {
            v4: filter_records(&self.v4, name),
            v6: filter_records(&self.v6, name),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ZoneRecord {
    pub zone: String,
    pub name: String,
}
