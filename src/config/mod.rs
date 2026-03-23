use std::fmt::Debug;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;

use crate::config::raw::RawConfig;

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

#[derive(Deserialize, Default, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LookupConfig {
    #[default]
    ICanHazIp,
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
