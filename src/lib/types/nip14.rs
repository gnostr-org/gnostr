//! NIP-14: Subject Tag in Text Events
//!
//! This NIP defines a standard `subject` tag for kind 1 (text note) events,
//! allowing clients to display messages in threaded lists with a clear subject.
//!
//! https://github.com/nostr-protocol/nips/blob/master/14.md

use anyhow::Result;

use crate::types::{Event, Tag};

/// The name of the subject tag.
pub const SUBJECT_TAG_NAME: &str = "subject";

/// Helper trait for NIP-14 subject tags on Event types.
pub trait NIP14Event {
    /// Extracts the subject string from the event's tags.
    /// Returns `None` if the "subject" tag is not found.
    fn subject(&self) -> Option<&str>;

    /// Adds a "subject" tag to the event with the given subject string.
    /// If a "subject" tag already exists, it will be replaced.
    fn add_subject_tag(&mut self, subject: String);

    /// Creates a "subject" tag with the given subject string.
    fn create_subject_tag(subject: String) -> Tag;
}

impl NIP14Event for Event {
    fn subject(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == SUBJECT_TAG_NAME {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn add_subject_tag(&mut self, subject: String) {
        // Remove existing subject tags to ensure only one is present
        self.tags
            .retain(|tag| tag.0.get(0) != Some(&SUBJECT_TAG_NAME.to_string()));
        self.tags.push(Self::create_subject_tag(subject));
    }

    fn create_subject_tag(subject: String) -> Tag {
        Tag::new(&[SUBJECT_TAG_NAME, &subject])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature, Unixtime};

    #[test]
    fn test_create_subject_tag() {
        let subject_str = "My Nostr Subject";
        let tag = Event::create_subject_tag(subject_str.to_string());
        assert_eq!(tag.0, vec!["subject", "My Nostr Subject"]);
    }

    #[test]
    fn test_subject_found() {
        let subject_str = "Nostr Dev Talk";
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_subject_tag(subject_str.to_string()),
            ],
            content: "This is a text note.".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(
            event.subject().map(|s| s.to_string()),
            Some(subject_str.to_string())
        );
    }

    #[test]
    fn test_subject_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])],
            content: "This is a text note without a subject.".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(event.subject(), None);
    }

    #[test]
    fn test_add_subject_tag() {
        let initial_subject = "Initial discussion";
        let new_subject = "Revised discussion topic";
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_subject_tag(initial_subject.to_string()),
            ],
            content: "Some content".to_string(),
            sig: Signature::zeroes(),
        };

        // Add a new subject tag
        event.add_subject_tag(new_subject.to_string());
        assert_eq!(
            event.subject().map(|s| s.to_string()),
            Some(new_subject.to_string())
        );
        // Ensure only one subject tag exists
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&SUBJECT_TAG_NAME.to_string()))
                .count(),
            1
        );

        // Add another subject tag, it should replace the existing one
        let even_newer_subject = "Final topic of conversation";
        event.add_subject_tag(even_newer_subject.to_string());
        assert_eq!(
            event.subject().map(|s| s.to_string()),
            Some(even_newer_subject.to_string())
        );
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&SUBJECT_TAG_NAME.to_string()))
                .count(),
            1
        );
    }
}
