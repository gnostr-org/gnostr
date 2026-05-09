//! NIP-36: Sensitive Content
//!
//! This NIP introduces a `content-warning` tag to indicate that an event's
//! content should be hidden by clients until the reader explicitly approves its
//! display.
//!
//! https://github.com/nostr-protocol/nips/blob/master/36.md

use anyhow::Result;

use crate::types::{Event, Tag};

/// The name of the content-warning tag.
pub const CONTENT_WARNING_TAG_NAME: &str = "content-warning";

/// Helper trait for NIP-36 content warning tags on Event types.
pub trait NIP36Event {
    /// Extracts the optional reason for the content warning from the event's
    /// tags. Returns `None` if the "content-warning" tag is not found or
    /// has no reason.
    fn content_warning_reason(&self) -> Option<&str>;

    /// Adds a "content-warning" tag to the event with an optional reason.
    /// If a "content-warning" tag already exists, it will be replaced.
    fn add_content_warning_tag(&mut self, reason: Option<String>);

    /// Creates a "content-warning" tag with an optional reason.
    fn create_content_warning_tag(reason: Option<String>) -> Tag;
}

impl NIP36Event for Event {
    fn content_warning_reason(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() >= 1 && tag.0[0] == CONTENT_WARNING_TAG_NAME {
                tag.0.get(1).map(|s| s.as_str()) // Reason is at index 1 if present
            } else {
                None
            }
        })
    }

    fn add_content_warning_tag(&mut self, reason: Option<String>) {
        // Remove existing content-warning tags to ensure only one is present
        self.tags
            .retain(|tag| tag.0.get(0) != Some(&CONTENT_WARNING_TAG_NAME.to_string()));
        self.tags.push(Self::create_content_warning_tag(reason));
    }

    fn create_content_warning_tag(reason: Option<String>) -> Tag {
        let mut tag_elements = vec![CONTENT_WARNING_TAG_NAME.to_string()];
        if let Some(r) = reason {
            tag_elements.push(r);
        }
        Tag::new(
            tag_elements
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature, Unixtime};

    // Helper to create a dummy event for testing
    fn create_dummy_event_with_warning(reason: Option<&str>) -> Event {
        let mut tags = Vec::new();
        tags.push(Event::create_content_warning_tag(
            reason.map(|s| s.to_string()),
        ));

        Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "This content is potentially sensitive.".to_string(),
            sig: Signature::zeroes(),
        }
    }

    #[test]
    fn test_create_content_warning_tag_with_reason() {
        let reason = "sexual content";
        let tag = Event::create_content_warning_tag(Some(reason.to_string()));
        assert_eq!(tag.0, vec![CONTENT_WARNING_TAG_NAME, reason]);
    }

    #[test]
    fn test_create_content_warning_tag_no_reason() {
        let tag = Event::create_content_warning_tag(None);
        assert_eq!(tag.0, vec![CONTENT_WARNING_TAG_NAME]);
    }

    #[test]
    fn test_content_warning_reason_found() {
        let reason = "hate speech";
        let event = create_dummy_event_with_warning(Some(reason));
        assert_eq!(event.content_warning_reason(), Some(reason));
    }

    #[test]
    fn test_content_warning_reason_not_found() {
        let event = create_dummy_event_with_warning(None);
        assert_eq!(event.content_warning_reason(), None);
    }

    #[test]
    fn test_content_warning_tag_not_present() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])], // No content-warning tag
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.content_warning_reason(), None);
    }

    #[test]
    fn test_add_content_warning_tag() {
        let reason1 = "graphic violence";
        let reason2 = "spoiler";
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_content_warning_tag(Some(reason1.to_string())),
            ],
            content: "Some content".to_string(),
            sig: Signature::zeroes(),
        };

        // Add a new content-warning tag, it should replace the existing one
        event.add_content_warning_tag(Some(reason2.to_string()));
        assert_eq!(event.content_warning_reason(), Some(reason2));
        // Ensure only one content-warning tag exists
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&CONTENT_WARNING_TAG_NAME.to_string()))
                .count(),
            1
        );

        // Add a content-warning tag without a reason
        event.add_content_warning_tag(None);
        assert_eq!(event.content_warning_reason(), None);
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&CONTENT_WARNING_TAG_NAME.to_string()))
                .count(),
            1
        );
    }
}
