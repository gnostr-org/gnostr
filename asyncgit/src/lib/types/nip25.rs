//! NIP-25: Reactions
//!
//! This NIP defines how users can express reactions to other events,
//! typically representing "likes", "dislikes", or other emoji-based feedback.
//!
//! https://github.com/nostr-protocol/nips/blob/master/25.md

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Event, EventKind, Id, PreEvent, PublicKey, Signature, Tag, Unixtime};

/// NIP-25 Reaction Event Kind
pub const REACTION_KIND: u32 = 7;
/// NIP-25 External Content Reaction Event Kind (optional, will focus on Kind 7
/// first)
pub const EXTERNAL_CONTENT_REACTION_KIND: u32 = 17;

/// Represents the content of a NIP-25 Reaction Event.
/// This is typically a single emoji or "+"/"-".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionContent {
    pub reaction: String,
}

/// Helper trait for NIP-25 reaction events.
pub trait NIP25Event {
    /// Extracts the ID of the event this reaction is responding to (from 'e'
    /// tag).
    fn reacted_event_id(&self) -> Option<Id>;

    /// Extracts the PublicKey of the author of the event this reaction is
    /// responding to (from 'p' tag).
    fn reacted_pubkey(&self) -> Option<PublicKey>;

    /// Extracts the original event coordinates from 'a' tag (for addressable
    /// events).
    // fn reacted_addressable_event_coords(&self) -> Option<Coordinate>; // Maybe later if
    // Coordinate is available

    /// Creates a NIP-25 Reaction event (Kind 7).
    fn new_reaction_event(
        public_key: PublicKey,
        reacted_event_id: Id,
        reacted_pubkey: Option<PublicKey>,
        reaction_content: String,
    ) -> Result<Event>;
}

impl NIP25Event for Event {
    fn reacted_event_id(&self) -> Option<Id> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "e" {
                Id::try_from_hex_string(tag.0[1].as_str()).ok()
            } else {
                None
            }
        })
    }

    fn reacted_pubkey(&self) -> Option<PublicKey> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "p" {
                PublicKey::try_from_hex_string(tag.0[1].as_str(), true).ok() // true to verify
            } else {
                None
            }
        })
    }

    fn new_reaction_event(
        public_key: PublicKey,
        reacted_event_id: Id,
        reacted_pubkey: Option<PublicKey>,
        reaction_content: String,
    ) -> Result<Event> {
        let content = ReactionContent {
            reaction: reaction_content.clone(),
        }
        .reaction;
        let mut tags: Vec<Tag> = vec![Tag::new(&["e", &reacted_event_id.as_hex_string()])];

        if let Some(pk) = reacted_pubkey {
            tags.push(Tag::new(&["p", &pk.as_hex_string()]));
        }
        // NIP-10 reply markers: reply, root
        // NIP-25 doesn't explicitly mention markers for e and p tags,
        // but it's good practice to include them for context if available
        // For a simple reaction, a "reply" marker might be appropriate for 'e' tag

        // Create PreEvent
        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Reaction, // Use the specific EventKind for reactions
            tags: tags.clone(),
            content: content.clone(),
        };

        let id = pre_event.hash().unwrap();

        Ok(Event {
            id,
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Reaction, // Use the specific EventKind for reactions
            tags,
            content,
            sig: Signature::zeroes(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Id, PublicKey, Signature};

    #[test]
    fn test_create_reaction_event() {
        let public_key = PublicKey::mock();
        let reacted_event_id = Id::mock();
        let reacted_pubkey = Some(PublicKey::mock_deterministic());
        let reaction_content = "+".to_string();

        let event = Event::new_reaction_event(
            public_key,
            reacted_event_id,
            reacted_pubkey,
            reaction_content.clone(),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::Reaction);
        assert_eq!(event.content, reaction_content);
        assert_eq!(event.reacted_event_id(), Some(reacted_event_id));
        assert_eq!(event.reacted_pubkey(), reacted_pubkey);

        // Test with custom emoji reaction
        let custom_reaction = "😂".to_string();
        let event_emoji = Event::new_reaction_event(
            public_key,
            reacted_event_id,
            None, // No reacted pubkey
            custom_reaction.clone(),
        )
        .unwrap();

        assert_eq!(event_emoji.kind, EventKind::Reaction);
        assert_eq!(event_emoji.content, custom_reaction);
        assert_eq!(event_emoji.reacted_event_id(), Some(reacted_event_id));
        assert_eq!(event_emoji.reacted_pubkey(), None);
    }

    #[test]
    fn test_reacted_event_id_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::Reaction,
            tags: vec![], // No 'e' tag
            content: "+".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.reacted_event_id(), None);
    }

    #[test]
    fn test_reacted_pubkey_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::Reaction,
            tags: vec![Tag::new(&["e", &Id::mock().as_hex_string()])], // No 'p' tag
            content: "+".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.reacted_pubkey(), None);
    }
}
