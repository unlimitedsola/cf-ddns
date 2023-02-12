use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "cf-ddns")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Update {
        ns: Option<String>,
    },
    Service {
        #[command(subcommand)]
        command: ServiceCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ServiceCommands {
    Install,
    Remove,
    Start,
    Stop,
    Run,
    Debug,
}
