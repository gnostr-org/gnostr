use std::{
    collections::HashSet,
    path::PathBuf,
    sync::{Arc, Mutex, OnceLock},
    thread,
    time::{SystemTime, UNIX_EPOCH},
};

use base64::Engine;
use chacha20poly1305::{
    aead::{Aead, KeyInit, Payload},
    ChaCha20Poly1305, Nonce,
};
use futures::StreamExt;
use libp2p::{
    gossipsub::IdentTopic,
    kad::{self, RecordKey as KadKey},
    swarm::SwarmEvent,
    Multiaddr, PeerId,
};
use gnostr_asyncgit::types::PrivateKey;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use tokio::{runtime::Builder, sync::{mpsc, oneshot}};
use tokio::sync::Notify;

use crate::{
    cli,
    event_handler,
    keypair_from_seed,
    network_config::{active_chat_topic, active_discovery_topic},
    time::run_time_sync_daemon,
    swarm_builder,
    utils::multiaddr_with_peer_id,
};
use gnostr_asyncgit::blockheight::blockheight_sync;
use gnostr_asyncgit::weeble::weeble_sync;
use gnostr_asyncgit::wobble::wobble_sync;

struct EmbeddedNetwork {
    status: Arc<Mutex<String>>,
    logs: Arc<Mutex<Vec<String>>>,
    chat_tx: mpsc::UnboundedSender<ChatCommand>,
    shutdown: Option<oneshot::Sender<()>>,
    join: Option<thread::JoinHandle<()>>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DiscoveredPeer {
    pub peer_id: String,
    pub source: String,
    pub addresses: Vec<String>,
    pub last_seen_secs: u64,
}

static NETWORK: OnceLock<Mutex<Option<EmbeddedNetwork>>> = OnceLock::new();
static LOGS: OnceLock<Mutex<Vec<String>>> = OnceLock::new();
static PEERS: OnceLock<Mutex<Vec<DiscoveredPeer>>> = OnceLock::new();
static CHAT_TOPICS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static SUBSCRIBED_CHAT_TOPICS: OnceLock<Mutex<HashSet<String>>> = OnceLock::new();
static CHAT_TOPIC_SYNC_NOTIFY: OnceLock<Notify> = OnceLock::new();

#[derive(Clone, Debug)]
struct ChatCommand {
    topic: String,
    message: String,
}

fn network_slot() -> &'static Mutex<Option<EmbeddedNetwork>> {
    NETWORK.get_or_init(|| Mutex::new(None))
}

fn logs_slot() -> &'static Mutex<Vec<String>> {
    LOGS.get_or_init(|| Mutex::new(Vec::new()))
}

fn peers_slot() -> &'static Mutex<Vec<DiscoveredPeer>> {
    PEERS.get_or_init(|| Mutex::new(Vec::new()))
}

fn chat_topics_slot() -> &'static Mutex<HashSet<String>> {
    CHAT_TOPICS.get_or_init(|| {
        let mut topics = HashSet::new();
        topics.insert(active_chat_topic());
        Mutex::new(topics)
    })
}

fn subscribed_chat_topics_slot() -> &'static Mutex<HashSet<String>> {
    SUBSCRIBED_CHAT_TOPICS.get_or_init(|| Mutex::new(HashSet::new()))
}

fn chat_topic_sync_notify() -> &'static Notify {
    CHAT_TOPIC_SYNC_NOTIFY.get_or_init(Notify::new)
}

fn chat_topics_file_path() -> Option<PathBuf> {
    let home = std::env::var_os("HOME")?;
    Some(PathBuf::from(home).join("Library/Application Support/gnostr/p2p-chat-topics.txt"))
}

#[derive(Clone, Debug, Deserialize, Serialize)]
struct EncryptedChatTopicsPayload {
    nonce: String,
    ciphertext: String,
    tag: String,
}

fn push_log(line: impl Into<String>) {
    let line = line.into();
    {
        let mut logs = logs_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        logs.push(line.clone());
        if logs.len() > 500 {
            let drain_to = logs.len().saturating_sub(500);
            logs.drain(0..drain_to);
        }
    }

    let guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(state) = guard.as_ref() {
        let mut logs = state.logs.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        logs.push(line);
        if logs.len() > 500 {
            let drain_to = logs.len().saturating_sub(500);
            logs.drain(0..drain_to);
        }
    }
}

