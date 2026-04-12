//! Minimal async Nostr relay client.
//!
//! Connects to a single relay over WebSocket (tokio-tungstenite), lets the
//! caller subscribe with NIP-01 filters, and streams received events back via
//! an `mpsc` channel.  Publishing signed events is also supported.
//!
//! The connection runs in a spawned task that reconnects automatically.
//!
//! # Wire protocol (NIP-01)
//! Client → Relay:
//!   `["REQ", <sub-id>, <filter>, …]`
//!   `["CLOSE", <sub-id>]`
//!   `["EVENT", <event-object>]`
//!
//! Relay → Client:
//!   `["EVENT", <sub-id>, <event-object>]`
//!   `["EOSE",  <sub-id>]`
//!   `["OK",    <event-id>, <true|false>, <message>]`
//!   `["NOTICE", <message>]`

use std::collections::HashMap;

use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tokio::sync::mpsc;
use tokio_tungstenite::{connect_async, tungstenite::Message};

// ── Public types ──────────────────────────────────────────────────────────────

/// A Nostr event as returned by relay subscriptions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelayEvent {
    pub id:         String,
    pub pubkey:     String,
    pub created_at: u64,
    pub kind:       u32,
    pub tags:       Vec<Vec<String>>,
    pub content:    String,
    pub sig:        String,
}

/// Messages sent from the relay task back to the UI.
#[derive(Debug, Clone)]
pub enum RelayMsg {
    /// An event received for a subscription.
    Event { sub_id: String, event: RelayEvent },
    /// End-of-stored-events for a subscription.
    Eose { sub_id: String },
    /// Result of an EVENT publish.
    Ok { event_id: String, accepted: bool, message: String },
    /// Human-readable message from the relay.
    Notice(String),
    /// WebSocket connected.
    Connected,
    /// WebSocket disconnected (will reconnect).
    Disconnected,
}

/// Commands sent from the UI to the relay task.
#[derive(Debug)]
pub enum RelayCmd {
    /// Subscribe with a filter. `sub_id` must be unique.
    Subscribe { sub_id: String, filter: Value },
    /// Close a subscription by id.
    Close { sub_id: String },
    /// Publish a signed event JSON object.
    Publish(Value),
    /// Disconnect and shut down.
    Shutdown,
}

// ── NIP-01 filter helper ──────────────────────────────────────────────────────

/// Build a NIP-01 filter JSON value.
pub fn filter(
    kinds: &[u32],
    authors: &[&str],
    limit: Option<u32>,
) -> Value {
    let mut f = serde_json::Map::new();
    if !kinds.is_empty() {
        f.insert("kinds".into(), json!(kinds));
    }
    if !authors.is_empty() {
        f.insert("authors".into(), json!(authors));
    }
    if let Some(l) = limit {
        f.insert("limit".into(), json!(l));
    }
    Value::Object(f)
}

/// Build a filter that fetches a single replaceable kind for an author.
pub fn filter_kind_author(kind: u32, author: &str) -> Value {
    filter(&[kind], &[author], Some(1))
}

// ── Task entry point ──────────────────────────────────────────────────────────

/// Spawn the relay connection task.
///
/// Returns `(cmd_tx, msg_rx)`:
/// - Send [`RelayCmd`]s on `cmd_tx` to subscribe, publish, etc.
/// - Receive [`RelayMsg`]s on `msg_rx` for events, EOSE, OK, etc.
pub fn spawn(url: String) -> (mpsc::Sender<RelayCmd>, mpsc::Receiver<RelayMsg>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<RelayCmd>(64);
    let (msg_tx, msg_rx) = mpsc::channel::<RelayMsg>(256);
    tokio::spawn(relay_task(url, cmd_rx, msg_tx));
    (cmd_tx, msg_rx)
}

// ── Internal relay task ───────────────────────────────────────────────────────

