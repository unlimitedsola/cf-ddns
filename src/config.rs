use std::collections::HashMap;
use std::env::current_exe;
use std::fmt::Debug;
use std::fs::File;
use std::iter::repeat;
use std::path::PathBuf;

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub token: String,
    #[serde(default)]
    pub zones: HashMap<String, Vec<String>>,
    pub lookup: Option<LookupConfig>,
    pub v4: Option<bool>,
    pub v6: Option<bool>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LookupConfig {
    #[default]
    ICANHAZIP,
}

impl Config {
    pub fn load(path: Option<&PathBuf>) -> Result<Config> {
        let default_path = Self::default_path()?;
        let path = path.unwrap_or(&default_path);
        let file = File::open(path)?;
        Ok(serde_yaml::from_reader(file)?)
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
    pub fn v4_enabled(&self) -> bool {
        self.v4.unwrap_or(true)
    }

    pub fn v6_enabled(&self) -> bool {
        self.v6.unwrap_or(true)
    }
    pub fn zone_records(&self) -> Vec<ZoneRecord> {
        self.zones
            .iter()
            .flat_map(|(zone, ns)| repeat(zone).zip(ns.iter()))
            .map(|(zone, ns)| ZoneRecord::new(zone, ns))
            .collect()
    }
}
