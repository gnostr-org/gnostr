use anyhow::{Error as AnyhowError, Result};
use clap::Args;

use crate::{
    types::{
        Client, Event, EventKind, Id, KeySigner, Keys, Metadata, PreEventV3, Signer, Tag,
        UncheckedUrl, Unixtime,
    },
    utils::{create_client, parse_private_key},
};

#[derive(Args, Debug)]
pub struct SetChannelMetadataSubCommand {
    /// Channel ID
    #[arg(short, long)]
    channel_id: String,
    /// Channel name
    #[arg(short, long)]
    name: Option<String>,
    /// Channel about
    #[arg(short, long)]
    about: Option<String>,
    /// Channel picture
    #[arg(short, long)]
    picture: Option<String>,
    #[arg(short, long)]
    recommended_relay: Option<String>,
}

pub async fn set_channel_metadata(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &SetChannelMetadataSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    // Process keypair and create a nostr client
    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays.clone(), difficulty_target).await?;

    let channel_id: Id = Id::try_from_hex_string(&sub_command_args.channel_id)?;

    // Build metadata
    let mut metadata = Metadata::new();

    if let Some(name) = sub_command_args.name.clone() {
        metadata.name = Some(name);
    }

    if let Some(about) = sub_command_args.about.clone() {
        metadata.about = Some(about);
    }

    if let Some(picture) = sub_command_args.picture.clone() {
        metadata.picture = Some(UncheckedUrl::from_str(&picture).to_string());
    }

    let relay_url: Option<String> = sub_command_args.recommended_relay.clone();

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::ChannelMetadata,
        tags: vec![Tag::new(&[
            "e",
            &channel_id.as_hex_string(),
            relay_url.as_ref().map_or("", |s| s.as_str()),
        ])],
        content: serde_json::to_string(&metadata)?,
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    let event_id = client.send_event(event.clone()).await?;

    // Print results
    println!(
        "\nSet new metadata for channel {}!",
        sub_command_args.channel_id.as_str()
    );
    println!("\nEvent ID:");
    println!("Hex: {}", event_id.as_hex_string());
    println!("Bech32: {}", event_id.as_bech32_string());

    Ok(())
}
