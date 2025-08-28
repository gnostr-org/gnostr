#![doc = include_str!("../../../README.md")]
use libp2p::{kad, kad::store::MemoryStore};

pub fn handle_input_line(
    kademlia: &mut kad::Behaviour<MemoryStore>,
    line: String,
) -> Result<(), String> {
    let mut args = line.split(' ');

    match args.next() {
        Some("GET") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => return Err("Expected key for GET command".to_string()),
                }
            };
            kademlia.get_record(key);
            Ok(())
        }
        Some("GET_PROVIDERS") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => return Err("Expected key for GET_PROVIDERS command".to_string()),
                }
            };
            kademlia.get_providers(key);
            Ok(())
        }
        Some("PUT") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => return Err("Expected key for PUT command".to_string()),
                }
            };
            let value = {
                match args.next() {
                    Some(value) => value.as_bytes().to_vec(),
                    None => return Err("Expected value for PUT command".to_string()),
                }
            };
            let record = kad::Record {
                key: key.clone(),
                value,
                publisher: None,
                expires: None,
            };
            kademlia
                .put_record(record, kad::Quorum::One)
                .map_err(|e| format!("Failed to store record locally: {e}"))?;
            kademlia
                .start_providing(key)
                .map_err(|e| format!("Failed to start providing key: {e}"))?;
            Ok(())
        }
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => return Err("Expected key for PUT_PROVIDER command".to_string()),
                }
            };

            kademlia
                .start_providing(key)
                .map_err(|e| format!("Failed to start providing key: {e}"))?;
            Ok(())
        }
        _ => Err("expected GET, GET_PROVIDERS, PUT or PUT_PROVIDER".to_string()),
    }
}
