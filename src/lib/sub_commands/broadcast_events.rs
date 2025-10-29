use crate::utils::{create_client, parse_private_key};
use clap::Args;
use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use log::debug;
use nostr_sdk_0_37_0::prelude::*;

#[derive(Args, Debug)]
pub struct BroadcastEventsSubCommand {
    /// Input file path, should contain an array of JSON events
    #[arg(short, long)]
    file_path: String,
}

pub async fn broadcast_events(
    nsec: Option<String>,
    mut relays: Vec<String>,
    sub_command_args: &BroadcastEventsSubCommand,
) -> Result<()> {
    let keys: Keys;
    if relays.is_empty() {
        for relay in relays {
            debug!("relay:{:?}", relay);
        }
        relays = BOOTSTRAP_RELAYS.clone()
    } else {
        debug!("relays:{:?}", relays);

        for relay in relays.clone() {
            debug!("relay:{:?}", relay);
        }
    }
    if nsec.is_none() {
        keys = parse_private_key(None, false).await?;
    } else {
        keys = parse_private_key(nsec, false).await?;
    }

    let client = create_client(&keys, relays.clone(), 0).await?;

    let file = std::fs::File::open(&sub_command_args.file_path)?;

    let events: Vec<Event> = serde_json::from_reader(file)?;

    for event in events.clone() {
        client.send_event(event).await?;
    }

    println!("Published {} events to {:?}", events.len(), relays);

    Ok(())
}