pub fn log_line(line: impl Into<String>) {
    push_log(line);
}

fn with_status<F, T>(f: F) -> T
where
    F: FnOnce(&mut String) -> T,
{
    let guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let Some(state) = guard.as_ref() else {
        return f(&mut String::from("not running"));
    };
    let mut status = state.status.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut status)
}

pub fn status() -> String {
    with_status(|status| status.clone())
}

pub fn logs() -> String {
    let logs = logs_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    logs.join("\n")
}

pub fn clear_logs() {
    logs_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}

pub fn clear_peers() {
    peers_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clear();
}

fn reset_topic_state() {
    {
        let mut topics = chat_topics_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        topics.clear();
        topics.insert(active_chat_topic());
    }
    {
        let mut subscribed = subscribed_chat_topics_slot()
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        subscribed.clear();
    }
}

fn merge_peer_discovery(peer_id: String, source: String, addresses: Vec<String>) {
    let mut peers = peers_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    let mut normalized = Vec::new();
    let mut seen = HashSet::new();
    for address in addresses {
        if seen.insert(address.clone()) {
            normalized.push(address);
        }
    }

    if let Some(existing) = peers.iter_mut().find(|peer| peer.peer_id == peer_id) {
        existing.source = source;
        existing.last_seen_secs = timestamp_secs();
        for address in normalized {
            if !existing.addresses.contains(&address) {
                existing.addresses.push(address);
            }
        }
    } else {
        peers.push(DiscoveredPeer {
            peer_id,
            source,
            addresses: normalized,
            last_seen_secs: timestamp_secs(),
        });
    }

    peers.sort_by(|a, b| {
        b.last_seen_secs
            .cmp(&a.last_seen_secs)
            .then_with(|| a.peer_id.cmp(&b.peer_id))
    });

    if peers.len() > 100 {
        peers.truncate(100);
    }
}

pub fn record_peer_discovery(peer_id: impl Into<String>, source: impl Into<String>, addresses: Vec<String>) {
    merge_peer_discovery(peer_id.into(), source.into(), addresses);
}

pub fn peers() -> String {
    let peers = peers_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    serde_json::to_string(&*peers).unwrap_or_else(|error| {
        push_log(format!("failed to serialize peers: {error}"));
        String::from("[]")
    })
}

const DISCOVERY_REFRESH_SECS: u64 = 20;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PeerPresence {
    pub peer_id: String,
    pub addresses: Vec<String>,
    pub timestamp_secs: u64,
}

fn discovery_topic() -> IdentTopic {
    IdentTopic::new(active_discovery_topic())
}

fn discovery_key() -> KadKey {
    let topic = active_discovery_topic();
    KadKey::new(&topic)
}

fn sync_registered_chat_topics(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
) -> Result<(), Box<dyn std::error::Error>> {
    let topics = {
        let topics = chat_topics_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        topics.clone()
    };
    let mut subscribed = subscribed_chat_topics_slot()
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    for topic in topics {
        if subscribed.insert(topic.clone()) {
            swarm
                .behaviour_mut()
                .gossipsub
                .subscribe(&IdentTopic::new(topic.clone()))?;
            push_log(format!("subscribed chat topic {topic}"));
        }
    }

    Ok(())
}

fn sync_chat_topics_from_disk() {
    let Some(path) = chat_topics_file_path() else {
        push_log("chat topic sync skipped: HOME is not set");
        return;
    };

    let Ok(contents) = std::fs::read(&path) else {
        return;
    };

    if let Ok(payload) = serde_json::from_slice::<EncryptedChatTopicsPayload>(&contents) {
        if let Some(private_key_seed) = topic_list_private_key_seed() {
            if let Ok(topics) = decrypt_chat_topics(payload, &private_key_seed) {
                for topic in topics {
                    let _ = register_chat_topic(topic);
                }
            }
        }
        return;
    }

    if let Ok(contents) = String::from_utf8(contents) {
        for topic in legacy_chat_topics(&contents) {
            let _ = register_chat_topic(topic);
        }
    }
}

fn sync_chat_topics(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
) -> Result<(), Box<dyn std::error::Error>> {
    sync_chat_topics_from_disk();
    sync_registered_chat_topics(swarm)
}

