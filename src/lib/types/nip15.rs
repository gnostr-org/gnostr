// NIP-15: End of Stored Events Notice
// https://github.com/nostr-protocol/nips/blob/master/15.md

use crate::types::event::{Event, UnsignedEvent};
use secp256k1::{SecretKey, XOnlyPublicKey};

/// Create an End of Stored Events (EOSE) event
pub fn end_of_stored_events(public_key: &XOnlyPublicKey, private_key: &SecretKey) -> Event {
    let unsigned_event = UnsignedEvent::new(public_key, 4, vec![], "".to_string());
    unsigned_event.sign(private_key).unwrap()
}
