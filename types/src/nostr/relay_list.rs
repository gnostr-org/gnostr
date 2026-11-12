use std::collections::HashMap;

use crate::nostr::{Event, RelayUrl, Tag};

/// Relay Usage
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RelayListUsage {
    /// The relay is used as an inbox (called 'read' in kind-10002)
    Inbox,

    /// The relay is used as an outbox (called 'write' in kind-10002)
    Outbox,

    /// The relay is used both as an inbox and an outbox
    #[default]
    Both,
}

impl RelayListUsage {
    /// A string marker used in a kind-10002 RelayList event for the variant
    pub fn marker(&self) -> Option<&'static str> {
        match self {
            RelayListUsage::Inbox => Some("read"),
            RelayListUsage::Outbox => Some("write"),
            RelayListUsage::Both => None,
        }
    }
}

/// A relay list, indicating usage for each relay, which can be used to
/// represent the data found in a kind 10002 RelayListMetadata event.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RelayList(pub HashMap<RelayUrl, RelayListUsage>);

impl RelayList {
    /// Parse a kind-10002 RelayList event into a RelayList
    ///
    /// This does not check the event kind, that is left up to the caller.
    pub fn from_event(event: &Event) -> RelayList {
        let mut relay_list: RelayList = Default::default();

        for tag in event.tags.iter() {
            if let Ok((uurl, optmarker)) = tag.parse_relay() {
                if let Ok(relay_url) = RelayUrl::try_from_unchecked_url(&uurl) {
                    if let Some(m) = optmarker {
                        match &*m.trim().to_lowercase() {
                            "read" => {
                                let _ = relay_list.0.insert(relay_url, RelayListUsage::Inbox);
                            }
                            "write" => {
                                let _ = relay_list.0.insert(relay_url, RelayListUsage::Outbox);
                            }
                            _ => {} // ignore unknown marker
                        }
                    } else {
                        let _ = relay_list.0.insert(relay_url, RelayListUsage::Both);
                    }
                }
            }
        }

        relay_list
    }

    /// Create a `Vec<Tag>` appropriate for forming a kind-10002 RelayList event
    pub fn to_event_tags(&self) -> Vec<Tag> {
        let mut tags: Vec<Tag> = Vec::new();
        for (relay_url, usage) in self.0.iter() {
            tags.push(Tag::new_relay(
                relay_url.to_unchecked_url(),
                usage.marker().map(|s| s.to_owned()),
            ));
        }
        tags
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nostr::{Event, UncheckedUrl};

    fn relay_tag(url: &str, marker: Option<&str>) -> Tag {
        Tag::new_relay(UncheckedUrl(url.to_string()), marker.map(|m| m.to_string()))
    }

    #[test]
    fn relay_list_usage_marker_mapping() {
        // Arrange / Act / Assert
        assert_eq!(RelayListUsage::Inbox.marker(), Some("read"));
        assert_eq!(RelayListUsage::Outbox.marker(), Some("write"));
        assert_eq!(RelayListUsage::Both.marker(), None);
    }

    #[test]
    fn from_event_parses_read_write_and_both() {
        // Arrange: a kind-10002-shaped event with all three marker shapes
        let mut event = Event::mock();
        event.tags = vec![
            relay_tag("wss://inbox.example.com", Some("read")),
            relay_tag("wss://outbox.example.com", Some("write")),
            relay_tag("wss://both.example.com", None),
        ];

        // Act
        let list = RelayList::from_event(&event);

        // Assert
        assert_eq!(list.0.len(), 3);
        let inbox = RelayUrl::try_from_str("wss://inbox.example.com").unwrap();
        let outbox = RelayUrl::try_from_str("wss://outbox.example.com").unwrap();
        let both = RelayUrl::try_from_str("wss://both.example.com").unwrap();
        assert_eq!(list.0.get(&inbox), Some(&RelayListUsage::Inbox));
        assert_eq!(list.0.get(&outbox), Some(&RelayListUsage::Outbox));
        assert_eq!(list.0.get(&both), Some(&RelayListUsage::Both));
    }

    #[test]
    fn from_event_ignores_unknown_markers_and_invalid_urls() {
        // Arrange
        let mut event = Event::mock();
        event.tags = vec![
            relay_tag("wss://valid.example.com", Some("bogus-marker")),
            relay_tag("not a url at all", Some("read")),
        ];

        // Act
        let list = RelayList::from_event(&event);

        // Assert: unknown marker entries and invalid URLs are dropped
        assert!(list.0.is_empty());
    }

    #[test]
    fn from_event_marker_matching_is_case_insensitive_and_trimmed() {
        // Arrange
        let mut event = Event::mock();
        event.tags = vec![relay_tag("wss://relay.example.com", Some("  READ  "))];

        // Act
        let list = RelayList::from_event(&event);

        // Assert
        let url = RelayUrl::try_from_str("wss://relay.example.com").unwrap();
        assert_eq!(list.0.get(&url), Some(&RelayListUsage::Inbox));
    }

    #[test]
    fn to_event_tags_round_trips_through_from_event() {
        // Arrange
        let inbox = RelayUrl::try_from_str("wss://inbox.example.com").unwrap();
        let outbox = RelayUrl::try_from_str("wss://outbox.example.com").unwrap();
        let both = RelayUrl::try_from_str("wss://both.example.com").unwrap();
        let mut expected = RelayList::default();
        let _ = expected.0.insert(inbox, RelayListUsage::Inbox);
        let _ = expected.0.insert(outbox, RelayListUsage::Outbox);
        let _ = expected.0.insert(both, RelayListUsage::Both);

        // Act
        let tags = expected.to_event_tags();
        let mut event = Event::mock();
        event.tags = tags;
        let parsed = RelayList::from_event(&event);

        // Assert
        assert_eq!(parsed, expected);
    }
}
