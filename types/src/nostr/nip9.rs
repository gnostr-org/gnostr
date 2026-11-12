// NIP-09: Event Deletion
// https://github.com/nostr-protocol/nips/blob/master/09.md

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::nostr::event::{Event, EventId, UnsignedEvent};

/// Create a deletion event
pub fn delete(
    ids_to_delete: Vec<EventId>,
    reason: Option<&str>,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let tags: Vec<Vec<String>> = ids_to_delete
        .into_iter()
        .map(|id: EventId| vec!["e".to_string(), id.as_hex_string()])
        .collect();

    let content = reason.unwrap_or("").to_string();

    let unsigned_event = UnsignedEvent::new(public_key, 5, tags, content);
    unsigned_event.sign(private_key).unwrap()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::nostr::{Id, PrivateKey};

    fn test_keys() -> (XOnlyPublicKey, SecretKey) {
        let private_key = PrivateKey::mock();
        (
            private_key.public_key().as_xonly_public_key(),
            private_key.0,
        )
    }

    #[test]
    fn delete_creates_kind5_event_with_one_e_tag_per_id() {
        // Arrange
        let (public_key, secret) = test_keys();
        let ids = vec![Id::mock(), Id::mock(), Id::mock()];

        // Act
        let event = delete(ids.clone(), None, &public_key, &secret);

        // Assert
        assert_eq!(event.kind, crate::nostr::EventKind::EventDeletion);
        assert_eq!(event.tags.len(), 3);
        for (tag, id) in event.tags.iter().zip(ids.iter()) {
            assert_eq!(tag.tagname(), "e");
            assert_eq!(tag.get_index(1), id.as_hex_string());
        }
        assert_eq!(event.pubkey.as_xonly_public_key(), public_key);
    }

    #[test]
    fn delete_with_reason_sets_content() {
        // Arrange
        let (public_key, secret) = test_keys();

        // Act
        let event = delete(vec![Id::mock()], Some("spam"), &public_key, &secret);

        // Assert
        assert_eq!(event.content, "spam");
    }

    #[test]
    fn delete_without_reason_has_empty_content() {
        // Arrange
        let (public_key, secret) = test_keys();

        // Act
        let event = delete(vec![Id::mock()], None, &public_key, &secret);

        // Assert
        assert_eq!(event.content, "");
    }

    #[test]
    fn deletion_event_has_valid_signature() {
        // Arrange
        let (public_key, secret) = test_keys();

        // Act
        let event = delete(vec![Id::mock()], Some("cleanup"), &public_key, &secret);

        // Assert: verifies id hash and schnorr signature
        event.verify(None).unwrap();
    }
}
