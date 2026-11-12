use std::{process::exit, time::Duration};

use clap::Args;
use tracing::debug;

use crate::{
    types::{Error, Event, Filter, Id},
    utils::{create_client, parse_private_key},
};

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
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &ReactionSubCommand,
) -> Result<(), Error> {
    let relays = if relays.is_empty() {
        crate::crawler::load_relays_or_bootstrap()
    } else {
        relays
    };

    if sub_command_args.reaction.trim().is_empty() {
        eprintln!("Reaction does not contain any content");
        exit(0);
    }

    let keys = parse_private_key(private_key, false).await?;
    let event_id = Id::try_from_hex_string(&sub_command_args.event_id)?;

    let mut filter = Filter::new();
    filter.add_id(&event_id.into());
    filter.limit = Some(1);

    // Try each relay individually until we find the event.  We cannot assume
    // every relay is available, trustworthy, or supports the event's kind/NIP.
    debug!("Fetching event {} to react to", event_id);
    let mut event_to_react_to: Option<Event> = None;
    for relay in &relays {
        debug!("Trying relay {}", relay);
        let single_client = create_client(&keys, vec![relay.clone()], difficulty_target).await;
        let single_client = match single_client {
            Ok(c) => c,
            Err(err) => {
                debug!("Failed to create client for relay {}: {}", relay, err);
                continue;
            }
        };

        match single_client
            .get_events_of(vec![filter.clone()], Some(Duration::from_secs(5)))
            .await
        {
            Ok(events) if !events.is_empty() => {
                event_to_react_to = Some(events.into_iter().next().unwrap());
                break;
            }
            Ok(_) => {
                debug!("Relay {} returned no events for {}", relay, event_id);
            }
            Err(err) => {
                debug!("Relay {} failed to fetch {}: {}", relay, event_id, err);
            }
        }
    }

    let event_to_react_to = match event_to_react_to {
        Some(e) => e,
        None => {
            eprintln!("Unable to find note with the provided event id");
            exit(0);
        }
    };

    // Publish the reaction to all configured relays; send_event succeeds if any
    // relay accepts it.
    let client = create_client(&keys, relays, difficulty_target).await?;
    let reaction_id = client
        .reaction(&event_to_react_to, sub_command_args.reaction.clone())
        .await?;

    if sub_command_args.hex {
        println!("{}", reaction_id.as_hex_string());
    } else {
        println!("{}", reaction_id.as_bech32_string());
    }

    Ok(())
}
