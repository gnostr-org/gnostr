use crate::utils::{create_client, parse_private_key};
use clap::Args;
use anyhow::{Result, Error as AnyhowError};
use crate::types::{Client, Event, EventKind, Id, Keys, PreEventV3, Tag, Unixtime, UncheckedUrl, KeySigner, Signer};

#[derive(Args, Debug)]
pub struct SendChannelMessageSubCommand {
    /// Channel id to send message to (must be hex)
    #[arg(short, long)]
    channel_id: String,
    /// Message content
    #[arg(short, long)]
    message: String,
    // Print keys as hex
    #[arg(long, default_value = "false")]
    hex: bool,
}

pub async fn send_channel_message(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &SendChannelMessageSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    // Process keypair and create a nostr client
    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays.clone(), difficulty_target).await?;

    let channel_id = Id::try_from_hex_string(&sub_command_args.channel_id)?;
    
    let tags = vec![Tag::new(&[
        "e",
        &channel_id.as_hex_string(),
        relays[0].as_str(),
        "root",
    ])];

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::ChannelMessage,
        tags,
        content: sub_command_args.message.clone(),
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    let event_id = client.send_event(event).await?;
    println!(
        "Public channel message sent with id: {}",
        event_id.as_bech32_string()
    );

    Ok(())
}
