//! NIP-30: Custom Emoji
//!
//! This NIP defines how to include custom emojis in Nostr events using a
//! special "emoji" tag. These emojis can be used in various event kinds like
//! kind 0, 1, 7, and 30315.
//!
//! https://github.com/nostr-protocol/nips/blob/master/30.md

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::types::{Event, Tag};

/// The name of the custom emoji tag.
pub const CUSTOM_EMOJI_TAG_NAME: &str = "emoji";

/// Represents a custom emoji, encapsulating its shortcode and image URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emoji {
    pub shortcode: String,
    pub image_url: String,
}

/// Helper trait for NIP-30 custom emoji tags on Event types.
pub trait NIP30Event {
    /// Extracts all custom emojis defined in "emoji" tags from the event.
    /// Returns a vector of `Emoji` structs. Invalid or malformed tags are
    /// ignored.
    fn extract_custom_emojis(&self) -> Vec<Emoji>;

    /// Creates an "emoji" tag with the given shortcode and image URL.
    fn create_custom_emoji_tag(shortcode: String, image_url: String) -> Tag;
}

impl NIP30Event for Event {
    fn extract_custom_emojis(&self) -> Vec<Emoji> {
        self.tags
            .iter()
            .filter(|tag| tag.0.len() == 3 && tag.0[0] == CUSTOM_EMOJI_TAG_NAME)
            .filter_map(|tag| {
                Some(Emoji {
                    shortcode: tag.0[1].clone(),
                    image_url: tag.0[2].clone(),
                })
            })
            .collect()
    }

    fn create_custom_emoji_tag(shortcode: String, image_url: String) -> Tag {
        Tag::new(&[CUSTOM_EMOJI_TAG_NAME, &shortcode, &image_url])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature, Unixtime};

    // Helper to create a dummy event for testing
    fn create_dummy_event_with_emojis(emojis: Vec<Emoji>) -> Event {
        let mut tags = Vec::new();
        for emoji in emojis {
            tags.push(Event::create_custom_emoji_tag(
                emoji.shortcode,
                emoji.image_url,
            ));
        }

        Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "Hello :wave: world :globe:".to_string(),
            sig: Signature::zeroes(),
        }
    }

    #[test]
    fn test_create_custom_emoji_tag() {
        let shortcode = "wave".to_string();
        let image_url = "https://example.com/wave.png".to_string();
        let tag = Event::create_custom_emoji_tag(shortcode.clone(), image_url.clone());
        assert_eq!(
            tag.0,
            vec![
                CUSTOM_EMOJI_TAG_NAME.to_string(),
                shortcode.clone(),
                image_url.clone()
            ]
        );
    }

    #[test]
    fn test_extract_custom_emojis() {
        let emoji1 = Emoji {
            shortcode: "wave".to_string(),
            image_url: "https://example.com/wave.png".to_string(),
        };
        let emoji2 = Emoji {
            shortcode: "globe".to_string(),
            image_url: "https://example.com/globe.png".to_string(),
        };
        let emojis = vec![emoji1.clone(), emoji2.clone()];
        let event = create_dummy_event_with_emojis(emojis.clone());

        let extracted_emojis = event.extract_custom_emojis();
        assert_eq!(extracted_emojis, emojis);
    }

    #[test]
    fn test_extract_custom_emojis_no_tags() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])], // No emoji tags
            content: "Hello world.".to_string(),
            sig: Signature::zeroes(),
        };

        let extracted_emojis = event.extract_custom_emojis();
        assert!(extracted_emojis.is_empty());
    }

    #[test]
    fn test_extract_custom_emojis_malformed_tag() {
        let emoji1 = Emoji {
            shortcode: "wave".to_string(),
            image_url: "https://example.com/wave.png".to_string(),
        };
        let mut tags = vec![Event::create_custom_emoji_tag(
            emoji1.shortcode.clone(),
            emoji1.image_url.clone(),
        )];
        // Malformed tag (missing image_url)
        tags.push(Tag::new(&[CUSTOM_EMOJI_TAG_NAME, "malformed"]));
        // Malformed tag (wrong name)
        tags.push(Tag::new(&["wrong_tag", "short", "url"]));

        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "Hello :wave: world :malformed:".to_string(),
            sig: Signature::zeroes(),
        };

        let extracted_emojis = event.extract_custom_emojis();
        assert_eq!(extracted_emojis, vec![emoji1]); // Only the valid emoji should be extracted
    }
}
