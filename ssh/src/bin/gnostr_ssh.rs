use clap::Parser;
use std::{path::PathBuf, sync::Arc};
use anyhow::{Context, anyhow};

use env_logger::Env;
use log::{error, info};
use tokio::sync::Mutex;
use toml;

use gnostr_ssh_lib::{state::State, ssh, utils::is_port_in_use, config::server::{load_server_config, ServerUser}};

// Define command line arguments
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Port to run the SSH server on
    #[arg(short, long, default_value_t = 2222)] // Default port from gnostr-ssh.toml? Or a new default?
    port: u16,
    /// Path to the gnostr-ssh.toml file
    #[arg(short, long, default_value = "gnostr-ssh.toml")]
    config: PathBuf,
    /// Path to the gnostr-repo.toml file
    #[arg(short = 'r', long, default_value = "gnostr-repo.toml")]
    repo_config: PathBuf,

    /// Add or update a user with the given username
    #[arg(long)]
    add_user: Option<String>,
    /// Set the user as an admin
    #[arg(long)]
    is_admin: bool,
    /// Set the public key for the user
    #[arg(long)]
    public_key: Option<String>,
    /// Allow the user to create repositories
    #[arg(long)]
    can_create_repos: bool,
}

async fn start(port: u16, config: PathBuf, repo_config: PathBuf) -> anyhow::Result<()> {
    if is_port_in_use(port).await {
        eprintln!("Error: Port {} is already in use.", port);
        return Err(anyhow!("Port {} is already in use.", port));
    }

    info!("Loading state...");
    let mut state = State::new(config, repo_config).await?; // Make state mutable to modify port
    // Update the server config port if it's different from the default in gnostr-ssh.toml
    // Or, if gnostr-ssh.toml doesn't exist, use the parsed port.
    // For now, let's assume we override the port from gnostr-ssh.toml if provided via CLI.
    state.server_config.port = port;

    let state = Arc::new(Mutex::new(state));

    info!("Starting server on port {}...", port); // Inform user about the port
    let _ = sd_notify::notify(true, &[sd_notify::NotifyState::Ready]);
    ssh::start_server(state).await?;
    Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse(); // Parse arguments
    let config_path = args.config.canonicalize().context("Could not canonicalize config path")?;
    let repo_config_path = args.repo_config.canonicalize().context("Could not canonicalize repo config path")?;

    env_logger::init_from_env(Env::default().default_filter_or("info"));

    if let Some(username) = args.add_user {
        let mut server_config = load_server_config(config_path.clone()).await?;

        let public_key_arg = args.public_key.context("Public key is required when adding a user")?;
        let public_key = if PathBuf::from(&public_key_arg).exists() {
            tokio::fs::read_to_string(&public_key_arg)
                .await
                .context(format!("Failed to read public key from file: {}", &public_key_arg))?
        } else {
            public_key_arg
        };

        let user = ServerUser {
            public_key,
            is_admin: Some(args.is_admin),
            can_create_repos: Some(args.can_create_repos),
        };
        server_config.users.insert(username.clone(), user);

        // Save the updated config
        let toml_string = toml::to_string_pretty(&server_config).context("Failed to serialize server config")?;
        tokio::fs::write(&config_path, toml_string).await.context("Failed to write updated server config")?;

        info!("User '{}' added/updated successfully.", username);
        return Ok(());
    }

    let result = async move {
        start(args.port, config_path, repo_config_path).await
    }.await;

    if let Err(e) = result {
        error!("{:#}", e);
    }

    Ok(())
}
