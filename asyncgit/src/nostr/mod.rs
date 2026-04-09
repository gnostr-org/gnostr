/// Nostr integration for asyncgit.
///
/// Exposes key management, a read-only signer, and an async client that
/// follows the same `std::thread` + crossbeam-channel pattern as the rest
/// of asyncgit (e.g. `AsyncPush`, `AsyncPull`).

pub mod client;
pub mod keys;
pub(crate) mod signer;

pub use client::{AsyncNostr, AsyncNostrNotification};
pub use keys::{
	generate_keys, load_identity, load_key_from_git_config,
	parse_key, save_key_to_git_config, NostrIdentity,
};
