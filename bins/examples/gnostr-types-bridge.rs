#![allow(clippy::uninlined_format_args)]

use gnostr::types::nostr::bridge::{decrypt_dm, encrypt_dm, encrypt_dm_with_algorithm};
use gnostr::types::ContentEncryptionAlgorithm;

fn main() {
    let sender_sk_hex = "0101010101010101010101010101010101010101010101010101010101010101";
    let recipient_sk_hex = "0202020202020202020202020202020202020202020202020202020202020202";

    let sender = gnostr::types::PrivateKey::try_from_hex_string(sender_sk_hex)
        .expect("valid sender key");
    let recipient = gnostr::types::PrivateKey::try_from_hex_string(recipient_sk_hex)
        .expect("valid recipient key");

    let sender_pk_hex = sender.public_key().as_hex_string();
    let recipient_pk_hex = recipient.public_key().as_hex_string();

    let plaintext = "hello from the types bridge";

    let nip04_ciphertext = encrypt_dm(sender_sk_hex, &recipient_pk_hex, plaintext)
        .expect("nip04 encrypt");
    let nip04_plaintext = decrypt_dm(recipient_sk_hex, &sender_pk_hex, &nip04_ciphertext)
        .expect("nip04 decrypt");

    let nip44_ciphertext = encrypt_dm_with_algorithm(
        sender_sk_hex,
        &recipient_pk_hex,
        plaintext,
        ContentEncryptionAlgorithm::Nip44v2,
    )
    .expect("nip44 encrypt");
    let nip44_plaintext = decrypt_dm(recipient_sk_hex, &sender_pk_hex, &nip44_ciphertext)
        .expect("nip44 decrypt");

    println!("sender pubkey: {sender_pk_hex}");
    println!("recipient pubkey: {recipient_pk_hex}");
    println!("nip04 ciphertext: {nip04_ciphertext}");
    println!("nip04 plaintext: {nip04_plaintext}");
    println!("nip44 ciphertext: {nip44_ciphertext}");
    println!("nip44 plaintext: {nip44_plaintext}");
}
