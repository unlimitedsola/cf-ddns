use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config::{Config, LookupConfig};
use crate::current_exe;

/// Raw configuration parsed from files.
#[derive(Deserialize, Debug)]
pub struct RawConfig {
    token: String,
    #[serde(default)]
    lookup: LookupConfig,
    #[serde(default)]
    interval: Interval,
    #[serde(default)]
    zones: Vec<RawRecordConfig>,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct Interval(u64); // in seconds
impl Default for Interval {
    fn default() -> Self {
        Interval(300) // 5 minutes
    }
}
impl Interval {
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.0)
    }
}

#[derive(Deserialize, Debug)]
struct RawRecordConfig {
    name: String,
    zone: String,
    #[serde(default)]
    v4: bool,
    #[serde(default)]
    v6: bool,
}

impl RawConfig {
    fn config_path() -> PathBuf {
        current_exe().with_file_name("config.toml")
    }

    pub fn load() -> Result<Self> {
        Self::from_path(Self::config_path())
    }

    pub fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = read_to_string(path.as_ref())
            .with_context(|| format!("unable to read config file: {:?}", path.as_ref()))?;

        Self::from_toml(&file)
            .with_context(|| format!("unable to parse config file: {:?}", path.as_ref()))
    }

    pub fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).context("unable to parse config content")
    }
}

impl From<RawConfig> for Config {
    fn from(value: RawConfig) -> Self {
        let mut records = crate::config::Records::default();
        for rec in value.zones {
            let zone_record = crate::config::ZoneRecord {
                zone: rec.zone,
                name: rec.name,
            };
            if rec.v4 {
                records.v4.push(zone_record.clone());
            }
            if rec.v6 {
                records.v6.push(zone_record);
            }
        }
        Config {
            token: value.token,
            lookup: value.lookup,
            interval: value.interval.duration(),
            records,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal() {
        let cfg = RawConfig::from_toml(
            // language=toml
            r#"token = "test""#,
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert_eq!(cfg.interval, Interval::default());
        assert!(cfg.zones.is_empty());
    }

    #[test]
    fn overridden() {
        let cfg = RawConfig::from_toml(
            // language=toml
            r#"
                token = "test"
                interval = 60
            "#,
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert_eq!(cfg.interval, Interval(60));
        assert!(cfg.zones.is_empty());
    }

    #[test]
    fn lookup() {
        let cfg = RawConfig::from_toml(
            // language=toml
            r#"
                token = "test"
                lookup = "icanhazip"
            "#,
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::ICanHazIp);
    }

    #[test]
    fn zones() {
        let cfg = RawConfig::from_toml(
            // language=toml
            r#"
                token = "test"
                [[zones]]
                name = "www.example.com"
                zone = "example.com"
                [[zones]]
                name = "v4.example.com"
                zone = "example.com"
                v4 = true
                [[zones]]
                name = "v6.example.com"
                zone = "example.com"
                v6 = true
            "#,
        )
        .unwrap();
        let www = &cfg.zones[0];
        assert!(!www.v4);
        assert!(!www.v6);
        let v4 = &cfg.zones[1];
        assert!(v4.v4);
        assert!(!v4.v6);
        let v6 = &cfg.zones[2];
        assert!(!v6.v4);
        assert!(v6.v6);
    }
}
