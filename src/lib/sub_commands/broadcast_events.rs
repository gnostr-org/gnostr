use anyhow::{Error as AnyhowError, Result};
use clap::Args;
use gnostr::crawler::processor::BOOTSTRAP_RELAYS;
use log::debug;

use crate::{
    types::{Client, Event, Filter, Id, Keys, PrivateKey, PublicKey},
    utils::{create_client, parse_private_key},
};

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
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        relays = BOOTSTRAP_RELAYS.clone()
    }

    let keys = if nsec.is_none() {
        parse_private_key(None, false).await?
    } else {
        parse_private_key(nsec, false).await?
    };

    let client = create_client(&keys, relays.clone(), 0).await?;

    let file = std::fs::File::open(&sub_command_args.file_path)?;

    let events: Vec<Event> = serde_json::from_reader(file)?;

    for event in events.clone() {
        client.send_event(event).await?;
    }

    println!("Published {} events to {:?}", events.len(), relays);

    Ok(())
}
