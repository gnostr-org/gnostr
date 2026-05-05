use std::{
    collections::HashSet,
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
    time::Duration,
};

#[allow(unused)]
// Working Nostr Client Implementation with proper interface
use anyhow::Result;
// NIP-44 related imports
use base64::{
    engine::general_purpose::{GeneralPurpose, STANDARD},
    Engine,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    XChaCha20Poly1305,
};
use futures_util::{SinkExt, StreamExt};
use futures::stream;
use hkdf::Hkdf;
use k256::{
    ecdsa::SigningKey,
    elliptic_curve::{
        sec1::{FromEncodedPoint, ToEncodedPoint},
        FieldBytes, SecretKey,
    },
    schnorr::Signature,
};
use rand::RngCore;
use secp256k1::ecdh::shared_secret_point; // Use secp256k1's shared_secret_point
use secp256k1::{
    Parity, SecretKey as Secp256k1SecretKey, XOnlyPublicKey as Secp256k1XOnlyPublicKey,
}; // Import secp256k1 types for ECDH and Parity
use serde_json::json;
use sha2::Sha256;
use futures::future::join_all;
use tokio::{net::TcpStream, sync::mpsc, time::timeout};
use tokio_tungstenite::{
    connect_async, tungstenite::Message as WsMessage, MaybeTlsStream, WebSocketStream,
};
use tracing::{debug, info, warn};

use crate::types::{
    private_key::content_encryption::ContentEncryptionAlgorithm, ClientMessage, Error, Event,
    EventBuilder, EventKind, Filter, Id, Keys, Metadata, PublicKey, RelayMessage, RelayUrl,
    SubscriptionId, Tag, UncheckedUrl, Unixtime,
};

fn broadcast_log_path() -> Result<PathBuf, Error> {
    let dirs = directories::ProjectDirs::from("org", "gnostr", "gnostr")
        .ok_or_else(|| Error::Custom("failed to resolve gnostr app data directory".into()))?;
    Ok(dirs.data_local_dir().join("asyncgit/broadcast.log"))
}

fn append_broadcast_log(line: &str) {
    let path = match broadcast_log_path() {
        Ok(path) => path,
        Err(_) => return,
    };

    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(file, "{line}");
    }
}

/// Filter behavior for relay subscriptions.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FilterOptions {
    ExitOnEOSE,
    // Add other options as needed
}

/// Client options that control relay send behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_copy_implementations)]
pub struct Options {
    send_timeout: Option<Duration>,
    wait_for_send: bool,
    difficulty: u8,
    // Add other options as needed
}

impl Options {
    /// Create a client with default options.
    pub fn new() -> Self {
        Self {
            send_timeout: None,
            wait_for_send: false,
            difficulty: 0,
        }
    }

    /// Set an optional send timeout.
    pub fn send_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.send_timeout = timeout;
        self
    }

    /// Wait for relay send completion before returning.
    pub fn wait_for_send(mut self, wait: bool) -> Self {
        self.wait_for_send = wait;
        self
    }

    /// Set the proof-of-work difficulty target.
    pub fn difficulty(mut self, difficulty: u8) -> Self {
        self.difficulty = difficulty;
        self
    }
}

/// Nostr client with relay connection and signing state.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Client {
    keys: Keys,
    relays: Vec<RelayUrl>,
    options: Options,
}

impl Client {
    /// Construct a client from signing keys and options.
    pub fn new(keys: &Keys, options: Options) -> Self {
        Self {
            keys: keys.clone(),
            relays: Vec::new(),
            options,
        }
    }

    /// Construct a client with explicit options.
    pub fn with_opts(keys: &Keys, options: Options) -> Self {
        Self::new(keys, options)
    }

    /// Add relay URLs to the client.
    pub async fn add_relays(&mut self, relays: Vec<String>) -> Result<(), Error> {
        for relay_str in relays {
            match RelayUrl::try_from_str(&relay_str) {
                Ok(url) => self.relays.push(url),
                Err(e) => return Err(Error::Custom(e.into())),
            }
        }
        Ok(())
    }

    /// Log a connection attempt to configured relays.
    pub async fn connect(&self) {
        info!("Client connecting to {} relays", self.relays.len());
        // In a real implementation, this would establish WebSocket connections
        // For now, just log connection attempt
    }

