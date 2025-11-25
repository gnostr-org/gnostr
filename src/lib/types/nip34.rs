//! NIP-34 implementation for creating git-related events.

use crate::{blockheight, blockhash, weeble, wobble};
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
    pub fn new(pubkey: &XOnlyPublicKey, kind: u16, mut tags: Vec<Vec<String>>, content: String) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Ok(val) = weeble::weeble() {
            tags.push(vec!["weeble".to_string(), val.to_string()]);
        }
        if let Ok(val) = blockheight::blockheight() {
            tags.push(vec!["blockheight".to_string(), val.to_string()]);
        }
        if let Ok(val) = wobble::wobble() {
            tags.push(vec!["wobble".to_string(), val.to_string()]);
        }
        if let Ok(val) = blockhash::blockhash() {
            tags.push(vec!["blockhash".to_string(), val]);
        }

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

    fn test_event_creation(kind: Nip34Kind, mut tags: Vec<Vec<String>>, content: String) {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let x_only_public_key = public_key.x_only_public_key().0;

        let unsigned_event = UnsignedEvent::new(
            &x_only_public_key,
            kind as u16,
            tags.clone(),
            content.clone(),
        );

        let event = unsigned_event.sign(&secret_key).unwrap();
        println!("Signed event for kind {:?}: {:?}", kind, event);

        assert_eq!(event.kind, kind as u16);
        assert_eq!(event.content, content);

        if let Ok(val) = weeble::weeble() {
            tags.push(vec!["weeble".to_string(), val.to_string()]);
        }
        if let Ok(val) = blockheight::blockheight() {
            tags.push(vec!["blockheight".to_string(), val.to_string()]);
        }
        if let Ok(val) = wobble::wobble() {
            tags.push(vec!["wobble".to_string(), val.to_string()]);
        }
        if let Ok(val) = blockhash::blockhash() {
            tags.push(vec!["blockhash".to_string(), val]);
        }

        assert_eq!(event.tags, tags);

        let event_id_bytes = hex::decode(event.id).unwrap();
        let message = Message::from_slice(&event_id_bytes).unwrap();
        let signature = schnorr::Signature::from_slice(&hex::decode(event.sig).unwrap()).unwrap();

        secp.verify_schnorr(&signature, &message, &x_only_public_key)
            .unwrap();
    }

    #[test]
    fn test_repo_announcement() {
        let tags = vec![
            vec!["d".to_string(), "gnostr".to_string()],
            vec!["name".to_string(), "gnostr".to_string()],
            vec!["description".to_string(), "A git implementation on nostr".to_string()],
            vec!["web".to_string(), "https://github.com/gnostr-org/gnostr".to_string()],
            vec!["clone".to_string(), "https://github.com/gnostr-org/gnostr.git".to_string()],
            vec!["relays".to_string(), "wss://relay.damus.io".to_string()],
        ];
        test_event_creation(Nip34Kind::RepoAnnouncement, tags, "".to_string());
    }

    #[test]
    fn test_repo_state() {
        let tags = vec![
            vec!["d".to_string(), "gnostr".to_string()],
            vec!["refs/heads/main".to_string(), "abcdef123456".to_string()],
            vec!["refs/tags/v0.1.0".to_string(), "fedcba654321".to_string()],
        ];
        test_event_creation(Nip34Kind::RepoState, tags, "".to_string());
    }

    #[test]
    fn test_patch() {
        let tags = vec![
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["commit".to_string(), "abcdef123456".to_string()],
        ];
        let content = "--- a/README.md\n+++ b/README.md\n@@ -1,3 +1,3 @@\n # gnostr\n-A git implementation on nostr\n+A git implementation over nostr".to_string();
        test_event_creation(Nip34Kind::Patch, tags, content);
    }

    #[test]
    fn test_pull_request() {
        let tags = vec![
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["subject".to_string(), "Add new feature".to_string()],
            vec!["branch-name".to_string(), "feature-branch".to_string()],
            vec!["merge-base".to_string(), "abcdef123456".to_string()],
        ];
        test_event_creation(Nip34Kind::PullRequest, tags, "".to_string());
    }

    #[test]
    fn test_pull_request_update() {
        let tags = vec![
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["e".to_string(), "event_id_of_pr".to_string()],
            vec!["c".to_string(), "new_commit_hash".to_string()],
        ];
        test_event_creation(Nip34Kind::PullRequestUpdate, tags, "".to_string());
    }

    #[test]
    fn test_issue() {
        let tags = vec![
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["subject".to_string(), "Bug report".to_string()],
        ];
        test_event_creation(Nip34Kind::Issue, tags, "This is a bug report.".to_string());
    }

    #[test]
    fn test_status_open() {
        let tags = vec![
            vec!["e".to_string(), "event_id_of_issue".to_string(), "root".to_string()],
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
        ];
        test_event_creation(Nip34Kind::StatusOpen, tags, "".to_string());
    }

    #[test]
    fn test_status_applied() {
        let tags = vec![
            vec!["e".to_string(), "event_id_of_patch".to_string(), "root".to_string()],
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["applied-as-commits".to_string(), "commit1,commit2".to_string()],
        ];
        test_event_creation(Nip34Kind::StatusApplied, tags, "".to_string());
    }

    #[test]
    fn test_status_closed() {
        let tags = vec![
            vec!["e".to_string(), "event_id_of_pr".to_string(), "root".to_string()],
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
        ];
        test_event_creation(Nip34Kind::StatusClosed, tags, "".to_string());
    }

    #[test]
    fn test_status_draft() {
        let tags = vec![
            vec!["e".to_string(), "event_id_of_patch".to_string(), "root".to_string()],
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
        ];
        test_event_creation(Nip34Kind::StatusDraft, tags, "".to_string());
    }

    #[test]
    fn test_user_grasp_list() {
        let tags = vec![
            vec!["g".to_string(), "wss://grasp.example.com".to_string()],
            vec!["g".to_string(), "wss://another-grasp.example.com".to_string()],
        ];
        test_event_creation(Nip34Kind::UserGraspList, tags, "".to_string());
    }
}