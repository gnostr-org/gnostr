//! NIP-40: Expiration Timestamp
//!
//! This NIP defines an `expiration` tag that can be added to Nostr events
//! to indicate a Unix timestamp (in seconds) at which the event should be
//! considered expired.
//!
//! https://github.com/nostr-protocol/nips/blob/master/40.md

use anyhow::Result;

use crate::types::{Event, Tag, Unixtime};

/// The name of the expiration tag.
pub const EXPIRATION_TAG_NAME: &str = "expiration";

/// Helper trait for NIP-40 expiration tags on Event types.
pub trait NIP40Event {
    /// Extracts the expiration timestamp from the event's tags.
    /// Returns `None` if the "expiration" tag is not found or is invalid.
    fn expiration_time(&self) -> Option<Unixtime>;

    /// Adds an "expiration" tag to the event with the given Unix timestamp.
    /// If an "expiration" tag already exists, it will be replaced.
    fn add_expiration_tag(&mut self, expiry_time: Unixtime);

    /// Creates an "expiration" tag with the given Unix timestamp.
    fn create_expiration_tag(expiry_time: Unixtime) -> Tag;
}

impl NIP40Event for Event {
    fn expiration_time(&self) -> Option<Unixtime> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == EXPIRATION_TAG_NAME {
                tag.0[1].parse::<i64>().ok().map(Unixtime::from)
            } else {
                None
            }
        })
    }

    fn add_expiration_tag(&mut self, expiry_time: Unixtime) {
        // Remove existing expiration tags to ensure only one is present
        self.tags
            .retain(|tag| tag.0.get(0) != Some(&EXPIRATION_TAG_NAME.to_string()));
        self.tags.push(Self::create_expiration_tag(expiry_time));
    }

    fn create_expiration_tag(expiry_time: Unixtime) -> Tag {
        Tag::new(&[EXPIRATION_TAG_NAME, &expiry_time.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature};

    #[test]
    fn test_create_expiration_tag() {
        let expiry = Unixtime::from(1678886400); // Example Unix timestamp
        let tag = Event::create_expiration_tag(expiry);
        assert_eq!(tag.0, vec!["expiration", "1678886400"]);
    }

    #[test]
    fn test_expiration_time_found() {
        let expiry = Unixtime::from(1678886400);
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_expiration_tag(expiry),
            ],
            content: "Test event".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(event.expiration_time(), Some(expiry));
    }

    #[test]
    fn test_expiration_time_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])],
            content: "Test event".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(event.expiration_time(), None);
    }

    #[test]
    fn test_expiration_time_invalid_format() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["expiration", "not_a_number"])],
            content: "Test event".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(event.expiration_time(), None);
    }

    #[test]
    fn test_add_expiration_tag() {
        let initial_expiry = Unixtime::from(1600000000);
        let new_expiry = Unixtime::from(1700000000);
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_expiration_tag(initial_expiry),
            ],
            content: "Test event".to_string(),
            sig: Signature::zeroes(),
        };

        // Add a new expiration tag
        event.add_expiration_tag(new_expiry);
        assert_eq!(event.expiration_time(), Some(new_expiry));
        // Ensure only one expiration tag exists
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&EXPIRATION_TAG_NAME.to_string()))
                .count(),
            1
        );

        // Add another expiration tag, it should replace the existing one
        let even_newer_expiry = Unixtime::from(1800000000);
        event.add_expiration_tag(even_newer_expiry);
        assert_eq!(event.expiration_time(), Some(even_newer_expiry));
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&EXPIRATION_TAG_NAME.to_string()))
                .count(),
            1
        );
    }
}