pub fn register_chat_topic(topic: impl Into<String>) -> String {
    let topic = topic.into().trim().to_string();
    if topic.is_empty() {
        push_log("skipping empty chat topic");
        return String::from("skipping empty chat topic");
    }

    let mut topics = chat_topics_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if topics.insert(topic.clone()) {
        push_log(format!("registered chat topic {topic}"));
        chat_topic_sync_notify().notify_one();
    }
    topic
}

pub fn send_chat_message(topic: impl Into<String>, message: impl Into<String>) -> String {
    let topic = topic.into().trim().to_string();
    let message = message.into().trim().to_string();

    if topic.is_empty() {
        push_log("skipping empty chat message topic");
        return String::from("skipping empty chat message topic");
    }

    if message.is_empty() {
        push_log(format!("skipping empty chat message on topic {topic}"));
        return String::from("skipping empty chat message");
    }

    let sender = {
        let guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        let Some(state) = guard.as_ref() else {
            push_log("cannot send chat message: network not running");
            return String::from("p2p network not running");
        };
        state.chat_tx.clone()
    };

    if sender.send(ChatCommand { topic: topic.clone(), message: message.clone() }).is_err() {
        push_log(format!("failed to queue chat message on topic {topic}"));
        return String::from("failed to queue chat message");
    }

    push_log(format!("queued chat message '{message}' on topic '{topic}'"));
    message
}

fn legacy_chat_topics(contents: &str) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut topics = Vec::new();

    for topic in contents
        .split(|c| c == '\n' || c == ',')
        .map(|topic| topic.trim())
        .filter(|topic| !topic.is_empty())
    {
        let topic = topic.to_string();
        if seen.insert(topic.clone()) {
            topics.push(topic);
        }
    }

    topics
}

fn topic_list_private_key_seed() -> Option<String> {
    let value = std::env::var("GNOSTR_NSEC").ok()?;
    let normalized = value.trim().to_lowercase();
    if normalized.is_empty() {
        return None;
    }

    if normalized.starts_with("nsec") {
        PrivateKey::try_from_bech32_string(&normalized).ok()?;
    } else {
        PrivateKey::try_from_hex_string(&normalized).ok()?;
    }

    Some(normalized)
}

fn chat_topics_key(seed: &str) -> [u8; 32] {
    Sha256::digest(seed.as_bytes()).into()
}

fn decrypt_chat_topics(
    payload: EncryptedChatTopicsPayload,
    private_key_seed: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let nonce = base64::engine::general_purpose::STANDARD.decode(payload.nonce)?;
    let ciphertext = base64::engine::general_purpose::STANDARD.decode(payload.ciphertext)?;
    let tag = base64::engine::general_purpose::STANDARD.decode(payload.tag)?;

    if nonce.len() != 12 || tag.len() != 16 {
        return Err("invalid encrypted chat topics payload".into());
    }

    let mut combined = ciphertext;
    combined.extend(tag);

    let key = chat_topics_key(private_key_seed);
    let cipher = ChaCha20Poly1305::new((&key).into());
    let plaintext = cipher.decrypt(
        Nonce::from_slice(&nonce),
        Payload {
            msg: &combined,
            aad: &[],
        },
    ).map_err(|error| {
        std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("chat topic decryption failed: {error}"),
        )
    })?;

    let topics: Vec<String> = serde_json::from_slice(&plaintext)?;
    let mut seen = HashSet::new();
    let mut normalized = Vec::new();
    for topic in topics {
        let topic = topic.trim().to_string();
        if !topic.is_empty() && seen.insert(topic.clone()) {
            normalized.push(topic);
        }
    }

    Ok(normalized)
}

fn timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default()
}

fn advertised_addresses(swarm: &libp2p::Swarm<crate::behaviour::Behaviour>) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut addresses = Vec::new();

    for addr in swarm.external_addresses() {
        let addr = addr.to_string();
        if seen.insert(addr.clone()) {
            addresses.push(addr);
        }
    }

    addresses
}

