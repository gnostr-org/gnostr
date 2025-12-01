use clap::{Parser, Subcommand};
use gnostr::queue::InternalEvent;
use gnostr::types::{
    EventKind, KeySigner, NostrClient, PreEventV3, PrivateKey, Signer, UncheckedUrl, Unixtime, EventV3, PublicKey, Nip05, TagV3, ContentEncryptionAlgorithm, Id
};
use gnostr::types::nip9;
use tokio::sync::mpsc;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, default_value = "wss://relay.damus.io")]
    relay_url: String,
    #[command(subcommand)]
    command: SubCommand,
}

#[derive(Subcommand, Debug)]
enum SubCommand {
    /// Publish a text note
    Publish {
        #[arg(short, long)]
        content: String,
    },
    /// Subscribe to a channel
    Channel {
        #[arg(short, long, default_value = "test")]
        id: String,
    },
    /// Subscribe to text notes
    Subscribe {
        #[arg(short, long)]
        pubkey: Option<String>,
    },
    /// Resolve a NIP-05 identifier
    Nip05 {
        #[arg(short, long)]
        identifier: String,
    },
    /// Send a direct message
    SendDm {
        #[arg(short, long)]
        recipient: String,
        #[arg(short, long)]
        content: String,
    },
    /// Get direct messages
    GetDms {
        #[arg(short, long)]
        private_key: String,
    },
    /// Delete an event
    Delete {
        #[arg(short, long)]
        event_id: String,
        #[arg(short, long)]
        reason: Option<String>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let (tx, mut rx) = mpsc::channel(100);

    let mut client = NostrClient::new(tx);

    let relay_url = UncheckedUrl(args.relay_url);
    client.connect_relay(relay_url).await?;

    let mut should_listen = true;
    let mut signer_for_decryption: Option<KeySigner> = None;

    match args.command {
        SubCommand::Publish { content } => {
            println!("Publishing: {}", content);
            let signer =
                KeySigner::from_private_key(PrivateKey::generate(), "", 1).unwrap();
            let pubkey = signer.public_key();
            let preevent = PreEventV3 {
                pubkey,
                created_at: Unixtime::now(),
                kind: EventKind::TextNote,
                tags: vec![],
                content,
            };
            let id = preevent.hash()?;
            let sig = signer.sign_id(id)?;
            let event = EventV3 {
                id,
                pubkey: preevent.pubkey,
                created_at: preevent.created_at,
                kind: preevent.kind,
                tags: preevent.tags,
                content: preevent.content,
                sig,
            };
            client.send_event(event).await?;
        }
        SubCommand::Channel { id } => {
            client.subscribe_to_channel(id).await;
        }
        SubCommand::Subscribe { pubkey } => {
            if let Some(pk_str) = pubkey {
                let pk = PublicKey::try_from_hex_string(&pk_str, true)?;
                println!("Subscribing to pubkey: {}", pk.as_hex_string());
                client.subscribe(Some(pk)).await;
            } else {
                println!("Subscribing to all text notes");
                client.subscribe(None).await;
            }
        }
        SubCommand::Nip05 { identifier } => {
            should_listen = false;
            let parts: Vec<&str> = identifier.split('@').collect();
            if parts.len() != 2 {
                println!("Invalid NIP-05 identifier");
                return Ok(());
            }
            let user = parts[0];
            let domain = parts[1];
            let url = format!("https://{}/.well-known/nostr.json?name={}", domain, user);
            let nip05: Nip05 = reqwest::get(&url).await?.json().await?;
            if let Some(pubkey) = nip05.names.get(user) {
                println!("Public key for {}: {}", identifier, pubkey);
            } else {
                println!("User {} not found at {}", user, domain);
            }
        }
        SubCommand::SendDm { recipient, content } => {
            let signer =
                KeySigner::from_private_key(PrivateKey::generate(), "", 1).unwrap();
            let recipient_pk = PublicKey::try_from_hex_string(&recipient, true)?;
            let encrypted_content = signer.encrypt(
                &recipient_pk,
                &content,
                ContentEncryptionAlgorithm::Nip04,
            )?;
            let pubkey = signer.public_key();
            let preevent = PreEventV3 {
                pubkey,
                created_at: Unixtime::now(),
                kind: EventKind::EncryptedDirectMessage,
                tags: vec![TagV3::new_pubkey(recipient_pk.into(), None, None)],
                content: encrypted_content,
            };
            let id = preevent.hash()?;
            let sig = signer.sign_id(id)?;
            let event = EventV3 {
                id,
                pubkey: preevent.pubkey,
                created_at: preevent.created_at,
                kind: preevent.kind,
                tags: preevent.tags,
                content: preevent.content,
                sig,
            };
            client.send_event(event).await?;
        }
        SubCommand::GetDms { private_key } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let signer = KeySigner::from_private_key(pk, "", 1).unwrap();
            let pubkey = signer.public_key();
            println!("Subscribing to DMs for {}", pubkey.as_hex_string());
            client.subscribe_to_dms(pubkey).await;
            signer_for_decryption = Some(signer);
        }
        SubCommand::Delete { event_id, reason } => {
            let private_key = PrivateKey::generate();
            let public_key = private_key.public_key();
            let secret_key = private_key.as_secret_key();

            let id = Id::try_from_hex_string(&event_id)?;
            let event = nip9::delete(
                vec![id],
                reason.as_deref(),
                &public_key.as_xonly_public_key(),
                &secret_key,
            );
            client.send_event(event.into()).await?;
        }
    }

    if should_listen {
        println!("Listening for events...");

        while let Some(internal_event) = rx.recv().await {
            match internal_event {
                InternalEvent::NostrEvent(event) => {
                    if event.kind == EventKind::EncryptedDirectMessage {
                        if let Some(signer) = &signer_for_decryption {
                            let decrypted = signer.decrypt(&event.pubkey, &event.content)?;
                            println!("DM from {}: {}", event.pubkey.as_hex_string(), decrypted);
                        }
                    } else {
                        println!("Received event: {:?}", event);
                    }
                }
                _ => {}
            }
        }
    }

    Ok(())
}