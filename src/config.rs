use std::collections::HashMap;
use std::fmt::Debug;
use std::fs::File;
use std::iter::repeat;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::current_exe;

#[derive(Deserialize, Debug)]
pub struct Config {
    pub token: String,
    #[serde(default)]
    pub zones: HashMap<String, Vec<String>>,
    #[serde(default)]
    pub lookup: LookupConfig,
    #[serde(default = "bool_true")]
    pub v4: bool,
    #[serde(default = "bool_true")]
    pub v6: bool,
    #[serde(default = "default_interval")]
    pub interval: u64,
}

/// Helper function to return true for serde default
const fn bool_true() -> bool {
    true
}

const fn default_interval() -> u64 {
    300
}

#[derive(Deserialize, Default, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LookupConfig {
    #[default]
    ICanHazIp,
}

impl Config {
    fn config_path() -> PathBuf {
        current_exe().with_file_name("config.yaml")
    }

    pub fn load() -> Result<Config> {
        Self::load_from(Self::config_path())
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> Result<Config> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("unable to open config file: {:?}", path.as_ref()))?;
        serde_yaml::from_reader(file)
            .with_context(|| format!("unable to parse config file: {:?}", path.as_ref()))
    }
}

#[derive(Debug, Clone)]
pub struct ZoneRecord<'a> {
    pub zone: &'a str,
    pub ns: &'a str,
}

impl<'a> ZoneRecord<'a> {
    pub fn new(zone: &'a str, ns: &'a str) -> Self {
        ZoneRecord { zone, ns }
    }
}

impl Config {
    pub fn zone_records(&self) -> Vec<ZoneRecord> {
        self.zones
            .iter()
            .flat_map(|(zone, ns)| repeat(zone).zip(ns.iter()))
            .map(|(zone, ns)| ZoneRecord::new(zone, ns))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal() {
        let cfg: Config = serde_yaml::from_str(
            // language=yaml
            "token: test",
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert!(cfg.v4);
        assert!(cfg.v6);
        assert_eq!(cfg.interval, 300)
    }

    #[test]
    fn overridden() {
        let cfg: Config = serde_yaml::from_str(
            // language=yaml
            r#"
                token: test
                v4: false
                v6: false
                interval: 60
            "#,
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert!(!cfg.v4);
        assert!(!cfg.v6);
        assert_eq!(cfg.interval, 60)
    }
}
