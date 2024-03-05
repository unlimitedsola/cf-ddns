use std::fmt::Debug;
use std::time::Duration;

use anyhow::Result;
use serde::Deserialize;

use crate::config::raw::RawConfig;

mod raw;

#[derive(Debug)]
pub struct Config {
    pub token: String,
    pub lookup: LookupConfig,
    pub interval: Duration,
    pub records: Records,
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
