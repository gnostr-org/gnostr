//! `gnostr-p2p` is the Rust crate that owns the P2P package namespace.
//!
//! The browser-side pure JavaScript implementation lives under `src/js/`.

extern crate gnostr_asyncgit as git2;

use std::path::PathBuf;

#[path = "../../src/lib/p2p/args.rs"]
pub mod args;
#[path = "../../src/lib/p2p/behaviour.rs"]
pub mod behaviour;
#[path = "../../src/lib/p2p/command_handler.rs"]
pub mod command_handler;
#[path = "../../src/lib/p2p/event_handler.rs"]
pub mod event_handler;
#[path = "../../src/lib/p2p/git_integration.rs"]
pub mod git_integration;
#[path = "../../src/lib/p2p/git_publisher.rs"]
pub mod git_publisher;
#[path = "../../src/lib/p2p/kvs.rs"]
pub mod kvs;
#[path = "../../src/lib/p2p/lookup.rs"]
pub mod lookup;
#[path = "../../src/lib/p2p/network_config.rs"]
pub mod network_config;
#[path = "../../src/lib/p2p/opt.rs"]
pub mod opt;
#[path = "../../src/lib/p2p/swarm_builder.rs"]
pub mod swarm_builder;
#[path = "../../src/lib/p2p/utils.rs"]
pub mod utils;

/// Crate name.
pub const PACKAGE_NAME: &str = "gnostr-p2p";

/// Return the absolute path to the bundled JavaScript source tree.
pub fn js_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/js")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exposes_js_source_dir() {
        assert!(js_source_dir().ends_with("p2p/src/js"));
    }
}

/// Compatibility namespace for the legacy `crate::p2p::...` module paths.
pub mod p2p {
    pub use crate::{
        args, behaviour, command_handler, event_handler, git_integration, git_publisher, kvs,
        lookup, network_config, opt, swarm_builder, utils,
    };
}
