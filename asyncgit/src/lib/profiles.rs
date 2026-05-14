use serde_json::Value;

use crate::types::{Metadata, PrivateKey, PublicKey, DEFAULT_AVATAR, DEFAULT_BANNER};

/// Deterministic Nostr profile fixture for tests and sample flows.
#[derive(Clone, Copy, Debug)]
pub struct NostrProfileFixture {
    pub label: &'static str,
    pub name: &'static str,
    pub display_name: &'static str,
    pub about: &'static str,
    pub picture: &'static str,
    pub nip05: &'static str,
    pub lud16: &'static str,
    pub website: &'static str,
    pub private_key_hex: &'static str,
}

impl NostrProfileFixture {
    pub fn private_key(self) -> PrivateKey {
        PrivateKey::try_from_hex_string(self.private_key_hex).expect("fixture private key must be valid")
    }

    pub fn public_key(self) -> PublicKey {
        self.private_key().public_key()
    }

    pub fn npub(self) -> String {
        self.public_key().as_bech32_string()
    }

    pub fn nsec(self) -> String {
        let mut private_key = self.private_key();
        private_key.as_bech32_string()
    }

    pub fn metadata(self) -> Metadata {
        let mut metadata = Metadata::new();
        metadata.name = Some(self.name.to_string());
        metadata.about = Some(self.about.to_string());
        metadata.picture = Some(self.picture.to_string());
        metadata.nip05 = Some(self.nip05.to_string());
        metadata
            .other
            .insert("display_name".to_string(), Value::String(self.display_name.to_string()));
        metadata
            .other
            .insert("lud16".to_string(), Value::String(self.lud16.to_string()));
        metadata
            .other
            .insert("website".to_string(), Value::String(self.website.to_string()));
        metadata
            .other
            .insert("banner".to_string(), Value::String(DEFAULT_BANNER.to_string()));
        metadata
    }

    pub fn metadata_json(self) -> String {
        serde_json::to_string(&self.metadata()).expect("fixture metadata must serialize")
    }
}

pub const BITCOINDEV_1: NostrProfileFixture = NostrProfileFixture {
    label: "bitcoindev_1",
    name: "bitcoindev_1",
    display_name: "Bitcoin Dev 1",
    about: "Public attestation fixture profile 1.",
    picture: DEFAULT_AVATAR,
    nip05: "bitcoindev1@gnostr.org",
    lud16: "bitcoindev1@strike.me",
    website: "https://example.com/bitcoindev/1",
    private_key_hex: "0ef991672c1c5a437eb81a203493b3320edfd469a5b4544ffb87fc119b55dd2e",
};

pub const BITCOINDEV_2: NostrProfileFixture = NostrProfileFixture {
    label: "bitcoindev_2",
    name: "bitcoindev_2",
    display_name: "Bitcoin Dev 2",
    about: "Public attestation fixture profile 2.",
    picture: DEFAULT_AVATAR,
    nip05: "bitcoindev2@gnostr.org",
    lud16: "bitcoindev2@strike.me",
    website: "https://example.com/bitcoindev/2",
    private_key_hex: "5ec6766ca89bb0bc374c8b48638416d310311cc9b7d0769912f52698c03b1cb5",
};

pub const BITCOINDEV_3: NostrProfileFixture = NostrProfileFixture {
    label: "bitcoindev_3",
    name: "bitcoindev_3",
    display_name: "Bitcoin Dev 3",
    about: "Public attestation fixture profile 3.",
    picture: DEFAULT_AVATAR,
    nip05: "bitcoindev3@gnostr.org",
    lud16: "bitcoindev3@strike.me",
    website: "https://example.com/bitcoindev/3",
    private_key_hex: "5be4d9ad6aacbb765acb8ad1db609b3d374a31756ec6af3d6405bab306ab7e83",
};

pub use BITCOINDEV_1 as bitcoindev_1;
pub use BITCOINDEV_2 as bitcoindev_2;
pub use BITCOINDEV_3 as bitcoindev_3;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn profiles_build_metadata_and_keys() {
        let profile = bitcoindev_1;
        let metadata = profile.metadata();
        println!(
            "profile={} npub={} nsec={} metadata={}",
            profile.label,
            profile.npub(),
            profile.nsec(),
            profile.metadata_json()
        );

        assert_eq!(metadata.name.as_deref(), Some("bitcoindev_1"));
        assert_eq!(metadata.nip05.as_deref(), Some("bitcoindev1@gnostr.org"));
        assert!(profile.npub().starts_with("npub1"));
        assert!(profile.nsec().starts_with("nsec1"));
    }
}
