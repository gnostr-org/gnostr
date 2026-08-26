use std::{str::FromStr, time::Duration};

use anyhow::{Error as AnyhowError, Result};
use clap::Args;

use crate::{
    types::{
        Client, Event, EventKind, Filter, Id, KeySigner, Keys, PreEventV3, Signer, Tag, Unixtime,
    },
    utils::{create_client, parse_private_key},
};

#[derive(Args, Debug)]
pub struct ProfileBadgesSubCommand {
    /// Badge definition event id
    #[arg(short, long, action = clap::ArgAction::Append)]
    badge_id: Vec<String>,
    /// Badge award event id
    #[arg(short, long, action = clap::ArgAction::Append)]
    award_id: Vec<String>,
}

pub async fn set_profile_badges(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &ProfileBadgesSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let badge_definition_event_ids: Vec<Id> = sub_command_args
        .badge_id
        .iter()
        .map(|badge_id| Id::try_from_hex_string(badge_id).unwrap())
        .collect();
    let mut badge_definition_filter = Filter::new();
    badge_definition_filter.ids = badge_definition_event_ids
        .into_iter()
        .map(|id| id.into())
        .collect();
    badge_definition_filter.kinds = vec![EventKind::BadgeDefinition];

    let badge_definition_events = client
        .get_events_of(vec![badge_definition_filter], Some(Duration::from_secs(10)))
        .await?;

    let award_event_ids: Vec<Id> = sub_command_args
        .award_id
        .iter()
        .map(|award_event_id| Id::try_from_hex_string(award_event_id).unwrap())
        .collect();
    let mut badge_award_filter = Filter::new();
    badge_award_filter.ids = award_event_ids.into_iter().map(|id| id.into()).collect();
    badge_award_filter.kinds = vec![EventKind::BadgeAward];

    let badge_award_events = client
        .get_events_of(vec![badge_award_filter], Some(Duration::from_secs(10)))
        .await?;

    let mut tags = Vec::new();
    for event in badge_definition_events {
        tags.push(Tag::new(&[
            "a",
            &format!("{}:{}", u32::from(event.kind), event.pubkey.as_hex_string()),
        ]));
    }
    for event in badge_award_events {
        tags.push(Tag::new(&["e", &event.id.as_hex_string()]));
    }

    let pre_event = PreEventV3 {
        pubkey: keys.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::ProfileBadges,
        tags,
        content: "".to_string(),
    };

    let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
    let event = signer.sign_event(pre_event)?;

    // Publish event
    let event_id = client.send_event(event).await?;
    println!("Published profile badges event with id:");
    println!("Hex: {}", event_id.as_hex_string());
    println!("Bech32: {}", event_id.as_bech32_string());

    Ok(())
}
