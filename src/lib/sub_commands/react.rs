use std::process::exit;
use std::time::Duration;

use clap::Args;
use nostr_sdk_0_32_0::prelude::*;

use gnostr_crawler::processor::BOOTSTRAP_RELAYS;
use crate::utils::{create_client, parse_private_key};

use tracing::{debug, info, error, trace, warn};

#[derive(Args, Debug)]
pub struct ReactionSubCommand {
    /// Event id to react to
    #[arg(short, long)]
    event_id: String,
    /// Author pubkey of the event you are reacting to. Must be hex format.
    #[arg(short, long, default_value = "")]
    author_pubkey: String,
    /// Reaction content. Set to '+' for like or '-' for dislike. Single emojis are also often used for reactions, such as in Damus Web.
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
) -> Result<()> {
    if relays.is_empty() {
        relays = BOOTSTRAP_RELAYS.to_vec();
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    if sub_command_args.reaction.trim().is_empty() {
        eprintln!("Reaction does not contain any content");
        exit(0)
    }

    let subscription: Filter;
    let event_id = EventId::from_hex(&sub_command_args.event_id)?;
    if sub_command_args.author_pubkey.len() > 0 {
        let author_pubkey = PublicKey::from_hex(sub_command_args.author_pubkey.clone())?;
        subscription = Filter::new().event(event_id).author(author_pubkey);
    } else {
        subscription = Filter::new().event(event_id);
    }

    debug!("{:?}", subscription);
    let events = client
        .get_events_of_with_opts(
            vec![subscription],
            Some(Duration::from_secs(30)),
            FilterOptions::ExitOnEOSE,
        )
        .await?;

    if events.is_empty() {
        eprintln!("Unable to find note with the provided event id");
        exit(0);
    }

    let event_to_react_to = events.first().unwrap();

    let id = client
        .reaction(event_to_react_to, sub_command_args.reaction.clone())
        .await?;

    if sub_command_args.hex {
        print!(
            "{{\"event_id\":\"{}\"}}{{\"reaction\":\"{}\"}}{{\"id\":\"{}\"}}",
            event_id, sub_command_args.reaction, id
        );
    } else {
        print!(
            "{{\"event_id\":\"{}\"}}{{\"reaction\":\"{}\"}}{{\"id\":\"{}\"}}",
            event_id.to_bech32()?,
            sub_command_args.reaction,
            id.to_bech32()?
        );
    }
    Ok(())
}
