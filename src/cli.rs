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
    use clap::{CommandFactory, Parser};

    #[test]
    fn test_cli_debug_assert() {
        Cli::command().debug_assert();
    }

    #[test]
    fn test_cli_env_bindings() {
        let cmd = Cli::command();

        let config_arg = cmd
            .get_arguments()
            .find(|a| a.get_id() == "config")
            .expect("config argument exists");
        assert_eq!(
            config_arg.get_env().map(std::ffi::OsStr::to_str),
            Some(Some("CF_DDNS_CONFIG"))
        );

        let id_cache_arg = cmd
            .get_arguments()
            .find(|a| a.get_id() == "id_cache")
            .expect("id_cache argument exists");
        assert_eq!(
            id_cache_arg.get_env().map(std::ffi::OsStr::to_str),
            Some(Some("CF_DDNS_ID_CACHE"))
        );
    }

    #[test]
    fn test_top_level_args_before_subcommand() -> Result<()> {
        let cli = Cli::try_parse_from(["cf-ddns", "--id-cache", "/tmp/cache.json", "update"])?;
        assert_eq!(
            cli.id_cache.context("id_cache is missing")?.to_str(),
            Some("/tmp/cache.json")
        );
        Ok(())
    }

    #[test]
    #[cfg(feature = "service")]
    fn test_service_subcommand_args() -> Result<()> {
        let cli = Cli::try_parse_from([
            "cf-ddns",
            "--id-cache",
            "/tmp/cache.json",
            "service",
            "status",
        ])?;
        assert_eq!(
            cli.id_cache.context("id_cache is missing")?.to_str(),
            Some("/tmp/cache.json")
        );
        Ok(())
    }
}
