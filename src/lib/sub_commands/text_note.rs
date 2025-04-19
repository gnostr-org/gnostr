use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;

use clap::Args;
use nostr_sdk_0_32_0::prelude::*;

use crate::utils::{create_client, parse_private_key};

#[derive(Args)]
pub struct TextNoteSubCommand {
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
    #[arg(long, action = clap::ArgAction::Append)]
    tag: Vec<String>,
    /// Seconds till expiration (NIP-40)
    #[arg(long)]
    expiration: Option<u64>,
    // Print event id as hex
    #[arg(long, default_value = "false")]
    hex: bool,
}

pub async fn broadcast_textnote(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &TextNoteSubCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    // Set up tags
    let mut tags: Vec<Tag> = vec![];

    // Subject tag (NIP-14)
    if let Some(subject) = &sub_command_args.subject {
        let subject_tag = Tag::custom(TagKind::Subject, vec![subject]);
        tags.push(subject_tag);
    }

    // Add p-tags
    for ptag in sub_command_args.ptag.iter() {
        // Parse pubkey to ensure we're sending hex keys
        let public_key = PublicKey::from_str(ptag.as_str())?;
        tags.push(Tag::public_key(public_key));
    }
    // Add e-tags
    for etag in sub_command_args.etag.iter() {
        let event_id = EventId::from_hex(etag)?;
        tags.push(Tag::event(event_id));
    }
    // Add tags
    for tag in sub_command_args.tag.iter() {
		//
        let hashtag = Tag::hashtag(tag);
        tags.push(hashtag);
    }

	for tag in &tags {
		println!("{:?}", tag);
	}

    // Set expiration tag
    if let Some(expiration) = sub_command_args.expiration {
        let timestamp = Timestamp::now().add(Duration::from_secs(expiration));
        tags.push(Tag::expiration(timestamp));
    }

    // Publish event
    let event_id = client
        .publish_text_note(sub_command_args.content.clone(), tags)
        .await?;
    if sub_command_args.hex {
        print!("{{\"id\":\"{}\"}}", event_id.to_hex());
    } else {
        print!("{{\"id\":\"{}\"}}", event_id.to_bech32()?);
    }
    std::process::exit(0);
    #[allow(unreachable_code)]
    Ok(())
}
