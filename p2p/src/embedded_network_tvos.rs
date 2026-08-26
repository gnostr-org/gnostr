use std::{
    collections::{HashSet, VecDeque},
    sync::{Mutex, OnceLock},
};

use serde::{Deserialize, Serialize};

const DEFAULT_DISCOVERY_TOPIC: &str = "gnostr/p2p/presence";
const DEFAULT_CHAT_TOPIC: &str = "gnostr-dev";

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub source: String,
    pub addresses: Vec<String>,
    pub last_seen_secs: u64,
}

#[derive(Default)]
struct EmbeddedNetwork {
    status: String,
    logs: VecDeque<String>,
    peers: Vec<DiscoveredPeer>,
    chat_topics: HashSet<String>,
}

static NETWORK: OnceLock<Mutex<EmbeddedNetwork>> = OnceLock::new();

fn network() -> &'static Mutex<EmbeddedNetwork> {
    NETWORK.get_or_init(|| {
        let mut state = EmbeddedNetwork::default();
        state.status = "tvOS stub: not running".to_string();
        state.chat_topics.insert(DEFAULT_CHAT_TOPIC.to_string());
        Mutex::new(state)
    })
}

fn push_log(line: impl Into<String>) {
    let mut state = network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.logs.push_back(line.into());
    while state.logs.len() > 500 {
        state.logs.pop_front();
    }
}

pub fn log_line(line: impl Into<String>) {
    push_log(line);
}

pub fn status() -> String {
    network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .status
        .clone()
}

pub fn logs() -> String {
    network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .logs
        .iter()
        .cloned()
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn clear_logs() {
    network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .logs
        .clear();
}

pub fn clear_peers() {
    network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .peers
        .clear();
}

pub fn peers() -> String {
    let peers = &network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .peers;
    serde_json::to_string(peers).unwrap_or_else(|_| "[]".to_string())
}

pub fn register_chat_topic(topic: impl Into<String>) -> String {
    let topic = topic.into();
    let mut state = network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.chat_topics.insert(topic.clone());
    format!("tvOS stub registered chat topic {topic}")
}

pub fn send_chat_message(topic: impl Into<String>, message: impl Into<String>) -> String {
    let topic = topic.into();
    let message = message.into();
    push_log(format!("tvOS stub chat send topic={topic} message={message}"));
    format!("tvOS stub queued chat message for {topic}")
}

pub fn start() -> String {
    let mut state = network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.status = format!(
        "tvOS stub: discovery={}, chat={}",
        DEFAULT_DISCOVERY_TOPIC,
        DEFAULT_CHAT_TOPIC
    );
    "tvOS stub network started".to_string()
}

pub fn stop() -> String {
    let mut state = network()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    state.status = "tvOS stub: stopped".to_string();
    "tvOS stub network stopped".to_string()
}
