use std::str::FromStr;

use clap::{Parser, Subcommand};
use gnostr::{
    queue::InternalEvent,
};
use gnostr_types::{ContentEncryptionAlgorithm, EventKind, EventV3, Id, KeySigner, Nip05V1,
        PreEventV3, PrivateKey, PublicKey, Rumor, Signature, Signer, TagV3, UncheckedUrl, Unixtime, Client, Keys, Options,
};
use nostr::nips::{nip02::{self, Contact}, nip09, nip18, nip26, nip59};
use nostr::{EventBuilder, Event as NostrEvent, UnsignedEvent, Timestamp, Kind, Keys as NostrKeys, Options as NostrOptions};

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
        private_key: String,
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

    // Generate a new keypair for the client
    let keys = Keys::generate();
    // Create default client options
    let options = Options::new();
    let mut client = Client::new(&keys, options);

    let relay_url = UncheckedUrl(args.relay_url);
    client.add_relays(vec![relay_url.to_string()]).await?;
    client.connect().await;

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
            // client.subscribe_to_channel(id).await;
            println!("Channel subscription not implemented in new client API.");
        }
        SubCommand::Subscribe { pubkey } => {
            if let Some(pk_str) = pubkey {
                let pk = PublicKey::try_from_hex_string(&pk_str, true)?;
                println!("Subscribing to pubkey: {}", pk.as_hex_string());
                // client.subscribe(Some(pk)).await;
                println!("Subscription by pubkey not implemented in new client API.");
            } else {
                println!("Subscribing to all text notes");
                // client.subscribe(None).await;
                println!("General subscription not implemented in new client API.");
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
            let nip05: Nip05V1 = reqwest::get(&url).await?.json().await?;
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
            // client.subscribe_to_dms(pubkey).await;
            println!("DM subscription not implemented in new client API.");
            signer_for_decryption = Some(signer);
        }
        SubCommand::Delete { event_id, reason } => {
            let private_key = PrivateKey::generate();
            let public_key = private_key.public_key();
            let secret_key = private_key.as_secret_key();

            let id = Id::try_from_hex_string(&event_id)?;
            let event = nip09::delete(
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
                nip02::set_contact_list(contacts, &public_key.as_xonly_public_key(), &secret_key);
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
                nip02::set_contact_list(contacts, &public_key.as_xonly_public_key(), &secret_key);
            client.send_event(event).await?;
        }
        SubCommand::GetContacts { private_key } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let signer = KeySigner::from_private_key(pk, "", 1).unwrap();
            let pubkey = signer.public_key();
            println!("Getting contacts for {}", pubkey.as_hex_string());
            // client.subscribe_to_contact_lists(pubkey).await;
            println!("Contact list subscription not implemented in new client API. Use set_contact_list for setting.");
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
            // client.subscribe_to_marketplace().await;
            println!("Marketplace subscription not implemented in new client API.");
        }
        SubCommand::Delegate {
            private_key,
            delegatee,
            event_kind,
            until,
            since,
        } => {
            let pk = PrivateKey::try_from_hex_string(&private_key)?;
            let delegator_keys = NostrKeys::new(pk.to_secp_private_key()); // Convert gnostr_types::PrivateKey to nostr::Keys
            let delegatee_pk_xonly = XOnlyPublicKey::from_str(&delegatee)?;

            let mut conditions = nip26::Conditions::new();
            conditions = conditions.kind(Kind::from(event_kind));
            if let Some(s) = since {
                conditions = conditions.created_at_after(Timestamp::from(s as i64));
            }
            if let Some(u) = until {
                conditions = conditions.created_at_before(Timestamp::from(u as i64));
            }

            let delegation_tag = nip26::create_delegation_tag(
                &delegator_keys,
                &delegatee_pk_xonly,
                conditions,
            )?;

            let preevent = PreEventV3 {
                pubkey: pk.public_key(),
                created_at: Unixtime::now(),
                kind: EventKind::TextNote, /* NIP-26 is a tag, not a kind. Using TextNote as
                                            * placeholder. */
                tags: vec![TagV3(delegation_tag.to_string().split(' ').map(|s| s.to_string()).collect())], // Convert DelegationTag to TagV3
                content: "Delegation proof".to_string(),
            };

            let id = preevent.hash()?;
            let sig = KeySigner::from_private_key(pk, "", 1).unwrap().sign_id(id)?;
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
        SubCommand::SendNip17Dm { private_key, recipient, content } => {
            let sender_private_key_gnostr = PrivateKey::try_from_hex_string(&private_key)?;
            let sender_private_key = NostrKeys::new(sender_private_key_gnostr.to_secp_private_key());
            let recipient_pk = PublicKey::try_from_hex_string(&recipient, true)?;
            let recipient_pk_xonly = recipient_pk.as_xonly_public_key();

            let rumor_unsigned_event = EventBuilder::new(
                Kind::TextNote,
                content,
                &[], // No tags for rumor
            )
            .to_unsigned_event(sender_private_key.public_key());

            let seal_event = nip59::create_seal(
                &sender_private_key,
                &recipient_pk_xonly,
                rumor_unsigned_event,
            )?;

            let gift_wrap_event = nip59::create_gift_wrap(
                &sender_private_key,
                &recipient_pk_xonly,
                &seal_event,
            )?;

            client.send_event(gift_wrap_event.into()).await?; // Convert nostr::Event to gnostr_types::EventV3
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
