//! Shared Nostr wire types used by the crawler query path.
//!
//! These are the same canonical types re-exported by `gnostr-p2p`. Both crates
//! depend on the shared asyncgit-backed wire surface, which keeps the graph
//! acyclic while preserving a common message format.

pub use gnostr_asyncgit::types::{
    ClientMessage, Event, EventBuilder, EventKind, Filter, GitNote, IdHex, PrivateKey,
    PublicKeyHex, RelayMessage, SubscriptionId, Unixtime,
};
