//! git-remotes — git remote helper binaries for multiple protocols.
//!
//! Provides git-remote helper binaries extending git with:
//! - `git-remote-blossom`: Blossom blob storage (BUD-01/02, kind:24242 auth)
//! - `git-remote-nostr`: NIP-34 repos resolved via Nostr relay → GRASP HTTP
//! - `git-remote-ipfs`: IPFS/Kubo MFS storage (ipfs://)
//! - `git-remote-pkarr`: PKARR DHT discovery → Blossom endpoint (pkarr://)
//! - `git-remote-tor`: Blossom over Tor SOCKS5 proxy (blossom+onion://, tor://)

pub mod auth;
pub mod blossom_backend;
pub mod ipfs_backend;
pub mod nostr_backend;
pub mod nostr_relay;
pub mod pkarr_backend;
pub mod protocol;
pub mod tor_backend;
