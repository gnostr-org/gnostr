//! Shared Nostr wire types used by the crawler query path.
//!
//! These are the same canonical types re-exported by `gnostr-p2p`, but the
//! crawler cannot depend on `gnostr-p2p` directly because `gnostr-p2p` already
//! depends on `gnostr-crawler`. Re-exporting the shared asyncgit-backed types
//! here keeps the crawler on the same wire format without creating a cycle.

pub use gnostr_asyncgit::types::{
    ClientMessage, Event, EventBuilder, EventKind, Filter, GitNote, IdHex, PrivateKey,
    PublicKeyHex, RelayMessage, SubscriptionId, Unixtime,
};
