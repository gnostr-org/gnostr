// NIP-26: Delegation
// https://github.com/nostr-protocol/nips/blob/master/26.md

use secp256k1::{schnorr::Signature, Secp256k1, SecretKey, XOnlyPublicKey};

pub struct Delegation {
    pub delegator: XOnlyPublicKey,
    pub delegatee: XOnlyPublicKey,
    pub event_kind: u16,
    pub until: Option<u64>,
    pub since: Option<u64>,
}

impl Delegation {
    pub fn create_tag(&self, private_key: &SecretKey) -> Result<String, secp256k1::Error> {
        let secp = Secp256k1::new();
        let conditions = self.build_conditions_string();
        let message = format!(
            "nostr:delegation:{}:{}",
            self.delegatee, conditions
        );
        let message_hash = secp256k1::Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(
            message.as_bytes(),
        );
        let signature = secp.sign_schnorr(&message_hash, private_key);
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
    let message_hash = secp256k1::Message::from_hashed_data::<secp256k1::hashes::sha256::Hash>(
        message.as_bytes(),
    );
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
