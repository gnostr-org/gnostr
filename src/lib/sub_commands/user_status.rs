use std::{ops::Add, str::FromStr, time::Duration};

use anyhow::{Error as AnyhowError, Result};
use clap::Args;

use gnostr_asyncgit::types::{Client, Event, EventKind, Id, KeySigner, Keys, PreEventV3, PublicKey, Signer, Tag, Unixtime};
use crate::utils::{create_client, parse_key_or_id_to_hex_string, parse_private_key};

#[derive(Args, Debug)]
pub struct UserStatusSubCommand {
    /// Text note content
    #[arg(short, long)]
    content: String,
    /// Status type (identifier tag)
    #[arg(short, long)]
    status_type: Option<String>,
    /// Pubkey references. Both hex and bech32 encoded keys are supported.
    #[arg(short, long)]
    ptag: Option<String>,
    /// Event references. Both hex and bech32 encoded keys are supported.
    #[arg(short, long)]
    etag: Option<String>,
    /// Reference tag
    #[arg(short, long)]
    rtag: Option<String>,
    /// Seconds till expiration (NIP-40)
    #[arg(long)]
    expiration: Option<u64>,
    // Print keys as hex
    #[arg(long, default_value = "false")]
    hex: bool,
}

pub async fn set_user_status(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &UserStatusSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    // Set up tags
    let mut tags: Vec<Tag> = vec![];

    // Add identifier tag
    if let Some(status) = &sub_command_args.status_type {
        tags.push(Tag::new(&["d", status]));
    }

    // Add expiration tag
    if let Some(expiration) = sub_command_args.expiration {
        let timestamp = Unixtime::now().0 + expiration as i64;
        tags.push(Tag::new(&["expiration", &timestamp.to_string()]));
    }

    // Add p-tag
    if let Some(p) = sub_command_args.ptag.clone() {
        let pubkey_hex = parse_key_or_id_to_hex_string(p).await?;
        let pubkey: gnostr_asyncgit::types::PublicKey = gnostr_asyncgit::types::PublicKey::try_from_hex_string(&pubkey_hex, true)?;
        tags.push(Tag::new(&["p", &pubkey.as_hex_string()]));
    }

    // Add e-tag
    if let Some(e) = sub_command_args.etag.clone() {
        let event_id_hex = parse_key_or_id_to_hex_string(e).await?;
        let event_id: gnostr_asyncgit::types::Id = gnostr_asyncgit::types::Id::try_from_hex_string(&event_id_hex)?;
        tags.push(Tag::new(&["e", &event_id.as_hex_string()]));
    }

    // Add r-tag
    if let Some(r) = sub_command_args.rtag.clone() {
        tags.push(Tag::new(&["r", &r]));
    }

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::from(30315),
        tags,
        content: sub_command_args.content.clone(),
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    let event_id = client.send_event(event).await?;
    if !sub_command_args.hex {
        println!(
            "Published user status with id: {}",
            event_id.as_bech32_string()
        );
    } else {
        println!(
            "Published user status with id: {}",
            event_id.as_hex_string()
        );
    }

    Ok(())
}
