#![allow(clippy::module_inception)]

// NIP-28: Public Chat Channels
// https://github.com/nostr-protocol/nips/blob/master/28.md

use crate::types::versioned::event3::PreEventV3;
use crate::types::versioned::event3::EventV3;
use crate::types::event_kind::{EventKind, EventKindOrRange};
use crate::types::{Id, PublicKey, PublicKeyHex, Signature, TagV3, Unixtime, Error, Signer, KeySecurity, NAddr, NostrBech32, NostrUrl, UncheckedUrl};
use secp256k1::{SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};
use std::str::FromStr;
use std::collections::HashSet;


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

/// Represents a parsed Kind 40 event for creating a public channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelCreationEvent {
    /// The unique identifier for the channel (from 'd' tag).
    pub channel_id: String,
    /// The name of the channel (from 'name' tag, optional).
    pub channel_name: Option<String>,
    /// The description of the channel (from 'description' tag, optional).
    pub channel_description: Option<String>,
    /// URL to the channel's picture (from 'picture' tag, optional).
    pub channel_picture: Option<String>,
    /// A recommended relay URL for the channel (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
    /// The public key of the event author.
    pub pubkey: PublicKey,
}

/// Represents a parsed Kind 41 event for setting channel metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelMetadataEvent {
    /// The unique identifier for the channel (from 'd' tag).
    pub channel_id: String,
    /// The name of the channel (from 'name' tag, optional).
    pub channel_name: Option<String>,
    /// The description of the channel (from 'description' tag, optional).
    pub channel_description: Option<String>,
    /// URL to the channel's picture (from 'picture' tag, optional).
    pub channel_picture: Option<String>,
    /// A recommended relay URL for the channel (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
    /// The public key of the event author.
    pub pubkey: PublicKey,
}

/// Creates a Kind 40 event for creating a public channel.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `channel_name`: The name of the channel (optional, 'name' tag).
/// * `channel_description`: The description of the channel (optional, 'description' tag).
/// * `channel_picture`: URL to the channel's picture (optional, 'picture' tag).
/// * `relay_url`: A recommended relay URL for the channel (optional, 'relay' tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on failure.
pub fn create_channel(
    signer: &dyn Signer,
    channel_id: &str,
    channel_name: &str,
    channel_description: &str,
    channel_picture: Option<&str>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'name' tag - optional
    if !channel_name.is_empty() {
        tags.push(TagV3::new(&["name", channel_name]));
    }

    // 'description' tag - optional
    if !channel_description.is_empty() {
        tags.push(TagV3::new(&["description", channel_description]));
    }

    // 'picture' tag - optional
    if let Some(picture_url) = channel_picture {
        if !picture_url.is_empty() {
            tags.push(TagV3::new(&["picture", picture_url]));
        }
    }

    // 'relay' tag - optional
    if let Some(relay) = relay_url {
        // NIP-28 doesn't explicitly define a marker for channel creation relay, so use None.
        tags.push(TagV3::new_relay(relay.clone(), None)); 
    }

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: CREATE_CHANNEL, // Kind 40
        tags,
        content: "".to_string(), // Channel creation event might not have content per NIP-28
    };

    // Sign the event
    signer.sign_event(pre_event)
}

/// Parses a generic `EventV3` into a `ChannelCreationEvent` if it matches Kind 40 and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `ChannelCreationEvent` on success, or an `Error` if parsing fails or the event is not a valid Kind 40 event.
pub fn parse_channel_creation(event: &EventV3) -> Result<ChannelCreationEvent, Error> {
    if event.kind != CREATE_CHANNEL {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut channel_name: Option<String> = None;
    let mut channel_description: Option<String> = None;
    let mut channel_picture: Option<String> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if tag.tagname() == "name" && !tag.value().is_empty() {
            channel_name = Some(tag.value().to_string());
        } else if tag.tagname() == "description" && !tag.value().is_empty() {
            channel_description = Some(tag.value().to_string());
        } else if tag.tagname() == "picture" && !tag.value().is_empty() {
            channel_picture = Some(tag.value().to_string());
        } else if let Ok((url, _)) = tag.parse_relay() {
            relay_url = Some(url);
        }
    }

    match channel_id {
        Some(id) => Ok(ChannelCreationEvent {
            channel_id: id,
            channel_name,
            channel_description,
            channel_picture,
            relay_url,
            pubkey: event.pubkey,
        }),
        None => Err(Error::AssertionFailed("Missing 'd' tag for channel ID.".to_string())),
    }
}

