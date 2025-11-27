use http::Uri;
use std::str::FromStr;

use anyhow::Result;
use clap::Args;
use tokio::task;

use crate::types::{
    Event, EventKind, Filter, Id, KeySigner, NAddr, PreEvent, PrivateKey, Signer, Tag, Unixtime,
};

#[derive(Args, Debug, Clone)]
pub struct ProfileBadgesSubCommand {
    /// Badge definition event id
    #[arg(short, long, action = clap::ArgAction::Append)]
    badge_id: Vec<String>,
    /// Badge award event id
    #[arg(short, long, action = clap::ArgAction::Append)]
    award_id: Vec<String>,
}

async fn get_events(relays: Vec<String>, filters: Vec<Filter>) -> Result<Vec<Event>> {
    task::spawn_blocking(move || {
        let mut events: Vec<Event> = Vec::new();
        let wire = crate::types::internal::filters_to_wire(filters);

        for relay_url_str in relays {
            let url = match url::Url::parse(&relay_url_str) {
                Ok(url) => url,
                Err(e) => {
                    eprintln!("Failed to parse relay url {}: {}", relay_url_str, e);
                    continue;
                }
            };
            let host = match url.host_str() {
                Some(h) => h.to_string(),
                None => {
                    eprintln!("No host in relay URL: {}", relay_url_str);
                    continue;
                }
            };
            let uri: Uri = match relay_url_str.parse() {
                Ok(uri) => uri,
                Err(e) => {
                    eprintln!("Failed to parse relay url {} as URI: {}", relay_url_str, e);
                    continue;
                }
            };
            let mut fetched = crate::types::internal::fetch(host, uri, wire.clone());
            events.append(&mut fetched);
        }
        Ok(events)
    })
    .await?
}

async fn send_event(relays: Vec<String>, event: Event) -> Result<Id> {
    let event_id = event.id;
    task::spawn_blocking(move || {
        let wire = crate::types::internal::event_to_wire(event);

        for relay_url_str in relays {
            let url = match url::Url::parse(&relay_url_str) {
                Ok(url) => url,
                Err(e) => {
                    eprintln!("Failed to parse relay url {}: {}", relay_url_str, e);
                    continue;
                }
            };
            let host = match url.host_str() {
                Some(h) => h.to_string(),
                None => {
                    eprintln!("No host in relay URL: {}", relay_url_str);
                    continue;
                }
            };
            let uri: Uri = match relay_url_str.parse() {
                Ok(uri) => uri,
                Err(e) => {
                    eprintln!("Failed to parse relay url {} as URI: {}", relay_url_str, e);
                    continue;
                }
            };
            crate::types::internal::post(host, uri, wire.clone());
        }
        Ok::<(), anyhow::Error>(())
    })
    .await??;
    Ok(event_id)
}

pub async fn set_profile_badges(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &ProfileBadgesSubCommand,
) -> Result<()> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let pk_str = match private_key {
        Some(pk) => pk,
        None => return Err(anyhow::anyhow!("Private key must be provided.")),
    };

    let pk = PrivateKey::try_from_bech32_string(&pk_str)
        .or_else(|_| PrivateKey::try_from_hex_string(&pk_str))?;

    let signer = KeySigner::from_private_key(pk, "", 10)?;

    let badge_definition_event_ids: Vec<Id> = sub_command_args
        .badge_id
        .iter()
        .map(|badge_id| Id::try_from_hex_string(badge_id).unwrap())
        .collect();

    let badge_definition_filter = Filter {
        ids: badge_definition_event_ids.iter().map(|id| (*id).into()).collect(),
        kinds: vec![EventKind::BadgeDefinition],
        ..Default::default()
    };
    let badge_definition_events = get_events(relays.clone(), vec![badge_definition_filter]).await?;

    let award_event_ids: Vec<Id> = sub_command_args
        .award_id
        .iter()
        .map(|award_event_id| Id::try_from_hex_string(award_event_id).unwrap())
        .collect();
    let badge_award_filter = Filter {
        ids: award_event_ids.iter().map(|id| (*id).into()).collect(),
        kinds: vec![EventKind::BadgeAward],
        ..Default::default()
    };
    let badge_award_events = get_events(relays.clone(), vec![badge_award_filter]).await?;

    let mut tags: Vec<Tag> = vec![];
    for event in badge_definition_events {
        tags.push(Tag::new_event(event.id, None, None));
    }

    for award_event in badge_award_events {
        let mut d_tag = "".to_string();
        for tag in &award_event.tags {
            if let Ok(id) = tag.parse_identifier() {
                d_tag = id;
                break;
            }
        }
        let naddr = NAddr {
            d: d_tag,
            relays: vec![],
            kind: EventKind::BadgeAward,
            author: award_event.pubkey,
        };
        tags.push(Tag::new_address(&naddr, None));
    }

    let pre_event = PreEvent {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::ProfileBadges,
        tags,
        content: "".to_string(),
    };

    let event = signer.sign_event_with_pow(pre_event, difficulty_target, None)?;

    let event_id = send_event(relays, event).await?;
    println!("Published profile badges event with id:");
    println!("Hex: {}", event_id.as_hex_string());
    println!("Bech32: {}", event_id.as_bech32_string());

    Ok(())
}
