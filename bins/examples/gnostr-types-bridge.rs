#![allow(clippy::uninlined_format_args)]

use clap::Parser;
use gnostr::types::bridge::{decrypt_dm, encrypt_dm, encrypt_dm_with_algorithm};
use gnostr::types::{ContentEncryptionAlgorithm, PrivateKey};

#[derive(Parser, Debug)]
#[command(author, version, about = "Demonstrate the gnostr-types bridge helpers")]
struct BridgeArgs {
    /// Nostr secret key used to encrypt the message
    #[arg(
        long = "nsec",
        default_value = "0000000000000000000000000000000000000000000000000000000000000001",
        help = "Sender secret key (hex or bech32)"
    )]
    nsec: String,

    /// Public key of the recipient
    #[arg(long, help = "Recipient public key (hex or bech32)")]
    recipient: String,

    /// Message content to encrypt
    #[arg(short, long, help = "Message to encrypt and round-trip through the bridge")]
    message: Option<String>,

    /// Relay URL used by the DM CLI; accepted here for parity
    #[arg(short, long, action = clap::ArgAction::Append, help = "Relay URL (accepted for parity with the DM CLI)")]
    relay: Vec<String>,

    /// Optional recipient secret key for the round-trip demo
    #[arg(long = "recipient-nsec", help = "Recipient secret key for the round-trip demo")]
    recipient_nsec: Option<String>,

    /// Limit used by the DM CLI inbox mode; accepted here for parity
    #[arg(short, long, help = "Inbox limit (accepted for parity with the DM CLI)")]
    limit: Option<i32>,

    /// Print the payload as JSON-style output
    #[arg(long, help = "Print bridge output as JSON-style output")]
    json: bool,

    /// Print extra bridge details
    #[arg(short = 'v', long = "verbose", action = clap::ArgAction::Count, help = "Print extra bridge details")]
    verbose: u8,
}

fn parse_private_key(value: &str) -> PrivateKey {
    PrivateKey::try_from_hex_string(value)
        .or_else(|_| PrivateKey::try_from_bech32_string(value))
        .expect("valid private key")
}

fn parse_public_key(value: &str) -> gnostr::types::PublicKey {
    gnostr::types::PublicKey::try_from_bech32_string(value, true)
        .or_else(|_| gnostr::types::PublicKey::try_from_hex_string(value, true))
        .expect("valid public key")
}

fn main() {
    let args = BridgeArgs::parse();

    let sender = parse_private_key(&args.nsec);
    let recipient = parse_public_key(&args.recipient);
    let sender_pk_hex = sender.public_key().as_hex_string();
    let recipient_pk_hex = recipient.as_hex_string();

    if args.verbose > 0 {
        eprintln!("recipient: {recipient_pk_hex}");
        eprintln!("relay count: {}", args.relay.len());
        eprintln!("limit: {:?}", args.limit);
    }

    let Some(message) = args.message.as_deref() else {
        println!("No --message supplied; this example only demonstrates encrypt/decrypt bridge usage.");
        return;
    };

    let nip04_ciphertext = encrypt_dm(&args.nsec, &recipient_pk_hex, message).expect("nip04 encrypt");

    let nip44_ciphertext = encrypt_dm_with_algorithm(
        &args.nsec,
        &recipient_pk_hex,
        message,
        ContentEncryptionAlgorithm::Nip44v2,
    )
    .expect("nip44 encrypt");

    let roundtrip = args.recipient_nsec.as_deref().map(|recipient_nsec| {
        let mut recipient_sk = parse_private_key(recipient_nsec);
        let recipient_sk_hex = recipient_sk.as_hex_string();
        let recipient_pk_from_sk_hex = recipient_sk.public_key().as_hex_string();
        assert_eq!(
            recipient_pk_from_sk_hex, recipient_pk_hex,
            "--recipient-nsec must match --recipient"
        );

        let nip04_plaintext = decrypt_dm(&recipient_sk_hex, &sender_pk_hex, &nip04_ciphertext)
            .expect("nip04 decrypt");
        let nip44_plaintext = decrypt_dm(&recipient_sk_hex, &sender_pk_hex, &nip44_ciphertext)
            .expect("nip44 decrypt");
        (nip04_plaintext, nip44_plaintext)
    });

    if args.json {
        if let Some((nip04_plaintext, nip44_plaintext)) = roundtrip {
            println!(
                "{{\"recipient\":\"{}\",\"nip04_ciphertext\":\"{}\",\"nip04_plaintext\":\"{}\",\"nip44_ciphertext\":\"{}\",\"nip44_plaintext\":\"{}\"}}",
                recipient_pk_hex, nip04_ciphertext, nip04_plaintext, nip44_ciphertext, nip44_plaintext
            );
        } else {
            println!(
                "{{\"recipient\":\"{}\",\"nip04_ciphertext\":\"{}\",\"nip44_ciphertext\":\"{}\"}}",
                recipient_pk_hex, nip04_ciphertext, nip44_ciphertext
            );
        }
    } else {
        println!("sender pubkey: {sender_pk_hex}");
        println!("recipient pubkey: {recipient_pk_hex}");
        println!("nip04 ciphertext: {nip04_ciphertext}");
        println!("nip44 ciphertext: {nip44_ciphertext}");
        if let Some((nip04_plaintext, nip44_plaintext)) = roundtrip {
            println!("nip04 plaintext: {nip04_plaintext}");
            println!("nip44 plaintext: {nip44_plaintext}");
        }
    }
}
