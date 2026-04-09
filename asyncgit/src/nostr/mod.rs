/// Nostr integration for asyncgit.
///
/// Exposes key management and an async relay client that follows the same
/// `std::thread` + crossbeam-channel pattern as the rest of asyncgit
/// (e.g. `AsyncPush`, `AsyncPull`).  No nostr-sdk dependency — uses
/// secp256k1, bech32, and tokio-tungstenite directly.

pub mod client;
pub mod keys;

pub use client::{AsyncNostr, AsyncNostrNotification, NostrEvent};
pub use keys::{
	generate_keys, load_identity, load_key_from_git_config,
	parse_key, save_key_to_git_config, NostrIdentity,
};
