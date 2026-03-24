use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::{Context, Result, bail};

use crate::lookup::LookupSpec;

pub struct ExecLookup {
    cmd: String,
}

impl ExecLookup {
    pub fn new(cmd: String) -> Self {
        Self { cmd }
    }

    async fn run(&self) -> Result<String> {
        let output = shell_command(&self.cmd)
            .output()
            .await
            .with_context(|| format!("failed to execute: {}", self.cmd))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("command exited with {}: {}", output.status, stderr.trim());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }
}

impl LookupSpec for ExecLookup {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        let out = self.run().await?;
        out.parse()
            .with_context(|| format!("failed to parse IPv4 address from command output: {out:?}"))
    }

    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        let out = self.run().await?;
        out.parse()
            .with_context(|| format!("failed to parse IPv6 address from command output: {out:?}"))
    }
}

/// Builds a [`tokio::process::Command`] that runs `cmd` through the system shell.
/// On Windows: `cmd.exe /C <cmd>`.
/// On Unix: `$SHELL -c <cmd>`, falling back to `bash` then `sh`.
#[cfg(windows)]
fn shell_command(cmd: &str) -> tokio::process::Command {
    let mut c = tokio::process::Command::new("cmd.exe");
    c.args(["/C", cmd]);
    c
}

#[cfg(not(windows))]
fn shell_command(cmd: &str) -> tokio::process::Command {
    let mut c = tokio::process::Command::new(find_shell());
    c.args(["-c", cmd]);
    c
}

/// Resolves the shell to use on Unix-like systems.
/// Prefers `$SHELL`, falls back to `bash` if found in `$PATH`, then `sh`.
#[cfg(not(windows))]
fn find_shell() -> String {
    if let Ok(shell) = std::env::var("SHELL")
        && !shell.is_empty()
    {
        return shell;
    }
    if find_in_path("bash") {
        return "bash".to_owned();
    }
    "sh".to_owned()
}

#[cfg(not(windows))]
fn find_in_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}
