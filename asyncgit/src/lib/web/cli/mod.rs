//! Command-line interface for gnostr-gnit
//!
//! This module provides argument parsing and configuration management
//! for the gnostr-gnit Git server.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::Duration,
};

use anyhow::Context;
use clap::Parser;
use humantime::format_duration;
use rocksdb::{Options, SliceTransform};
use tracing::warn;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter};

use crate::{database::schema::SCHEMA_VERSION, git::Git};

/// Configuration for the gnostr-gnit server
#[derive(Debug, Clone)]
pub struct Config {
    /// Path to a directory where the RocksDB database should be stored
    pub db_store: PathBuf,
    /// The IP address to bind to
    pub bind_address: std::net::IpAddr,
    /// The socket port to bind to
    pub bind_port: u16,
    /// The path in which bare Git repositories reside
    pub scan_path: PathBuf,
    /// Configures the metadata refresh interval for Git repositories
    pub refresh_interval: RefreshInterval,
    /// Configures the request timeout for incoming HTTP requests
    pub request_timeout: humantime::Duration,
    /// Whether debug logging is enabled
    pub debug: bool,
}

/// Refresh interval configuration
#[derive(Debug, Clone, Copy)]
pub enum RefreshInterval {
    Never,
    Duration(Duration),
}

impl std::fmt::Display for RefreshInterval {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Never => write!(f, "never"),
            Self::Duration(s) => write!(f, "{}", humantime::format_duration(*s)),
        }
    }
}

impl std::str::FromStr for RefreshInterval {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "never" {
            Ok(Self::Never)
        } else if let Ok(v) = humantime::parse_duration(s) {
            Ok(Self::Duration(v))
        } else {
            Err("must be seconds, a human readable duration (eg. '10m') or 'never'")
        }
    }
}

/// Parse command-line arguments
pub fn parse_args() -> Result<Config, anyhow::Error> {
    parse_args_internal()
}

/// Initialize configuration from parsed arguments
pub fn initialize_config(args: &Args) -> Result<Config, anyhow::Error> {
    parse_args_internal()
}

/// Parse command-line arguments (internal function)
fn parse_args_internal() -> Result<Config, anyhow::Error> {
    let args = Args::parse();

    // Set up logging if RUST_LOG is not already set
    if std::env::var_os("RUST_LOG").is_none() {
        if args.debug {
            std::env::set_var("RUST_LOG", "debug");
        } else if args.info {
            std::env::set_var("RUST_LOG", "info");
        } else {
            std::env::set_var("RUST_LOG", "warn");
        }
    }

    // Initialize logging
    let logger_layer = tracing_subscriber::fmt::layer()
        .with_span_events(tracing_subscriber::fmt::format::FmtSpan::CLOSE);
    let env_filter = EnvFilter::from_default_env();

    tracing_subscriber::registry()
        .with(env_filter)
        .with(logger_layer)
        .init();

    // Create configuration
    let scan_path = args
        .scan_path
        .canonicalize()
        .context("Could not canonicalize scan path")?;

    Ok(Config {
        db_store: args.db_store,
        bind_address: args.bind_address,
        bind_port: args.bind_port,
        scan_path,
        refresh_interval: args.refresh_interval,
        request_timeout: args.request_timeout,
        debug: args.debug,
    })
}

/// Command-line argument definitions
#[derive(Debug, clap::Parser)]
#[clap(author, version = env!("CARGO_PKG_VERSION"), about = "gnostr:git server")]
pub struct Args {
    /// Path to a directory where the RocksDB database should be stored.
    ///
    /// This directory will be created if it doesn't exist. The RocksDB database is
    /// quick to generate, so it can be pointed to temporary storage.
    #[clap(short, long, default_value = ".gnostr/web")]
    pub db_store: PathBuf,

    /// The IP address to bind to (e.g., 127.0.0.1, 0.0.0.0).
    #[clap(long, default_value = "127.0.0.1")]
    pub bind_address: std::net::IpAddr,

    /// The socket port to bind to (e.g., 3333).
    #[clap(short, long, default_value = "3333", env = "GNOSTR_GNIT_BIND_PORT")]
    pub bind_port: u16,

    /// The path in which your bare Git repositories reside.
    ///
    /// This directory will be scanned recursively for Git repositories.
    #[clap(short, long, default_value = ".")]
    pub scan_path: PathBuf,

    /// Configures the metadata refresh interval for Git repositories (e.g., "never" or "60s").
    #[clap(long, default_value_t = RefreshInterval::Duration(std::time::Duration::from_secs(30)), env = "GNOSTR_GNIT_REFRESH_INTERVAL")]
    pub refresh_interval: RefreshInterval,

    /// Configures the request timeout for incoming HTTP requests (e.g., "10s").
    #[clap(long, default_value_t = humantime::Duration(std::time::Duration::from_secs(10)).into(), env = "GNOSTR_GNIT_REQUEST_TIMEOUT")]
    pub request_timeout: humantime::Duration,

    /// Enable debug logging
    #[clap(long, default_value = "false")]
    pub debug: bool,

    /// Enable info logging
    #[clap(long, default_value = "false")]
    pub info: bool,
}
