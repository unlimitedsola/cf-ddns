use std::net::{Ipv4Addr, Ipv6Addr};

use anyhow::{Context, Result, bail};

use crate::lookup::LookupSpec;

pub struct ExecLookup {
    v4: Option<String>,
    v6: Option<String>,
}

impl ExecLookup {
    pub fn new(v4: Option<String>, v6: Option<String>) -> Self {
        Self { v4, v6 }
    }

    async fn run(cmd: &str) -> Result<String> {
        let output = tokio::process::Command::new(find_shell())
            .arg("-c")
            .arg(cmd)
            .output()
            .await
            .with_context(|| format!("failed to execute: {cmd}"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            bail!("command exited with {}: {}", output.status, stderr.trim());
        }

        Ok(String::from_utf8_lossy(&output.stdout).trim().to_owned())
    }
}

impl LookupSpec for ExecLookup {
    async fn lookup_v4(&self) -> Result<Ipv4Addr> {
        let cmd = self.v4.as_deref().context("no v4 command configured")?;
        let out = ExecLookup::run(cmd).await?;
        out.parse()
            .with_context(|| format!("failed to parse IPv4 address from command output: {out:?}"))
    }

    async fn lookup_v6(&self) -> Result<Ipv6Addr> {
        let cmd = self.v6.as_deref().context("no v6 command configured")?;
        let out = ExecLookup::run(cmd).await?;
        out.parse()
            .with_context(|| format!("failed to parse IPv6 address from command output: {out:?}"))
    }
}

/// Resolves the shell to use for executing commands.
/// Prefers `$SHELL`, falls back to `bash` if found in `$PATH`, then `sh`.
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

fn find_in_path(name: &str) -> bool {
    std::env::var_os("PATH")
        .map(|paths| std::env::split_paths(&paths).any(|dir| dir.join(name).is_file()))
        .unwrap_or(false)
}
