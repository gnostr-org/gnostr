use anyhow::Result;
use clap::Parser;
use crate::types::{NostrBech32, PrivateKey};
use nostr_0_34_1::bech32;
use serde_json::{json, Value};

#[derive(Parser, Debug, Clone)]
pub struct Bech32ToAnySubCommand {
    /// Bech32 string to convert
    #[arg(value_name = "BECH32_STRING")]
    pub bech32_string: String,
}

pub fn bech32_to_any(sub_command_args: &Bech32ToAnySubCommand) -> Result<()> {
    let bech32 = sub_command_args.bech32_string.trim();

    if let Some(nb32) = NostrBech32::try_from_string(bech32) {
        let json_output = match nb32 {
            NostrBech32::NAddr(na) => json!({
                "type": "Event Address",
                "d": na.d,
                "relays": na.relays.iter().map(|r| r.as_str().to_owned()).collect::<Vec<String>>(),
                "kind": Into::<u32>::into(na.kind),
                "author": na.author.as_hex_string(),
            }),
            NostrBech32::NEvent(ne) => json!({
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
                Value::Object(map)
            }
            NostrBech32::Profile(profile) => json!({
                "type": "Profile",
                "pubkey": profile.pubkey.as_hex_string(),
                "relays": profile.relays.iter().map(|r| r.as_str().to_owned()).collect::<Vec<String>>(),
            }),
            NostrBech32::Pubkey(pubkey) => {
                let (hrp, _) = bech32::decode(bech32).unwrap();
                let mut map = serde_json::Map::new();
                map.insert(hrp.to_string(), Value::String(pubkey.as_hex_string()));
                Value::Object(map)
            }
            NostrBech32::Relay(url) => json!({ "relay_url": url.0 }),
            NostrBech32::CryptSec(_) => json!({ "error": "CryptSec not implemented" }),
        };
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else if let Ok(mut key) = PrivateKey::try_from_bech32_string(bech32) {
        let (hrp, _) = bech32::decode(bech32).unwrap();
        let mut map = serde_json::Map::new();
        map.insert(hrp.to_string(), Value::String(key.as_hex_string()));
        let json_output = Value::Object(map);
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    } else {
        let (hrp, data) = bech32::decode(bech32).unwrap();
        let json_output = json!({
            "hrp": hrp.to_string(),
            "data": String::from_utf8_lossy(&data),
        });
        println!("{}", serde_json::to_string_pretty(&json_output)?);
    }
    Ok(())
}