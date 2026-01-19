use anyhow::Result;
use clap::Parser;
use nostr_0_34_1::bech32;
use serde_json::{json, Value};

use crate::types::{NEvent, NostrBech32, PrivateKey};

#[derive(Parser, Debug, Clone)]
pub struct Bech32ToAnySubCommand {
    /// Bech32 string to convert
    #[arg(value_name = "BECH32_STRING")]
    pub bech32_string: String,
    #[arg(long, value_name = "BECH32_RAW", group = "output_format")]
    pub raw: bool,
    #[arg(long, value_name = "BECH32_JSON", group = "output_format")]
    pub json: bool,
    /// Only output the event ID (hex string) if the input is a nevent
    #[arg(long, group = "output_format")]
    pub event_id: bool,
}

pub fn bech32_to_any(sub_command_args: &Bech32ToAnySubCommand) -> Result<()> {
    let bech32 = sub_command_args.bech32_string.trim();

    // Extract bech32 part from nostr URLs if present
    let bech32 = if bech32.starts_with("nostr://") {
        // Extract the bech32 part after "nostr://" and before any "/"
        let after_prefix = &bech32[8..]; // Remove "nostr://"
        if let Some(slash_pos) = after_prefix.find('/') {
            &after_prefix[..slash_pos]
        } else {
            after_prefix
        }
    } else {
        bech32
    };

    if sub_command_args.event_id {
        // Try nevent first
        if let Ok(nevent) = crate::types::NEvent::try_from_bech32_string(bech32) {
            println!("{}", nevent.id.as_hex_string());
            return Ok(());
        }
        // Try note (direct event ID)
        else if let Ok(note) = crate::types::Id::try_from_bech32_string(bech32) {
            println!("{}", note.as_hex_string());
            return Ok(());
        }
        // Try naddr (parameterized replaceable event)
        else if let Ok(_naddr) = crate::types::NAddr::try_from_bech32_string(bech32) {
            // naddr doesn't have a specific event ID, it's a coordinate
            eprintln!(
                "Error: --event-id cannot be used with naddr (parameterized replaceable event)."
            );
            eprintln!("naddr represents an event coordinate, not a specific event ID.");
            return Err(anyhow::anyhow!("naddr doesn't have a specific event ID"));
        } else {
            eprintln!("Error: --event-id can only be used with nevent or note bech32 strings.");
            eprintln!("Supported types for --event-id:");
            eprintln!("  nevent1... - Event pointer with optional relays, author, kind");
            eprintln!("  note1...    - Direct event ID");
            return Err(anyhow::anyhow!("Invalid bech32 string for --event-id"));
        }
    }

    let output_value: Value;
    let mut raw_output_string: Option<String> = None;

    if let Some(nb32) = NostrBech32::try_from_string(bech32) {
        match nb32 {
            NostrBech32::NAddr(na) => {
                output_value = json!({
                    "type": "Event Address",
                    "d": na.d,
                    "relays": na.relays.iter().map(|r| r.as_str().to_owned()).collect::<Vec<String>>(),
                    "kind": Into::<u32>::into(na.kind),
                    "author": na.author.as_hex_string(),
                })
            }
            NostrBech32::NEvent(ne) => {
                output_value = json!({
                    "type": "Event Pointer",
                    "id": ne.id.as_hex_string(),
                    "relays": ne.relays.iter().map(|r| r.as_str().to_owned()).collect::<Vec<String>>(),
                    "kind": ne.kind.map(Into::<u32>::into),
                    "author": ne.author.map(|a| a.as_hex_string()),
                })
            }
            NostrBech32::Id(id) => {
                let (hrp, _) = bech32::decode(bech32)
                    .map_err(|e| anyhow::anyhow!("Invalid bech32 encoding: {}", e))?;
                let mut map = serde_json::Map::new();
                map.insert(hrp.to_string(), Value::String(id.as_hex_string()));
                raw_output_string = Some(id.as_hex_string());
                output_value = Value::Object(map);
            }
            NostrBech32::Profile(profile) => {
                output_value = json!({
                    "type": "Profile",
                    "pubkey": profile.pubkey.as_hex_string(),
                    "relays": profile.relays.iter().map(|r| r.as_str().to_owned()).collect::<Vec<String>>(),
                })
            }
            NostrBech32::Pubkey(pubkey) => {
                let (hrp, _) = bech32::decode(bech32)
                    .map_err(|e| anyhow::anyhow!("Invalid bech32 encoding: {}", e))?;
                let mut map = serde_json::Map::new();
                map.insert(hrp.to_string(), Value::String(pubkey.as_hex_string()));
                raw_output_string = Some(pubkey.as_hex_string());
                output_value = Value::Object(map);
            }
            NostrBech32::Relay(url) => output_value = json!({ "relay_url": url.0 }),
            NostrBech32::CryptSec(_) => {
                output_value = json!({ "error": "CryptSec not implemented" })
            }
        };
    } else if let Ok(mut key) = PrivateKey::try_from_bech32_string(bech32) {
        let (hrp, _) = bech32::decode(bech32)
            .map_err(|e| anyhow::anyhow!("Invalid bech32 encoding: {}", e))?;
        let mut map = serde_json::Map::new();
        map.insert(hrp.to_string(), Value::String(key.as_hex_string()));
        raw_output_string = Some(key.as_hex_string());
        output_value = Value::Object(map);
    } else {
        let (hrp, data) = bech32::decode(bech32)
            .map_err(|e| anyhow::anyhow!("Invalid bech32 encoding: {}", e))?;
        output_value = json!({
            "hrp": hrp.to_string(),
            "data": String::from_utf8_lossy(&data),
        });
    }

    if sub_command_args.raw && raw_output_string.is_some() && !sub_command_args.json {
        println!("{}", raw_output_string.unwrap());
    } else if sub_command_args.json || (!sub_command_args.raw && !sub_command_args.json) {
        // Default to pretty JSON if --json is specified or if neither --raw nor --json
        // are
        println!("{}", serde_json::to_string_pretty(&output_value)?);
    } else {
        // Fallback for raw output of complex types or when raw is requested but no
        // specific raw_output_string is available
        println!("{}", serde_json::to_string(&output_value)?);
    }
    Ok(())
}