    /// Fetch events matching filters with explicit options.
    pub async fn get_events_of_with_opts(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
        _opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        debug!("Getting events with {} filters", filters.len());

        let timeout = timeout.unwrap_or(Duration::from_secs(10));
        let relay_urls: Vec<String> = self
            .relays
            .iter()
            .map(|relay| relay.as_str().to_string())
            .collect();

        if relay_urls.is_empty() {
            return Ok(Vec::new());
        }

        let results = join_all(relay_urls.into_iter().map(|relay_url| {
            let filters = filters.clone();
            async move { fetch_events_from_relay(&relay_url, filters, timeout).await }
        }))
        .await;

        let mut seen = HashSet::new();
        let mut events = Vec::new();
        let mut first_error: Option<Error> = None;

        for result in results {
            match result {
                Ok(relay_events) => {
                    for event in relay_events {
                        if seen.insert(event.id) {
                            events.push(event);
                        }
                    }
                }
                Err(err) => {
                    warn!("relay fetch failed: {err}");
                    if first_error.is_none() {
                        first_error = Some(err);
                    }
                }
            }
        }

        if !events.is_empty() || first_error.is_none() {
            Ok(events)
        } else {
            Err(first_error.expect("checked above"))
        }
    }

    /// Fetch events matching filters.
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        self.get_events_of_with_opts(filters, timeout, FilterOptions::ExitOnEOSE)
            .await
    }

    /// Send a reaction event to configured relays.
    pub async fn reaction(&self, event: &Event, reaction: String) -> Result<Id, Error> {
        let reaction_event = EventBuilder::new(
            EventKind::Reaction,
            reaction,
            vec![
                Tag::new_event(event.id, None, None),
                Tag::new_pubkey(event.pubkey, None, None),
            ],
        )
        .to_event(
            &self
                .keys
                .secret_key()
                .map_err(|e| Error::Custom(e.into()))?,
        )
        .map_err(|e| Error::Custom(e.into()))?;

        self.send_event(reaction_event).await
    }

    /// Send an event deletion event.
    pub async fn delete_event(&self, event_id: Id) -> Result<Id, Error> {
        let delete_event = EventBuilder::new(
            EventKind::EventDeletion,
            "".to_string(),
            vec![Tag::new_event(event_id, None, None)],
        )
        .to_event(
            &self
                .keys
                .secret_key()
                .map_err(|e| Error::Custom(e.into()))?,
        )
        .map_err(|e| Error::Custom(e.into()))?;

        self.send_event(delete_event).await
    }

    /// Publish profile metadata.
    pub async fn set_metadata(&self, metadata: &Metadata) -> Result<Id, Error> {
        let content = serde_json::to_string(metadata).map_err(|e| Error::Custom(e.into()))?;

        let metadata_event = EventBuilder::new(EventKind::Metadata, content, Vec::new())
            .to_event(
                &self
                    .keys
                    .secret_key()
                    .map_err(|e| Error::Custom(e.into()))?,
            )
            .map_err(|e| Error::Custom(e.into()))?;

        self.send_event(metadata_event).await
    }

    /// Hide a message in a public channel.
    pub async fn hide_channel_msg(&self, channel_id: Id, reason: String) -> Result<Id, Error> {
        let moderation_event = EventBuilder::new(
            EventKind::ChannelHideMessage,
            reason,
            vec![Tag::new_event(channel_id, None, None)],
        )
        .to_event(
            &self
                .keys
                .secret_key()
                .map_err(|e| Error::Custom(e.into()))?,
        )
        .map_err(|e| Error::Custom(e.into()))?;

        self.send_event(moderation_event).await
    }

    /// Mute a user in a public channel.
    pub async fn mute_channel_user(
        &self,
        pubkey_to_mute: PublicKey,
        reason: String,
    ) -> Result<Id, Error> {
        let mute_event = EventBuilder::new(
            EventKind::ChannelMuteUser,
            reason,
            vec![Tag::new_pubkey(pubkey_to_mute, None, None)],
        )
        .to_event(
            &self
                .keys
                .secret_key()
                .map_err(|e| Error::Custom(e.into()))?,
        )
        .map_err(|e| Error::Custom(e.into()))?;

        self.send_event(mute_event).await
    }

    /// Publish a text note with tags.
    pub async fn publish_text_note(&self, content: String, tags: Vec<Tag>) -> Result<Id, Error> {
        let text_note = EventBuilder::new(EventKind::TextNote, content, tags)
            .to_event(
                &self
                    .keys
                    .secret_key()
                    .map_err(|e| Error::Custom(e.into()))?,
            )
            .map_err(|e| Error::Custom(e.into()))?;

        self.send_event(text_note).await
    }

    /// Publish a contact list.
    pub async fn set_contact_list(&self, contacts: Vec<Tag>) -> Result<(), Error> {
        let contact_event = EventBuilder::new(EventKind::ContactList, "".to_string(), contacts)
            .to_event(
                &self
                    .keys
                    .secret_key()
                    .map_err(|e| Error::Custom(e.into()))?,
            )
            .map_err(|e| Error::Custom(e.into()))?;

        let contact_event = self.send_event(contact_event).await?;
        debug!("contact_event={}", contact_event);
        Ok(())
    }

    /// Publish an encrypted NIP-44 direct message.
    pub async fn nip44_direct_message(
        &self,
        recipient_pubkey: PublicKey,
        content: String,
    ) -> Result<Id, Error> {
        let direct_message_event =
            self.build_nip44_direct_message_event(recipient_pubkey, content)?;
        self.send_event(direct_message_event).await
    }

    /// Build an encrypted NIP-44 direct message event.
    fn build_nip44_direct_message_event(
        &self,
        recipient_pubkey: PublicKey,
        content: String,
    ) -> Result<Event, Error> {
        let sender = self.keys.secret_key()?;
        let encrypted_content = sender.encrypt(
            &recipient_pubkey,
            &content,
            ContentEncryptionAlgorithm::Nip44v2,
        )?;

        EventBuilder::new(
            EventKind::EncryptedDirectMessage,
            encrypted_content,
            vec![Tag::new_pubkey(recipient_pubkey, None, None)],
        )
        .to_event(&sender)
        .map_err(|e| Error::Custom(e.into()))
    }

    /// Serialize and send an event to each configured relay.
    pub async fn send_event(&self, event: Event) -> Result<Id, Error> {
        append_broadcast_log(&format!(
            "sending event {} to {} relays",
            event.id,
            self.relays.len()
        ));

        // Serialize event to JSON
        let _event_json = serde_json::to_string(&event).map_err(|e| Error::Custom(e.into()))?;
        append_broadcast_log(&format!(
            "serialized event {} for {} relays",
            event.id,
            self.relays.len()
        ));

        // Create client message
        let client_message = ClientMessage::Event(Box::new(event.clone()));
        let message_json =
            serde_json::to_string(&client_message).map_err(|e| Error::Custom(e.into()))?;

        let relay_urls: Vec<String> = self
            .relays
            .iter()
            .map(|relay_url| relay_url.as_str().to_string())
            .collect();

        let relay_timeout = self.options.send_timeout.unwrap_or(Duration::from_secs(1));

        let results = stream::iter(relay_urls.into_iter().map(|ws_url| {
            let message_json = message_json.clone();
            let event_id = event.id;
            async move {
                append_broadcast_log(&format!("connecting relay {ws_url}"));

                match timeout(relay_timeout, async {
                    match connect_async(&ws_url).await {
                        Ok((ws_stream, _)) => {
                            let (mut ws_write, _) = ws_stream.split();

                            if let Err(e) = ws_write
                                .send(WsMessage::Text(message_json.clone().into()))
                                .await
                            {
                                append_broadcast_log(&format!(
                                    "broadcast failed relay={ws_url} error={e}"
                                ));
                                false
                            } else {
                                append_broadcast_log(&format!(
                                    "broadcast succeeded relay={ws_url} event={event_id}"
                                ));
                                true
                            }
                        }
                        Err(e) => {
                            append_broadcast_log(&format!(
                                "broadcast failed relay={ws_url} error={e}"
                            ));
                            false
                        }
                    }
                })
                .await
                {
                    Ok(sent) => sent,
                    Err(_) => {
                        append_broadcast_log(&format!(
                            "broadcast failed relay={ws_url} timeout={}s",
                            relay_timeout.as_secs_f32()
                        ));
                        false
                    }
                }
            }
        }))
        .buffer_unordered(8)
        .collect::<Vec<bool>>()
        .await;

        if results.into_iter().any(|sent| sent) {
            Ok(event.id)
        } else {
            Err(Error::Custom(
                "Failed to send event to any configured relay.".into(),
            ))
        }
    }
}

