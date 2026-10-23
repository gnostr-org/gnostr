//! Primitive packet types for the `perfect_ip` protocol.
//!
//! These are the pure data types exchanged on the wire (serialized as JSON).
//! Networking/behaviour code lives in `gnostr-p2p`.
//!
//! This module re-exports the types from [`crate::utils::protocol_slice`].

pub use crate::utils::protocol_slice::*;
