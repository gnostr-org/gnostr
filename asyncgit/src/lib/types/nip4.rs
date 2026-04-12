use aes::{
    Aes256,
    cipher::block_padding::Pkcs7,
};
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};
use cbc::{
    Decryptor, Encryptor,
    cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit},
};
use rand::RngCore;
use secp256k1::{Secp256k1, SecretKey, XOnlyPublicKey, ecdh};

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
