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

/// NIP-34 event kinds.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone)]
pub enum Nip34Kind {
    RepoAnnouncement = 30617,
    RepoState = 30618,
    Patch = 1617,
    PullRequest = 1618,
    PullRequestUpdate = 1619,
    Issue = 1621,
    StatusOpen = 1630,
    StatusApplied = 1631,
    StatusClosed = 1632,
    StatusDraft = 1633,
    UserGraspList = 10317,
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::{schnorr, Secp256k1};
    use rand::rngs::OsRng;

    #[test]
    fn test_sign_and_verify() {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let x_only_public_key = public_key.x_only_public_key().0;

        let unsigned_event = UnsignedEvent::new(
            &x_only_public_key,
            1,
            vec![vec!["p".to_string(), "pubkey_hex".to_string()]],
            "Hello, Nostr!".to_string(),
        );

        let event = unsigned_event.sign(&secret_key).unwrap();

        let event_id_bytes = hex::decode(event.id).unwrap();
        let message = Message::from_slice(&event_id_bytes).unwrap();
        let signature = schnorr::Signature::from_slice(&hex::decode(event.sig).unwrap()).unwrap();

        secp.verify_schnorr(&signature, &message, &x_only_public_key)
            .unwrap();
    }

    #[test]
    fn test_nip34_kinds() {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let x_only_public_key = public_key.x_only_public_key().0;

        let kinds = vec![
            Nip34Kind::RepoAnnouncement,
            Nip34Kind::RepoState,
            Nip34Kind::Patch,
            Nip34Kind::PullRequest,
            Nip34Kind::PullRequestUpdate,
            Nip34Kind::Issue,
            Nip34Kind::StatusOpen,
            Nip34Kind::StatusApplied,
            Nip34Kind::StatusClosed,
            Nip34Kind::StatusDraft,
            Nip34Kind::UserGraspList,
        ];

        for kind in kinds {
            let unsigned_event = UnsignedEvent::new(
                &x_only_public_key,
                kind as u16,
                vec![],
                "test".to_string(),
            );
            let event = unsigned_event.sign(&secret_key).unwrap();
            assert_eq!(event.kind, kind as u16);
        }
    }
}