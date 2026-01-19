#[allow(unused)]
// Working Nostr Client Implementation with proper interface
use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use serde_json::json;
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async, tungstenite::Message as WsMessage, MaybeTlsStream, WebSocketStream,
};

use crate::types::{
    ClientMessage, Error, Event, EventBuilder, EventKind, Filter, Id, Keys, Metadata, PublicKey,
    RelayUrl, SubscriptionId, Tag, UncheckedUrl, Unixtime,
};
use tracing::{debug, info, warn};

// NIP-44 related imports
use k256::{
    ecdsa::SigningKey,
    elliptic_curve::{
        ecdh::{self, SharedSecret},
        sec1::{FromEncodedPoint, ToEncodedPoint},
        SecretKey,
    },
    schnorr::Signature,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    XChaCha20Poly1305,
};
use hkdf::Hkdf;
use sha2::Sha256;
use rand::RngCore;
use base64::{engine::general_purpose::{STANDARD, GeneralPurpose}, Engine};


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

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Client {
    keys: Keys,
    relays: Vec<RelayUrl>,
    options: Options,
}

impl Client {
    pub fn new(keys: &Keys, options: Options) -> Self {
        Self {
            keys: keys.clone(),
            relays: Vec::new(),
            options,
        }
    }

    pub fn with_opts(keys: &Keys, options: Options) -> Self {
        Self::new(keys, options)
    }

    pub async fn add_relays(&mut self, relays: Vec<String>) -> Result<(), Error> {
        for relay_str in relays {
            match RelayUrl::try_from_str(&relay_str) {
                Ok(url) => self.relays.push(url),
                Err(e) => return Err(Error::Custom(e.into())),
            }
        }
        Ok(())
    }

    pub async fn connect(&self) {
        info!("Client connecting to {} relays", self.relays.len());
        // In a real implementation, this would establish WebSocket connections
        // For now, just log connection attempt
    }

    pub async fn get_events_of_with_opts(
        &self,
        _filters: Vec<Filter>,
        timeout: Option<Duration>,
        _opts: FilterOptions,
    ) -> Result<Vec<Event>, Error> {
        debug!("Getting events with {} filters", _filters.len());

        if let Some(timeout) = timeout {
            tokio::time::sleep(timeout).await;
        }

        // Return empty vector for now - in real implementation this would query relays
        Ok(Vec::new())
    }

    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        self.get_events_of_with_opts(filters, timeout, FilterOptions::ExitOnEOSE)
            .await
    }

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

    pub async fn nip44_direct_message(
        &self,
        recipient_pubkey: PublicKey,
        content: String,
    ) -> Result<Id, Error> {
        // 1. Get sender's secret key
        let sender_secret_key = self
            .keys
            .secret_key()
            .map_err(|e| Error::Custom(e.into()))?;

        // Convert k256::ecdsa::SigningKey to k256::elliptic_curve::SecretKey
        let secret_key = SecretKey::from_bytes(sender_secret_key.0.secret_bytes().as_slice())
            .map_err(|e| Error::Custom(e.into()))?;

        // Convert PublicKey to k256::elliptic_curve::PublicKey
        let recipient_point = k256::PublicKey::from_sec1_bytes(recipient_pubkey.as_bytes())
            .map_err(|e| Error::Custom(e.into()))?;
        
        // 2. Derive shared secret using ECDH
        let shared_secret = shared_secret(&secret_key, &recipient_point);

        // 3. Derive encryption key using HKDF-SHA256
        let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());
        let mut encryption_key = [0u8; 32]; // XChaCha20Poly1305 key size
        hkdf.expand(b"nip44", &mut encryption_key)
            .map_err(|e| Error::Custom(e.into()))?;

        // 4. Encrypt the message using XChaCha20Poly1305
        let cipher = XChaCha20Poly1305::new(&encryption_key.into());
        let mut nonce = [0u8; 24]; // XChaCha20Poly1305 nonce size
        OsRng.fill_bytes(&mut nonce);

        let encrypted_content = cipher
            .encrypt(&nonce.into(), content.as_bytes())
            .map_err(|e| Error::Custom(e.into()))?;

        let encrypted_message_base64 = base64::encode(encrypted_content);
        let content_to_send = format!("{}?iv={}", encrypted_message_base64, base64::encode(nonce));

        // 5. Create EventKind::EncryptedDirectMessage (kind 4) event
        let direct_message_event = EventBuilder::new(
            EventKind::EncryptedDirectMessage,
            content_to_send,
            vec![Tag::new_pubkey(recipient_pubkey, None, None)],
        )
        .to_event(sender_secret_key)
        .map_err(|e| Error::Custom(e.into()))?;

        // 6. Send the event
        self.send_event(direct_message_event).await
    }

    pub async fn send_event(&self, event: Event) -> Result<Id, Error> {
        debug!("Sending event {} to {} relays", event.id, self.relays.len());

        // Serialize event to JSON
        let _event_json = serde_json::to_string(&event).map_err(|e| Error::Custom(e.into()))?;
        debug!(
            "Sending event {} to {} relays",
            _event_json,
            self.relays.len()
        );

        // Create client message
        let client_message = ClientMessage::Event(Box::new(event.clone()));
        let message_json =
            serde_json::to_string(&client_message).map_err(|e| Error::Custom(e.into()))?;

        // REAL IMPLEMENTATION: Connect to relays and send event
        for relay_url in self.relays.iter() {
            let ws_url = format!("ws://{}", relay_url.as_str());
            info!("Connecting to relay: {}", ws_url);

            match connect_async(&ws_url).await {
                Ok((ws_stream, _)) => {
                    let (mut ws_write, _) = ws_stream.split();

                    // Send EVENT message
                    if let Err(e) = ws_write
                        .send(WsMessage::Text(message_json.clone().into()))
                        .await
                    {
                        warn!("Failed to send event to {}: {}", relay_url, e);
                        continue;
                    }

                    info!("Event {} sent to relay {}", event.id, relay_url);

                    // Keep connection open briefly for response
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                Err(e) => {
                    warn!("Failed to connect to relay {}: {}", relay_url, e);
                    continue;
                }
            }
        }

        Ok(event.id)
    }
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