/// Represents a parsed Kind 42 event for a message within a channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelMessageEvent {
    /// The unique identifier of the channel (from 'd' tag).
    pub channel_id: String,
    /// The content of the message.
    pub message: String,
    /// The ID of the message this message is replying to ('e' tag with 'reply' marker).
    pub reply_to: Option<Id>,
    /// The ID of the root message in a thread ('e' tag with 'root' marker).
    pub root_message: Option<Id>,
    /// The public key of the sender.
    pub pubkey: PublicKey,
    /// A recommended relay URL for context (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
}

/// Parses a generic `EventV3` into a `ChannelMessageEvent` if it matches Kind 42 and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `ChannelMessageEvent` on success, or an `Error` if parsing fails or the event is not a valid Kind 42 event.
pub fn parse_channel_message(event: &EventV3) -> Result<ChannelMessageEvent, Error> {
    if event.kind != CREATE_CHANNEL_MESSAGE {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut reply_to: Option<Id> = None;
    let mut root_message: Option<Id> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if let Ok((id, recommended_relay_url, marker)) = tag.parse_event() {
            if marker.as_deref() == Some("reply") {
                reply_to = Some(id);
                // Store relay if present, prioritizing explicit relay tags on reply/root.
                relay_url = recommended_relay_url;
            } else if marker.as_deref() == Some("root") {
                root_message = Some(id);
                relay_url = recommended_relay_url; // Store relay if present
            }
        } else if let Ok((url, _)) = tag.parse_relay() {
            // If no explicit relay tag was found on reply/root, check for a standalone 'r' tag.
            if relay_url.is_none() {
                relay_url = Some(url);
            }
        }
    }

    match channel_id {
        Some(id) => Ok(ChannelMessageEvent {
            channel_id: id,
            message: event.content.clone(),
            reply_to,
            root_message,
            pubkey: event.pubkey,
            relay_url,
        }),
        None => Err(Error::AssertionFailed("Missing 'd' tag for channel ID.".to_string())),
    }
}

/// Represents a parsed Kind 43 event for hiding a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HideMessageEvent {
    /// The unique identifier of the channel (from 'd' tag).
    pub channel_id: String,
    /// The ID of the message that was hidden ('e' tag).
    pub message_id_to_hide: Id,
    /// An optional reason for hiding the message (from 'reason' tag).
    pub reason: Option<String>,
    /// A recommended relay URL for context (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
    /// The public key of the sender of the hide message event.
    pub pubkey: PublicKey,
}

/// Parses a generic `EventV3` into a `HideMessageEvent` if it matches Kind 43 and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `HideMessageEvent` on success, or an `Error` if parsing fails or the event is not a valid Kind 43 event.
pub fn parse_hide_message(event: &EventV3) -> Result<HideMessageEvent, Error> {
    if event.kind != HIDE_MESSAGE {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut message_id_to_hide: Option<Id> = None;
    let mut reason: Option<String> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if let Ok((id, recommended_relay_url, _)) = tag.parse_event() {
            // Assume the first 'e' tag is the message to hide.
            if message_id_to_hide.is_none() {
                message_id_to_hide = Some(id);
                relay_url = recommended_relay_url;
            }
        } else if let Ok((url, _)) = tag.parse_relay() {
            // Capture relay URL if not already set by an 'e' tag.
            if relay_url.is_none() {
                relay_url = Some(url);
            }
        } else if tag.tagname() == "reason" && !tag.value().is_empty() {
            reason = Some(tag.value().to_string());
        }
    }

    match (channel_id, message_id_to_hide) {
        (Some(id), Some(msg_id)) => Ok(HideMessageEvent {
            channel_id: id,
            message_id_to_hide: msg_id,
            reason,
            relay_url,
            pubkey: event.pubkey,
        }),
        (None, _) => Err(Error::AssertionFailed("Missing 'd' tag for channel ID.".to_string())),
        (_, None) => Err(Error::AssertionFailed("Missing 'e' tag for message ID.".to_string())),
    }
}

