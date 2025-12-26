// NIP-10: Text Notes and Threads
// https://github.com/nostr-protocol/nips/blob/master/10.md

use crate::types::event::{Event, EventId, UnsignedEvent};
use secp256k1::{SecretKey, XOnlyPublicKey};

/// Create a reply to an event.
pub fn create_reply(
    root_id: EventId,
    replied_to_id: EventId,
    content: String,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let tags = vec![
        vec!["e".to_string(), root_id.as_hex_string(), "root".to_string()],
        vec![
            "e".to_string(),
            replied_to_id.as_hex_string(),
            "reply".to_string(),
        ],
    ];
    let unsigned_event = UnsignedEvent::new(public_key, 1, tags, content);
    unsigned_event.sign(private_key).unwrap()
}
