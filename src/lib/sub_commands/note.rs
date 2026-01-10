use crate::utils::{create_client, parse_private_key};
use clap::Args;
use anyhow::{Result, Error as AnyhowError};
use crate::types::{Client, Event, EventKind, Id, Keys, PublicKey, Tag, PreEventV3, Unixtime, KeySigner, Signer};
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;
use tracing::trace;

#[derive(Args, Debug)]
pub struct NoteSubCommand {
    /// Text note content
    #[arg(short, long)]
    content: String,
    /// Subject tag (NIP-14)
    #[arg(short, long)]
    subject: Option<String>,
    /// Pubkey references. Both hex and bech32 encoded keys are supported.
    #[arg(long, action = clap::ArgAction::Append)]
    ptag: Vec<String>,
    /// Event references
    #[arg(long, action = clap::ArgAction::Append)]
    etag: Vec<String>,
    #[arg(short, long, action = clap::ArgAction::Append)]
    tag: Vec<String>,
    /// Seconds till expiration (NIP-40)
    #[arg(long)]
    expiration: Option<u64>,
    // Print event id as hex
    #[arg(long, default_value = "false")]
    hex: bool,
    #[arg(long, default_value = "false")]
    verbose: bool,
}

pub async fn broadcast_textnote(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &NoteSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    // Set up tags
    let mut tags: Vec<Tag> = vec![];

    // Subject tag (NIP-14)
    if let Some(subject) = &sub_command_args.subject {
        tags.push(Tag::new(&["subject", subject]));
    }

    // Add p-tags
    for ptag in sub_command_args.ptag.iter() {
        let public_key = PublicKey::try_from_hex_string(ptag, true)?;
        tags.push(Tag::new(&["p", &public_key.as_hex_string()]));
    }
    // Add e-tags
    for etag in sub_command_args.etag.iter() {
        let event_id = Id::try_from_hex_string(etag)?;
        tags.push(Tag::new(&["e", &event_id.as_hex_string()]));
    }
    // Add tags
    for tag in sub_command_args.tag.iter() {
        tags.push(Tag::new(&["t", tag]));
    }

    for tag in &tags {
        trace!("{:?}", tag);
    }

    // Set expiration tag
    if let Some(expiration) = sub_command_args.expiration {
        let timestamp = Unixtime::now().0 + expiration as i64;
        tags.push(Tag::new(&["expiration", &timestamp.to_string()]));
    }

    if sub_command_args.verbose {
        println!("{}", sub_command_args.content.clone());
    }
    
    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::TextNote,
        tags,
        content: sub_command_args.content.clone(),
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    // Publish event
    let event_id = client.send_event(event).await?;
    
    if sub_command_args.hex {
        print!("{{\"id\":\"{}\"}}", event_id.as_hex_string());
    } else {
        print!("{{\"id\":\"{}\"}}", event_id.as_bech32_string());
    }
    std::process::exit(0);
    #[allow(unreachable_code)]
    Ok(())
}
