// NIP-26: Delegation
// https://github.com/nostr-protocol/nips/blob/master/26.md

use secp256k1::{schnorr::Signature, Keypair, Message, Secp256k1, SecretKey, XOnlyPublicKey};
use sha2::{Digest, Sha256};

/// A delegation, which allows one key to sign an event on behalf of another
/// key.
#[derive(Debug, Copy, Clone)]
pub struct Delegation {
    /// The public key of the delegator
    pub delegator: XOnlyPublicKey,
    /// The public key of the delegatee
    pub delegatee: XOnlyPublicKey,
    /// The kind of event being delegated
    pub event_kind: u16,
    /// An optional expiration timestamp for the delegation
    pub until: Option<u64>,
    /// An optional creation timestamp for the delegation
    pub since: Option<u64>,
}

impl Delegation {
    /// Create a delegation tag
    pub fn create_tag(&self, private_key: &SecretKey) -> Result<String, secp256k1::Error> {
        let secp = Secp256k1::new();
        let keypair = Keypair::from_secret_key(&secp, private_key);
        let conditions = self.build_conditions_string();
        let message = format!("nostr:delegation:{}:{}", self.delegatee, conditions);
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let message_hash = Message::from_digest_slice(&hasher.finalize()).unwrap();
        let signature = secp.sign_schnorr(&message_hash, &keypair);
        Ok(format!(
            "delegation:{}:{}:{}",
            self.delegator, conditions, signature
        ))
    }

    fn build_conditions_string(&self) -> String {
        let mut conditions = format!("kind={}", self.event_kind);
        if let Some(until) = self.until {
            conditions.push_str(&format!("&created_at<{}", until));
        }
        if let Some(since) = self.since {
            conditions.push_str(&format!("&created_at>{}", since));
        }
        conditions
    }
}

/// Verify a delegation tag
pub fn verify(
    delegation_tag: &str,
    delegatee_pubkey: &XOnlyPublicKey,
    event_kind: u16,
    created_at: u64,
) -> Result<bool, anyhow::Error> {
    let mut parts = delegation_tag.split(':');
    if parts.next() != Some("delegation") {
        return Ok(false);
    }

    let delegator_str = parts.next().ok_or(anyhow::anyhow!("Missing delegator"))?;
    let conditions = parts.next().ok_or(anyhow::anyhow!("Missing conditions"))?;
    let signature_str = parts.next().ok_or(anyhow::anyhow!("Missing signature"))?;

    // Verify signature
    let secp = Secp256k1::new();
    let message = format!("nostr:delegation:{}:{}", delegatee_pubkey, conditions);
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let message_hash = Message::from_digest_slice(&hasher.finalize()).unwrap();
    let signature = Signature::from_slice(&hex::decode(signature_str)?)?;
    let delegator = XOnlyPublicKey::from_slice(&hex::decode(delegator_str)?)?;
    secp.verify_schnorr(&signature, &message_hash, &delegator)?;

    // Verify conditions
    for condition in conditions.split('&') {
        let mut parts = condition.split('=');
        let key = parts.next();
        let value = parts.next();

        if let (Some(key), Some(value)) = (key, value) {
            match key {
                "kind" => {
                    if value.parse::<u16>()? != event_kind {
                        return Ok(false);
                    }
                }
                "created_at<" => {
                    if created_at >= value.parse::<u64>()? {
                        return Ok(false);
                    }
                }
                "created_at>" => {
                    if created_at <= value.parse::<u64>()? {
                        return Ok(false);
                    }
                }
                _ => {} // Unknown conditions are ignored
            }
        }
    }

    Ok(true)
}
