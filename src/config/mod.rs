use std::fs::read_to_string;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::time::Duration;

use anyhow::{Context, Result};
use serde::Deserialize;

use crate::current_exe;
use crate::lookup::{ExecLookup, ICanHazIp, InterfaceLookup, Provider};

mod de;

pub use crate::util::matcher::{Ipv4Matcher, Ipv6Matcher};

/// Parsed configuration.
#[derive(Debug, Deserialize)]
pub struct Config {
    pub token: String,
    #[serde(default, deserialize_with = "de::string_or_struct")]
    pub lookup: LookupConfig,
    #[serde(
        default = "de::default_interval",
        deserialize_with = "de::duration_from_secs"
    )]
    pub interval: Duration,
    #[serde(default)]
    pub retry: RetryConfig,
    // FIXME: remove the backward compatibility alias in a future version
    #[serde(default, alias = "zones", deserialize_with = "de::deserialize_records")]
    pub records: Records,
}

impl Config {
    pub fn load(custom_path: Option<&Path>) -> Result<Self> {
        let path = match custom_path {
            Some(p) => p.to_path_buf(),
            None => Self::default_config_path(),
        };
        Self::from_path(path)
    }

    pub fn default_config_path() -> PathBuf {
        current_exe().with_file_name("config.toml")
    }

    fn from_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let file = read_to_string(path.as_ref())
            .with_context(|| format!("unable to read config file: {}", path.as_ref().display()))?;
        Self::from_toml(&file)
            .with_context(|| format!("unable to parse config file: {}", path.as_ref().display()))
    }

    pub(crate) fn from_toml(s: &str) -> Result<Self> {
        toml::from_str(s).context("unable to parse config content")
    }
}

/// Per-protocol lookup provider configuration.
///
/// Accepts the deprecated `lookup = "icanhazip"` string shorthand (sets both
/// protocols to icanhazip) or a `[lookup]` table with explicit `v4`/`v6` keys.
#[derive(Deserialize, Debug, Default, Clone, Eq, PartialEq)]
pub struct LookupConfig {
    #[serde(default, deserialize_with = "de::string_or_struct")]
    pub v4: ProviderConfig,
    #[serde(default, deserialize_with = "de::string_or_struct")]
    pub v6: ProviderConfig,
}

// FIXME: remove the deprecated string form in a future version
impl FromStr for LookupConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "icanhazip" => {
                tracing::warn!(
                    "`lookup = \"icanhazip\"` is deprecated; \
                     use `[lookup]` with explicit `v4`/`v6` keys instead"
                );
                Ok(LookupConfig::default())
            }
            _ => Err(format!(
                "unknown variant `{s}`, expected `icanhazip` or a lookup table"
            )),
        }
    }
}

#[derive(Deserialize, Debug, Default, Clone, Eq, PartialEq, Hash)]
pub struct MatcherConfig {
    #[serde(default)]
    pub v4: Vec<Ipv4Matcher>,
    #[serde(default)]
    pub v6: Vec<Ipv6Matcher>,
}

/// Lookup provider for a single protocol.
///
/// Accepts either a provider name string (e.g. `"icanhazip"`) or a provider
/// config table (e.g. `{ provider = "exec", cmd = "..." }`).
#[derive(Deserialize, Debug, Default, Clone, Eq, PartialEq, Hash)]
#[serde(tag = "provider", rename_all = "lowercase")]
pub enum ProviderConfig {
    #[default]
    ICanHazIp,
    /// Run a shell command and parse its stdout as an IP address.
    Exec { cmd: String },
    /// Read the address assigned to a specific network interface.
    Interface {
        interface: String,
        #[serde(default)]
        matchers: MatcherConfig,
    },
}

impl FromStr for ProviderConfig {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "icanhazip" => Ok(Self::ICanHazIp),
            "exec" => Err(
                r#"provider "exec" requires `cmd`: use `{ provider = "exec", cmd = "..." }`"#
                    .to_owned(),
            ),
            "interface" => Err(
                r#"provider "interface" requires `interface`: use `{ provider = "interface", interface = "eth0" }`"#
                    .to_owned(),
            ),
            _ => Err(format!(
                "unknown provider `{s}`, expected one of: icanhazip, exec, interface"
            )),
        }
    }
}

