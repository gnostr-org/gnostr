//! Bridge helpers that wrap canonical Nostr crypto and asset metadata.
//!
//! This module stays inside `types` so other crates can later wire the same
//! APIs into WASM or browser bindings without re-implementing the crypto.
//!
//! # Usage
//! Use [`encrypt_dm`] and [`decrypt_dm`] with hex-encoded secret/public keys:
//! ```ignore
//! use gnostr_types::nostr::bridge::{decrypt_dm, encrypt_dm};
//!
//! let ciphertext = encrypt_dm(sender_sk_hex, recipient_pk_hex, "hello")?;
//! let plaintext = decrypt_dm(recipient_sk_hex, sender_pk_hex, &ciphertext)?;
//! # Ok::<(), gnostr_types::nostr::Error>(())
//! ```

pub mod dm;
pub mod mime;

pub use dm::{decrypt_dm, encrypt_dm, encrypt_dm_with_algorithm};
pub use mime::asset_content_type;
