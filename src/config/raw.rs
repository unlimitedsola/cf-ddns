use std::collections::HashMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::Context;
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
    zones: HashMap<String, RawZoneConfig>,
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
struct RawZoneConfig(HashMap<String, RawRecordConfig>);

#[derive(Deserialize, Debug)]
struct RawRecordConfig {
    #[serde(default)]
    v4: bool,
    #[serde(default)]
    v6: bool,
}

impl RawConfig {
    fn config_path() -> PathBuf {
        current_exe().with_file_name("config.yaml")
    }

    pub fn load() -> anyhow::Result<Self> {
        Self::load_from(Self::config_path())
    }

    pub fn load_from<P: AsRef<Path>>(path: P) -> anyhow::Result<Self> {
        let file = File::open(path.as_ref())
            .with_context(|| format!("unable to open config file: {:?}", path.as_ref()))?;
        serde_yaml::from_reader(file)
            .with_context(|| format!("unable to parse config file: {:?}", path.as_ref()))
    }
}

impl From<RawConfig> for Config {
    fn from(value: RawConfig) -> Self {
        let mut records = crate::config::Records::default();
        for (zone, zone_records) in value.zones {
            for (name, record) in zone_records.0 {
                let zone_record = crate::config::ZoneRecord {
                    zone: zone.clone(),
                    name,
                };
                if record.v4 {
                    records.v4.push(zone_record.clone());
                }
                if record.v6 {
                    records.v6.push(zone_record);
                }
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
        let cfg: RawConfig = serde_yaml::from_str(
            // language=yaml
            "token: test",
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert_eq!(cfg.interval, Interval::default());
        assert!(cfg.zones.is_empty());
    }

    #[test]
    fn overridden() {
        let cfg: RawConfig = serde_yaml::from_str(
            // language=yaml
            r#"
                token: test
                interval: 60
            "#,
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert_eq!(cfg.interval, Interval(60));
        assert!(cfg.zones.is_empty());
    }

    #[test]
    fn lookup() {
        let cfg: RawConfig = serde_yaml::from_str(
            // language=yaml
            r#"
                token: test
                lookup: icanhazip
            "#,
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::ICanHazIp);
    }

    #[test]
    fn zones() {
        let cfg: RawConfig = serde_yaml::from_str(
            // language=yaml
            r#"
                token: test
                zones:
                  example.com:
                    www.example.com:
                    v4.example.com:
                      v4: true
                    v6.example.com:
                      v6: true
            "#,
        )
        .unwrap();
        let www = &cfg.zones["example.com"].0["www.example.com"];
        assert!(!www.v4);
        assert!(!www.v6);
        let v4 = &cfg.zones["example.com"].0["v4.example.com"];
        assert!(v4.v4);
        assert!(!v4.v6);
        let v6 = &cfg.zones["example.com"].0["v6.example.com"];
        assert!(!v6.v4);
        assert!(v6.v6);
    }
}
