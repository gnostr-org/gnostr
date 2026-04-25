use aes::{cipher::block_padding::Pkcs7, Aes256};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use anyhow::anyhow;
use cbc::{
    cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Decryptor, Encryptor,
};
use rand::RngCore;
use secp256k1::{ecdh, Secp256k1, SecretKey, XOnlyPublicKey};

type Aes256CbcEncryptor = Encryptor<Aes256>;
type Aes256CbcDecryptor = Decryptor<Aes256>;

/// Encrypt content
pub fn encrypt(
    sender_private_key: &SecretKey,
    recipient_public_key: &XOnlyPublicKey,
    content: &str,
) -> Result<String, anyhow::Error> {
    let _secp = Secp256k1::new();

    // NIP-04 specifies using the first 32 bytes of the sha256 of the shared secret
    // point
    let shared_secret = ecdh::shared_secret_point(
        &recipient_public_key.public_key(secp256k1::Parity::Even), // Simplified assumption
        sender_private_key,
    );
    let shared_key = &shared_secret[..32];

    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    let cipher = Aes256CbcEncryptor::new(shared_key.into(), &iv.into());
    let encrypted_content = cipher.encrypt_padded_vec_mut::<Pkcs7>(content.as_bytes());

    let iv_base64 = BASE64.encode(iv);
    let content_base64 = BASE64.encode(encrypted_content);

    Ok(format!("{}?iv={}", content_base64, iv_base64))
}

/// Decrypt content
pub fn decrypt(
    recipient_private_key: &SecretKey,
    sender_public_key: &XOnlyPublicKey,
    encrypted_content: &str,
) -> Result<String, anyhow::Error> {
    let mut parts = encrypted_content.split("?iv=");
    let content_base64 = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid encrypted content format"))?;
    let iv_base64 = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid encrypted content format: missing iv"))?;

    let iv = BASE64.decode(iv_base64)?;
    if iv.len() != 16 {
        return Err(anyhow!("Invalid IV length"));
    }
    let encrypted_bytes = BASE64.decode(content_base64)?;

    let _secp = Secp256k1::new();
    let shared_secret = ecdh::shared_secret_point(
        &sender_public_key.public_key(secp256k1::Parity::Even), // Simplified assumption
        recipient_private_key,
    );
    let shared_key = &shared_secret[..32];

    let cipher = Aes256CbcDecryptor::new(shared_key.into(), iv.as_slice().into());
    let decrypted_bytes = cipher.decrypt_padded_vec_mut::<Pkcs7>(&encrypted_bytes)?;

    Ok(String::from_utf8(decrypted_bytes)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
    use secp256k1::{Keypair, Secp256k1};

    fn test_keypair(seed: u8) -> (SecretKey, XOnlyPublicKey) {
        let secp = Secp256k1::new();
        let sk = SecretKey::from_slice(&[seed; 32]).unwrap();
        let keypair = Keypair::from_secret_key(&secp, &sk);
        let (pk, _) = XOnlyPublicKey::from_keypair(&keypair);
        (sk, pk)
    }

    #[test]
    fn decrypt_rejects_invalid_iv_length() {
        let (sender_sk, sender_pk) = test_keypair(1);
        let (recipient_sk, _) = test_keypair(2);
        let payload = format!("{}?iv={}", BASE64.encode(b"ciphertext"), BASE64.encode([7u8; 15]));

        let err = decrypt(&recipient_sk, &sender_pk, &payload).unwrap_err();
        assert!(err.to_string().contains("Invalid IV length"));
        let _ = sender_sk;
    }

    #[test]
    fn decrypt_rejects_missing_iv() {
        let (_, sender_pk) = test_keypair(1);
        let (recipient_sk, _) = test_keypair(2);

        let err = decrypt(&recipient_sk, &sender_pk, "ciphertext").unwrap_err();
        assert!(err.to_string().contains("missing iv"));
    }
}