/// Represents a parsed Kind 44 event for muting a user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MuteUserEvent {
    /// The unique identifier of the channel (from 'd' tag).
    pub channel_id: String,
    /// The public key of the user being muted ('p' tag).
    pub user_pubkey: PublicKey,
    /// An optional reason for muting the user (from 'reason' tag).
    pub reason: Option<String>,
    /// A recommended relay URL for context (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
    /// The public key of the sender of the mute event.
    pub pubkey: PublicKey,
}

/// Parses a generic `EventV3` into a `MuteUserEvent` if it matches Kind 44 and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `MuteUserEvent` on success, or an `Error` if parsing fails or the event is not a valid Kind 44 event.
pub fn parse_mute_user(event: &EventV3) -> Result<MuteUserEvent, Error> {
    if event.kind != MUTE_USER {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut user_pubkey: Option<PublicKey> = None;
    let mut reason: Option<String> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if let Ok((pubkey, recommended_relay_url, _)) = tag.parse_pubkey() {
            // NIP-28 specifies the 'p' tag for the muted user's public key.
            user_pubkey = Some(pubkey);
            relay_url = recommended_relay_url; // Capture relay if present
        } else if let Ok((url, _)) = tag.parse_relay() {
            // Capture relay URL if not already set by a 'p' tag.
            if relay_url.is_none() {
                relay_url = Some(url);
            }
        } else if tag.tagname() == "reason" && !tag.value().is_empty() {
            reason = Some(tag.value().to_string());
        }
    }

    match (channel_id, user_pubkey) {
        (Some(id), Some(pk)) => Ok(MuteUserEvent {
            channel_id: id,
            user_pubkey: pk,
            reason,
            relay_url,
            pubkey: event.pubkey,
        }),
        (None, _) => Err(Error::AssertionFailed("Missing 'd' tag for channel ID.".to_string())),
        (_, None) => Err(Error::AssertionFailed("Missing 'p' tag for user public key.".to_string())),
    }
}

/// Creates a Kind 41 event for setting channel metadata.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `channel_name`: The new name of the channel (optional, 'name' tag).
/// * `channel_description`: The new description of the channel (optional, 'description' tag).
/// * `channel_picture`: New URL to the channel's picture (optional, 'picture' tag).
/// * `relay_url`: A recommended relay URL for the channel (optional, 'relay' tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on failure.
pub fn set_channel_metadata(
    signer: &dyn Signer,
    channel_id: &str,
    channel_name: Option<&str>,
    channel_description: Option<&str>,
    channel_picture: Option<&str>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'name' tag - optional
    if let Some(name) = channel_name {
        if !name.is_empty() {
            tags.push(TagV3::new(&["name", name]));
        }
    }

    // 'description' tag - optional
    if let Some(description) = channel_description {
        if !description.is_empty() {
            tags.push(TagV3::new(&["description", description]));
        }
    }

    // 'picture' tag - optional
    if let Some(picture_url) = channel_picture {
        if !picture_url.is_empty() {
            tags.push(TagV3::new(&["picture", picture_url]));
        }
    }

    // 'relay' tag - optional
    if let Some(relay) = relay_url {
        tags.push(TagV3::new_relay(relay.clone(), None)); // Metadata updates might also include relay recommendations
    }

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: SET_CHANNEL_METADATA, // Kind 41
        tags,
        content: "".to_string(), // Metadata events typically have empty content
    };

    // Sign the event
    signer.sign_event(pre_event)
}

/// Parses a generic `EventV3` into a `ChannelMetadataEvent` if it matches Kind 41 and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `ChannelMetadataEvent` on success, or an `Error` if parsing fails or the event is not a valid Kind 41 event.
pub fn parse_set_channel_metadata(event: &EventV3) -> Result<ChannelMetadataEvent, Error> {
    if event.kind != SET_CHANNEL_METADATA {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut channel_name: Option<String> = None;
    let mut channel_description: Option<String> = None;
    let mut channel_picture: Option<String> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if tag.tagname() == "name" && !tag.value().is_empty() {
            channel_name = Some(tag.value().to_string());
        } else if tag.tagname() == "description" && !tag.value().is_empty() {
            channel_description = Some(tag.value().to_string());
        } else if tag.tagname() == "picture" && !tag.value().is_empty() {
            channel_picture = Some(tag.value().to_string());
        } else if let Ok((url, _)) = tag.parse_relay() {
            relay_url = Some(url);
        }
    }

    match channel_id {
        Some(id) => Ok(ChannelMetadataEvent {
            channel_id: id,
            channel_name,
            channel_description,
            channel_picture,
            relay_url,
            pubkey: event.pubkey,
        }),
        None => Err(Error::AssertionFailed("Missing 'd' tag for channel ID.".to_string())),
    }
}



