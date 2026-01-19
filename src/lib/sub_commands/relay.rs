use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use tokio::process::Command;
use tracing::{debug, info};

#[derive(Debug, Deserialize)]
struct DataConfig {
    path: Option<PathBuf>,
}

#[derive(Debug, Deserialize)]
struct GnostrConfig {
    data: Option<DataConfig>,
}

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

#[derive(clap::Parser, Debug, Clone)]
pub struct RelaySubCommand {
    /// Path to configuration file.
    #[arg(short, long)]
    pub config: Option<PathBuf>,

    /// Path to data directory.
    #[arg(short, long)]
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

//TODO web actix runtime
pub async fn relay(args: RelaySubCommand) -> Result<()> {
    info!("Start relay server with args: {:?}", args);

    // Handle detach mode - run as daemon
    if args.detach {
        #[cfg(unix)]
        {
            return run_as_daemon(args).await;
        }

        #[cfg(not(unix))]
        {
            return Err(anyhow!(
                "Detach functionality is currently only supported on Unix-like systems"
            ));
        }
    }

    let mut final_config_path: Option<PathBuf> = None;
    let mut _final_data_path: Option<PathBuf> = None;

    // Determine configuration file path
    if let Some(config_arg_path) = args.config {
        final_config_path = Some(config_arg_path);
    } else {
        let default_config_file = PathBuf::from("config/gnostr.toml");
        if default_config_file.exists() {
            debug!("Using default config file: {:?}", default_config_file);
            final_config_path = Some(default_config_file);
        }
    }

    // If a config file is determined, try to load it to get default data path
    if let Some(config_path) = &final_config_path {
        if let Ok(config_content) = tokio::fs::read_to_string(config_path).await {
            match toml::from_str::<GnostrConfig>(&config_content) {
                Ok(gnostr_config) => {
                    if let Some(data_config) = gnostr_config.data {
                        if let Some(data_path_from_config) = data_config.path {
                            debug!("Data path from config file: {:?}", data_path_from_config);
                            _final_data_path = Some(data_path_from_config);
                        }
                    }
                }
                Err(e) => {
                    // Log error but don't fail, continue with other defaults/args
                    info!("Failed to parse config file {:?}: {}", config_path, e);
                }
            }
        }
    }

    // Override data path if provided by command line args
    if let Some(data_arg_path) = args.data {
        debug!(
            "Overriding data path with command line argument: {:?}",
            data_arg_path
        );
        _final_data_path = Some(data_arg_path);
    }

    // Check if gnostr-relay is installed
    let which_output = Command::new("which")
        .arg("gnostr-relay")
        .output()
        .await
        .context("Failed to run `which gnostr-relay`")?;

    let gnostr_relay_path = if which_output.status.success() && !which_output.stdout.is_empty() {
        String::from_utf8_lossy(&which_output.stdout)
            .trim()
            .to_string()
    } else {
        info!("gnostr-relay not found. Attempting to install...");
        let install_status = Command::new("cargo")
            .args(["install", "gnostr-relay"]) //, "--path", "relay"])
            .spawn()
            .context("Failed to spawn `cargo install gnostr-relay`")?
            .wait()
            .await
            .context("Failed to await `cargo install gnostr-relay`")?;

        if !install_status.success() {
            return Err(anyhow!("Failed to install gnostr-relay"));
        }
        info!("gnostr-relay installed successfully. Checking path again...");
        let which_output_after_install =
            Command::new("which")
                .arg("gnostr-relay")
                .output()
                .await
                .context("Failed to run `which gnostr-relay` after install")?;

        if which_output_after_install.status.success()
            && !which_output_after_install.stdout.is_empty()
        {
            String::from_utf8_lossy(&which_output_after_install.stdout)
                .trim()
                .to_string()
        } else {
            return Err(anyhow!(
                "gnostr-relay not found in PATH after installation."
            ));
        }
    };

    info!("Running gnostr-relay from: {}", gnostr_relay_path);

    let mut cmd = Command::new(gnostr_relay_path);
    //if let Some(config_path) = final_config_path {
    //    cmd.arg("--config").arg(config_path);
    //}
    //if let Some(data_path) = final_data_path {
    //    cmd.arg("--data").arg(data_path);
    //}
    //if args.watch {
    //    cmd.arg("--watch");
    //}
    cmd.arg("--logging").arg(args.logging.to_string());

    let status = cmd
        .spawn()
        .context("Failed to spawn gnostr-relay process")?
        .wait()
        .await
        .context("Failed to await gnostr-relay process")?;

    if !status.success() {
        return Err(anyhow!(
            "gnostr-relay process exited with non-zero status: {:?}",
            status.code()
        ));
    }

    info!("Relay server shutdown");
    Ok(())
}

#[cfg(unix)]
async fn run_as_daemon(args: RelaySubCommand) -> Result<()> {
    info!("Starting relay in daemon mode...");

    // Get current executable path
    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;

    // Build daemon command arguments
    let daemon_args = build_daemon_args(&args);

    // Set environment variables for minimal logging
    let mut cmd = tokio::process::Command::new(&current_exe);
    cmd.args(&daemon_args);
    cmd.env("RUST_LOG", "error");
    cmd.env("GNOSTR_RELAY_MODE", "daemon");

    // Detach from terminal
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        use std::os::unix::process::CommandExt;
        cmd.stdin(std::process::Stdio::null());
        cmd.stdout(std::process::Stdio::null());
        cmd.stderr(std::process::Stdio::null());
    }

    // Spawn daemon process
    cmd.spawn().context("Failed to spawn daemon process")?;

    println!("Relay started in background");
    println!("Process ID: {}", std::process::id());

    Ok(())
}

#[cfg(unix)]
fn build_daemon_args(args: &RelaySubCommand) -> Vec<String> {
    let mut daemon_args = Vec::new();

    // Add relay subcommand
    daemon_args.push("relay".to_string());

    // Pass through essential arguments
    if let Some(config) = &args.config {
        daemon_args.push("--config".to_string());
        daemon_args.push(config.to_string_lossy().to_string());
    }

    if let Some(data) = &args.data {
        daemon_args.push("--data".to_string());
        daemon_args.push(data.to_string_lossy().to_string());
    }

    if args.watch {
        daemon_args.push("--watch".to_string());
    }

    // Force error-level logging for daemon mode to reduce noise
    daemon_args.push("--logging".to_string());
    daemon_args.push("error".to_string());

    daemon_args
}