pub fn publish_presence(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
    peer_id: PeerId,
) -> Result<(), Box<dyn std::error::Error>> {
    let addresses = advertised_addresses(swarm);
    if addresses.is_empty() {
        push_log("skipping presence publish: no external addresses yet");
        return Ok(());
    }

    let presence = PeerPresence {
        peer_id: peer_id.to_string(),
        addresses,
        timestamp_secs: timestamp_secs(),
    };
    let payload = serde_json::to_vec(&presence)?;
    swarm
        .behaviour_mut()
        .gossipsub
        .publish(discovery_topic(), payload)?;
    push_log(format!("published presence for peer={peer_id}"));
    Ok(())
}

pub fn subscribe_to_discovery_topic(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
) -> Result<(), Box<dyn std::error::Error>> {
    swarm
        .behaviour_mut()
        .gossipsub
        .subscribe(&discovery_topic())?;
    Ok(())
}

pub fn subscribe_to_chat_topic(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
) -> Result<(), Box<dyn std::error::Error>> {
    sync_chat_topics(swarm)
}

pub fn bootstrap_public_dht(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
) -> Result<(), Box<dyn std::error::Error>> {
    for (addr, peer_id) in crate::network_config::Network::Ipfs.bootnodes() {
        let address_with_peer = multiaddr_with_peer_id(&addr, &peer_id);
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&peer_id, address_with_peer.clone());
        swarm
            .behaviour_mut()
            .autonat
            .add_server(peer_id, Some(address_with_peer.clone()));
        if let Err(error) = swarm.dial(address_with_peer) {
            push_log(format!("failed to dial bootstrap peer {peer_id}: {error}"));
        }
    }

    if let Err(error) = swarm.behaviour_mut().kademlia.start_providing(discovery_key()) {
        push_log(format!("failed to start providing discovery key: {error}"));
    }

    match swarm.behaviour_mut().kademlia.bootstrap() {
        Ok(_) => push_log("started kademlia bootstrap"),
        Err(error) => push_log(format!("kademlia bootstrap deferred: {error}")),
    }

    Ok(())
}

pub fn refresh_wide_area_discovery(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
    peer_id: PeerId,
) -> Result<(), Box<dyn std::error::Error>> {
    sync_chat_topics(swarm)?;
    subscribe_to_discovery_topic(swarm)?;
    bootstrap_public_dht(swarm)?;
    publish_presence(swarm, peer_id)?;
    swarm.behaviour_mut().kademlia.get_providers(discovery_key());
    Ok(())
}

pub fn handle_presence_message(
    swarm: &mut libp2p::Swarm<crate::behaviour::Behaviour>,
    payload: &[u8],
) -> Result<(), Box<dyn std::error::Error>> {
    let presence: PeerPresence = serde_json::from_slice(payload)?;
    let local_peer_id = swarm.local_peer_id().clone();
    let remote_peer_id: PeerId = presence.peer_id.parse()?;

    if remote_peer_id == local_peer_id {
        return Ok(());
    }

    let mut discovered_addresses = Vec::new();
    let mut announced = 0usize;
    for addr in presence.addresses {
        let Ok(addr) = addr.parse::<Multiaddr>() else {
            continue;
        };

        let addr_with_peer = multiaddr_with_peer_id(&addr, &remote_peer_id);
        discovered_addresses.push(addr_with_peer.to_string());
        swarm
            .behaviour_mut()
            .kademlia
            .add_address(&remote_peer_id, addr_with_peer.clone());
        swarm
            .behaviour_mut()
            .autonat
            .add_server(remote_peer_id, Some(addr_with_peer.clone()));

        if let Err(error) = swarm.dial(addr_with_peer.clone()) {
            push_log(format!(
                "failed to dial presence peer {remote_peer_id} at {addr_with_peer}: {error}"
            ));
        } else {
            announced += 1;
        }
    }

    if !discovered_addresses.is_empty() {
        record_peer_discovery(
            remote_peer_id.to_string(),
            "presence",
            discovered_addresses,
        );
    }

    if announced > 0 {
        push_log(format!(
            "presence discovery announced peer={remote_peer_id} addrs={announced}"
        ));
    }

    Ok(())
}

