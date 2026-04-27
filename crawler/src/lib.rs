pub mod processor;
pub mod api;
pub mod cli;
pub mod relay_metadata;
pub mod relay_fetch;
pub mod relay_io;
pub mod pubkeys;
pub mod commands;
pub mod query;
pub mod relay_manager;
pub mod relays;
pub mod stats;
mod api_cache;
mod api_routes;
mod git_helpers;

pub use cli::{dispatch_cli_command, run, Cli, CliArgs, Commands};
pub use query::{build_gnostr_query, send, Config, ConfigBuilder};
pub use api::{run_api_server, run_api_server_detached};
pub use commands::{run_nip34, run_sniper, run_watch};
pub use relay_metadata::Relay;
pub use relay_fetch::{fetch_relay_texts, parse_relay_metadata, websocket_http_url};
pub use relay_io::{load_file, load_relays_or_bootstrap, load_shitlist, preprocess_line};
pub use git_helpers::{
    log_message_matches, match_with_parent, print_commit, print_time, sig_matches,
};

pub fn init_tracing() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive("hyper::client::trace=trace".parse()?)
                .add_directive("hyper::client::connect=trace".parse()?)
                .add_directive("hyper::client::connect::http=off".parse()?)
                .add_directive("hyper::proto=off".parse()?)
                .add_directive("nostr_sdk::relay=off".parse()?)
                .add_directive("nostr_relay_pool=off".parse()?)
                .add_directive("nostr_relay_pool::relay::inner=off".parse()?),
        )
        .init();
    Ok(())
}
