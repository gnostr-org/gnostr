//! NIP-13: Proof of Work
//! https://github.com/nostr-protocol/nips/blob/master/13.md

use anyhow::Result;
use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::types::{
    Id, PublicKey, Signature, Tag, Unixtime,
    event::{Event, UnsignedEvent},
};

/// The name of the nonce tag.
pub const NONCE_TAG_NAME: &str = "nonce";

/// Helper trait for NIP-13 proof of work tags on Event types.
pub trait NIP13Event {
    /// Extracts the nonce value and target difficulty from the event's "nonce"
    /// tag. Returns `None` if the "nonce" tag is not found or is malformed.
    fn nonce_data(&self) -> Option<(u64, u8)>;

    /// Adds a "nonce" tag to the event with the given nonce value and target
    /// difficulty. If a "nonce" tag already exists, it will be replaced.
    fn add_nonce_tag(&mut self, nonce_value: u64, target_difficulty: u8);

    /// Creates a "nonce" tag with the given nonce value and target difficulty.
    fn create_nonce_tag(nonce_value: u64, target_difficulty: u8) -> Tag;
}

impl NIP13Event for Event {
    fn nonce_data(&self) -> Option<(u64, u8)> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() == 3 && tag.0[0] == NONCE_TAG_NAME {
                if let (Ok(nonce), Ok(difficulty)) =
                    (tag.0[1].parse::<u64>(), tag.0[2].parse::<u8>())
                {
                    Some((nonce, difficulty))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn add_nonce_tag(&mut self, nonce_value: u64, target_difficulty: u8) {
        // Remove existing nonce tags to ensure only one is present
        self.tags
            .retain(|tag| tag.0.get(0) != Some(&NONCE_TAG_NAME.to_string()));
        self.tags
            .push(Self::create_nonce_tag(nonce_value, target_difficulty));
    }

    fn create_nonce_tag(nonce_value: u64, target_difficulty: u8) -> Tag {
        Tag::new(&[
            NONCE_TAG_NAME,
            &nonce_value.to_string(),
            &target_difficulty.to_string(),
        ])
    }
}

/// Generate a Proof of Work event
pub fn generate_pow_event(
    content: String,
    mut tags: Vec<Tag>, // Change to Vec<Tag>
    difficulty: u8,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let mut nonce: u64 = 0;
    loop {
        // Remove any existing nonce tag before adding a new one for current iteration
        tags.retain(|tag| tag.0.get(0) != Some(&NONCE_TAG_NAME.to_string()));
        tags.push(Event::create_nonce_tag(nonce, difficulty));

        let unsigned_event = UnsignedEvent::new(
            public_key,
            1,
            tags.iter().map(|tag| tag.0.clone()).collect(), // Convert Vec<Tag> to Vec<Vec<String>>
            content.clone(),
        );
        let event = unsigned_event.sign(private_key).unwrap();
        if crate::types::get_leading_zero_bits(&event.id.0) >= difficulty {
            return event;
        }
        nonce += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature, Unixtime};

    // Helper to create a dummy event for testing
    fn create_dummy_event_with_nonce(nonce_value: u64, difficulty: u8) -> Event {
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![],
            content: "Test event for PoW".to_string(),
            sig: Signature::zeroes(),
        };
        event.add_nonce_tag(nonce_value, difficulty);
        event
    }

    #[test]
    fn test_create_nonce_tag() {
        let nonce_val = 12345;
        let difficulty_val = 20;
        let tag = Event::create_nonce_tag(nonce_val, difficulty_val);
        assert_eq!(
            tag.0,
            vec![
                NONCE_TAG_NAME.to_string(),
                nonce_val.to_string(),
                difficulty_val.to_string()
            ]
        );
    }

    #[test]
    fn test_nonce_data_found() {
        let nonce_val = 56789;
        let difficulty_val = 15;
        let event = create_dummy_event_with_nonce(nonce_val, difficulty_val);
        assert_eq!(event.nonce_data(), Some((nonce_val, difficulty_val)));
    }

    #[test]
    fn test_nonce_data_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])], // No nonce tag
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.nonce_data(), None);
    }

    #[test]
    fn test_nonce_data_malformed_tag() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&[NONCE_TAG_NAME, "not_a_number", "10"])], // Malformed nonce value
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.nonce_data(), None);

        let event2 = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&[NONCE_TAG_NAME, "123", "not_a_difficulty"])], /* Malformed difficulty */
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event2.nonce_data(), None);

        let event3 = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&[NONCE_TAG_NAME, "123"])], // Too few elements
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event3.nonce_data(), None);
    }

    #[test]
    fn test_add_nonce_tag() {
        let nonce1 = 1;
        let difficulty1 = 5;
        let nonce2 = 2;
        let difficulty2 = 10;
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_nonce_tag(nonce1, difficulty1),
            ],
            content: "Some content".to_string(),
            sig: Signature::zeroes(),
        };

        // Add a new nonce tag, it should replace the existing one
        event.add_nonce_tag(nonce2, difficulty2);
        assert_eq!(event.nonce_data(), Some((nonce2, difficulty2)));
        // Ensure only one nonce tag exists
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&NONCE_TAG_NAME.to_string()))
                .count(),
            1
        );

        // Add another nonce tag, it should replace the existing one
        let nonce3 = 3;
        let difficulty3 = 15;
        event.add_nonce_tag(nonce3, difficulty3);
        assert_eq!(event.nonce_data(), Some((nonce3, difficulty3)));
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&NONCE_TAG_NAME.to_string()))
                .count(),
            1
        );
    }
}
