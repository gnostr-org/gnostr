use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use clap::Parser;
use tokio::process::Command;
use tracing::{debug, info, warn};

#[derive(clap::ValueEnum, Clone, Debug, Copy)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
}

impl std::fmt::Display for LogLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LogLevel::Trace => write!(f, "trace"),
            LogLevel::Debug => write!(f, "debug"),
            LogLevel::Info => write!(f, "info"),
            LogLevel::Warn => write!(f, "warn"),
            LogLevel::Error => write!(f, "error"),
        }
    }
}

#[derive(Parser, Debug, Clone)]
pub struct RelaySubCommand {
    /// Path to configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Path to data directory.
    #[arg(short, long, default_value = ".gnostr/relay")]
    pub data: Option<PathBuf>,

    /// Watch for configuration file changes.
    #[arg(short, long)]
    pub watch: bool,

    /// Set logging level.
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub logging: LogLevel,

    /// Run the relay in background (daemon mode).
    #[arg(long, help = "Run the relay server in background as a daemon")]
    pub detach: bool,
}

pub async fn relay(args: RelaySubCommand) -> Result<()> {
    info!("Start relay server with args: {:?}", args);

    if args.detach {
        #[cfg(unix)]
        {
            return run_detached(args).await;
        }

        #[cfg(not(unix))]
        {
            return Err(anyhow::anyhow!(
                "Detach functionality is currently only supported on Unix-like systems"
            ));
        }
    }

    if args.watch {
        debug!("relay --watch is accepted but handled by the local relay crate");
    }
    if args.data.is_some() {
        debug!("relay --data is accepted but handled by the local relay crate defaults");
    }

    run_local_relay(resolve_setting_path(&args)).await
}

async fn run_detached(args: RelaySubCommand) -> Result<()> {
    info!("Starting relay in daemon mode...");

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let mut cmd = Command::new(current_exe);
    cmd.arg("relay");

    if let Some(config) = args.config {
        cmd.arg("--config").arg(config);
    }
    if let Some(data) = args.data {
        cmd.arg("--data").arg(data);
    }
    if args.watch {
        cmd.arg("--watch");
    }
    cmd.arg("--logging").arg(args.logging.to_string());
    cmd.env("RUST_LOG", "error");

    cmd.spawn()
        .context("Failed to spawn detached relay process")?;

    println!("Relay started in background");
    println!("Process ID: {}", std::process::id());
    Ok(())
}

async fn run_local_relay(setting_path: Option<String>) -> Result<()> {
    let setting_path = setting_path.as_deref();
    let local_set = tokio::task::LocalSet::new();

    local_set
        .run_until(async move {
            let app_data = gnostr_relay::App::create(
                setting_path,
                true,
                Some("NOSTR".to_owned()),
                None,
            )
            .map_err(anyhow::Error::from)?;
            app_data.web_server()?.await.map_err(anyhow::Error::from)
        })
        .await?;

    info!("Relay server shutdown");
    Ok(())
}

fn resolve_setting_path(args: &RelaySubCommand) -> Option<String> {
    if let Some(config) = &args.config {
        return Some(config.to_string_lossy().into_owned());
    }

    let config_file_path = Path::new("config/gnostr.toml");
    if config_file_path.exists() {
        return Some(config_file_path.to_string_lossy().into_owned());
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    struct CwdGuard(PathBuf);

    impl Drop for CwdGuard {
        fn drop(&mut self) {
            let _ = std::env::set_current_dir(&self.0);
        }
    }

    #[test]
    fn resolve_setting_path_uses_explicit_config() {
        let args = RelaySubCommand {
            config: Some(PathBuf::from("/tmp/custom.toml")),
            data: None,
            watch: false,
            logging: LogLevel::Info,
            detach: false,
        };

        assert_eq!(
            resolve_setting_path(&args),
            Some(String::from("/tmp/custom.toml"))
        );
    }

    #[test]
    fn resolve_setting_path_uses_repo_config_when_present() {
        let tempdir = TempDir::new().expect("tempdir");
        let original = std::env::current_dir().expect("cwd");
        let _guard = CwdGuard(original);
        std::env::set_current_dir(tempdir.path()).expect("set cwd");
        std::fs::create_dir_all("config").expect("config dir");
        std::fs::write("config/gnostr.toml", "[server]\nport = 0\nhost = \"127.0.0.1\"\n")
            .expect("write config");

        let args = RelaySubCommand {
            config: None,
            data: None,
            watch: false,
            logging: LogLevel::Info,
            detach: false,
        };

        assert_eq!(
            resolve_setting_path(&args),
            Some(String::from("config/gnostr.toml"))
        );
    }
}