/// Creates a Kind 42 event for sending a message within a channel.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `message`: The content of the message.
/// * `reply_to_id`: The ID of the message this message is replying to (optional, 'e' tag with 'reply' marker).
/// * `root_message_id`: The ID of the root message in a thread (optional, 'e' tag with 'root' marker).
/// * `relay_url`: A recommended relay URL for context (optional, 'relay' tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on failure.
pub fn create_channel_message(
    signer: &dyn Signer,
    channel_id: &str,
    message: &str,
    reply_to_id: Option<Id>,
    root_message_id: Option<Id>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'e' tag for reply
    if let Some(id) = reply_to_id {
        tags.push(TagV3::new_event(id, relay_url.cloned(), Some("reply".to_string())));
    }

    // 'e' tag for root message
    if let Some(id) = root_message_id {
        tags.push(TagV3::new_event(id, relay_url.cloned(), Some("root".to_string())));
    }

    // 'relay' tag
    if let Some(url) = relay_url {
        tags.push(TagV3::new_relay(url.clone(), None));
    }

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: CREATE_CHANNEL_MESSAGE, // Kind 42
        tags,
        content: message.to_string(),
	};
    // Sign the event
    signer.sign_event(pre_event)
}

/// Creates a Kind 43 event for hiding a message.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `message_id_to_hide`: The ID of the message to hide (required, 'e' tag).
/// * `reason`: An optional reason for hiding the message.
/// * `relay_url`: A recommended relay URL for context (optional, 'relay' tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on failure.
pub fn hide_message(
    signer: &dyn Signer,
    channel_id: &str,
    message_id_to_hide: &Id,
    reason: Option<&str>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'e' tag for the message to hide
    tags.push(TagV3::new_event(*message_id_to_hide, relay_url.cloned(), None));

    // 'reason' tag - optional
    if let Some(r) = reason {
        if !r.is_empty() {
            tags.push(TagV3::new(&["reason", r]));
        }
    }

    // 'relay' tag - optional
    if let Some(url) = relay_url {
        tags.push(TagV3::new_relay(url.clone(), None));
    }

    // Filter out any 'p' tags to ensure NIP-28 compliance for Kind 43 events
    let filtered_tags: Vec<TagV3> = tags.into_iter().filter(|tag| tag.tagname() != "p").collect();

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: HIDE_MESSAGE, // Kind 43
        tags: filtered_tags,
        content: "".to_string(),
	};
    // Sign the event
    signer.sign_event(pre_event)
}

/// Creates a Kind 44 event for muting a user.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `user_pubkey`: The public key of the user to mute (required, 'p' tag).
/// * `reason`: An optional reason for muting the user.
/// * `relay_url`: A recommended relay URL for context (optional, 'relay' tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on failure.
pub fn mute_user(
    signer: &dyn Signer,
    channel_id: &str,
    user_pubkey: &PublicKey,
    reason: Option<&str>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'p' tag for the user to mute
    tags.push(TagV3::new_pubkey(user_pubkey.clone(), relay_url.cloned(), None));

    // 'reason' tag - optional
    if let Some(r) = reason {
        if !r.is_empty() {
            tags.push(TagV3::new(&["reason", r]));
        }
    }

    // 'relay' tag - optional
    if let Some(url) = relay_url {
        tags.push(TagV3::new_relay(url.clone(), None));
    }

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: MUTE_USER, // Kind 44
        tags,
        content: "".to_string(), // Mute user events typically have empty content
    };

    // Sign the event
    signer.sign_event(pre_event)
}


