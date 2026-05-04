//! `gnostr-p2p` is the Rust crate that owns the P2P package namespace.
//!
//! The browser-side pure JavaScript implementation lives under `src/js/`.

extern crate gnostr_asyncgit as git2;

use std::path::PathBuf;

use libp2p::identity;
use sha2::{Digest, Sha256};

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
pub mod message;
pub mod template_html;
pub mod swarm_builder;
pub mod utils;

/// Crate name.
pub const PACKAGE_NAME: &str = "gnostr-p2p";

/// Return the absolute path to the bundled JavaScript source tree.
pub fn js_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/js")
}

/// Build a deterministic libp2p keypair from an optional secret seed string.
///
/// Hex SHA-256 seeds are used directly; any other input is hashed into a 32-byte seed.
pub fn keypair_from_seed(secret_key_seed: Option<String>) -> identity::Keypair {
    match secret_key_seed {
        Some(seed) => identity::Keypair::ed25519_from_bytes(seed_bytes(&seed))
            .expect("only errors on wrong length"),
        None => identity::Keypair::generate_ed25519(),
    }
}

fn seed_bytes(seed: &str) -> [u8; 32] {
    if seed.len() == 64 && seed.chars().all(|c| c.is_ascii_hexdigit()) {
        let mut bytes = [0u8; 32];
        for (idx, chunk) in seed.as_bytes().chunks_exact(2).enumerate() {
            bytes[idx] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16)
                .expect("validated hex digest");
        }
        return bytes;
    }

    let digest = Sha256::digest(seed.as_bytes());
    digest.into()
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
        keypair_from_seed, lookup, message, network_config, opt, swarm_builder, utils,
    };
}
