/// Nostr integration for asyncgit.
///
/// Exposes key management and an async relay client that follows the same
/// `std::thread` + crossbeam-channel pattern as the rest of asyncgit
/// (e.g. `AsyncPush`, `AsyncPull`).  No nostr-sdk dependency — uses
/// secp256k1, bech32, and tokio-tungstenite directly.

pub mod client;
/// Nostr key management: parse, generate, load/save via git config.
pub mod keys;
/// NIP-34 Git Stuff — types, builders and parsers.
pub mod nip34;

#[allow(unused_imports)]
pub use client::*;
#[allow(unused_imports)]
pub use keys::*;
#[allow(unused_imports)]
pub use nip34::*;

#[allow(unused_imports)]
pub use client::{AsyncNostr, AsyncNostrNotification, NostrEvent};
#[allow(unused_imports)]
pub use keys::{
	generate_keypair_strings, generate_keys, load_identity,
	load_key_from_git_config, parse_key, save_key_to_git_config,
	NostrIdentity, DEFAULT_NOSTR_KEY,
};
#[allow(unused_imports)]
pub use nip34::{
	GitIssue, GitPatch, GitRepoAnnouncement, PatchStatus,
};
