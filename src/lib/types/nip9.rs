// NIP-09: Event Deletion
// https://github.com/nostr-protocol/nips/blob/master/09.md

use crate::event::{Event, EventId, UnsignedEvent};
use secp256k1::{XOnlyPublicKey, SecretKey};

pub fn delete(
    ids_to_delete: Vec<EventId>,
    reason: Option<&str>,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let tags: Vec<Vec<String>> = ids_to_delete
        .into_iter()
        .map(|id| vec!["e".to_string(), id.to_string()])
        .collect();

    let content = reason.unwrap_or("").to_string();

    let unsigned_event = UnsignedEvent::new(public_key, 5, tags, content);
    unsigned_event.sign(private_key).unwrap()
}
