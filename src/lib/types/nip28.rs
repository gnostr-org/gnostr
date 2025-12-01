#![allow(clippy::module_inception)]

// NIP-28: Public Chat Channels
// https://github.com/nostr-protocol/nips/blob/master/28.md

use crate::types::event::{Event, UnsignedEvent};
use crate::types::event_kind::{EventKind, EventKindOrRange};
use crate::types::{Id, PublicKey, Signature, Tag, Unixtime};
use secp256k1::{SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};


/// Event Kind 40: Create channel
/// Used to create a public chat channel, including initial metadata like name, description, and picture.
pub const CREATE_CHANNEL: EventKind = EventKind::ChannelCreation;

/// Event Kind 41: Set channel metadata
/// Used to update a channel's public metadata. Clients should treat these like replaceable events,
/// only storing the most recent one, and ignore updates from pubkeys other than the channel creator.
pub const SET_CHANNEL_METADATA: EventKind = EventKind::ChannelMetadata;

/// Event Kind 42: Create channel message
/// Used to send text messages within a channel. It supports NIP-10 tags for relay recommendations
/// and to indicate if a message is a reply or a root message within a thread.
pub const CREATE_CHANNEL_MESSAGE: EventKind = EventKind::ChannelMessage;

/// Event Kind 43: Hide message
/// Allows a user to hide a specific message within a channel. Clients can optionally hide messages
/// for other users based on multiple hide events.
pub const HIDE_MESSAGE: EventKind = EventKind::ChannelHideMessage;

/// Event Kind 44: Mute user
/// Allows a user to mute another user, hiding their messages within the channel. Similar to hiding messages,
/// clients can extend this moderation to multiple users.
pub const MUTE_USER: EventKind = EventKind::ChannelMuteUser;

// Placeholder for potential future NIP-28 related structs or functions.
// For now, we only need to define the event kinds as constants.

// If specific data structures are required for these event kinds in the future,
// they would be defined here.

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{EventKind, Tag};
    use crate::types::{PublicKey, PrivateKey, Unixtime, Id, Error, PublicKeyHex, UncheckedUrl};
    use crate::test_serde;
    use secp256k1::{Keypair, Secp256k1, SecretKey, XOnlyPublicKey};
    use sha2::{Digest, Sha256};
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_nip28_event_kinds() {
        assert_eq!(CREATE_CHANNEL, EventKind::from(40));
        assert_eq!(SET_CHANNEL_METADATA, EventKind::from(41));
        assert_eq!(CREATE_CHANNEL_MESSAGE, EventKind::from(42));
        assert_eq!(HIDE_MESSAGE, EventKind::from(43));
        assert_eq!(MUTE_USER, EventKind::from(44));
    }
}