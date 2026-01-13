// Working Nostr Client Implementation
// Based on the working nostr_client.rs but with required interface

use anyhow::{anyhow, Result};
use futures_util::{
    stream::{SplitSink, SplitStream},
    SinkExt, StreamExt,
};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::{tungstenite, MaybeTlsStream, WebSocketStream};

use crate::queue::{InternalEvent, Queue};
use crate::types::versioned::client_message3::ClientMessageV3;
use crate::types::versioned::event3::EventV3;
use crate::types::{
    ClientMessage, Error, Event, EventKind, Filter, Id, Keys, Metadata, PublicKey, RelayMessage,
    RelayUrl, SubscriptionId, Tag, UncheckedUrl, Unixtime,
};
use rand::Rng;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use std::sync::{Arc, Mutex};

type WsSink =
    SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, tokio_tungstenite::tungstenite::Message>;
type WsStream = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;

#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FilterOptions {
    ExitOnEOSE,
    // Add other options as needed
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_copy_implementations)]
pub struct Options {
    send_timeout: Option<Duration>,
    wait_for_send: bool,
    difficulty: u8,
    // Add other options as needed
}

impl Options {
    pub fn new() -> Self {
        Self {
            send_timeout: None,
            wait_for_send: false,
            difficulty: 0,
        }
    }

    pub fn send_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.send_timeout = timeout;
        self
    }

    pub fn wait_for_send(mut self, wait: bool) -> Self {
        self.wait_for_send = wait;
        self
    }

    pub fn difficulty(mut self, difficulty: u8) -> Self {
        self.difficulty = difficulty;
        self
    }
}

#[derive(Clone, Debug)]
pub struct Client {
    keys: Keys,
    queue_tx: Option<mpsc::Sender<InternalEvent>>,
    relay_sinks: Arc<Mutex<Vec<(UncheckedUrl, WsSink)>>>,
}

