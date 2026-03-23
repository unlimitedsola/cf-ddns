use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::config::{Config, LookupConfig, RetryConfig};
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
    retry: RawRetryConfig,
    // FIXME: remove the backward compatibility alias in a future version
    #[serde(default, alias = "zones")]
    records: Vec<RawRecordConfig>,
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

#[derive(Deserialize, Debug, Default)]
struct RawRetryConfig {
    #[serde(default)]
    base_delay: BaseDelay,
    #[serde(default)]
    backoff_multiplier: BackoffMultiplier,
    #[serde(default)]
    max_attempts: MaxAttempts,
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct BaseDelay(u64); // in seconds
impl Default for BaseDelay {
    fn default() -> Self {
        BaseDelay(5)
    }
}
impl BaseDelay {
    pub fn duration(&self) -> Duration {
        Duration::from_secs(self.0)
    }
}

#[derive(Deserialize, Debug)]
struct BackoffMultiplier(f64);
impl Default for BackoffMultiplier {
    fn default() -> Self {
        BackoffMultiplier(2.0)
    }
}

#[derive(Deserialize, Debug, Eq, PartialEq)]
struct MaxAttempts(u32);
impl Default for MaxAttempts {
    fn default() -> Self {
        MaxAttempts(5)
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
        for rec in value.records {
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
            retry: RetryConfig {
                base_delay: value.retry.base_delay.duration(),
                backoff_multiplier: value.retry.backoff_multiplier.0,
                max_attempts: value.retry.max_attempts.0,
            },
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
        assert_eq!(cfg.retry.base_delay, BaseDelay::default());
        assert_eq!(cfg.retry.max_attempts, MaxAttempts::default());
        assert!(cfg.records.is_empty());
    }

    #[test]
    fn overridden() {
        let cfg = RawConfig::from_toml(
            // language=toml
            r#"
                token = "test"
                interval = 60
                [retry]
                base_delay = 10
                backoff_multiplier = 3.0
                max_attempts = 3
            "#,
        )
        .unwrap();
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert_eq!(cfg.interval, Interval(60));
        assert_eq!(cfg.retry.base_delay, BaseDelay(10));
        assert_eq!(cfg.retry.max_attempts, MaxAttempts(3));

        let config: Config = cfg.into();
        assert_eq!(config.interval, Duration::from_secs(60));
        assert_eq!(config.retry.base_delay, Duration::from_secs(10));
        assert!((config.retry.backoff_multiplier - 3.0).abs() < f64::EPSILON);
        assert_eq!(config.retry.max_attempts, 3);
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
    fn records() {
        let cfg = RawConfig::from_toml(
            // language=toml
            r#"
                token = "test"
                [[records]]
                name = "www.example.com"
                zone = "example.com"
                [[records]]
                name = "v4.example.com"
                zone = "example.com"
                v4 = true
                [[records]]
                name = "v6.example.com"
                zone = "example.com"
                v6 = true
            "#,
        )
        .unwrap();
        let www = &cfg.records[0];
        assert!(!www.v4);
        assert!(!www.v6);
        let v4 = &cfg.records[1];
        assert!(v4.v4);
        assert!(!v4.v6);
        let v6 = &cfg.records[2];
        assert!(!v6.v4);
        assert!(v6.v6);
    }
}
