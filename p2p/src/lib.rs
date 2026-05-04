//! `gnostr-p2p` is the Rust crate that owns the P2P package namespace.
//!
//! The browser-side pure JavaScript implementation lives under `src/js/`.

extern crate gnostr_asyncgit as git2;

use std::path::PathBuf;

pub mod args;
pub mod cli;
pub mod behaviour;
pub mod command_handler;
pub mod event_handler;
pub mod git_integration;
pub mod git_publisher;
pub mod kvs;
pub mod lookup;
pub mod network_config;
pub mod opt;
pub mod bridge;
pub mod js;
pub mod template_html;
pub mod swarm_builder;
pub mod utils;

/// Crate name.
pub const PACKAGE_NAME: &str = "gnostr-p2p";

/// Return the absolute path to the bundled JavaScript source tree.
pub fn js_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/js")
}

pub use bridge::{asset_content_type, asset_response, shell_html};
pub use js::get_js_assets;
pub use template_html::{get_template_assets, TemplateHtml};

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
        args, behaviour, cli, command_handler, event_handler, git_integration, git_publisher, kvs,
        lookup, network_config, opt, swarm_builder, utils,
    };
}
