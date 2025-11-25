// NIP-02: Contact List and Petnames
// https://github.com/nostr-protocol/nips/blob/master/02.md

use crate::event::{Event, UnsignedEvent};
use secp256k1::XOnlyPublicKey;

pub struct Contact {
    pub public_key: XOnlyPublicKey,
    pub relay_url: Option<String>,
    pub petname: Option<String>,
}

pub fn set_contact_list(
    contacts: Vec<Contact>,
    public_key: &XOnlyPublicKey,
    private_key: &secp256k1::SecretKey,
) -> Event {
    let tags: Vec<Vec<String>> = contacts
        .into_iter()
        .map(|contact| {
            let mut tag = vec!["p".to_string(), contact.public_key.to_string()];
            if let Some(relay_url) = contact.relay_url {
                tag.push(relay_url);
            }
            if let Some(petname) = contact.petname {
                tag.push(petname);
            }
            tag
        })
        .collect();

    let unsigned_event = UnsignedEvent::new(public_key, 3, tags, "".to_string());
    unsigned_event.sign(private_key).unwrap()
}
