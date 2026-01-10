use crate::types::{NostrBech32, PrivateKey};
use anyhow::Result;
use clap::Parser;
use nostr_0_34_1::bech32;
use serde_json::{json, Value};

#[derive(Parser, Debug, Clone)]
pub struct Bech32ToAnySubCommand {
    /// Bech32 string to convert
    #[arg(value_name = "BECH32_STRING")]
    pub bech32_string: String,
    #[arg(long, value_name = "BECH32_RAW")]
    pub raw: bool,
    #[arg(long, value_name = "BECH32_JSON")]
    pub json: bool,
}

pub fn bech32_to_any(sub_command_args: &Bech32ToAnySubCommand) -> Result<()> {
    let bech32 = sub_command_args.bech32_string.trim();

    let output_value: Value;
    let mut raw_output_string: Option<String> = None;

    if let Some(nb32) = NostrBech32::try_from_string(bech32) {
        match nb32 {
            NostrBech32::NAddr(na) => output_value = json!({
                "type": "Event Address",
                "d": na.d,
                "relays": na.relays.iter().map(|r| r.as_str().to_owned()).collect::<Vec<String>>(),
                "kind": Into::<u32>::into(na.kind),
                "author": na.author.as_hex_string(),
            }),
            NostrBech32::NEvent(ne) => output_value = json!({
                "type": "Event Pointer",
                "id": ne.id.as_hex_string(),
                "relays": ne.relays.iter().map(|r| r.as_str().to_owned()).collect::<Vec<String>>(),
                "kind": ne.kind.map(Into::<u32>::into),
                "author": ne.author.map(|a| a.as_hex_string()),
            }),
            NostrBech32::Id(id) => {
                let (hrp, _) = bech32::decode(bech32).unwrap();
                let mut map = serde_json::Map::new();
                map.insert(hrp.to_string(), Value::String(id.as_hex_string()));
                raw_output_string = Some(id.as_hex_string());
                output_value = Value::Object(map);
            }
            NostrBech32::Profile(profile) => output_value = json!({
                "type": "Profile",
                "pubkey": profile.pubkey.as_hex_string(),
                "relays": profile.relays.iter().map(|r| r.as_str().to_owned()).collect::<Vec<String>>(),
            }),
            NostrBech32::Pubkey(pubkey) => {
                let (hrp, _) = bech32::decode(bech32).unwrap();
                let mut map = serde_json::Map::new();
                map.insert(hrp.to_string(), Value::String(pubkey.as_hex_string()));
                raw_output_string = Some(pubkey.as_hex_string());
                output_value = Value::Object(map);
            }
            NostrBech32::Relay(url) => output_value = json!({ "relay_url": url.0 }),
            NostrBech32::CryptSec(_) => output_value = json!({ "error": "CryptSec not implemented" }),
        };
    } else if let Ok(mut key) = PrivateKey::try_from_bech32_string(bech32) {
        let (hrp, _) = bech32::decode(bech32).unwrap();
        let mut map = serde_json::Map::new();
        map.insert(hrp.to_string(), Value::String(key.as_hex_string()));
        raw_output_string = Some(key.as_hex_string());
        output_value = Value::Object(map);
    } else {
        let (hrp, data) = bech32::decode(bech32).unwrap();
        output_value = json!({
            "hrp": hrp.to_string(),
            "data": String::from_utf8_lossy(&data),
        });
    }

    if sub_command_args.raw && raw_output_string.is_some() && !sub_command_args.json {
        println!("{}", raw_output_string.unwrap());
    } else if sub_command_args.json || (!sub_command_args.raw && !sub_command_args.json) {
        // Default to pretty JSON if --json is specified or if neither --raw nor --json are
        println!("{}", serde_json::to_string_pretty(&output_value)?);
    } else {
        // Fallback for raw output of complex types or when raw is requested but no specific raw_output_string is available
        println!("{}", serde_json::to_string(&output_value)?);
    }
    Ok(())
}