#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{EventKind, TagV3, PublicKey, PrivateKey, Unixtime, Id, Error, PublicKeyHex, UncheckedUrl, Signer, KeySecurity};
    use crate::KeySigner;
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

    #[test]
    fn test_create_channel_event() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "my-cool-channel";
        let channel_name = "My Cool Channel";
        let channel_description = "A channel for cool people.";
        let channel_picture = Some("https://example.com/picture.jpg");
        let relay_url = UncheckedUrl::from_str("wss://relay.example.com");

        let event = create_channel(
            &signer,
            channel_id,
            channel_name,
            channel_description,
            channel_picture,
            Some(&relay_url).cloned().as_ref(),
        ).unwrap();

        assert_eq!(event.kind, EventKind::ChannelCreation);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, "");

        // Check tags
        let mut found_d_tag = false;
        let mut found_name_tag = false;
        let mut found_description_tag = false;
        let mut found_picture_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "name" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_name);
                found_name_tag = true;
            } else if tag.tagname() == "description" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_description);
                found_description_tag = true;
            } else if tag.tagname() == "picture" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_picture.unwrap());
                found_picture_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_name_tag);
        assert!(found_description_tag);
        assert!(found_picture_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_set_channel_metadata_event() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "my-channel-id";
        let channel_name = Some("Updated Channel Name");
        let channel_description = Some("Updated description.");
        let channel_picture = Some("https://example.com/new_picture.jpg");
        let relay_url = UncheckedUrl::from_str("wss://new-relay.example.com");

        let event = set_channel_metadata(
            &signer,
            channel_id,
            channel_name,
            channel_description,
            channel_picture,
            Some(&relay_url).cloned().as_ref(),
        ).unwrap();

        assert_eq!(event.kind, EventKind::ChannelMetadata);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, ""); // Metadata events typically have empty content

        // Check tags
        let mut found_d_tag = false;
        let mut found_name_tag = false;
        let mut found_description_tag = false;
        let mut found_picture_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "name" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_name.unwrap());
                found_name_tag = true;
            } else if tag.tagname() == "description" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_description.unwrap());
                found_description_tag = true;
            } else if tag.tagname() == "picture" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_picture.unwrap());
                found_picture_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_name_tag);
        assert!(found_description_tag);
        assert!(found_picture_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_set_channel_metadata_with_optional_args() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "another-channel";

        // Test with only required arguments
        let event_minimal = set_channel_metadata(
            &signer,
            channel_id,
            None, // name
            None, // description
            None, // picture
            None, // relay_url
        ).unwrap();

        assert_eq!(event_minimal.kind, EventKind::ChannelMetadata);
        assert_eq!(event_minimal.pubkey, signer.public_key());
        assert_eq!(event_minimal.content, "");
        // Should only contain the 'd' tag
        assert_eq!(event_minimal.tags.len(), 1);
        if event_minimal.tags[0].tagname() == "d" {
            let d = event_minimal.tags[0].parse_identifier().unwrap();
            assert_eq!(d, channel_id);
        } else {
            panic!("Expected 'd' tag for channel ID");
        }

        // Test with some optional arguments
        let channel_name = Some("Channel Name");
        let channel_picture = Some("https://example.com/pic.png");
        let event_partial = set_channel_metadata(
            &signer,
            channel_id,
            channel_name,
            None, // description
            channel_picture,
            None, // relay_url
        ).unwrap();

        assert_eq!(event_partial.kind, EventKind::ChannelMetadata);
        assert_eq!(event_partial.pubkey, signer.public_key());
        assert_eq!(event_partial.content, "");

        let mut found_d = false;
        let mut found_name = false;
        let mut found_picture = false;
        for tag in event_partial.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d = true;
            } else if tag.tagname() == "name" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_name.unwrap());
                found_name = true;
            } else if tag.tagname() == "picture" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_picture.unwrap());
                found_picture = true;
            }
        }
        assert!(found_d && found_name && found_picture);
    }

    #[test]
    fn test_create_channel_message_event() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "chat-channel";
        let message = "Hello, this is a chat message!";
        let reply_to_id = Some(Id::mock());
        let root_message_id = Some(Id::mock());
        let relay_url = UncheckedUrl::from_str("wss://chat-relay.example.com");

        let event = create_channel_message(
            &signer,
            channel_id,
            message,
            reply_to_id,
            root_message_id,
            Some(&relay_url).cloned().as_ref(),
        ).unwrap();

        assert_eq!(event.kind, EventKind::ChannelMessage);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, message);

        // Check tags
        let mut found_d_tag = false;
        let mut found_reply_e_tag = false;
        let mut found_root_e_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "e" {
                let (id, recommended_relay_url, marker) = tag.parse_event().unwrap();
                if marker.as_deref() == Some("reply") {
                    assert_eq!(id, reply_to_id.unwrap());
                    assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                    found_reply_e_tag = true;
                } else if marker.as_deref() == Some("root") {
                    assert_eq!(id, root_message_id.unwrap());
                    assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                    found_root_e_tag = true;
                }
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_reply_e_tag);
        assert!(found_root_e_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_hide_message_event() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "secret-channel";
        let message_id_to_hide = Id::mock();
        let reason = Some("spam");
        let relay_url = UncheckedUrl::from_str("wss://hide-relay.example.com");

        let event = hide_message(
            &signer,
            channel_id,
            &message_id_to_hide,
            reason,
            Some(&relay_url).cloned().as_ref(),
        ).unwrap();

        assert_eq!(event.kind, EventKind::ChannelHideMessage);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, "");

        // Check tags
        let mut found_d_tag = false;
        let mut found_e_tag = false;
        let mut found_p_tag = false;
        let mut found_reason_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "e" {
                let (id, recommended_relay_url, _) = tag.parse_event().unwrap();
                assert_eq!(id, message_id_to_hide);
                assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                found_e_tag = true;
            } else if tag.tagname() == "reason" && !tag.value().is_empty() {
                assert_eq!(tag.value(), reason.unwrap());
                found_reason_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_e_tag);
        assert!(found_reason_tag);
        assert!(found_relay_tag);

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "p" {
                let (pubkey, recommended_relay_url, petname) = tag.parse_pubkey().unwrap();
                assert!(pubkey.len() > 0);
                assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                assert!(petname.is_none()); // Mute user tag should not have petname
                if petname.is_none() { found_p_tag = false; }
                assert_eq!(petname.is_none(), !found_p_tag);
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            } else if tag.tagname() == "reason" && !tag.value().is_empty() {
                assert_eq!(tag.value(), reason.unwrap());
                found_reason_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(!found_p_tag);
        assert!(found_reason_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_parse_channel_creation() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "my-cool-channel";
        let channel_name = "My Cool Channel";
        let channel_description = "A channel for cool people.";
        let channel_picture = Some("https://example.com/picture.jpg");
        let relay_url = UncheckedUrl::from_str("wss://relay.example.com");

        let event = create_channel(
            &signer,
            channel_id,
            channel_name,
            channel_description,
            channel_picture,
            Some(&relay_url).cloned().as_ref(),
        ).unwrap();

        let parsed_event = parse_channel_creation(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.channel_name, Some(channel_name.to_string()));
        assert_eq!(parsed_event.channel_description, Some(channel_description.to_string()));
        assert_eq!(parsed_event.channel_picture, Some(channel_picture.unwrap().to_string()));
        assert_eq!(parsed_event.relay_url, Some(&relay_url).cloned());
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_channel_creation_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "minimal-channel";
        
        let event = create_channel(
            &signer,
            channel_id,
            "", // Empty name
            "", // Empty description
            None, // No picture
            None, // No relay URL
        ).unwrap();

        let parsed_event = parse_channel_creation(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.channel_name, None);
        assert_eq!(parsed_event.channel_description, None);
        assert_eq!(parsed_event.channel_picture, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_channel_message() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "chat-channel";
        let message = "Hello, this is a chat message!";
        let reply_to_id = Some(Id::mock());
        let root_message_id = Some(Id::mock());
        let relay_url = UncheckedUrl::from_str("wss://chat-relay.example.com");

        let event = create_channel_message(
            &signer,
            channel_id,
            message,
            reply_to_id,
            root_message_id,
            Some(&relay_url).cloned().as_ref(),
        ).unwrap();

        let parsed_event = parse_channel_message(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.message, message);
        assert_eq!(parsed_event.reply_to, reply_to_id);
        assert_eq!(parsed_event.root_message, root_message_id);
        assert_eq!(parsed_event.relay_url, Some(&relay_url).cloned());
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_channel_message_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "minimal-chat";
        let message = "Just a simple message";

        let event = create_channel_message(
            &signer,
            channel_id,
            message,
            None, // reply_to
            None, // root_message
            None, // relay_url
        ).unwrap();

        let parsed_event = parse_channel_message(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.message, message);
        assert_eq!(parsed_event.reply_to, None);
        assert_eq!(parsed_event.root_message, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_hide_message_event_duplicate() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "secret-channel";
        let message_id_to_hide = Id::mock();
        let reason = Some("spam");
        let relay_url = UncheckedUrl::from_str("wss://hide-relay.example.com");

        let event = hide_message(
            &signer,
            channel_id,
            &message_id_to_hide,
            reason,
            Some(&relay_url),
        ).unwrap();

        assert_eq!(event.kind, EventKind::ChannelHideMessage);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, "");

        // Check tags
        let mut found_d_tag = false;
        let mut found_e_tag = false;
        let mut found_reason_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "e" {
                let (id, recommended_relay_url, _) = tag.parse_event().unwrap();
                assert_eq!(id, message_id_to_hide);
                assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                found_e_tag = true;
            } else if tag.tagname() == "reason" && !tag.value().is_empty() {
                assert_eq!(tag.value(), reason.unwrap());
                found_reason_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_e_tag);
        assert!(found_reason_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_parse_hide_message() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "secret-channel";
        let message_id_to_hide = Id::mock();
        let reason = Some("spam");
        let relay_url = UncheckedUrl::from_str("wss://hide-relay.example.com");

        let event = hide_message(
            &signer,
            channel_id,
            &message_id_to_hide,
            reason,
            Some(&relay_url).cloned().as_ref(),
        ).unwrap();

        let parsed_event = parse_hide_message(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.message_id_to_hide, message_id_to_hide);
        assert_eq!(parsed_event.reason, reason.map(|r| r.to_string()));
        assert_eq!(parsed_event.relay_url, Some(&relay_url).cloned());
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_hide_message_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "minimal-hide-channel";
        let message_id_to_hide = Id::mock();

        let event = hide_message(
            &signer,
            channel_id,
            &message_id_to_hide,
            None, // reason
            None, // relay_url
        ).unwrap();

        let parsed_event = parse_hide_message(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.message_id_to_hide, message_id_to_hide);
        assert_eq!(parsed_event.reason, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_mute_user_event_duplicate() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "muted-channel";
        let user_pubkey = PublicKey::mock_deterministic();
        let reason = Some("trolling");
        let relay_url = UncheckedUrl::from_str("wss://mute-relay.example.com");

        let event = mute_user(
            &signer,
            channel_id,
            &user_pubkey,
            reason,
            Some(&relay_url),
        ).unwrap();

        assert_eq!(event.kind, EventKind::ChannelMuteUser);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, "");

        // Check tags
        let mut found_d_tag = false;
        let mut found_p_tag = false;
        let mut found_reason_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "p" {
                let (pubkey, recommended_relay_url, petname) = tag.parse_pubkey().unwrap();
                assert_eq!(pubkey, user_pubkey);
                assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                assert!(petname.is_none()); // Mute user tag should not have petname
                found_p_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            } else if tag.tagname() == "reason" && !tag.value().is_empty() {
                assert_eq!(tag.value(), reason.unwrap());
                found_reason_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_p_tag);
        assert!(found_reason_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_parse_mute_user_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "muted-channel";
        let user_pubkey = PublicKey::mock_deterministic();

        let event = mute_user(
            &signer,
            channel_id,
            &user_pubkey,
            None, // reason
            None, // relay_url
        ).unwrap();

        let parsed_event = parse_mute_user(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.user_pubkey, user_pubkey);
        assert_eq!(parsed_event.reason, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_set_channel_metadata() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "test-channel-id";
        let channel_name = Some("Test Channel Name");
        let channel_description = Some("This is a test channel description.");
        let channel_picture = Some("https://example.com/test_picture.jpg");
        let relay_url = UncheckedUrl::from_str("wss://test-relay.example.com");

        let event = set_channel_metadata(
            &signer,
            channel_id,
            channel_name,
            channel_description,
            channel_picture,
            Some(&relay_url).cloned().as_ref(),
        ).unwrap();

        let parsed_event = parse_set_channel_metadata(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.channel_name, channel_name.map(|s| s.to_string()));
        assert_eq!(parsed_event.channel_description, channel_description.map(|s| s.to_string()));
        assert_eq!(parsed_event.channel_picture, channel_picture.map(|s| s.to_string()));
        assert_eq!(parsed_event.relay_url, Some(&relay_url).cloned());
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_set_channel_metadata_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "minimal-channel-id";

        let event = set_channel_metadata(
            &signer,
            channel_id,
            None, // name
            None, // description
            None, // picture
            None, // relay_url
        ).unwrap();

        let parsed_event = parse_set_channel_metadata(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.channel_name, None);
        assert_eq!(parsed_event.channel_description, None);
        assert_eq!(parsed_event.channel_picture, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }
}