impl ProviderConfig {
    pub fn to_provider(&self) -> Result<Provider> {
        match self {
            ProviderConfig::ICanHazIp => Ok(Provider::ICanHazIp(ICanHazIp::new()?)),
            ProviderConfig::Exec { cmd } => Ok(Provider::Exec(ExecLookup::new(cmd.clone()))),
            ProviderConfig::Interface {
                interface,
                matchers,
            } => Ok(Provider::Interface(InterfaceLookup::new(
                interface.clone(),
                matchers.clone(),
            )?)),
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(default)]
pub struct RetryConfig {
    #[serde(deserialize_with = "de::duration_from_secs")]
    pub base_delay: Duration,
    pub backoff_multiplier: f64,
    pub max_attempts: u32,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            base_delay: Duration::from_secs(5),
            backoff_multiplier: 2.0,
            max_attempts: 5,
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
    /// Per-record lookup provider override. `None` means use the global provider.
    pub lookup: Option<ProviderConfig>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn minimal() -> Result<()> {
        let cfg = Config::from_toml(r#"token = "test""#)?;
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert_eq!(cfg.interval, Duration::from_mins(5));
        assert_eq!(cfg.retry.base_delay, Duration::from_secs(5));
        assert_eq!(cfg.retry.max_attempts, 5);
        assert!(cfg.records.v4.is_empty());
        assert!(cfg.records.v6.is_empty());
        Ok(())
    }

    #[test]
    fn overridden() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                interval = 60
                [retry]
                base_delay = 10
                backoff_multiplier = 3.0
                max_attempts = 3
            "#,
        )?;
        assert_eq!(cfg.lookup, LookupConfig::default());
        assert_eq!(cfg.interval, Duration::from_mins(1));
        assert_eq!(cfg.retry.base_delay, Duration::from_secs(10));
        assert!((cfg.retry.backoff_multiplier - 3.0).abs() < f64::EPSILON);
        assert_eq!(cfg.retry.max_attempts, 3);
        Ok(())
    }

    #[test]
    fn load_custom_config_path() -> Result<()> {
        let temp_dir = std::env::temp_dir();
        let temp_file = temp_dir.join("cf-ddns-test-config.toml");
        std::fs::write(&temp_file, r#"token = "custom_path_test_token""#)?;

        let cfg = Config::load(Some(&temp_file))?;
        assert_eq!(cfg.token, "custom_path_test_token");

        let _ = std::fs::remove_file(temp_file);
        Ok(())
    }

    #[test]
    fn lookup_icanhazip_string() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                lookup = "icanhazip"
            "#,
        )?;
        assert_eq!(cfg.lookup, LookupConfig::default());
        Ok(())
    }

    #[test]
    fn lookup_unknown_provider() {
        let result = Config::from_toml(
            r#"
                token = "test"
                lookup = "unknown-provider"
            "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn lookup_split_v4_only() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [lookup]
                v4 = "icanhazip"
            "#,
        )?;
        assert_eq!(
            cfg.lookup,
            LookupConfig {
                v4: ProviderConfig::ICanHazIp,
                v6: ProviderConfig::ICanHazIp,
            }
        );
        Ok(())
    }

