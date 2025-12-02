// NIP-18: Reposts
// https://github.com/nostr-protocol/nips/blob/master/18.md

use crate::types::event::{Event, EventId, UnsignedEvent};
use crate::types::{PublicKey, RelayUrl, Tag};
use secp256k1::{SecretKey, XOnlyPublicKey};

/// Create a kind 6 repost event for a text note (kind 1).
pub fn create_repost_text_note(
    reposted_event: &Event,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Result<Event, crate::types::Error> {
    let content = serde_json::to_string(reposted_event)?;
    let tags = vec![
        Tag::new_event(reposted_event.id, None, None).as_vec(),
        Tag::new_pubkey(reposted_event.pubkey, None, None).as_vec(),
    ];
    let unsigned_event = UnsignedEvent::new(public_key, 6, tags, content);
    Ok(unsigned_event.sign(private_key).unwrap())
}

/// Create a kind 16 generic repost event for any event other than kind 1.
pub fn create_generic_repost(
    reposted_event: &Event,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Result<Event, crate::types::Error> {
    let content = serde_json::to_string(reposted_event)?;
    let tags = vec![
        Tag::new_event(reposted_event.id, None, None).as_vec(),
        Tag::new_pubkey(reposted_event.pubkey, None, None).as_vec(),
        Tag::new_kind(reposted_event.kind).as_vec(),
    ];
    let unsigned_event = UnsignedEvent::new(public_key, 16, tags, content);
    Ok(unsigned_event.sign(private_key).unwrap())
}
