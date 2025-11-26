use anyhow::{anyhow, Context, Result};
use gnostr_relay::App;
use std::path::PathBuf;
use tokio::process::Command;
use tracing::info;

use crate::p2p::chat::p2p::evt_loop;
use crate::queue::InternalEvent;
use libp2p::gossipsub;
use tokio::sync::mpsc;

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
    /// Path to the configuration file.
    #[clap(short, long)]
    pub config: Option<PathBuf>,

    /// Path to the data directory.
    #[clap(short, long)]
    pub data: Option<PathBuf>,

    /// Watch for configuration file changes.
    #[clap(short, long)]
    pub watch: bool,

    /// Set the logging level.
    #[clap(long, value_enum, default_value_t = LogLevel::Info)]
    pub logging: LogLevel,
}
//TODO web actix runtime
pub async fn relay(args: RelaySubCommand) -> Result<()> {
    info!("Start relay server with args: {:?}", args);

    info!("Starting background p2p chat relay on topic 'gnostr'");
    // Channels to handle communication with the p2p event loop, though they won't be actively used in this background setup.
    let (_input_tx, input_rx) = mpsc::channel::<InternalEvent>(100);
    let (peer_tx, mut _peer_rx) = mpsc::channel::<InternalEvent>(100);
    let topic = gossipsub::IdentTopic::new("gnostr");

    tokio::spawn(async move {
        if let Err(e) = evt_loop(input_rx, peer_tx, topic).await {
            tracing::error!("p2p chat relay event loop failed: {}", e);
        }
    });
    info!("p2p chat relay started in the background.");

    // Check if gnostr-relay is installed
    let which_output = Command::new("which")
        .arg("gnostr-relay")
        .output()
        .await
        .context("Failed to run `which gnostr-relay`")?;

    let gnostr_relay_path = if which_output.status.success() && !which_output.stdout.is_empty() {
        String::from_utf8_lossy(&which_output.stdout).trim().to_string()
    } else {
        info!("gnostr-relay not found. Attempting to install...");
        let install_status = Command::new("cargo")
            .args(&["install", "gnostr-relay"]) //, "--path", "relay"])
            .spawn()
            .context("Failed to spawn `cargo install gnostr-relay`")?
            .wait()
            .await
            .context("Failed to await `cargo install gnostr-relay`")?;

        if !install_status.success() {
            return Err(anyhow!("Failed to install gnostr-relay"));
        }
        info!("gnostr-relay installed successfully. Checking path again...");
        let which_output_after_install = Command::new("which")
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
    if let Some(config_path) = args.config {
        cmd.arg("--config").arg(config_path);
    }
    if let Some(data_path) = args.data {
        cmd.arg("--data").arg(data_path);
    }
    if args.watch {
        cmd.arg("--watch");
    }
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