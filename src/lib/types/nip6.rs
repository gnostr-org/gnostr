// NIP-06: Basic key derivation from mnemonic seed phrase
// https://github.com/nostr-protocol/nips/blob/master/06.md

use bip39::{Language, Mnemonic, Seed};
use secp256k1::{Secp256k1, SecretKey};
use tiny_hderive::bip32::ExtendedPrivKey;

/// Get a secret key from a mnemonic phrase
pub fn from_mnemonic(mnemonic: &str, passphrase: Option<&str>) -> Result<SecretKey, anyhow::Error> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    let seed = Seed::new(&mnemonic, passphrase.unwrap_or(""));

    let ext_priv_key = ExtendedPrivKey::derive(seed.as_bytes(), "m/44'/1237'/0'/0/0")
        .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;
    let private_key = SecretKey::from_slice(&ext_priv_key.secret())?;

    Ok(private_key)
}
