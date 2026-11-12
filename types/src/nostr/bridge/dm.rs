use super::super::{ContentEncryptionAlgorithm, Error, PrivateKey, PublicKey};

fn parse_private_key(hex_key: &str) -> Result<PrivateKey, Error> {
    PrivateKey::try_from_hex_string(hex_key)
}

fn parse_public_key(hex_key: &str) -> Result<PublicKey, Error> {
    PublicKey::try_from_hex_string(hex_key, true)
}

/// Encrypt a DM using the canonical Nostr content-encryption implementation.
///
/// The sender key must be a 32-byte hex secret key, and the recipient key must
/// be the recipient's x-only public key encoded as hex.
///
/// # Example
/// ```ignore
/// use gnostr_types::nostr::bridge::encrypt_dm;
///
/// let ciphertext = encrypt_dm(sender_sk_hex, recipient_pk_hex, "hello")?;
/// # Ok::<_, gnostr_types::nostr::Error>(())
/// ```
pub fn encrypt_dm(
    sender_private_key_hex: &str,
    recipient_public_key_hex: &str,
    plaintext: &str,
) -> Result<String, Error> {
    encrypt_dm_with_algorithm(
        sender_private_key_hex,
        recipient_public_key_hex,
        plaintext,
        ContentEncryptionAlgorithm::Nip04,
    )
}

/// Encrypt a DM with an explicit content-encryption algorithm.
///
/// Use this when you need to force NIP-04 or NIP-44 v2 rather than relying on
/// the default DM helper.
pub fn encrypt_dm_with_algorithm(
    sender_private_key_hex: &str,
    recipient_public_key_hex: &str,
    plaintext: &str,
    algorithm: ContentEncryptionAlgorithm,
) -> Result<String, Error> {
    let sender = parse_private_key(sender_private_key_hex)?;
    let recipient = parse_public_key(recipient_public_key_hex)?;
    sender.encrypt(&recipient, plaintext, algorithm)
}

/// Decrypt a DM using the canonical Nostr content-encryption implementation.
///
/// The recipient key must be the recipient's 32-byte hex secret key, and the
/// sender key must be the sender's x-only public key encoded as hex.
///
/// The cipher version is auto-detected by the underlying `PrivateKey::decrypt`
/// implementation, so the caller only needs to provide the key pair.
///
/// # Example
/// ```ignore
/// use gnostr_types::nostr::bridge::decrypt_dm;
///
/// let plaintext = decrypt_dm(recipient_sk_hex, sender_pk_hex, &ciphertext)?;
/// # Ok::<_, gnostr_types::nostr::Error>(())
/// ```
pub fn decrypt_dm(
    recipient_private_key_hex: &str,
    sender_public_key_hex: &str,
    ciphertext: &str,
) -> Result<String, Error> {
    let recipient = parse_private_key(recipient_private_key_hex)?;
    let sender = parse_public_key(sender_public_key_hex)?;
    recipient.decrypt(&sender, ciphertext)
}

#[cfg(test)]
mod tests {
    use super::{decrypt_dm, encrypt_dm, encrypt_dm_with_algorithm};
    use crate::nostr::ContentEncryptionAlgorithm;

    fn private_key_hex(seed: u8) -> String {
        hex::encode([seed; 32])
    }

    #[test]
    fn nip04_round_trip() {
        let sender_hex = private_key_hex(1);
        let recipient_hex = private_key_hex(2);
        let recipient = crate::nostr::PrivateKey::try_from_hex_string(&recipient_hex).unwrap();
        let sender = crate::nostr::PrivateKey::try_from_hex_string(&sender_hex).unwrap();
        let recipient_pubkey_hex = recipient.public_key().as_hex_string();
        let sender_pubkey_hex = sender.public_key().as_hex_string();

        let ciphertext = encrypt_dm(&sender_hex, &recipient_pubkey_hex, "hello dm").unwrap();
        let plaintext = decrypt_dm(&recipient_hex, &sender_pubkey_hex, &ciphertext).unwrap();

        assert_eq!(plaintext, "hello dm");
    }

    #[test]
    fn nip44_v2_round_trip() {
        let sender_hex = private_key_hex(3);
        let recipient_hex = private_key_hex(4);
        let recipient = crate::nostr::PrivateKey::try_from_hex_string(&recipient_hex).unwrap();
        let sender = crate::nostr::PrivateKey::try_from_hex_string(&sender_hex).unwrap();
        let recipient_pubkey_hex = recipient.public_key().as_hex_string();
        let sender_pubkey_hex = sender.public_key().as_hex_string();

        let ciphertext = encrypt_dm_with_algorithm(
            &sender_hex,
            &recipient_pubkey_hex,
            "hello rust bridge",
            ContentEncryptionAlgorithm::Nip44v2,
        )
        .unwrap();
        let plaintext = decrypt_dm(&recipient_hex, &sender_pubkey_hex, &ciphertext).unwrap();

        assert_eq!(plaintext, "hello rust bridge");
    }
}