pub fn start() -> String {
    let mut guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
    if let Some(state) = guard.as_ref() {
        push_log("p2p network already running");
        return state
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone();
    }

    reset_topic_state();
    clear_logs();
    clear_peers();
    let status = Arc::new(Mutex::new(String::from("starting p2p network")));
    let thread_status = Arc::clone(&status);
    let thread_logs = Arc::new(Mutex::new(Vec::new()));
    let thread_logs_clone = Arc::clone(&thread_logs);
    let (shutdown_tx, mut shutdown_rx) = oneshot::channel::<()>();
    let (chat_tx, mut chat_rx) = mpsc::unbounded_channel::<ChatCommand>();

    let join = thread::spawn(move || {
        let runtime = Builder::new_multi_thread()
            .enable_all()
            .worker_threads(2)
            .build()
            .expect("tokio runtime");

        runtime.block_on(async move {
            tokio::spawn(async {
                if let Err(error) = run_time_sync_daemon().await {
                    push_log(format!("p2p time sync daemon failed: {error}"));
                }
            });

            let identity_seed = relay_identity_seed();
            let keypair = keypair_from_seed(Some(identity_seed.clone()));
            let peer_id = keypair.public().to_peer_id();
            push_log(format!("relay identity {} peer={peer_id}", relay_identity_label()));
            push_log(format!("p2p network starting peer={peer_id}"));

            {
                let mut status = thread_status
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *status = format!("starting p2p network peer={peer_id}");
            }

            let mut swarm = match swarm_builder::build_swarm(keypair).await {
                Ok(swarm) => swarm,
                Err(error) => {
                    push_log(format!("p2p network failed to build: {error}"));
                    let mut status = thread_status
                        .lock()
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    *status = format!("p2p network failed to build: {error}");
                    return;
                }
            };

            swarm.behaviour_mut().kademlia.set_mode(Some(kad::Mode::Server));

            if let Err(error) = cli::listen_default_addresses(&mut swarm, None, 0, false) {
                push_log(format!("p2p network failed to listen: {error}"));
                let mut status = thread_status
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *status = format!("p2p network failed to listen: {error}");
                return;
            }

            if let Err(error) = refresh_wide_area_discovery(&mut swarm, peer_id) {
                push_log(format!("wide-area discovery init failed: {error}"));
            }

            {
                let mut status = thread_status
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                *status = format!("running p2p network peer={peer_id}");
            }
            push_log(format!("p2p network running peer={peer_id}"));

            let mut discovery_tick =
                tokio::time::interval(std::time::Duration::from_secs(DISCOVERY_REFRESH_SECS));
            discovery_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut chat_tick = tokio::time::interval(std::time::Duration::from_secs(1));
            chat_tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut chat_topic_sync = Box::pin(chat_topic_sync_notify().notified());

            loop {
                tokio::select! {
                    _ = &mut shutdown_rx => {
                        push_log(format!("p2p network stopping peer={peer_id}"));
                        break;
                    }
                    Some(command) = chat_rx.recv() => {
                        let payload = serde_json::json!({
                            "from": relay_identity_label(),
                            "content": [command.message.clone()],
                        });
                        match serde_json::to_vec(&payload) {
                            Ok(payload) => {
                                let topic = IdentTopic::new(command.topic.clone());
                                match swarm.behaviour_mut().gossipsub.publish(topic, payload) {
                                    Ok(_) => {
                                        push_log(format!(
                                            "sent message '{}' on topic '{}'",
                                            command.message, command.topic
                                        ));
                                    }
                                    Err(error) => {
                                        push_log(format!(
                                            "failed to publish chat message on topic {}: {}",
                                            command.topic, error
                                        ));
                                    }
                                }
                            }
                            Err(error) => {
                                push_log(format!("failed to encode chat message: {error}"));
                            }
                        }
                    }
                    _ = chat_topic_sync.as_mut() => {
                        if let Err(error) = sync_chat_topics(&mut swarm) {
                            push_log(format!("chat topic sync failed: {error}"));
                        }
                        chat_topic_sync = Box::pin(chat_topic_sync_notify().notified());
                    }
                    _ = discovery_tick.tick() => {
                        if let Err(error) = refresh_wide_area_discovery(&mut swarm, peer_id) {
                            push_log(format!("wide-area discovery refresh failed: {error}"));
                        }
                    }
                    _ = chat_tick.tick() => {
                        if let Err(error) = sync_chat_topics(&mut swarm) {
                            push_log(format!("chat topic sync failed: {error}"));
                        }
                    }
                    event = swarm.select_next_some() => {
                        if let SwarmEvent::NewListenAddr { address, .. } = &event {
                            push_log(format!("p2p network listening peer={peer_id} addr={address}"));
                            let mut status = thread_status
                                .lock()
                                .unwrap_or_else(|poisoned| poisoned.into_inner());
                            *status = format!("running p2p network peer={peer_id} listen={address}");
                        }
                        event_handler::handle_swarm_event(&mut swarm, event).await;
                    }
                }
            }

            let mut status = thread_status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            *status = format!("stopped p2p network peer={peer_id}");
            push_log(format!("p2p network stopped peer={peer_id}"));
        });
    });

    let snapshot = status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    *guard = Some(EmbeddedNetwork {
        status,
        logs: thread_logs_clone,
        chat_tx,
        shutdown: Some(shutdown_tx),
        join: Some(join),
    });
    snapshot
}

