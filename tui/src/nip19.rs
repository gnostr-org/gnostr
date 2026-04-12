//! NIP-19 bech32-encoded Nostr entities.
//!
//! Supports the simple single-key TLV-less formats (npub/nsec/note)
//! that cover the TUI's needs without pulling in the full nostr-sdk.
//!
//! Reference: <https://github.com/nostr-protocol/nips/blob/master/19.md>

use bech32::{Bech32, Hrp};

// ── Encode ────────────────────────────────────────────────────────────────────

/// Encode a 32-byte public key (hex) as `npub1…`.
pub fn pubkey_to_npub(hex_pubkey: &str) -> Result<String, Nip19Error> {
    let bytes = hex::decode(hex_pubkey).map_err(|_| Nip19Error::InvalidHex)?;
    if bytes.len() != 32 {
        return Err(Nip19Error::WrongLength);
    }
    let hrp = Hrp::parse("npub").expect("static hrp");
    Ok(bech32::encode::<Bech32>(hrp, &bytes).expect("encode"))
}

/// Encode a 32-byte secret key (hex) as `nsec1…`.
pub fn seckey_to_nsec(hex_seckey: &str) -> Result<String, Nip19Error> {
    let bytes = hex::decode(hex_seckey).map_err(|_| Nip19Error::InvalidHex)?;
    if bytes.len() != 32 {
        return Err(Nip19Error::WrongLength);
    }
    let hrp = Hrp::parse("nsec").expect("static hrp");
    Ok(bech32::encode::<Bech32>(hrp, &bytes).expect("encode"))
}

/// Encode a 32-byte event id (hex) as `note1…`.
pub fn event_to_note(hex_event_id: &str) -> Result<String, Nip19Error> {
    let bytes = hex::decode(hex_event_id).map_err(|_| Nip19Error::InvalidHex)?;
    if bytes.len() != 32 {
        return Err(Nip19Error::WrongLength);
    }
    let hrp = Hrp::parse("note").expect("static hrp");
    Ok(bech32::encode::<Bech32>(hrp, &bytes).expect("encode"))
}

// ── Decode ────────────────────────────────────────────────────────────────────

/// Decode an `npub1…` string to a 64-char hex pubkey.
pub fn npub_to_pubkey(npub: &str) -> Result<String, Nip19Error> {
    let (hrp, data) = bech32::decode(npub).map_err(|_| Nip19Error::Bech32)?;
    if hrp.as_str() != "npub" {
        return Err(Nip19Error::WrongHrp);
    }
    if data.len() != 32 {
        return Err(Nip19Error::WrongLength);
    }
    Ok(hex::encode(data))
}

/// Decode an `nsec1…` string to a 64-char hex secret key.
pub fn nsec_to_seckey(nsec: &str) -> Result<String, Nip19Error> {
    let (hrp, data) = bech32::decode(nsec).map_err(|_| Nip19Error::Bech32)?;
    if hrp.as_str() != "nsec" {
        return Err(Nip19Error::WrongHrp);
    }
    if data.len() != 32 {
        return Err(Nip19Error::WrongLength);
    }
    Ok(hex::encode(data))
}

/// Decode a `note1…` string to a 64-char hex event id.
pub fn note_to_event(note: &str) -> Result<String, Nip19Error> {
    let (hrp, data) = bech32::decode(note).map_err(|_| Nip19Error::Bech32)?;
    if hrp.as_str() != "note" {
        return Err(Nip19Error::WrongHrp);
    }
    if data.len() != 32 {
        return Err(Nip19Error::WrongLength);
    }
    Ok(hex::encode(data))
}

/// Attempt to decode any bech32 Nostr entity; return `(hrp, hex_data)`.
pub fn decode_any(encoded: &str) -> Result<(String, String), Nip19Error> {
    let (hrp, data) = bech32::decode(encoded).map_err(|_| Nip19Error::Bech32)?;
    Ok((hrp.to_string(), hex::encode(data)))
}

// ── Error ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nip19Error {
    InvalidHex,
    WrongLength,
    WrongHrp,
    Bech32,
}

impl std::fmt::Display for Nip19Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidHex   => write!(f, "invalid hex string"),
            Self::WrongLength  => write!(f, "key must be exactly 32 bytes"),
            Self::WrongHrp     => write!(f, "unexpected bech32 human-readable part"),
            Self::Bech32       => write!(f, "bech32 decode error"),
        }
    }
}

impl std::error::Error for Nip19Error {}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    const HEX_KEY: &str =
        "7e7e9c42a91bfef19fa929e5fda1b72e0ebc1a4c1141673e2794234d86addf4e";

    #[test]
    fn roundtrip_npub() {
        let npub = pubkey_to_npub(HEX_KEY).unwrap();
        assert!(npub.starts_with("npub1"));
        assert_eq!(npub_to_pubkey(&npub).unwrap(), HEX_KEY);
    }

    #[test]
    fn roundtrip_nsec() {
        let nsec = seckey_to_nsec(HEX_KEY).unwrap();
        assert!(nsec.starts_with("nsec1"));
        assert_eq!(nsec_to_seckey(&nsec).unwrap(), HEX_KEY);
    }

    #[test]
    fn roundtrip_note() {
        let note = event_to_note(HEX_KEY).unwrap();
        assert!(note.starts_with("note1"));
        assert_eq!(note_to_event(&note).unwrap(), HEX_KEY);
    }

    #[test]
    fn wrong_hrp_rejected() {
        let npub = pubkey_to_npub(HEX_KEY).unwrap();
        assert_eq!(nsec_to_seckey(&npub), Err(Nip19Error::WrongHrp));
    }

    #[test]
    fn invalid_hex_rejected() {
        assert_eq!(pubkey_to_npub("not_hex!"), Err(Nip19Error::InvalidHex));
    }
}
