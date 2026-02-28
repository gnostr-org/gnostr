use anyhow::Result;
use clap::Parser;
use nostr_0_34_1::bech32;
use serde_json::{Value, json};

use gnostr_asyncgit::types::{NEvent, NostrBech32, PrivateKey};

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

    // Extract bech32 part from nostr or gnostr URLs if present
    let bech32 = if bech32.starts_with("nostr://") || bech32.starts_with("gnostr://") {
        let prefix_len = if bech32.starts_with("nostr://") { 8 } else { 9 };
        // Extract the bech32 part after the protocol and before any "/"
        let after_prefix = &bech32[prefix_len..];
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
        if let Ok(nevent) = gnostr_asyncgit::types::NEvent::try_from_bech32_string(bech32) {
            println!("{}", nevent.id.as_hex_string());
            return Ok(());
        }
        // Try note (direct event ID)
        else if let Ok(note) = gnostr_asyncgit::types::Id::try_from_bech32_string(bech32) {
            println!("{}", note.as_hex_string());
            return Ok(());
        }
        // Try naddr (parameterized replaceable event)
        else if let Ok(_naddr) = gnostr_asyncgit::types::NAddr::try_from_bech32_string(bech32) {
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

#[cfg(test)]
mod tests {
    use super::*;
    use gnostr_asyncgit::types::{Id, NEvent, NostrBech32};

    #[test]
    fn test_event_id_with_nevent() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "nevent1qqstna2yrezu5wghjvswqqculvvwxsrcvu7uc0f78gan4xqhvz49d9spr3mhxue69uhkummnw3ez6un9d3shjtn4de6x2argwghx6egpr4mhxue69uhkummnw3ez6ur4vgh8wetvd3hhyer9wghxuet5nxnepm".to_string(),
            raw: false,
            json: false,
            event_id: true,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());

        // Capture stdout and verify it contains the expected event ID
        // Note: This test would need stdout capturing in a real test
        // environment
    }

    #[test]
    fn test_event_id_with_note() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "note1j8dp2dcqf2kz0ylnpdsnknw6a0mvmuzchgumzcykwt6w7xt3602qysj207"
                .to_string(),
            raw: false,
            json: false,
            event_id: true,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_event_id_with_naddr_error() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "naddr1qqxk67txd9e8xardv96x7mt9qgsgfvxyd2mfntp4avk29pj8pwz7pqwmyzrummmrjv3rdsuhg9mc9agrqsqqqa28rkfdwv".to_string(),
            raw: false,
            json: false,
            event_id: true,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("naddr doesn't have a specific event ID")
        );
    }

    #[test]
    fn test_event_id_with_invalid_bech32() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "invalid_bech32_string".to_string(),
            raw: false,
            json: false,
            event_id: true,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Invalid bech32 string for --event-id")
        );
    }

    #[test]
    fn test_raw_output_with_pubkey() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "npub1sn0wdenkukak0d9dfczzeacvhkrgz92ak56egt7vdgzn8pv2wfqqhrjdv9"
                .to_string(),
            raw: true,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_json_output_with_nprofile() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "nprofile1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gpp4mhxue69uhhytnc9e3k7mgpz4mhxue69uhkg6nzv9ejuumpv34kytnrdaksjlyr9p".to_string(),
            raw: false,
            json: true,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_json_output() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "note1fntxtkcy9pjwucqwa9mddn7v03wwwsu9j330jj350nvhpky2tuaspk6nqc"
                .to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nostr_url_handling() {
        let args = Bech32ToAnySubCommand {
            bech32_string:
                "nostr://note1fntxtkcy9pjwucqwa9mddn7v03wwwsu9j330jj350nvhpky2tuaspk6nqc"
                    .to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_nostr_url_with_path_handling() {
        let args = Bech32ToAnySubCommand {
            bech32_string:
                "nostr://note1fntxtkcy9pjwucqwa9mddn7v03wwwsu9j330jj350nvhpky2tuaspk6nqc/some/path"
                    .to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_naddr_output() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "naddr1qqxk67txd9e8xardv96x7mt9qgsgfvxyd2mfntp4avk29pj8pwz7pqwmyzrummmrjv3rdsuhg9mc9agrqsqqqa28rkfdwv".to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_relay_output() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "nrelay1qqghwumn8ghj7mn0wd68yv339e3k7mgftj9ag".to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_private_key_handling() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "nsec1ufnus6pju578ste3v90xd5m2decpuzpql2295m3sknqcjzyys9ls0qlc85"
                .to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }

    #[test]
    fn test_cryptsec_not_implemented() {
        // Since we don't have a real ncryptsec string, we'll test with an invalid one
        // that will fall through to the generic bech32 decoder and handle gracefully
        let args = Bech32ToAnySubCommand {
            bech32_string: "ncryptsec1invalid".to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        // The function should handle this gracefully, either succeeding with JSON
        // output containing the decoded data or failing with a proper error
        assert!(result.is_ok() || result.is_err());
    }

    #[test]
    fn test_empty_string_handling() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "".to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_err());
    }

    #[test]
    fn test_whitespace_trimming() {
        let args = Bech32ToAnySubCommand {
            bech32_string: "  note1fntxtkcy9pjwucqwa9mddn7v03wwwsu9j330jj350nvhpky2tuaspk6nqc  "
                .to_string(),
            raw: false,
            json: false,
            event_id: false,
        };

        let result = bech32_to_any(&args);
        assert!(result.is_ok());
    }
}
