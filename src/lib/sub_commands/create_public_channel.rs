use anyhow::{Error as AnyhowError, Result};
use clap::Args;

use gnostr_asyncgit::types::{Client, Event, EventBuilder, EventKind, Id, Keys, Metadata, PublicKey, Tag, UncheckedUrl, Unixtime};
use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct CreatePublicChannelSubCommand {
    /// Channel name
    #[arg(short, long)]
    name: String,
    /// Channel about
    #[arg(short, long)]
    about: Option<String>,
    /// Channel picture
    #[arg(short, long)]
    picture: Option<String>,
}

pub async fn create_public_channel(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &CreatePublicChannelSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    // Process keypair and create a nostr client
    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays.clone(), difficulty_target).await?;

    // Create metadata
    let mut metadata = Metadata::new();
    metadata.name = Some(sub_command_args.name.clone());

    if let Some(about) = sub_command_args.about.clone() {
        metadata.about = Some(about);
    }

    if let Some(picture) = sub_command_args.picture.clone() {
        // TODO: Ensure UncheckedUrl::try_from_str works correctly with Url::parse
        // behavior
        metadata.picture = Some(UncheckedUrl::from_str(&picture).to_string());
    }

    let private_key = keys.secret_key()?;
    let event_builder = EventBuilder::channel(&metadata, &private_key.public_key());
    let event = event_builder.to_event(&private_key)?;
    let event_id = client.send_event(event).await?;

    // Print results
    println!("\nCreated new public channel!");
    println!("Channel ID:");
    println!("Hex: {}", event_id.as_hex_string());
    println!("Bech32: {}", event_id.as_bech32_string());

    Ok(())
}