async fn fetch_events_from_relay(
    relay_url: &str,
    filters: Vec<Filter>,
    timeout: Duration,
) -> Result<Vec<Event>, Error> {
    let (websocket, _) = tokio::time::timeout(timeout, connect_async(relay_url))
        .await
        .map_err(|_| Error::Custom(format!("connection timeout for relay {relay_url}").into()))?
        .map_err(|e| Error::Custom(e.into()))?;

    let (mut ws_write, mut ws_read) = websocket.split();
    let subscription_id = SubscriptionId(format!("nips-{}", rand::random::<u64>()));
    let request = ClientMessage::Req(subscription_id.clone(), filters);
    let request_json = serde_json::to_string(&request).map_err(|e| Error::Custom(e.into()))?;
    ws_write
        .send(WsMessage::Text(request_json.into()))
        .await
        .map_err(|e| Error::Custom(e.into()))?;

    let mut events = Vec::new();
    loop {
        let message = match tokio::time::timeout(timeout, ws_read.next()).await {
            Ok(Some(Ok(message))) => message,
            Ok(Some(Err(e))) => return Err(Error::Custom(e.into())),
            Ok(None) => break,
            Err(_) => break,
        };

        match message {
            WsMessage::Text(text) => {
                let relay_message: RelayMessage =
                    serde_json::from_str(&text).map_err(|e| Error::Custom(e.into()))?;
                match relay_message {
                    RelayMessage::Event(_, event) => events.push(*event),
                    RelayMessage::Eose(_) => break,
                    RelayMessage::Closed(_, message) => {
                        warn!("relay {relay_url} closed subscription: {message}");
                        break;
                    }
                    RelayMessage::Notice(message) => {
                        debug!("relay {relay_url} notice: {message}");
                    }
                    RelayMessage::Notify(message) => {
                        debug!("relay {relay_url} notify: {message}");
                    }
                    RelayMessage::Ok(_, _, _) | RelayMessage::Auth(_) => {}
                }
            }
            WsMessage::Ping(payload) => {
                ws_write
                    .send(WsMessage::Pong(payload))
                    .await
                    .map_err(|e| Error::Custom(e.into()))?;
            }
            WsMessage::Close(_) => break,
            WsMessage::Binary(_) | WsMessage::Pong(_) | WsMessage::Frame(_) => {}
        }
    }

    Ok(events)
}

