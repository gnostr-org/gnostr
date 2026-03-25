use std::sync::{Arc, Mutex};

use anyhow::{anyhow, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use rand::Rng;
use tokio::{net::TcpStream, sync::mpsc};
use tokio_tungstenite::{connect_async, tungstenite, MaybeTlsStream, WebSocketStream};
use tracing::{debug, info, warn};

use crate::types::{
    ClientMessage, EventKind, Filter, PublicKey, RelayMessage, SubscriptionId, UncheckedUrl,
    Unixtime,
}; // Added PublicKey
use crate::{
    queue::{InternalEvent, Queue},
    types::versioned::{client_message3::ClientMessageV3, event3::EventV3},
};

type WsSink =
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Clone, Debug)]
pub struct NostrClient {
    queue_tx: mpsc::Sender<InternalEvent>,
    relay_sinks: Arc<Mutex<Vec<(UncheckedUrl, WsSink)>>>,
}

impl NostrClient {
    pub fn new(queue_tx: mpsc::Sender<InternalEvent>) -> Self {
        Self {
            queue_tx,
            relay_sinks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn connect_relay(&mut self, url: UncheckedUrl) -> Result<()> {
        info!("Connecting to Nostr relay: {}", url.0);
        let (ws_stream, _) = connect_async(&url.0).await?;
        info!("Connected to Nostr relay: {}", url.0);

        let (sink, stream) = ws_stream.split();
        self.spawn_listener_task(url.clone(), stream);
        self.relay_sinks.lock().unwrap().push((url, sink));

        Ok(())
    }

    fn spawn_listener_task(&self, url: UncheckedUrl, mut stream: WsStream) {
        let queue_tx_clone = self.queue_tx.clone();

        let _ = crate::p2p::chat::global_rt().spawn(async move {
            while let Some(message_result) = stream.next().await {
                match message_result {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                        debug!("Received from {}: {}", url.0, text);
                        match serde_json::from_str::<RelayMessage>(&text) {
                            Ok(RelayMessage::Event(_sub_id, event)) => {
                                info!("Received Nostr event from {}: {:?}", url.0, event);
                                if let Err(e) =
                                    queue_tx_clone.send(InternalEvent::NostrEvent(*event)).await
                                {
                                    warn!("Failed to send NostrEvent to queue: {}", e);
                                }
                            }
                            Ok(other) => {
                                debug!("Received other relay message from {}: {:?}", url.0, other)
                            }
                            Err(e) => warn!(
                                "Failed to parse relay message from {}: {}. Message: {}",
                                url.0, e, text
                            ),
                        }
                    }
                    Ok(other) => debug!("Received non-text message from {}: {:?}", url.0, other),
                    Err(e) => {
                        warn!("WebSocket error on {}: {}", url.0, e);
                        break;
                    }
                }
            }
            info!("Relay listener for {} disconnected.", url.0);
        });
    }

    pub async fn send_event(&self, event: EventV3) -> Result<()> {
        let client_message = ClientMessage::Event(Box::new(event));
        let json = serde_json::to_string(&client_message)?;
        let websocket_message = tokio_tungstenite::tungstenite::Message::Text(json.into());

        // Temporarily take ownership of the sinks from the Mutex
        let mut sinks_to_send = {
            let mut sinks_guard = self.relay_sinks.lock().unwrap();
            sinks_guard.drain(..).collect::<Vec<_>>()
        };

        for (url, sink) in sinks_to_send.iter_mut() {
            info!("Sending Nostr event to relay {}", url.0);
            if let Err(e) = sink.send(websocket_message.clone()).await {
                warn!("Failed to send event to relay {}: {}", url.0, e);
            }
        }

        // Put the sinks back into the Mutex, preserving their state (e.g., if one
        // failed it's still here)
        {
            let mut sinks_guard = self.relay_sinks.lock().unwrap();
            sinks_guard.extend(sinks_to_send.drain(..));
        }

        Ok(())
    }

    pub async fn subscribe(&self, public_key: Option<PublicKey>) {
        let subscription_id = if let Some(pk) = public_key {
            info!("Subscribing to text notes from {}", pk.as_hex_string());
            SubscriptionId(format!("notes:{}", pk.as_hex_string()))
        } else {
            info!("Subscribing to all text notes");
            SubscriptionId("notes:all".to_string())
        };

        let filter = {
            let mut f = Filter::new();
            if let Some(pk) = public_key {
                let _ = f.add_author(&pk.into());
            }
            let _ = f.add_event_kind(EventKind::TextNote);
            f.since = Some(Unixtime::now());
            f
        };
        let client_message = ClientMessage::Req(subscription_id, vec![filter.clone()]);
        let json = serde_json::to_string(&client_message).unwrap();
        let websocket_message = tokio_tungstenite::tungstenite::Message::Text(json.into());

        let mut sinks = self.relay_sinks.lock().unwrap();
        for (url, sink) in sinks.iter_mut() {
            if let Err(e) = sink.send(websocket_message.clone()).await {
                warn!("Failed to send REQ to relay {}: {}", url.0, e);
            }
        }
    }

    pub async fn subscribe_to_dms(&self, public_key: PublicKey) {
        info!("Subscribing to DMs for {}", public_key.as_hex_string());

        let filter = {
            let mut f = Filter::new();
            let _ = f.add_tag_value('p', public_key.as_hex_string());
            let _ = f.add_event_kind(EventKind::EncryptedDirectMessage);
            let _ = f.add_event_kind(EventKind::GiftWrap);
            f.since = Some(Unixtime::now());
            f
        };
        let subscription_id = SubscriptionId(format!("dms:{}", public_key.as_hex_string()));

        let client_message = ClientMessage::Req(subscription_id, vec![filter.clone()]);
        let json = serde_json::to_string(&client_message).unwrap();
        let websocket_message = tokio_tungstenite::tungstenite::Message::Text(json.into());

        let mut sinks = self.relay_sinks.lock().unwrap();
        for (url, sink) in sinks.iter_mut() {
            if let Err(e) = sink.send(websocket_message.clone()).await {
                warn!("Failed to send REQ to relay {}: {}", url.0, e);
            }
        }
    }

    pub async fn subscribe_to_contact_lists(&self, public_key: PublicKey) {
        info!(
            "Subscribing to contact lists for {}",
            public_key.as_hex_string()
        );

        let filter = {
            let mut f = Filter::new();
            let _ = f.add_author(&public_key.into());
            let _ = f.add_event_kind(EventKind::ContactList);
            f.limit = Some(1);
            f
        };
        let subscription_id = SubscriptionId(format!("contacts:{}", public_key.as_hex_string()));

        let client_message = ClientMessage::Req(subscription_id, vec![filter.clone()]);
        let json = serde_json::to_string(&client_message).unwrap();
        let websocket_message = tokio_tungstenite::tungstenite::Message::Text(json.into());

        let mut sinks = self.relay_sinks.lock().unwrap();
        for (url, sink) in sinks.iter_mut() {
            if let Err(e) = sink.send(websocket_message.clone()).await {
                warn!("Failed to send REQ to relay {}: {}", url.0, e);
            }
        }
    }

    pub async fn subscribe_to_channel(&self, channel_id: String) {
        info!("Subscribing to Nostr channel: {}", channel_id);

        let filter = {
            let mut f = Filter::new();
            let _ = f.add_tag_value('d', channel_id.clone());
            let _ = f.add_event_kind(EventKind::ChannelMessage);
            f.since = Some(Unixtime::now());
            f
        };

        let subscription_id = SubscriptionId(format!("channel:{}", channel_id));

        let client_message = ClientMessage::Req(subscription_id, vec![filter.clone()]);
        let json = serde_json::to_string(&client_message).unwrap();
        let websocket_message = tokio_tungstenite::tungstenite::Message::Text(json.into());

        let mut sinks = self.relay_sinks.lock().unwrap();
        for (url, sink) in sinks.iter_mut() {
            if let Err(e) = sink.send(websocket_message.clone()).await {
                warn!("Failed to send REQ to relay {}: {}", url.0, e);
            }
        }
    }

    pub async fn subscribe_to_marketplace(&self) {
        info!("Subscribing to marketplace events");

        let filter = {
            let mut f = Filter::new();
            let _ = f.add_event_kind(EventKind::MarketplaceUi);
            f.since = Some(Unixtime::now());
            f
        };

        let subscription_id = SubscriptionId("marketplace".to_string());

        let client_message = ClientMessage::Req(subscription_id, vec![filter.clone()]);
        let json = serde_json::to_string(&client_message).unwrap();
        let websocket_message = tokio_tungstenite::tungstenite::Message::Text(json.into());

        let mut sinks = self.relay_sinks.lock().unwrap();
        for (url, sink) in sinks.iter_mut() {
            if let Err(e) = sink.send(websocket_message.clone()).await {
                warn!("Failed to send REQ to relay {}: {}", url.0, e);
            }
        }
    }
}
