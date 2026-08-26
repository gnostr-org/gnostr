//! `gnostr-p2p` owns the libp2p transport namespace and swarm composition.
//!
//! It re-exports the shared asyncgit-backed Nostr wire types for downstream
//! crates, and it keeps relay bucket, command handling, and quorum-time helpers
//! local so the dependency graph stays one-way: `types -> asyncgit -> p2p`.
//!
//! The crate builds the node transport stack, exposes browser-side assets under
//! the `js` feature, and adds the optional local Tor transport under `tor`.
//!
//! Attestation syndication follows the same deterministic structure used by
//! `asyncgit`: mined commit, signed attestation event, mined git note, and a
//! chronological `notes_ref` chain that links each public attestation to the
//! previous one.
//#![feature(trivial_bounds)]

#[cfg(not(target_os = "tvos"))]
pub mod git2 {
    pub use gnostr_asyncgit::git2::*;
    pub use gnostr_asyncgit::types;
}

use std::path::PathBuf;
use std::process::{Command, Stdio};

#[cfg(not(target_os = "tvos"))]
use libp2p::identity;
#[cfg(not(target_os = "tvos"))]
use sha2::{Digest, Sha256};

#[cfg(not(target_os = "tvos"))]
pub mod args;
#[cfg(not(target_os = "tvos"))]
pub mod cli;
#[cfg(not(target_os = "tvos"))]
pub mod behaviour;
#[cfg(target_os = "tvos")]
#[path = "embedded_network_tvos.rs"]
pub mod embedded_network;
#[cfg(not(target_os = "tvos"))]
pub mod embedded_network;
#[cfg(not(target_os = "tvos"))]
pub mod fractal;
#[cfg(not(target_os = "tvos"))]
pub mod perfect_ip;
#[cfg(not(target_os = "tvos"))]
pub mod command_handler;
#[cfg(not(target_os = "tvos"))]
pub mod event_handler;
#[cfg(not(target_os = "tvos"))]
pub mod git_integration;
#[cfg(not(target_os = "tvos"))]
pub mod git_publisher;
#[cfg(not(target_os = "tvos"))]
pub mod kvs;
#[cfg(not(target_os = "tvos"))]
pub mod lookup;
#[cfg(not(target_os = "tvos"))]
pub mod network_config;
#[cfg(not(target_os = "tvos"))]
pub mod opt;
#[cfg(not(doc))]
#[cfg(all(feature = "js", not(target_os = "tvos")))]
pub mod bridge;
#[cfg(not(doc))]
#[cfg(all(feature = "js", not(target_os = "tvos")))]
pub mod js;
#[cfg(not(doc))]
#[cfg(all(feature = "tor", not(target_os = "tvos")))]
pub mod tor;
#[cfg(not(target_os = "tvos"))]
pub mod crawler_broadcast;
#[cfg(not(doc))]
#[cfg(not(target_os = "tvos"))]
pub mod message;
#[cfg(not(target_os = "tvos"))]
pub mod repo_state;
#[cfg(not(doc))]
#[cfg(not(target_os = "tvos"))]
pub mod relay_bridge;
#[cfg(not(target_os = "tvos"))]
pub mod relay_paths;
#[cfg(not(target_os = "tvos"))]
pub mod time;
#[cfg(not(doc))]
#[cfg(all(feature = "js", not(target_os = "tvos")))]
pub mod template_html;
#[cfg(not(target_os = "tvos"))]
pub mod swarm_builder;
#[cfg(not(target_os = "tvos"))]
pub mod utils;
#[cfg(not(target_os = "tvos"))]
pub use crawler_broadcast as relay_buckets;

/// Crate name.
pub const PACKAGE_NAME: &str = "gnostr-p2p";

/// Return the absolute path to the bundled JavaScript source tree.
pub fn js_source_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/js")
}

/// Build a deterministic libp2p keypair from an optional secret seed string.
///
/// Hex SHA-256 seeds are used directly; any other input is hashed into a 32-byte seed.
#[cfg(not(target_os = "tvos"))]
pub fn keypair_from_seed(secret_key_seed: Option<String>) -> identity::Keypair {
    match secret_key_seed {
        Some(seed) => identity::Keypair::ed25519_from_bytes(seed_bytes(&seed))
            .expect("only errors on wrong length"),
        None => identity::Keypair::generate_ed25519(),
    }
}

#[cfg(not(target_os = "tvos"))]
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

#[cfg(all(not(doc), feature = "js", not(target_os = "tvos")))]
pub use bridge::{asset_content_type, asset_response, shell_html};
#[cfg(all(not(doc), feature = "js", not(target_os = "tvos")))]
pub use js::get_js_assets;
#[cfg(all(not(doc), feature = "tor", not(target_os = "tvos")))]
pub use tor::{build_transport as build_tor_transport, AddressConversion, TorError, TorTransport, TorTransportError, TokioTorStream};
#[cfg(not(target_os = "tvos"))]
#[cfg(not(doc))]
pub use message::*;
#[cfg(not(target_os = "tvos"))]
pub use repo_state::{RepoStateQuorum, RepoStateRefs, RepoStateSnapshot};
#[cfg(not(target_os = "tvos"))]
#[cfg(not(doc))]
pub use relay_bridge::{RelayBridgeCommand, RelayBridgeNotification, RelayBridgeSession, NostrRelayConnection};
#[cfg(not(target_os = "tvos"))]
pub use fractal::{build_fractal_swarm, run_fractal_engine, FractalBehaviour, FractalBehaviourEvent, IntegrityManager, ProtocolSlice};
#[cfg(not(target_os = "tvos"))]
pub use perfect_ip::{calculate_parity, process_slice, Header, ProtocolSlice as PerfectProtocolSlice, MTU_PAYLOAD};
#[cfg(all(not(doc), feature = "js", not(target_os = "tvos")))]
pub use template_html::{get_template_assets, TemplateHtml};

pub fn spawn_detached_current_exe<I, S>(args: I) -> Result<u32, Box<dyn std::error::Error>>
where
    I: IntoIterator<Item = S>,
    S: AsRef<std::ffi::OsStr>,
{
    let current_exe = std::env::current_exe()?;
    let mut command = Command::new(current_exe);
    command.args(args);
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    let child = command.spawn()?;
    Ok(child.id())
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
#[cfg(not(target_os = "tvos"))]
pub mod p2p {
    pub use crate::{
        args, behaviour, cli, command_handler, event_handler, git_integration, git_publisher, kvs,
        keypair_from_seed, lookup, network_config, opt, repo_state, relay_paths, time, crawler_broadcast,
        fractal,
        swarm_builder, utils,
    };
    #[cfg(not(doc))]
    pub use crate::{message, relay_bridge};
    #[cfg(feature = "tor")]
    pub use crate::tor;
}
