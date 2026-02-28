use std::{process::exit, time::Duration};

use clap::Args;
use crate::crawler::processor::BOOTSTRAP_RELAYS;
use tracing::debug;

use gnostr_asyncgit::types::{Error, Event, Filter, Id, PublicKey};
use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct ReactionSubCommand {
    /// Event id to react to
    #[arg(short, long)]
    event_id: String,
    /// Author pubkey of the event you are reacting to. Must be hex format.
    #[arg(short, long, default_value = "")]
    author_pubkey: String,
    /// Reaction content. Set to '+' for like or '-' for dislike. Single emojis
    /// are also often used for reactions, such as in Damus Web.
    #[arg(short, long)]
    reaction: String,
    // Print keys as hex
    #[arg(long, default_value = "false")]
    hex: bool,
}

pub async fn react_to_event(
    private_key: Option<String>,
    mut relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &ReactionSubCommand,
) -> Result<(), Error> {
    if relays.is_empty() {
        relays = BOOTSTRAP_RELAYS.to_vec();
    }

    let keys = parse_private_key(private_key, false).await?;
    // TODO: The client must also be reimplemented without nostr_sdk.
    // For now, client and its methods are assumed to be compatible or stubbed out.
    let _client = create_client(&keys, relays, difficulty_target).await?;

    if sub_command_args.reaction.trim().is_empty() {
        eprintln!("Reaction does not contain any content");
        exit(0)
    }

    let event_id = gnostr_asyncgit::types::Id::try_from_hex_string(&sub_command_args.event_id)?;
    // TODO: Implement Filter::event and Filter::author methods in
    // src/lib/types/filter.rs
    let subscription: Filter = Filter::new(); // Placeholder

    debug!("{:?}", subscription);
    // TODO: Replace with gnostr client logic to fetch and react to events
    // This functionality needs to be reimplemented without nostr_sdk
    let events: Vec<Event> = Vec::new(); // Placeholder for fetched events
    let id: Id = event_id; // Placeholder for the reaction event ID (or use event_id)

    if events.is_empty() {
        eprintln!("Unable to find note with the provided event id");
        exit(0);
    }

    // The following lines were part of nostr_sdk client interaction and are now
    // commented out. let event_to_react_to = events.first().unwrap();
    // let id = client
    //     .reaction(event_to_react_to, sub_command_args.reaction.clone())
    //     .await?;

    if sub_command_args.hex {
        print!(
            "{{\"event_id\":\"{}\"}}{{\"reaction\":\"{}\"}}{{\"id\":\"{}\"}}",
            event_id.as_hex_string(),
            sub_command_args.reaction,
            id.as_hex_string()
        );
    } else {
        print!(
            "{{\"event_id\":\"{}\"}}{{\"reaction\":\"{}\"}}{{\"id\":\"{}\"}}",
            event_id.as_bech32_string(),
            sub_command_args.reaction,
            id.as_bech32_string()
        );
    }
    Ok(())
}
