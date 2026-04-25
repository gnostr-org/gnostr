// NIP-02: Contact List and Petnames
// https://github.com/nostr-protocol/nips/blob/master/02.md

use secp256k1::XOnlyPublicKey;

use crate::types::event::{Event, UnsignedEvent};

/// A contact
#[derive(Debug, Clone)]
pub struct Contact {
    /// Their public key
    pub public_key: XOnlyPublicKey,
    /// A relay URL for them
    pub relay_url: Option<String>,
    /// A petname for them
    pub petname: Option<String>,
}

/// Set a contact list
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_contact_list_builds_kind_3_with_expected_tags() {
        let public_key = XOnlyPublicKey::from_slice(&[1u8; 32]).unwrap();
        let private_key = secp256k1::SecretKey::from_slice(&[2u8; 32]).unwrap();
        let contacts = vec![Contact {
            public_key: XOnlyPublicKey::from_slice(&[3u8; 32]).unwrap(),
            relay_url: Some("wss://relay.damus.io".to_string()),
            petname: Some("alice".to_string()),
        }];

        let event = set_contact_list(contacts, &public_key, &private_key);

        assert_eq!(u32::from(event.kind), 3);
        assert_eq!(event.content, "");
        assert_eq!(event.tags.len(), 1);
        assert_eq!(event.tags[0], vec![
            "p".to_string(),
            XOnlyPublicKey::from_slice(&[3u8; 32]).unwrap().to_string(),
            "wss://relay.damus.io".to_string(),
            "alice".to_string(),
        ]);
    }
}
