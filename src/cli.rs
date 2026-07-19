use anyhow::Result;
use clap::{Parser, Subcommand};

use crate::AppContext;
use crate::debug::DebugCommand;
#[cfg(feature = "service")]
use crate::service::ServiceCommand;

#[derive(Debug, Parser)]
#[command(name = "cf-ddns")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Path to the configuration file
    #[arg(short, long, value_name = "FILE", env = "CF_DDNS_CONFIG")]
    pub config: Option<std::path::PathBuf>,

    /// Path to the zone/record ID cache file
    #[arg(long, value_name = "FILE", env = "CF_DDNS_ID_CACHE")]
    pub id_cache: Option<std::path::PathBuf>,

    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, Subcommand, Clone)]
pub enum Command {
    Update {
        name: Option<String>,
    },
    #[cfg(feature = "service")]
    #[command(subcommand)]
    Service(ServiceCommand),
    /// Diagnostic commands for inspecting runtime state.
    #[command(subcommand)]
    Debug(DebugCommand),
}

impl AppContext {
    pub async fn run(&self) -> Result<()> {
        match self.cli.command.clone() {
            None => self.update(None).await?,
            Some(cmd) => match cmd {
                Command::Update { name } => self.update(name.as_deref()).await?,
                #[cfg(feature = "service")]
                Command::Service(command) => self.run_service_command(&command).await?,
                Command::Debug(_) => {
                    unreachable!("debug commands are handled before config is loaded")
                }
            },
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;
    use clap::Parser;

    #[test]
    fn test_cli_env_variables() -> Result<()> {
        unsafe {
            std::env::set_var("CF_DDNS_CONFIG", "/tmp/config_env.toml");
            std::env::set_var("CF_DDNS_ID_CACHE", "/tmp/cache_env.json");
        }

        // Parse from empty args to ensure it falls back to env variables
        let cli = Cli::try_parse_from(["cf-ddns"])?;
        assert_eq!(cli.config.context("config is missing")?.to_str(), Some("/tmp/config_env.toml"));
        assert_eq!(cli.id_cache.context("id_cache is missing")?.to_str(), Some("/tmp/cache_env.json"));

        unsafe {
            std::env::remove_var("CF_DDNS_CONFIG");
            std::env::remove_var("CF_DDNS_ID_CACHE");
        }
        Ok(())
    }

    #[test]
    fn test_cli_args_override_env() -> Result<()> {
        unsafe {
            std::env::set_var("CF_DDNS_CONFIG", "/tmp/config_env.toml");
        }

        // Command line arg should override env variable
        let cli = Cli::try_parse_from(["cf-ddns", "-c", "/tmp/override.toml"])?;
        assert_eq!(cli.config.context("config is missing")?.to_str(), Some("/tmp/override.toml"));

        unsafe {
            std::env::remove_var("CF_DDNS_CONFIG");
        }
        Ok(())
    }
}