pub fn stop() -> String {
    let state = {
        let mut guard = network_slot().lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        guard.take()
    };
    let Some(mut state) = state else {
        return String::from("p2p network not running");
    };

    if let Some(shutdown) = state.shutdown.take() {
        let _ = shutdown.send(());
    }
    if let Some(join) = state.join.take() {
        let _ = join.join();
    }

    let status = state
        .status
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    push_log(status.clone());
    status
}

fn private_key_seed_from_env() -> Option<String> {
    std::env::var("GNOSTR_NSEC")
        .ok()
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn padded_metric_identity(metric: &str) -> String {
    format!("{:0>64}", metric.trim())
}

fn padded_blockheight_identity(blockheight: &str) -> String {
    padded_metric_identity(blockheight)
}

fn padded_weeble_identity(weeble: &str) -> String {
    padded_metric_identity(weeble)
}

fn padded_wobble_identity(wobble: &str) -> String {
    padded_metric_identity(wobble)
}

fn blockheight_relay_seed() -> String {
    padded_blockheight_identity(&blockheight_sync())
}

fn weeble_relay_seed() -> String {
    padded_weeble_identity(&weeble_sync().unwrap_or(0.0).to_string())
}

fn wobble_relay_seed() -> String {
    padded_wobble_identity(&wobble_sync().unwrap_or(0.0).to_string())
}

fn relay_kind_from_env() -> Option<String> {
    std::env::var("GNOSTR_RELAY_KIND")
        .ok()
        .map(|value| value.trim().to_lowercase())
        .filter(|value| !value.is_empty())
}

fn relay_identity_seed() -> String {
    if let Some(seed) = private_key_seed_from_env() {
        return seed;
    }

    match relay_kind_from_env().as_deref() {
        Some("wobble") => wobble_relay_seed(),
        Some("weeble") => weeble_relay_seed(),
        _ => blockheight_relay_seed(),
    }
}

fn relay_identity_label() -> &'static str {
    if private_key_seed_from_env().is_some() {
        "explicit"
    } else {
        match relay_kind_from_env().as_deref() {
            Some("wobble") => "wobble_relay",
            Some("weeble") => "weeble_relay",
            _ => "blockheight_relay",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{padded_blockheight_identity, padded_weeble_identity, padded_wobble_identity};

    #[test]
    fn padded_blockheight_identity_is_sha256_length() {
        let identity = padded_blockheight_identity("12345");
        println!("blockheight identity seed: {identity}");
        assert_eq!(identity.len(), 64);
        assert!(identity.ends_with("12345"));
        assert!(identity.chars().take(59).all(|c| c == '0'));
    }

    #[test]
    fn padded_weeble_identity_is_sha256_length() {
        let identity = padded_weeble_identity("9876.5");
        println!("weeble identity seed: {identity}");
        assert_eq!(identity.len(), 64);
        assert!(identity.ends_with("9876.5"));
        assert!(identity.chars().take(58).all(|c| c == '0'));
    }

    #[test]
    fn padded_wobble_identity_is_sha256_length() {
        let identity = padded_wobble_identity("123.456");
        println!("wobble identity seed: {identity}");
        assert_eq!(identity.len(), 64);
        assert!(identity.ends_with("123.456"));
        assert!(identity.chars().take(57).all(|c| c == '0'));
    }
}