impl std::fmt::Display for Client {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Client {{ pubkey: {}, relays: {} }}",
            self.keys.public_key().as_hex_string(),
            self.relays.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_private_key(seed: u8) -> crate::types::PrivateKey {
        crate::types::PrivateKey(
            secp256k1::SecretKey::from_slice(&[seed; 32]).unwrap(),
            crate::types::KeySecurity::Weak,
        )
    }

    #[test]
    fn build_nip44_direct_message_event_uses_real_keys_and_recipient_tag() {
        let sender =
            crate::types::PrivateKey(crate::default_gnostr_private_key(), crate::types::KeySecurity::Weak);
        let recipient = test_private_key(2);
        let client = Client::new(&Keys::new(sender.clone()), Options::new());

        let content = "hello from asyncgit";
        let event = client
            .build_nip44_direct_message_event(recipient.public_key(), content.to_string())
            .unwrap();

        println!("direct message event id: {}", event.id);
        assert_eq!(event.kind, EventKind::EncryptedDirectMessage);
        assert_eq!(event.pubkey, sender.public_key());
        assert_eq!(event.tags.len(), 1);
        assert_eq!(event.tags[0].tagname(), "p");
        assert_eq!(event.tags[0].parse_pubkey().unwrap().0, recipient.public_key());
        assert_eq!(
            recipient
                .decrypt(&sender.public_key(), &event.content)
                .unwrap(),
            content
        );
    }
}
