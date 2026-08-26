// NIP-03: OpenTimestamps Attestations for Events
// https://github.com/nostr-protocol/nips/blob/master/03.md

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::nostr::{
    event::{Event, EventId, PreEvent},
    EventKind, KeySecurity, KeySigner, PrivateKey, PublicKey, Signer, Unixtime,
};

/// Create an OpenTimestamps attestation event for another event.
///
/// The content must be a base64-encoded .ots file.
pub fn create_attestation(
    event_id: EventId,
    ots_base64: String,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    create_attestation_with_pow(event_id, ots_base64, public_key, private_key, 0)
}

/// Create an OpenTimestamps attestation event and mine it with POW.
pub fn create_attestation_with_pow(
    event_id: EventId,
    ots_base64: String,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
    difficulty: u8,
) -> Event {
    let preevent = PreEvent {
        pubkey: PublicKey::from_bytes(&public_key.serialize(), false).unwrap(),
        created_at: Unixtime::now(),
        kind: EventKind::Timestamp,
        tags: vec![crate::nostr::TagV3::new_tag("e", &event_id.as_hex_string())],
        content: ots_base64,
    };

    if difficulty > 0 {
        let signer = KeySigner::from_private_key(
            PrivateKey(private_key.clone(), KeySecurity::NotTracked),
            "",
            1,
        )
        .unwrap();
        signer.sign_event_with_pow(preevent, difficulty, None).unwrap()
    } else {
        Event::sign_with_private_key(
            preevent,
            &PrivateKey(private_key.clone(), KeySecurity::NotTracked),
        )
        .unwrap()
    }
}