    #[test]
    fn lookup_split_both_simple() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [lookup]
                v4 = "icanhazip"
                v6 = "icanhazip"
            "#,
        )?;
        assert_eq!(cfg.lookup, LookupConfig::default());
        Ok(())
    }

    #[test]
    fn lookup_split_exec_detailed() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [lookup]
                v4 = { provider = "exec", cmd = "curl -s ipv4.icanhazip.com" }
                v6 = { provider = "exec", cmd = "curl -s ipv6.icanhazip.com" }
            "#,
        )?;
        assert_eq!(
            cfg.lookup,
            LookupConfig {
                v4: ProviderConfig::Exec {
                    cmd: "curl -s ipv4.icanhazip.com".to_owned()
                },
                v6: ProviderConfig::Exec {
                    cmd: "curl -s ipv6.icanhazip.com".to_owned()
                },
            }
        );
        Ok(())
    }

    #[test]
    fn lookup_split_interface_detailed() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [lookup]
                v6 = { provider = "interface", interface = "eth0" }
            "#,
        )?;
        assert_eq!(
            cfg.lookup,
            LookupConfig {
                v4: ProviderConfig::ICanHazIp,
                v6: ProviderConfig::Interface {
                    interface: "eth0".to_owned(),
                    matchers: MatcherConfig::default(),
                },
            }
        );
        Ok(())
    }

    #[test]
    fn lookup_split_mixed() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [lookup]
                v4 = "icanhazip"
                v6 = { provider = "exec", cmd = "dig -6 +short myip.opendns.com @resolver1.opendns.com" }
            "#,
        )?;
        assert_eq!(
            cfg.lookup,
            LookupConfig {
                v4: ProviderConfig::ICanHazIp,
                v6: ProviderConfig::Exec {
                    cmd: "dig -6 +short myip.opendns.com @resolver1.opendns.com".to_owned(),
                },
            }
        );
        Ok(())
    }

    #[test]
    fn lookup_split_icanhazip_detailed() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [lookup]
                v4 = { provider = "icanhazip" }
            "#,
        )?;
        assert_eq!(
            cfg.lookup,
            LookupConfig {
                v4: ProviderConfig::ICanHazIp,
                v6: ProviderConfig::ICanHazIp,
            }
        );
        Ok(())
    }

    #[test]
    fn lookup_exec_bare_string_errors() {
        let result = Config::from_toml(
            r#"
                token = "test"
                lookup = "exec"
            "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn lookup_split_exec_bare_string_errors() {
        let result = Config::from_toml(
            r#"
                token = "test"
                [lookup]
                v4 = "exec"
            "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn lookup_split_interface_bare_string_errors() {
        let result = Config::from_toml(
            r#"
                token = "test"
                [lookup]
                v6 = "interface"
            "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn records() -> Result<()> {
        let cfg = Config::from_toml(
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
        )?;
        assert!(!cfg.records.v4.iter().any(|r| r.name == "www.example.com"));
        assert!(!cfg.records.v6.iter().any(|r| r.name == "www.example.com"));
        assert!(cfg.records.v4.iter().any(|r| r.name == "v4.example.com"));
        assert!(!cfg.records.v6.iter().any(|r| r.name == "v4.example.com"));
        assert!(!cfg.records.v4.iter().any(|r| r.name == "v6.example.com"));
        assert!(cfg.records.v6.iter().any(|r| r.name == "v6.example.com"));
        // Boolean true should produce no per-record override (use global provider).
        assert!(cfg.records.v4.iter().all(|r| r.lookup.is_none()));
        assert!(cfg.records.v6.iter().all(|r| r.lookup.is_none()));
        Ok(())
    }

    #[test]
    fn record_per_record_lookup_icanhazip() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v4 = { lookup = { provider = "icanhazip" } }
            "#,
        )?;
        assert_eq!(cfg.records.v4.len(), 1);
        assert_eq!(cfg.records.v4[0].lookup, Some(ProviderConfig::ICanHazIp),);
        Ok(())
    }

    #[test]
    fn record_per_record_lookup_string_shorthand() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v4 = { lookup = "icanhazip" }
            "#,
        )?;
        assert_eq!(cfg.records.v4.len(), 1);
        assert_eq!(cfg.records.v4[0].lookup, Some(ProviderConfig::ICanHazIp));
        Ok(())
    }

    #[test]
    fn record_per_record_lookup_interface() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v6 = { lookup = { provider = "interface", interface = "eth0" } }
            "#,
        )?;
        assert_eq!(cfg.records.v6.len(), 1);
        assert_eq!(
            cfg.records.v6[0].lookup,
            Some(ProviderConfig::Interface {
                interface: "eth0".to_owned(),
                matchers: MatcherConfig::default(),
            })
        );
        Ok(())
    }

    #[test]
    fn record_per_record_lookup_exec() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v4 = { lookup = { provider = "exec", cmd = "curl -s ipv4.icanhazip.com" } }
            "#,
        )?;
        assert_eq!(cfg.records.v4.len(), 1);
        assert_eq!(
            cfg.records.v4[0].lookup,
            Some(ProviderConfig::Exec {
                cmd: "curl -s ipv4.icanhazip.com".to_owned()
            })
        );
        Ok(())
    }

    #[test]
    fn record_per_record_lookup_mixed_v4_bool_v6_custom() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v4 = true
                v6 = { lookup = { provider = "interface", interface = "eth0" } }
            "#,
        )?;
        assert_eq!(cfg.records.v4.len(), 1);
        assert_eq!(cfg.records.v4[0].lookup, None);
        assert_eq!(cfg.records.v6.len(), 1);
        assert_eq!(
            cfg.records.v6[0].lookup,
            Some(ProviderConfig::Interface {
                interface: "eth0".to_owned(),
                matchers: MatcherConfig::default(),
            })
        );
        Ok(())
    }

    #[test]
    fn record_per_record_v6_filter_suffix() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v6 = { lookup = { provider = "interface", interface = "eth0", matchers = { v6 = ["::20/-64"] } } }
            "#,
        )?;
        assert_eq!(cfg.records.v6.len(), 1);
        assert_eq!(
            cfg.records.v6[0].lookup,
            Some(ProviderConfig::Interface {
                interface: "eth0".to_owned(),
                matchers: MatcherConfig {
                    v4: Vec::new(),
                    v6: vec!["::20/-64".parse()?],
                },
            })
        );
        Ok(())
    }

    #[test]
    fn record_per_record_v6_filter_prefix() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v6 = { lookup = { provider = "interface", interface = "eth0", matchers = { v6 = ["2001:db8::/64"] } } }
            "#,
        )?;
        assert_eq!(cfg.records.v6.len(), 1);
        assert_eq!(
            cfg.records.v6[0].lookup,
            Some(ProviderConfig::Interface {
                interface: "eth0".to_owned(),
                matchers: MatcherConfig {
                    v4: Vec::new(),
                    v6: vec!["2001:db8::/64".parse()?],
                },
            })
        );
        Ok(())
    }

    #[test]
    fn record_per_record_v6_both_filters() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v6 = { lookup = { provider = "interface", interface = "eth0", matchers = { v6 = ["2001:db8::/64", "::20/-64"] } } }
            "#,
        )?;
        assert_eq!(cfg.records.v6.len(), 1);
        assert_eq!(
            cfg.records.v6[0].lookup,
            Some(ProviderConfig::Interface {
                interface: "eth0".to_owned(),
                matchers: MatcherConfig {
                    v4: Vec::new(),
                    v6: vec!["2001:db8::/64".parse()?, "::20/-64".parse()?,],
                },
            })
        );
        Ok(())
    }

    #[test]
    fn record_per_record_v6_invalid_matcher_errors() {
        let result = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v6 = { lookup = { provider = "interface", interface = "eth0", matchers = { v6 = ["invalid-filter"] } } }
            "#,
        );
        assert!(result.is_err());
    }

    #[test]
    fn record_per_record_v4_filter_suffix() -> Result<()> {
        let cfg = Config::from_toml(
            r#"
                token = "test"
                [[records]]
                name = "abc.example.com"
                zone = "example.com"
                v4 = { lookup = { provider = "interface", interface = "eth0", matchers = { v4 = ["0.0.0.20/-24"] } } }
            "#,
        )?;
        assert_eq!(cfg.records.v4.len(), 1);
        assert_eq!(
            cfg.records.v4[0].lookup,
            Some(ProviderConfig::Interface {
                interface: "eth0".to_owned(),
                matchers: MatcherConfig {
                    v4: vec!["0.0.0.20/-24".parse()?],
                    v6: Vec::new(),
                },
            })
        );
        Ok(())
    }
}
