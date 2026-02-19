// NIP-05: Mapping Nostr keys to DNS-based internet identifiers
// https://github.com/nostr-protocol/nips/blob/master/05.md

use std::collections::HashMap;

use anyhow::{Result, anyhow};
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    types::event::{Event, UnsignedEvent},
    utils::ureq_async,
};

/// A Nip05 record
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Nip05 {
    /// A map of names to public keys
    pub names: HashMap<String, String>,
}

/// A metadata record
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata {
    /// The user's name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A description of the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// A URL to the user's profile picture
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    /// A URL to the user's banner image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    /// The user's Nip05 identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nip05: Option<String>,
    /// The user's lightning address (LUD-06)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lud06: Option<String>,
    /// The user's lightning address (LUD-16)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lud16: Option<String>,
    /// Extra fields
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Set metadata for a user
pub fn set_metadata(
    metadata: &Metadata,
    tags: Vec<Vec<String>>,
    public_key: &XOnlyPublicKey,
    private_key: &secp256k1::SecretKey,
) -> Result<Event, serde_json::Error> {
    let content = serde_json::to_string(metadata)?;
    let unsigned_event = UnsignedEvent::new(public_key, 0, tags, content);
    let signed_event = unsigned_event.sign(private_key).unwrap();
    Ok(signed_event)
}

/// Verify a nip05 identifier
pub async fn verify(public_key: &XOnlyPublicKey, nip05_identifier: &str) -> Result<bool> {
    let mut parts = nip05_identifier.split('@');
    let name = parts.next();
    let domain = parts.next();

    if name.is_none() || domain.is_none() {
        return Err(anyhow!("Invalid NIP-05 identifier format"));
    }

    let name = name.unwrap();
    let domain = domain.unwrap();

    let url = format!("https://{}/.well-known/nostr.json?name={}", domain, name);

    let response_str = ureq_async(url).await.map_err(|e| anyhow!(e))?;
    let nip05_data: Nip05 = serde_json::from_str(&response_str)?;

    if let Some(found_pubkey) = nip05_data.names.get(name) {
        let pk_hex = public_key.to_string();
        return Ok(found_pubkey == &pk_hex);
    }

    Ok(false)
}
