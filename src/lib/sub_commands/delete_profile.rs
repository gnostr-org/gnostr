use std::time::Duration;

use clap::Args;
use anyhow::{Result, Error as AnyhowError};
use crate::types::{Client, Event, EventKind, Filter, Id, Keys, Metadata, PublicKey, Tag, PreEventV3, Unixtime, KeySigner};
use serde_json::Value;

use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct DeleteProfileSubCommand {
    /// Delete just the events instead of the profile
    #[arg(long, default_value = "false")]
    events_only: bool,
    /// If events only are selected, allows specifying kinds
    #[arg(short, long, action = clap::ArgAction::Append)]
    kinds: Option<Vec<u64>>,
    /// Reason for deleting the events
    #[arg(short, long)]
    reason: Option<String>,
    // Print keys as hex
    #[arg(long, default_value = "false")]
    hex: bool,
    /// Timeout in seconds
    #[arg(long)]
    timeout: Option<u64>,
}

pub async fn delete(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &DeleteProfileSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let timeout = sub_command_args.timeout.map(Duration::from_secs);

    if sub_command_args.events_only {
        // go through all of the user events
        let authors: Vec<PublicKey> = vec![keys.public_key()];
        println!("checking author events...");

        // Convert kind number to Kind struct
        let kinds: Vec<EventKind> = sub_command_args
            .kinds
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|x| EventKind::from(x as u32))
            .collect();

        let mut filter = Filter::new();
        filter.authors = authors.iter().map(|p| (*p).into()).collect();
        filter.kinds = kinds;

        let events: Vec<Event> = client
            .get_events_of(vec![filter], timeout)
            .await?;

        let event_tags: Vec<Tag> = events
            .iter()
            .map(|event| Tag::new(&["e", &event.id.as_hex_string()]))
            .collect();

        println!("Retrieved events to delete: {}", events.len());

        let pre_event = PreEventV3 {
            pubkey: keys.public_key(),
            created_at: Unixtime::now(),
            kind: EventKind::EventDeletion,
            tags: event_tags,
            content: sub_command_args.reason.clone().unwrap_or_default(),
        };

        let signer = KeySigner::from_private_key(keys.secret_key()?, "", 1)?;
        let delete_event = signer.sign_event(pre_event)?;

        let event_id = client.send_event(delete_event).await?;

        if !sub_command_args.hex {
            println!("All event deleted in event {}", event_id.as_bech32_string());
        } else {
            println!("All event deleted in event {}", event_id.as_hex_string());
        }
    } else {
        // Not a perfect delete but multiple clients trigger off of this metadata
        let mut metadata = Metadata::default();
        metadata.name = Some("Deleted".to_string());
        metadata.display_name = Some("Deleted".to_string());
        metadata.about = Some("Deleted".to_string());
        let mut custom_fields = std::collections::BTreeMap::new();
        custom_fields.insert("deleted".to_string(), Value::Bool(true));
        metadata.custom = custom_fields;

        let event_id = client.set_metadata(&metadata).await?;
        println!("Metadata updated ({})", event_id.as_bech32_string());
    }
    Ok(())
}
