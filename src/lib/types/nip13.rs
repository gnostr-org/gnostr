// NIP-13: Proof of Work
// https://github.com/nostr-protocol/nips/blob/master/13.md

use crate::types::event::{Event, UnsignedEvent};
use crate::types::unixtime::Unixtime;
use secp256k1::{SecretKey, XOnlyPublicKey};

/// Generate a Proof of Work event
pub fn generate_pow_event(
    content: String,
    tags: Vec<Vec<String>>,
    difficulty: u8,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let mut nonce: u64 = 0;
    loop {
        let mut temp_tags = tags.clone();
        temp_tags.push(vec![
            "nonce".to_string(),
            nonce.to_string(),
            difficulty.to_string(),
        ]);
        let unsigned_event = UnsignedEvent::new(public_key, 1, temp_tags, content.clone());
        let event = unsigned_event.sign(private_key).unwrap();
        if crate::types::get_leading_zero_bits(&event.id.0) >= difficulty {
            return event;
        }
        nonce += 1;
    }
}