async fn relay_task(
    url: String,
    mut cmd_rx: mpsc::Receiver<RelayCmd>,
    msg_tx: mpsc::Sender<RelayMsg>,
) {
    // Pending subscriptions to re-issue on reconnect.
    let mut subs: HashMap<String, Value> = HashMap::new();

    loop {
        let ws = match connect_async(&url).await {
            Ok((ws, _)) => ws,
            Err(e) => {
                eprintln!("relay connect failed: {e}");
                let _ = msg_tx.send(RelayMsg::Disconnected).await;
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                continue;
            }
        };

        let _ = msg_tx.send(RelayMsg::Connected).await;
        let (mut sink, mut stream) = ws.split();

        // Re-issue all active subscriptions.
        for (id, filter) in &subs {
            let req = json!(["REQ", id, filter]).to_string();
            if sink.send(Message::Text(req.into())).await.is_err() {
                break;
            }
        }

        loop {
            tokio::select! {
                // Relay → UI
                Some(msg) = stream.next() => {
                    match msg {
                        Ok(Message::Text(txt)) => {
                            handle_relay_msg(txt.as_str(), &msg_tx).await;
                        }
                        Ok(Message::Ping(payload)) => {
                            let _ = sink.send(Message::Pong(payload)).await;
                        }
                        Ok(Message::Close(_)) | Err(_) => {
                            let _ = msg_tx.send(RelayMsg::Disconnected).await;
                            break;
                        }
                        _ => {}
                    }
                }
                // UI → Relay
                Some(cmd) = cmd_rx.recv() => {
                    match cmd {
                        RelayCmd::Shutdown => return,
                        RelayCmd::Subscribe { sub_id, filter } => {
                            let req = json!(["REQ", &sub_id, &filter]).to_string();
                            subs.insert(sub_id, filter);
                            if sink.send(Message::Text(req.into())).await.is_err() {
                                let _ = msg_tx.send(RelayMsg::Disconnected).await;
                                break;
                            }
                        }
                        RelayCmd::Close { sub_id } => {
                            subs.remove(&sub_id);
                            let msg = json!(["CLOSE", &sub_id]).to_string();
                            let _ = sink.send(Message::Text(msg.into())).await;
                        }
                        RelayCmd::Publish(event) => {
                            let msg = json!(["EVENT", event]).to_string();
                            if sink.send(Message::Text(msg.into())).await.is_err() {
                                let _ = msg_tx.send(RelayMsg::Disconnected).await;
                                break;
                            }
                        }
                    }
                }
                else => break,
            }
        }

        // Short back-off before reconnect.
        tokio::time::sleep(std::time::Duration::from_secs(3)).await;
    }
}

async fn handle_relay_msg(raw: &str, tx: &mpsc::Sender<RelayMsg>) {
    let Ok(arr) = serde_json::from_str::<Value>(raw) else {
        return;
    };
    let Some(arr) = arr.as_array() else { return };
    let verb = arr.first().and_then(|v| v.as_str()).unwrap_or("");

    match verb {
        "EVENT" => {
            let sub_id = arr.get(1).and_then(|v| v.as_str()).unwrap_or("").to_owned();
            if let Some(ev_val) = arr.get(2) {
                if let Ok(event) = serde_json::from_value::<RelayEvent>(ev_val.clone()) {
                    let _ = tx.send(RelayMsg::Event { sub_id, event }).await;
                }
            }
        }
        "EOSE" => {
            let sub_id = arr.get(1).and_then(|v| v.as_str()).unwrap_or("").to_owned();
            let _ = tx.send(RelayMsg::Eose { sub_id }).await;
        }
        "OK" => {
            let event_id = arr.get(1).and_then(|v| v.as_str()).unwrap_or("").to_owned();
            let accepted = arr.get(2).and_then(|v| v.as_bool()).unwrap_or(false);
            let message  = arr.get(3).and_then(|v| v.as_str()).unwrap_or("").to_owned();
            let _ = tx.send(RelayMsg::Ok { event_id, accepted, message }).await;
        }
        "NOTICE" => {
            let msg = arr.get(1).and_then(|v| v.as_str()).unwrap_or("").to_owned();
            let _ = tx.send(RelayMsg::Notice(msg)).await;
        }
        _ => {}
    }
}

// ── Tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn filter_kinds_and_authors() {
        let f = filter(&[0, 3], &["aabbcc"], Some(10));
        assert_eq!(f["kinds"], json!([0, 3]));
        assert_eq!(f["authors"], json!(["aabbcc"]));
        assert_eq!(f["limit"], json!(10));
    }

    #[test]
    fn filter_kind_author_helper() {
        let f = filter_kind_author(0, "deadbeef");
        assert_eq!(f["kinds"], json!([0]));
        assert_eq!(f["limit"], json!(1));
    }
}
