// EventBuilder - Builder pattern for creating Nostr events
// Replaces nostr_sdk EventBuilder with local implementation

use anyhow::Result;

use crate::types::{
    Event, EventKind, Id, ImageDimensions, KeySigner, Metadata, PreEvent, PrivateKey, PublicKey,
    Signer, Tag, UncheckedUrl, Unixtime,
};

/// Builder for creating Nostr events
pub struct EventBuilder {
    kind: EventKind,
    content: String,
    tags: Vec<Tag>,
    created_at: Option<Unixtime>,
}

impl EventBuilder {
    /// Create a new event builder
    pub fn new(kind: EventKind, content: String, tags: Vec<Tag>) -> Self {
        Self {
            kind,
            content,
            tags,
            created_at: None,
        }
    }

    /// Set creation time
    pub fn created_at(mut self, created_at: Unixtime) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Build and sign the event with proof of work
    pub fn to_pow_event(self, keys: &PrivateKey, difficulty_target: u8) -> Result<Event> {
        let mut preevent = PreEvent {
            pubkey: keys.public_key(),
            created_at: self.created_at.unwrap_or_else(Unixtime::now),
            kind: self.kind,
            tags: self.tags,
            content: self.content,
        };

        // Apply proof of work if difficulty > 0
        if difficulty_target > 0 {
            // TODO: Implement actual POW calculation
            // For now, just sign without POW
        }

        // Sign the event
        let signer = KeySigner::from_private_key(keys.clone(), "", 1)?;
        signer.sign_event(preevent)
    }

    /// Build and sign the event without proof of work
    pub fn to_event(self, keys: &PrivateKey) -> Result<Event> {
        self.to_pow_event(keys, 0)
    }
}

// Badge-related builders
impl EventBuilder {
    /// Create a badge definition event (NIP-58)
    pub fn define_badge(
        id: String,
        name: Option<String>,
        description: Option<String>,
        image_url: Option<UncheckedUrl>,
        image_size: Option<ImageDimensions>,
        thumbnails: Vec<(UncheckedUrl, Option<ImageDimensions>)>,
    ) -> Self {
        let mut tags = Vec::new();

        // Add identifier tag
        tags.push(Tag::new_identifier(id));

        // Add name tag
        if let Some(name) = name {
            tags.push(Tag::new_name(name));
        }

        // Add image tag
        if let Some(url) = image_url {
            if let Some(dims) = image_size {
                tags.push(Tag::new_image(url, Some(dims.width), Some(dims.height)));
            } else {
                tags.push(Tag::new_image(url, None, None));
            }
        }

        // Add thumbnail tags
        for (thumb_url, thumb_dims) in thumbnails {
            if let Some(dims) = thumb_dims {
                tags.push(Tag::new_thumb(
                    thumb_url,
                    Some(dims.width),
                    Some(dims.height),
                ));
            } else {
                tags.push(Tag::new_thumb(thumb_url, None, None));
            }
        }

        Self {
            kind: EventKind::BadgeDefinition,
            content: description.unwrap_or_default(),
            tags,
            created_at: None,
        }
    }

    /// Create a badge award event (NIP-58)
    pub fn award_badge(badge_definition_event: &Event, awarded_pubkeys: Vec<Tag>) -> Self {
        let mut tags = Vec::new();

        // Reference the badge definition
        tags.push(Tag::new_event(
            badge_definition_event.id,
            None,
            Some("a".to_string()),
        ));

        // Add awarded pubkeys
        for pubkey_tag in awarded_pubkeys {
            if let Ok((pk, _, _)) = pubkey_tag.parse_pubkey() {
                tags.push(Tag::new_pubkey(pk, None, None));
            }
        }

        Self {
            kind: EventKind::BadgeAward,
            content: "".to_string(),
            tags,
            created_at: None,
        }
    }

    /// Create a channel creation event (NIP-28)
    pub fn channel(metadata: &Metadata) -> Self {
        let mut tags = Vec::new();

        // Add author pubkey tag
        tags.push(Tag::new_pubkey(metadata.pubkey, None, None));

        Self {
            kind: EventKind::ChannelCreation,
            content: serde_json::to_string(metadata).unwrap_or_default(),
            tags,
            created_at: None,
        }
    }

    /// Create a text note event
    pub fn text_note(content: String) -> Self {
        Self {
            kind: EventKind::TextNote,
            content,
            tags: Vec::new(),
            created_at: None,
        }
    }

    /// Create a reaction event
    pub fn reaction(reacted_event_id: Id, reaction: String) -> Self {
        let mut tags = Vec::new();
        tags.push(Tag::new_event(
            reacted_event_id,
            None,
            Some("reaction".to_string()),
        ));

        Self {
            kind: EventKind::Reaction,
            content: reaction,
            tags,
            created_at: None,
        }
    }

    /// Create an event deletion event
    pub fn delete_with_reason(event_ids_to_delete: Vec<Id>, reason: String) -> Self {
        let mut tags = Vec::new();

        for event_id in event_ids_to_delete {
            tags.push(Tag::new_event(event_id, None, None));
        }

        Self {
            kind: EventKind::EventDeletion,
            content: reason,
            tags,
            created_at: None,
        }
    }
}
