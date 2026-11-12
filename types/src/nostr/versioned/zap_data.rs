use serde::{Deserialize, Serialize};

use crate::nostr::{EventReference, Id, MilliSatoshi, PublicKey};

/// Data about a Zap
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ZapDataV2 {
    /// The event that was zapped. If missing we can't use the zap receipt
    /// event.
    pub zapped_event: EventReference,

    /// The amount that the event was zapped
    pub amount: MilliSatoshi,

    /// The public key of the person who received the zap
    pub payee: PublicKey,

    /// The public key of the person who paid the zap, if it was in the receipt
    pub payer: PublicKey,

    /// The public key of the zap provider, for verification purposes
    pub provider_pubkey: PublicKey,
}

/// Data about a Zap
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ZapDataV1 {
    /// The event that was zapped
    pub id: Id,

    /// The amount that the event was zapped
    pub amount: MilliSatoshi,

    /// The public key of the person who provided the zap
    pub pubkey: PublicKey,

    /// The public key of the zap provider, for verification purposes
    pub provider_pubkey: PublicKey,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_zap_data_v1() {
        let id = Id::mock();
        let pubkey = PublicKey::mock_deterministic();
        let provider_pubkey = PublicKey::mock();
        let zap = ZapDataV1 {
            id,
            amount: MilliSatoshi::mock(),
            pubkey,
            provider_pubkey,
        };

        let copied = zap;

        assert_eq!(copied.id, id);
        assert_eq!(copied.amount, MilliSatoshi::mock());
        assert_eq!(copied.pubkey, pubkey);
        assert_eq!(copied.provider_pubkey, provider_pubkey);
    }

    #[test]
    fn test_zap_data_v2() {
        let zapped_event = EventReference::Id {
            id: Id::mock(),
            author: Some(PublicKey::mock_deterministic()),
            relays: Vec::new(),
            marker: Some("root".to_owned()),
        };
        let payee = PublicKey::mock_deterministic();
        let payer = PublicKey::mock();
        let provider_pubkey = PublicKey::mock();
        let zap = ZapDataV2 {
            zapped_event: zapped_event.clone(),
            amount: MilliSatoshi::mock(),
            payee,
            payer,
            provider_pubkey,
        };

        assert_eq!(zap.zapped_event, zapped_event);
        assert_eq!(zap.amount, MilliSatoshi::mock());
        assert_eq!(zap.payee, payee);
        assert_eq!(zap.payer, payer);
        assert_eq!(zap.provider_pubkey, provider_pubkey);
    }
}
