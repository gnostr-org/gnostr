// NIP-06: Basic key derivation from mnemonic seed phrase
// https://github.com/nostr-protocol/nips/blob/master/06.md

use bip32::{DerivationPath, XPrv};
use bip39::{Language, Mnemonic, Seed};
use secp256k1::SecretKey;

/// Get a secret key from a mnemonic phrase
pub fn from_mnemonic(mnemonic: &str, passphrase: Option<&str>) -> Result<SecretKey, anyhow::Error> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    let seed = Seed::new(&mnemonic, passphrase.unwrap_or(""));

    let path: DerivationPath = "m/44'/1237'/0'/0/0".parse()?;
    let ext_priv_key = XPrv::derive_from_path(seed.as_bytes(), &path)?;
    let private_key = SecretKey::from_slice(&ext_priv_key.private_key().to_bytes())?;

    Ok(private_key)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_mnemonic_smoke() {
        // BIP39 test vector mnemonic; NIP-06 path m/44'/1237'/0'/0/0
        let mnemonic = "abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon abandon about";
        let secret_key = from_mnemonic(mnemonic, None).unwrap();
        // Valid secp256k1 secret key: 32 non-zero bytes
        let bytes = secret_key.secret_bytes();
        assert_eq!(bytes.len(), 32);
        assert_ne!(bytes, [0u8; 32]);
    }
}
