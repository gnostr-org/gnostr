//! NIP-32: Labeling
//!
//! This NIP defines a system for labeling Nostr events using two new indexable tags:
//! `L` for label namespaces and `l` for labels. It also defines a new event kind (1985)
//! for attaching these labels to existing events.
//!
//! https://github.com/nostr-protocol/nips/blob/master/32.md

use crate::types::{Event, Id, PublicKey, Tag, Unixtime, EventKind, PreEvent, Signature};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

/// NIP-32 Label Event Kind
pub const LABEL_EVENT_KIND: u32 = 1985;
/// The name of the label tag.
pub const LABEL_TAG_NAME: &str = "l";
/// The name of the label namespace tag.
pub const LABEL_NAMESPACE_TAG_NAME: &str = "L";

/// Represents a label, composed of a value and an optional namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub value: String,
    pub namespace: Option<String>,
}

/// Helper trait for NIP-32 labeling events.
pub trait NIP32Event {
    /// Extracts all labels defined in "l" and "L" tags from the event.
    /// Returns a vector of `Label` structs. Malformed tags are ignored.
    fn extract_labels(&self) -> Vec<Label>;

    /// Adds an "l" tag (label value) to the event.
    /// Optionally includes a "mark" if the label is associated with a namespace.
    fn add_label_tag(&mut self, label_value: String, mark: Option<String>);

    /// Adds an "L" tag (label namespace) to the event.
    fn add_label_namespace_tag(&mut self, namespace: String);

    /// Creates an "l" tag with the given label value and optional mark.
    fn create_label_tag(label_value: String, mark: Option<String>) -> Tag;

    /// Creates an "L" tag with the given namespace.
    fn create_label_namespace_tag(namespace: String) -> Tag;

    /// Creates a NIP-32 Label event (Kind 1985) to label a target event.
    fn new_label_event(
        public_key: PublicKey,
        label_value: String,
        namespace: Option<String>,
        target_event_id: Id, // For simplicity, initially target an Event ID
        content: Option<String>,
    ) -> Result<Event>;
}

impl NIP32Event for Event {
    fn extract_labels(&self) -> Vec<Label> {
        let mut labels = Vec::new();
        let mut namespace_map: std::collections::HashMap<String, String> = std::collections::HashMap::new();

        // First, process 'L' tags to build the namespace map
        for tag in &self.tags {
            if tag.0.len() == 2 && tag.0[0] == LABEL_NAMESPACE_TAG_NAME {
                namespace_map.insert(tag.0[1].clone(), tag.0[1].clone()); // Mark is usually same as namespace for 'L' tags
            }
        }

        // Then, process 'l' tags
        for tag in &self.tags {
            if tag.0.len() >= 2 && tag.0[0] == LABEL_TAG_NAME {
                let label_value = tag.0[1].clone();
                let mark = tag.0.get(2).map(|s| s.clone());
                
                let namespace = if let Some(m) = mark {
                    // If a mark is present, check if it corresponds to a namespace
                    namespace_map.get(&m).map(|s| s.clone())
                } else {
                    // If no mark, NIP-32 implies "ugc" if omitted.
                    Some("ugc".to_string()) // TODO: NIP-32 says "ugc" is implied if omitted, check for existing "L" tag
                };
                
                labels.push(Label { value: label_value, namespace });
            }
        }
        labels
    }

    fn add_label_tag(&mut self, label_value: String, mark: Option<String>) {
        let mut tag_elements = vec![LABEL_TAG_NAME.to_string(), label_value];
        if let Some(m) = mark {
            tag_elements.push(m);
        }
        self.tags.push(Tag(tag_elements));
    }

    fn add_label_namespace_tag(&mut self, namespace: String) {
        self.tags.push(Tag::new(&[LABEL_NAMESPACE_TAG_NAME, &namespace]));
    }

    fn create_label_tag(label_value: String, mark: Option<String>) -> Tag {
        let mut tag_elements = vec![LABEL_TAG_NAME.to_string(), label_value];
        if let Some(m) = mark {
            tag_elements.push(m);
        }
        Tag(tag_elements)
    }

    fn create_label_namespace_tag(namespace: String) -> Tag {
        Tag::new(&[LABEL_NAMESPACE_TAG_NAME, &namespace])
    }

