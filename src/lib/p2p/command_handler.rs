use libp2p::{gossipsub, kad, kad::store::MemoryStore};
use tracing::debug;

pub async fn handle_input_line(swarm: &mut libp2p::Swarm<super::behaviour::Behaviour>, line: String) {
    let mut args = line.split_whitespace();
    match args.next() {
        Some("TOPIC") => {
            if let Some(key_str) = args.next() {
                let topic = gossipsub::IdentTopic::new(key_str);
                println!("subscribe topic={}", topic.clone());
                swarm
                    .behaviour_mut()
                    .gossipsub
                    .subscribe(&topic)
                    .expect("failed to subscribe to TOPIC");
            } else {
                eprintln!("Usage: TOPIC <topic_string>");
            }
        }
        Some("GET") => {
            if let Some(key_str) = args.next() {
                let key = kad::RecordKey::new(&key_str);
                swarm.behaviour_mut().kademlia.get_record(key);
            } else {
                eprintln!("Usage: GET <key>");
            }
        }
        Some("GET_PROVIDERS") => {
            if let Some(key_str) = args.next() {
                let key = kad::RecordKey::new(&key_str);
                swarm.behaviour_mut().kademlia.get_providers(key);
            } else {
                eprintln!("Usage: GET_PROVIDERS <key>");
            }
        }
        Some("PUT") => {
            if let (Some(key_str), Some(value_str)) = (args.next(), args.next()) {
                let key = kad::RecordKey::new(&key_str);
                let value = value_str.as_bytes().to_vec();
                let record = kad::Record {
                    key: key.clone(),
                    value,
                    publisher: None,
                    expires: None,
                };
                if let Err(e) = swarm
                    .behaviour_mut()
                    .kademlia
                    .put_record(record.clone(), kad::Quorum::Majority)
                {
                    debug!("Failed to store record locally: {:?}", e);
                } else {
                    debug!(
                        "put record.key:{:?} record.value:{:?}",
                        record.key, record.value
                    );
                }
                if let Err(e) = swarm.behaviour_mut().kademlia.start_providing(key.clone()) {
                    debug!("Failed to store record locally: {:?}", e);
                } else {
                    debug!(
                        "started providing put record.key:{:?} record.value:{:?} key:{:?}",
                        record.key.clone(),
                        record.value,
                        key.clone()
                    );

                    let topic = gossipsub::IdentTopic::new(format!(
                        "{}",
                        std::str::from_utf8(record.key.as_ref()).unwrap_or("invalid utf8"),
                    ));

                    println!("subscribe topic={}", topic.clone());
                    swarm
                        .behaviour_mut()
                        .gossipsub
                        .subscribe(&topic)
                        .expect("failed to subscribe to TOPIC");
                }
            } else {
                eprintln!("Usage: PUT <key> <value>");
            }
        }
        Some("PUT_PROVIDER") => {
            let key = {
                match args.next() {
                    Some(key) => kad::RecordKey::new(&key),
                    None => {
                        eprint!("gnostr> ");
                        return;
                    }
                }
            };
            if let Err(e) = swarm.behaviour_mut().kademlia.start_providing(key) {
                eprintln!("Failed to store record locally: {:?}", e);
            }
        }
        Some("QUIT") | Some("Q") | Some("EXIT") => {
            std::process::exit(0);
        }
        _ => {
            eprintln!("Commands: TOPIC, GET, GET_PROVIDERS, PUT, PUT_PROVIDER, QUIT");
        }
    }
}