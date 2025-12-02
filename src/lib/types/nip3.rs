// NIP-03: OpenTimestamps Attestations for Events
// https://github.com/nostr-protocol/nips/blob/master/03.md

use crate::types::event::{Event, EventId, UnsignedEvent};
use secp256k1::{SecretKey, XOnlyPublicKey};

/// Create an OpenTimestamps attestation event for another event.
///
/// The content must be a base64-encoded .ots file.
pub fn create_attestation(
    event_id: EventId,
    ots_base64: String,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let tags = vec![vec!["e".to_string(), event_id.as_hex_string()]];
    let unsigned_event = UnsignedEvent::new(public_key, 1040, tags, ots_base64);
    unsigned_event.sign(private_key).unwrap()
}