    fn new_label_event(
        public_key: PublicKey,
        label_value: String,
        namespace: Option<String>,
        target_event_id: Id,
        content_description: Option<String>,
    ) -> Result<Event> {
        let mut tags: Vec<Tag> = vec![
            Tag::new(&["e", &target_event_id.as_hex_string()]), // Target the event ID
            Self::create_label_tag(label_value, namespace.clone().or(Some("ugc".to_string()))), // Default mark to "ugc"
        ];
        if let Some(ns) = namespace {
            tags.push(Self::create_label_namespace_tag(ns));
        }

        let content = content_description.unwrap_or_default();

        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Label, // Use the specific EventKind for NIP-32 labels
            tags: tags.clone(),
            content: content.clone(),
        };

        let id = pre_event.hash().unwrap();

        Ok(Event {
            id,
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Label,
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
    fn test_create_label_tag() {
        let tag = Event::create_label_tag("permies".to_string(), None);
        assert_eq!(tag.0, vec![LABEL_TAG_NAME, "permies"]);

        let tag_with_mark = Event::create_label_tag("humor".to_string(), Some("ugc".to_string()));
        assert_eq!(tag_with_mark.0, vec![LABEL_TAG_NAME, "humor", "ugc"]);
    }

    #[test]
    fn test_create_label_namespace_tag() {
        let tag = Event::create_label_namespace_tag("ugc".to_string());
        assert_eq!(tag.0, vec![LABEL_NAMESPACE_TAG_NAME, "ugc"]);
    }

    #[test]
    fn test_extract_labels() {
        let mut tags = Vec::new();
        tags.push(Event::create_label_namespace_tag("ISO-3166-2".to_string()));
        tags.push(Event::create_label_tag("IT-MI".to_string(), Some("ISO-3166-2".to_string())));
        tags.push(Event::create_label_tag("bug".to_string(), Some("ugc".to_string())));
        tags.push(Tag::new(&["e", &Id::mock().as_hex_string()])); // Non-label tag

        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "An event with labels".to_string(),
            sig: Signature::zeroes(),
        };

        let labels = event.extract_labels();
        assert_eq!(labels.len(), 2);
        assert!(labels.contains(&Label { value: "IT-MI".to_string(), namespace: Some("ISO-3166-2".to_string()) }));
        assert!(labels.contains(&Label { value: "bug".to_string(), namespace: Some("ugc".to_string()) }));
    }

    #[test]
    fn test_extract_labels_no_namespace_tag() {
        let mut tags = Vec::new();
        tags.push(Event::create_label_tag("feature".to_string(), None)); // No mark
        tags.push(Tag::new(&["p", &PublicKey::mock().as_hex_string()]));

        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "Another event with labels".to_string(),
            sig: Signature::zeroes(),
        };

        let labels = event.extract_labels();
        assert_eq!(labels.len(), 1);
        // NIP-32 implies "ugc" mark if omitted in the 'l' tag.
        assert!(labels.contains(&Label { value: "feature".to_string(), namespace: Some("ugc".to_string()) }));
    }

    #[test]
    fn test_new_label_event() {
        let public_key = PublicKey::mock();
        let target_event_id = Id::mock();
        let label_value = "approved".to_string();
        let namespace = Some("moderation".to_string());
        let content_description = Some("This event has been approved by moderators.".to_string());

        let event = Event::new_label_event(
            public_key,
            label_value.clone(),
            namespace.clone(),
            target_event_id,
            content_description.clone(),
        ).unwrap();

        assert_eq!(event.kind, EventKind::Label);
        assert_eq!(event.content, content_description.unwrap());
        assert_eq!(event.extract_labels().len(), 1);
        assert_eq!(event.extract_labels()[0].value, label_value);
        assert_eq!(event.extract_labels()[0].namespace, namespace);
        assert!(event.tags.iter().any(|tag| tag.0.len() == 2 && tag.0[0] == "e" && tag.0[1] == target_event_id.as_hex_string()));
        assert!(event.tags.iter().any(|tag| tag.0.len() == 2 && tag.0[0] == LABEL_NAMESPACE_TAG_NAME && tag.0[1] == namespace.unwrap()));
    }
}