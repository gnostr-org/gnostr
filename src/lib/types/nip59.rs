// NIP-59: Gift Wrap
// https://github.com/nostr-protocol/nips/blob/master/59.md

use crate::types::event::{Event, Rumor, UnsignedEvent};
use crate::types::{ContentEncryptionAlgorithm, PrivateKey, PublicKey, Signer};
use secp256k1::{SecretKey, XOnlyPublicKey};

/// Create a Seal event (kind 13) which wraps a Rumor
pub fn create_seal(
    rumor: Rumor,
    private_key: &PrivateKey,
    recipient_pubkey: &PublicKey,
) -> Result<Event, crate::types::Error> {
    let rumor_json = serde_json::to_string(&rumor)?;
    let encrypted_content = private_key.encrypt(
        recipient_pubkey,
        &rumor_json,
        ContentEncryptionAlgorithm::Nip44v2,
    )?;
    let unsigned_event = UnsignedEvent::new(
        &private_key.public_key().as_xonly_public_key(),
        13,
        vec![],
        encrypted_content,
    );
    unsigned_event.sign(&private_key.as_secret_key())
}

/// Create a Gift Wrap event (kind 1059) which wraps a Seal
pub fn create_gift_wrap(
    seal: Event,
    private_key: &PrivateKey,
    recipient_pubkey: &PublicKey,
) -> Result<Event, crate::types::Error> {
    let seal_json = serde_json::to_string(&seal)?;
    let encrypted_content = private_key.encrypt(
        recipient_pubkey,
        &seal_json,
        ContentEncryptionAlgorithm::Nip44v2,
    )?;
    let tags = vec![vec!["p".to_string(), recipient_pubkey.as_hex_string()]];
    let unsigned_event = UnsignedEvent::new(
        &private_key.public_key().as_xonly_public_key(),
        1059,
        tags,
        encrypted_content,
    );
    unsigned_event.sign(&private_key.as_secret_key())
}
