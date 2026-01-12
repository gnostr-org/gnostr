use std::str::FromStr;

use clap::{Parser, Subcommand};
use gnostr::{
    queue::InternalEvent,
    types::{
        nip18,
        nip2::{self, Contact},
        nip26, nip59, nip9, ContentEncryptionAlgorithm, EventKind, EventV3, Id, KeySigner, Nip05,
        NostrClient, PreEventV3, PrivateKey, PublicKey, Rumor, Signature, Signer, TagV3,
        UncheckedUrl, Unixtime,
    },
};
use secp256k1::XOnlyPublicKey;
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
        #[arg(long)]
        subject: Option<String>,
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
    /// Add a contact to your contact list
    AddContact {
        #[arg(short, long)]
        private_key: String,
        #[arg(short, long)]
        pubkey: String,
        #[arg(short, long)]
        relay_url: Option<String>,
        #[arg(short, long)]
        petname: Option<String>,
    },
    /// Remove a contact from your contact list
    RemoveContact {
        #[arg(short, long)]
        private_key: String,
        #[arg(short, long)]
        pubkey: String,
    },
    /// Get your contact list
    GetContacts {
        #[arg(short, long)]
        private_key: String,
    },
    /// Publish a new product to the marketplace
    MarketProduct {
        #[arg(short, long)]
        private_key: String,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        description: String,
        #[arg(short, long)]
        price: u64,
        #[arg(short, long)]
        currency: String,
    },
    /// Publish a new stall to the marketplace
    MarketStall {
        #[arg(short, long)]
        private_key: String,
        #[arg(short, long)]
        name: String,
        #[arg(short, long)]
        description: String,
    },
    /// Subscribe to marketplace events
    MarketSubscribe,
    /// Delegate event signing to another key
    Delegate {
        #[arg(short, long)]
        private_key: String,
        #[arg(short, long)]
        delegatee: String,
        #[arg(short, long)]
        event_kind: u16,
        #[arg(short, long)]
        until: Option<u64>,
        #[arg(short, long)]
        since: Option<u64>,
    },
    /// Send a NIP-17 private direct message
    SendNip17Dm {
        #[arg(short, long)]
        recipient: String,
        #[arg(short, long)]
        content: String,
    },
    /// Repost a text note (kind 1)
    RepostTextNote {
        #[arg(short, long)]
        private_key: String,
        #[arg(short, long)]
        event_id: String,
    },
    /// Repost any generic event (kind other than 1)
    RepostGeneric {
        #[arg(short, long)]
        private_key: String,
        #[arg(short, long)]
        event_id: String,
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
        SubCommand::Publish { content, subject } => {
            println!("Publishing: {}", content);
            let signer = KeySigner::from_private_key(PrivateKey::generate(), "", 1).unwrap();
            let pubkey = signer.public_key();

            let mut tags = vec![];
            if let Some(s) = subject {
                tags.push(TagV3(vec!["subject".to_string(), s]));
            }

            let preevent = PreEventV3 {
                pubkey,
                created_at: Unixtime::now(),
                kind: EventKind::TextNote,
                tags,
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
            let signer = KeySigner::from_private_key(PrivateKey::generate(), "", 1).unwrap();
            let recipient_pk = PublicKey::try_from_hex_string(&recipient, true)?;
            let encrypted_content =
                signer.encrypt(&recipient_pk, &content, ContentEncryptionAlgorithm::Nip04)?;
            let pubkey = signer.public_key();
            let preevent = PreEventV3 {
                pubkey,
                created_at: Unixtime::now(),
                kind: EventKind::EncryptedDirectMessage,
                tags: vec![TagV3::new_pubkey(recipient_pk, None, None)],
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
            client.send_event(event).await?;
        }
        SubCommand::AddContact {
            private_key,
            pubkey,
            relay_url,
            petname,
        } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let public_key = pk.public_key();
            let secret_key = pk.as_secret_key();

            // TODO: Fetch current contact list
            let mut contacts: Vec<Contact> = vec![];

            let new_contact_pk = XOnlyPublicKey::from_str(&pubkey)?;
            contacts.push(Contact {
                public_key: new_contact_pk,
                relay_url,
                petname,
            });

            let event =
                nip2::set_contact_list(contacts, &public_key.as_xonly_public_key(), &secret_key);
            client.send_event(event).await?;
        }
        SubCommand::RemoveContact {
            private_key,
            pubkey,
        } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let public_key = pk.public_key();
            let secret_key = pk.as_secret_key();

            // TODO: Fetch current contact list
            let mut contacts: Vec<Contact> = vec![];

            let remove_pk = XOnlyPublicKey::from_str(&pubkey)?;
            contacts.retain(|c| c.public_key != remove_pk);

            let event =
                nip2::set_contact_list(contacts, &public_key.as_xonly_public_key(), &secret_key);
            client.send_event(event).await?;
        }
        SubCommand::GetContacts { private_key } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let signer = KeySigner::from_private_key(pk, "", 1).unwrap();
            let pubkey = signer.public_key();
            println!("Getting contacts for {}", pubkey.as_hex_string());
            client.subscribe_to_contact_lists(pubkey).await;
        }
        SubCommand::MarketProduct {
            private_key,
            name,
            description,
            price,
            currency,
        } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let signer = KeySigner::from_private_key(pk, "", 1).unwrap();
            let pubkey = signer.public_key();

            let content = serde_json::json!({
                "name": name,
                "description": description,
                "price": price,
                "currency": currency,
            })
            .to_string();

            let preevent = PreEventV3 {
                pubkey,
                created_at: Unixtime::now(),
                kind: EventKind::MarketplaceUi,
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
        SubCommand::MarketStall {
            private_key,
            name,
            description,
        } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let signer = KeySigner::from_private_key(pk, "", 1).unwrap();
            let pubkey = signer.public_key();

            let content = serde_json::json!({
                "name": name,
                "description": description,
            })
            .to_string();

            let preevent = PreEventV3 {
                pubkey,
                created_at: Unixtime::now(),
                kind: EventKind::MarketplaceUi, /* This should be a stall kind, but MarketplaceUi
                                                 * is the only one available */
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
        SubCommand::MarketSubscribe => {
            client.subscribe_to_marketplace().await;
        }
        SubCommand::Delegate {
            private_key,
            delegatee,
            event_kind,
            until,
            since,
        } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let signer = KeySigner::from_private_key(pk.clone(), "", 1).unwrap();
            let public_key = signer.public_key();
            let secret_key = pk.as_secret_key();

            let delegatee_pk = XOnlyPublicKey::from_str(&delegatee)?;

            let delegation = nip26::Delegation {
                delegator: public_key.as_xonly_public_key(),
                delegatee: delegatee_pk,
                event_kind,
                until,
                since,
            };

            let tag = delegation.create_tag(&secret_key)?;

            let preevent = PreEventV3 {
                pubkey: public_key,
                created_at: Unixtime::now(),
                kind: EventKind::TextNote, /* NIP-26 is a tag, not a kind. Using TextNote as
                                            * placeholder. */
                tags: vec![TagV3(tag.split(' ').map(|s| s.to_string()).collect())],
                content: "Delegation proof".to_string(),
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
        SubCommand::SendNip17Dm { recipient, content } => {
            let sender_private_key = PrivateKey::generate();
            let recipient_pk = PublicKey::try_from_hex_string(&recipient, true)?;

            let rumor = Rumor {
                id: Id::default(), // This will be replaced by the create_seal function
                pubkey: sender_private_key.public_key(),
                created_at: Unixtime::now(),
                kind: EventKind::TextNote, /* NIP-17 says Kind 14 for chat messages, but NIP-59
                                            * wraps generic rumors. Use TextNote for the inner
                                            * rumor. */
                tags: vec![],
                content,
            };

            let seal_event = nip59::create_seal(rumor, &sender_private_key, &recipient_pk)?;

            let gift_wrap_event =
                nip59::create_gift_wrap(seal_event, &sender_private_key, &recipient_pk)?;

            client.send_event(gift_wrap_event).await?;
        }
        SubCommand::RepostTextNote {
            private_key,
            event_id,
        } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            // TODO: Fetch the event to be reposted. For now, create a dummy event.
            println!("Reposting text note {}", event_id);
            let dummy_event = EventV3 {
                id: Id::try_from_hex_string(&event_id)?,
                pubkey: pk.public_key(),
                created_at: Unixtime::now(),
                kind: EventKind::TextNote,
                sig: Signature::zeroes(),
                content: "".to_string(),
                tags: vec![],
            };
            let repost_event = nip18::create_repost_text_note(
                &dummy_event,
                &pk.public_key().as_xonly_public_key(),
                &pk.as_secret_key(),
            )?;
            client.send_event(repost_event).await?;
        }
        SubCommand::RepostGeneric {
            private_key,
            event_id,
        } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            // TODO: Fetch the event to be reposted. For now, create a dummy event.
            println!("Reposting generic event {}", event_id);
            let dummy_event = EventV3 {
                id: Id::try_from_hex_string(&event_id)?,
                pubkey: pk.public_key(),
                created_at: Unixtime::now(),
                kind: EventKind::TextNote, // Assume TextNote for dummy
                sig: Signature::zeroes(),
                content: "".to_string(),
                tags: vec![],
            };
            let repost_event = nip18::create_generic_repost(
                &dummy_event,
                &pk.public_key().as_xonly_public_key(),
                &pk.as_secret_key(),
            )?;
            client.send_event(repost_event).await?;
        }
    }

    if should_listen {
        println!("Listening for events...");

        while let Some(internal_event) = rx.recv().await {
            if let InternalEvent::NostrEvent(event) = internal_event {
                if event.kind == EventKind::EncryptedDirectMessage {
                    if let Some(signer) = &signer_for_decryption {
                        let decrypted = signer.decrypt(&event.pubkey, &event.content)?;
                        println!("DM from {}: {}", event.pubkey.as_hex_string(), decrypted);
                    }
                } else if event.kind == EventKind::ContactList {
                    println!("Contact list updated:");
                    for tag in &event.tags {
                        if tag.tagname() == "p" {
                            let v: Vec<&str> = tag.value().split(' ').collect();
                            let pubkey = v.first().unwrap_or(&"");
                            let relay = v.get(1).unwrap_or(&"");
                            let petname = v.get(2).unwrap_or(&"");
                            println!(
                                "  pubkey: {}, relay: {}, petname: {}",
                                pubkey, relay, petname
                            );
                        }
                    }
                } else if event.kind == EventKind::GiftWrap {
                    if let Some(signer) = &signer_for_decryption {
                        // Unwrap GiftWrap to get the Seal
                        let seal_json = signer.decrypt(&event.pubkey, &event.content)?;
                        let seal_event: EventV3 = serde_json::from_str(&seal_json)?;

                        // Unwrap Seal to get the Rumor
                        let rumor_json = signer.decrypt(&seal_event.pubkey, &seal_event.content)?;
                        let rumor: Rumor = serde_json::from_str(&rumor_json)?;

                        println!(
                            "NIP-17 DM from {}: {}",
                            seal_event.pubkey.as_hex_string(),
                            rumor.content
                        );
                    }
                } else if event.kind == EventKind::MarketplaceUi {
                    println!("Marketplace event: {:?}", event);
                } else {
                    println!("Received event: {:?}", event);
                }
            }
        }
    }

    Ok(())
}
