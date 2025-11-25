//! NIP-34 implementation for creating git-related events.

use secp256k1::{Message, SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

/// A signed Nostr event.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    /// 32-byte, hex-encoded SHA256 hash of the serialized event data.
    pub id: String,
    /// 32-byte, hex-encoded public key of the event creator.
    pub pubkey: String,
    /// Unix timestamp in seconds.
    pub created_at: u64,
    /// Event kind.
    pub kind: u16,
    /// A list of tags.
    pub tags: Vec<Vec<String>>,
    /// Event content.
    pub content: String,
    /// 64-byte signature of the event ID hash.
    pub sig: String,
}

/// An unsigned Nostr event.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsignedEvent {
    /// 32-byte, hex-encoded public key of the event creator.
    pub pubkey: String,
    /// Unix timestamp in seconds.
    pub created_at: u64,
    /// Event kind.
    pub kind: u16,
    /// A list of tags.
    pub tags: Vec<Vec<String>>,
    /// Event content.
    pub content: String,
}

impl UnsignedEvent {
    /// Create a new unsigned event.
    pub fn new(pubkey: &XOnlyPublicKey, kind: u16, tags: Vec<Vec<String>>, content: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        Self {
            pubkey: pubkey.to_string(),
            created_at,
            kind,
            tags,
            content,
        }
    }

    /// Serialize the event data for hashing and signing.
    fn serialize(&self) -> Result<String, serde_json::Error> {
        let data = (
            0,
            &self.pubkey,
            self.created_at,
            self.kind,
            &self.tags,
            &self.content,
        );
        serde_json::to_string(&data)
    }

    /// Sign the event and return a signed `Event`.
    pub fn sign(self, secret_key: &SecretKey) -> Result<Event, Box<dyn std::error::Error>> {
        let serialized_event = self.serialize()?;
        let mut hasher = Sha256::new();
        hasher.update(serialized_event.as_bytes());
        let event_id_bytes = hasher.finalize();
        let id = hex::encode(event_id_bytes);

        let secp = secp256k1::Secp256k1::new();
        let message = Message::from_slice(&event_id_bytes)?;
        let sig = secp.sign_schnorr(&message, &secret_key.keypair(&secp));

        Ok(Event {
            id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig: sig.to_string(),
        })
    }
}
