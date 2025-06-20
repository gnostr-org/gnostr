use clap::Args;
use nostr_sdk_0_32_0::prelude::*;

use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct BroadcastEventsSubCommand {
    /// Input file path, should contain an array of JSON events
    #[arg(short, long)]
    file_path: String,
}

pub async fn broadcast_events(
    relays: Vec<String>,
    sub_command_args: &BroadcastEventsSubCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(None, false).await?;
    let client = create_client(&keys, relays.clone(), 0).await?;

    let file = std::fs::File::open(&sub_command_args.file_path)?;

    let events: Vec<Event> = serde_json::from_reader(file)?;

    for event in events.clone() {
        client.send_event(event).await?;
    }

    println!("Published {} events to {:?}", events.len(), relays);

    Ok(())
}
