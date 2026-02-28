use std::{process::exit, str::FromStr, time::Duration};

use anyhow::{Error as AnyhowError, Result};
use clap::Args;

use crate::{
    utils::{create_client, parse_private_key},
};
use gnost_asyncgit::{
    types::{
        Client, Event, EventBuilder, EventKind, Filter, FilterOptions, Id, IdHex, Keys, Nip19,
        Options, PrivateKey, PublicKey, Tag,
    },
};

#[derive(Args, Debug)]
pub struct AwardBadgeSubCommand {
    /// Badge definition event id
    #[arg(short, long)]
    badge_event_id: String,
    /// Awarded pubkeys
    #[arg(short, long, action = clap::ArgAction::Append)]
    ptag: Vec<String>,
}

pub async fn award_badge(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &AwardBadgeSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let event_id = Id::try_from_hex_string(sub_command_args.badge_event_id.as_str())?;
    // TODO: Implement Filter::id method
    let mut filter = Filter::new();
    filter.add_id(&event_id.into()); // Assuming Id can be converted to IdHex

    let badge_definition_query = client
        .get_events_of_with_opts(
            vec![filter],
            Some(Duration::from_secs(10)),
            FilterOptions::ExitOnEOSE,
        )
        .await?;

    if badge_definition_query.len() != 1 {
        eprintln!("Expected one event, got {}", badge_definition_query.len());
        exit(1)
    };

    let badge_definition_event = badge_definition_query.first().unwrap();
    // Verify that this event is a badge definition event
    if badge_definition_event.kind != EventKind::BadgeDefinition {
        eprintln!(
            "Unexpected badge definition event. Expected event of kind {} but got {}",
            u32::from(EventKind::BadgeDefinition), // Convert EventKind to u32 for printing
            u32::from(badge_definition_event.kind)  // Convert EventKind to u32 for printing
        );
        exit(1)
    }

    // Verify that the user trying to award the badge is actually the author of the
    // badge definition
    if badge_definition_event.pubkey != keys.public_key() {
        eprint!(
            "Incorrect private key. Only the private key used for issuing the badge definition can award it to other public keys"
        );
        exit(1)
    }

    let awarded_pubkeys: Vec<Tag> = sub_command_args
        .ptag
        .iter()
        .map(|pubkey_string| {
            // TODO: Ensure PublicKey::try_from_hex_string is robust enough
            Tag::new_pubkey(
                gnostr_asyncgit::types::PublicKey::try_from_hex_string(pubkey_string, true)
                    .expect("Unable to parse public key"),
                None, // No recommended relay URL
                None, // No petname
            )
        })
        .collect();

    // TODO: Implement EventBuilder::award_badge and to_pow_event without nostr_sdk
    let mut event = Event::new_dummy(); // Placeholder event
    // Modify dummy event with relevant tags and kind
    event.kind = EventKind::BadgeAward;
    event.tags.push(Tag::new_event(
        badge_definition_event.id,
        None,
        Some("e".to_string()),
    ));
    for pubkey_tag in awarded_pubkeys {
        if let Ok((pk, _, _)) = pubkey_tag.parse_pubkey() {
            event.tags.push(Tag::new_pubkey(pk, None, None));
        }
    }
    // For to_pow_event, set difficulty_target in the event or tags if needed
    // For now, assume dummy event can be published.

    // Publish event
    // TODO: Replace with actual client.send_event implementation
    let event_id = client.send_event(event).await?;

    println!("Published badge award event with id:");
    println!("Hex: {}", event_id.as_hex_string());
    println!("Bech32: {}", event_id.as_bech32_string());

    Ok(())
}
