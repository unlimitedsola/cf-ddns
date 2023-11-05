use std::collections::HashMap;
use std::env::current_exe;
use std::fmt::Debug;
use std::fs::File;
use std::iter::repeat;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::Deserialize;

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
}

/// Helper function to return true for serde default
const fn bool_true() -> bool {
    true
}

#[derive(Deserialize, Default, Debug, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum LookupConfig {
    #[default]
    ICanHazIp,
}

impl Config {
    pub fn load(path: Option<&PathBuf>) -> Result<Config> {
        let default_path = Self::default_path()?;
        let path = path.unwrap_or(&default_path);
        let file = File::open(path)?;
        serde_yaml::from_reader(file)
            .with_context(|| format!("unable to load config from {:?}", path))
    }

    fn default_path() -> Result<PathBuf> {
        Ok(current_exe()?.with_file_name("config.yaml"))
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
            "token: test"
        ).unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert!(cfg.v4);
        assert!(cfg.v6);
    }

    #[test]
    fn overridden() {
        let cfg: Config = serde_yaml::from_str(
            // language=yaml
            r#"
                token: test
                v4: false
                v6: false
            "#
        ).unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert!(!cfg.v4);
        assert!(!cfg.v6);
    }
}
