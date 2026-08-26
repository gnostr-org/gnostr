use std::{
    fs,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

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
#[command(
    about = "Run the gnostr relay server",
    long_about = "Run the local gnostr relay. By default it loads .gnostr/relay.toml when present, stores data under .gnostr/relay, and writes logs to stderr and the gnostr log file. Use --detach on Unix-like systems to keep it running in the background.",
    after_help = "Examples:\n  gnostr relay\n  gnostr relay --watch\n  gnostr relay --detach --logging info\n  gnostr relay --config .gnostr/relay.toml --data .gnostr/relay"
)]
pub struct RelaySubCommand {
    /// Path to the relay config file.
    ///
    /// Defaults to .gnostr/relay.toml when present in the current directory.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Path to the relay data directory.
    ///
    /// The relay stores its LMDB database under <data>/events.
    #[arg(short, long, default_value = ".gnostr/relay")]
    pub data: Option<PathBuf>,

    /// Watch the config file for changes and reload on update.
    #[arg(short, long)]
    pub watch: bool,

    /// Set the logging level written to stderr and gnostr.log.
    #[arg(long, value_enum, default_value_t = LogLevel::Info)]
    pub logging: LogLevel,

    /// Run the relay in background (daemon mode).
    #[arg(long, help = "Run the relay server in background as a daemon")]
    pub detach: bool,
}

/// Launch the relay wrapper.
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
        debug!("relay --data is forwarded to the relay app");
    }

    let current_dir = std::env::current_dir().context("Failed to get current directory")?;
    run_local_relay(
        resolve_setting_path(&args, &current_dir),
        args.data.clone(),
    )
    .await
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

async fn run_local_relay(
    setting_path: Option<PathBuf>,
    data_path: Option<PathBuf>,
) -> Result<()> {
    tokio::task::spawn_blocking(move || {
        let system = actix_web::rt::System::new();
        system.block_on(async move {
            let setting_path = setting_path.as_deref();
            let data_path = data_path.as_deref();
            let app_data = create_relay_app(setting_path, data_path).await?;
            gnostr_relay::run_app_with_endpoint(app_data)
                .await
                .map_err(anyhow::Error::from)
        })
    })
    .await
    .context("failed to join relay runtime thread")??;

    info!("Relay server shutdown");
    Ok(())
}

async fn create_relay_app(
    setting_path: Option<&Path>,
    data_path: Option<&Path>,
) -> Result<gnostr_relay::App> {
    match gnostr_relay::App::create(setting_path, true, Some("NOSTR".to_owned()), data_path) {
        Ok(app) => Ok(app),
        Err(err) if is_lmdb_version_mismatch(&err) => {
            warn!("relay database version mismatch detected; backing up stale LMDB and recreating it");
            backup_relay_events_dir(setting_path, data_path)?;
            gnostr_relay::App::create(setting_path, true, Some("NOSTR".to_owned()), data_path)
                .map_err(anyhow::Error::from)
        }
        Err(err) => Err(anyhow::Error::from(err)),
    }
}

fn resolve_setting_path(args: &RelaySubCommand, current_dir: &Path) -> Option<PathBuf> {
    if let Some(config) = &args.config {
        return Some(config.clone());
    }

    let config_file_path = current_dir.join(".gnostr/relay.toml");
    if config_file_path.exists() {
        return Some(config_file_path);
    }

    None
}

fn is_lmdb_version_mismatch(err: &impl std::fmt::Display) -> bool {
    let message = err.to_string();
    message.contains("MDB_VERSION_MISMATCH")
        || message.contains("Database environment version mismatch")
}

fn resolve_relay_data_path(
    setting_path: Option<&Path>,
    data_path: Option<&Path>,
) -> Result<PathBuf> {
    if let Some(data_path) = data_path {
        return Ok(data_path.to_path_buf());
    }

    if let Some(setting_path) = setting_path {
        return gnostr_relay::Setting::read(setting_path, Some("NOSTR".to_owned()))
            .map(|setting| setting.data.path)
            .map_err(anyhow::Error::from);
    }

    Ok(gnostr_relay::Setting::default().data.path)
}

fn backup_relay_events_dir(setting_path: Option<&Path>, data_path: Option<&Path>) -> Result<()> {
    let relay_data_path = resolve_relay_data_path(setting_path, data_path)?;
    let events_dir = relay_data_path.join("events");
    if !events_dir.exists() {
        return Ok(());
    }

    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .context("system time is before UNIX_EPOCH")?
        .as_secs();
    let backup_dir = relay_data_path.join(format!("events.mdb-version-mismatch-{timestamp}"));
    fs::rename(&events_dir, &backup_dir).with_context(|| {
        format!(
            "failed to back up stale relay database from {:?} to {:?}",
            events_dir, backup_dir
        )
    })?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

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
            resolve_setting_path(&args, Path::new(".")),
            Some(PathBuf::from("/tmp/custom.toml"))
        );
    }

    #[test]
    fn resolve_setting_path_uses_repo_config_when_present() {
        let tempdir = TempDir::new().expect("tempdir");
        std::fs::create_dir_all(tempdir.path().join(".gnostr")).expect("config dir");
        std::fs::write(
            tempdir.path().join(".gnostr/relay.toml"),
            "[server]\nport = 0\nhost = \"127.0.0.1\"\n",
        )
        .expect("write config");

        let args = RelaySubCommand {
            config: None,
            data: None,
            watch: false,
            logging: LogLevel::Info,
            detach: false,
        };

        assert_eq!(
            resolve_setting_path(&args, tempdir.path()),
            Some(tempdir.path().join(".gnostr/relay.toml"))
        );
    }

    #[test]
    fn resolve_relay_data_path_prefers_explicit_data_path() {
        let tempdir = TempDir::new().expect("tempdir");
        let explicit = tempdir.path().join("custom-data");

        assert_eq!(
            resolve_relay_data_path(None, Some(explicit.as_path())).expect("data path"),
            explicit
        );
    }

    #[test]
    fn backup_relay_events_dir_renames_existing_database() {
        let tempdir = TempDir::new().expect("tempdir");
        let relay_data = tempdir.path().join("relay");
        let events_dir = relay_data.join("events");
        std::fs::create_dir_all(&events_dir).expect("events dir");
        std::fs::write(events_dir.join("data.mdb"), b"old-db").expect("db file");

        backup_relay_events_dir(None, Some(relay_data.as_path())).expect("backup");

        assert!(!events_dir.exists());
        let entries = std::fs::read_dir(&relay_data)
            .expect("read relay dir")
            .map(|entry| entry.expect("entry").file_name().to_string_lossy().into_owned())
            .collect::<Vec<_>>();
        assert!(
            entries
                .iter()
                .any(|entry| entry.starts_with("events.mdb-version-mismatch-"))
        );
    }
}