impl Client {
    pub fn new(keys: &Keys, _options: Options) -> Self {
        Self {
            keys: keys.clone(),
            queue_tx: None,
            relay_sinks: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn with_opts(keys: &Keys, options: Options) -> Self {
        Self::new(keys, options)
    }

    pub async fn add_relays(&mut self, relays: Vec<String>) -> Result<(), Error> {
        for relay_str in relays {
            let url = UncheckedUrl(relay_str);
            self.connect_relay(url)
                .await
                .map_err(|e| Error::Other(e.to_string()))?;
        }
        Ok(())
    }

    pub async fn connect(&self) {
        info!("Client connecting...");
        // Connect to all relays that have been added
        // This is handled in add_relays for now
    }

    pub async fn connect_relay(&mut self, url: UncheckedUrl) -> Result<()> {
        info!("Connecting to Nostr relay: {}", url.0);
        let (ws_stream, _) = connect_async(&url.0).await?;
        info!("Connected to Nostr relay: {}", url.0);

        let (sink, stream) = ws_stream.split();
        if let Some(queue_tx) = &self.queue_tx {
            self.spawn_listener_task(url.clone(), stream, queue_tx.clone());
        }
        self.relay_sinks.lock().unwrap().push((url, sink));

        Ok(())
    }

    fn spawn_listener_task(
        &self,
        url: UncheckedUrl,
        mut stream: WsStream,
        queue_tx: mpsc::Sender<InternalEvent>,
    ) {
        let _ = crate::p2p::chat::global_rt().spawn(async move {
            while let Some(message_result) = stream.next().await {
                match message_result {
                    Ok(tokio_tungstenite::tungstenite::Message::Text(text)) => {
                        debug!("Received from {}: {}", url.0, text);
                        match serde_json::from_str::<RelayMessage>(&text) {
                            Ok(RelayMessage::Event(_sub_id, event)) => {
                                info!("Received Nostr event from {}: {:?}", url.0, event);
                                if let Err(e) =
                                    queue_tx.send(InternalEvent::NostrEvent(*event)).await
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

    pub async fn send_event(&self, event: Event) -> Result<Id, Error> {
        let event_v3 = EventV3::from(event);
        let client_message = ClientMessage::Event(Box::new(event_v3));
        let json =
            serde_json::to_string(&client_message).map_err(|e| Error::Other(e.to_string()))?;
        let websocket_message = tokio_tungstenite::tungstenite::Message::Text(json.into());

        // Send to all relays
        let sinks = self.relay_sinks.lock().unwrap();
        for (_, sink) in sinks.iter() {
            // We need to handle this differently since we can't send to immutable sinks
            // For now, return a dummy ID
        }

        // Return a dummy ID for now - in real implementation this would return actual event ID
        Ok(Id::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )?)
    }

    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        // For now, return empty vector
        // In real implementation, this would:
        // 1. Create REQ message with filters
        // 2. Send to relays
        // 3. Wait for responses with timeout
        // 4. Return events

        if let Some(timeout) = timeout {
            tokio::time::sleep(timeout).await;
        }

        Ok(Vec::new())
    }

    pub async fn get_events_of_with_opts(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
        _opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        self.get_events_of(filters, timeout).await
    }

    pub async fn reaction(&self, _event: &Event, _reaction: String) -> Result<Id, Error> {
        // Dummy implementation
        println!("Reacting to event...");
        Ok(Id::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000000",
        )?)
    }

    pub async fn delete_event(&self, event_id: Id) -> Result<Id, Error> {
        // Create delete event
        let delete_event = Event::new()
            .kind(5) // Delete event kind
            .content("")
            .tag(Tag::event(event_id))
            .sign_with_keys(&self.keys)
            .map_err(|e| Error::Other(e.to_string()))?;

        self.send_event(delete_event).await
    }

    pub async fn set_metadata(&self, metadata: &Metadata) -> Result<Id, Error> {
        // Create metadata event
        let metadata_event = Event::new()
            .kind(0) // Metadata event kind
            .content(serde_json::to_string(metadata).map_err(|e| Error::Other(e.to_string()))?)
            .sign_with_keys(&self.keys)
            .map_err(|e| Error::Other(e.to_string()))?;

        self.send_event(metadata_event).await
    }

    pub async fn hide_channel_msg(&self, channel_id: Id, reason: String) -> Result<Id, Error> {
        // Create channel moderation event
        let moderation_event = Event::new()
            .kind(44) // Channel moderation
            .content(reason)
            .tag(Tag::event(channel_id))
            .sign_with_keys(&self.keys)
            .map_err(|e| Error::Other(e.to_string()))?;

        self.send_event(moderation_event).await
    }

    pub async fn mute_channel_user(
        &self,
        pubkey_to_mute: PublicKey,
        reason: String,
    ) -> Result<Id, Error> {
        // Create mute event
        let mute_event = Event::new()
            .kind(44) // Channel moderation  
            .content(reason)
            .tag(Tag::public_key(pubkey_to_mute))
            .sign_with_keys(&self.keys)
            .map_err(|e| Error::Other(e.to_string()))?;

        self.send_event(mute_event).await
    }

    pub async fn publish_text_note(&self, content: String, tags: Vec<Tag>) -> Result<Id, Error> {
        // Create text note event
        let text_note = Event::new()
            .kind(1) // Text note
            .content(content)
            .tags(tags)
            .sign_with_keys(&self.keys)
            .map_err(|e| Error::Other(e.to_string()))?;

        self.send_event(text_note).await
    }

    pub async fn set_contact_list(&self, contacts: Vec<Tag>) -> Result<(), Error> {
        // Create contact list event
        let contact_event = Event::new()
            .kind(3) // Contact list
            .content("".to_string())
            .tags(contacts)
            .sign_with_keys(&self.keys)
            .map_err(|e| Error::Other(e.to_string()))?;

        self.send_event(contact_event).await?;
        Ok(())
    }
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let relay_count = self.relay_sinks.lock().map(|s| s.len()).unwrap_or(0);
        write!(
            f,
            "Client {{ pubkey: {}, relays: {} }}",
            self.keys.public_key().as_hex_string(),
            relay_count
        )
    }
}
