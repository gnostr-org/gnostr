use clap::Args;
use anyhow::{Result, Error as AnyhowError};
use crate::types::{Client, Id, Keys};

use crate::utils::{create_client, parse_private_key};

#[derive(Args, Debug)]
pub struct DeleteEventSubCommand {
    /// Event id to delete. Must be in hex format.
    #[arg(short, long)]
    event_id: String,
    /// Print keys as hex
    #[arg(long, default_value = "false")]
    hex: bool,
}

pub async fn delete(
    private_key: Option<String>,
    relays: Vec<String>,
    difficulty_target: u8,
    sub_command_args: &DeleteEventSubCommand,
) -> Result<(), AnyhowError> {
    if relays.is_empty() {
        panic!("No relays specified, at least one relay is required!")
    }

    let keys = parse_private_key(private_key, false).await?;
    let client = create_client(&keys, relays, difficulty_target).await?;

    let event_id_to_delete = Id::try_from_hex_string(&sub_command_args.event_id)?;

    let event_id = client.delete_event(event_id_to_delete).await?;
    if !sub_command_args.hex {
        println!("Deleted event with id: {}", event_id.as_bech32_string());
    } else {
        println!("Deleted event with id: {}", event_id.as_hex_string());
    }
    Ok(())
}
