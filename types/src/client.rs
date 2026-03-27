use std::{
    fmt,
    time::Duration,
};

#[allow(unused)]
// Working Nostr Client Implementation with proper interface
use anyhow::Result;
// NIP-44 related imports
use futures_util::SinkExt;
use futures_util::StreamExt;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::{debug, info, warn};


use crate::{
    Error, Event, EventKind, Filter, Id, Keys, Metadata, PublicKey,
    RelayUrl, Tag,
    client_message::ClientMessage,
    EventBuilder,
    private_key::ContentEncryptionAlgorithm,
};

/// Options for filtering events from relays.
#[derive(Debug, Clone, PartialEq, Eq, Copy)]
pub enum FilterOptions {
    /// Exit after receiving an End Of Stored Events (EOSE) message.
    ExitOnEOSE,
    // Add other options as needed
}

/// Options for configuring the Nostr client behavior.
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(missing_copy_implementations)]
pub struct Options {
    /// Timeout for sending messages to relays.
    send_timeout: Option<Duration>,
    /// Whether to wait for a successful send to at least one relay.
    wait_for_send: bool,
    /// Proof of Work difficulty target for events.
    difficulty: u8,
    // Add other options as needed
}

impl Options {
    /// Create new default `Options`.
    pub fn new() -> Self {
        Self {
            send_timeout: None,
            wait_for_send: false,
            difficulty: 0,
        }
    }

    /// Set the send timeout.
    pub fn send_timeout(mut self, timeout: Option<Duration>) -> Self {
        self.send_timeout = timeout;
        self
    }

    /// Set whether to wait for a successful send.
    pub fn wait_for_send(mut self, wait: bool) -> Self {
        self.wait_for_send = wait;
        self
    }

    /// Set the Proof of Work difficulty target.
    pub fn difficulty(mut self, difficulty: u8) -> Self {
        self.difficulty = difficulty;
        self
    }
}

/// A Nostr client for interacting with relays.
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct Client {
    keys: Keys,
    relays: Vec<RelayUrl>,
    options: Options,
}

impl Client {
    /// Create a new `Client` with the given `Keys` and `Options`.
    pub fn new(keys: &Keys, options: Options) -> Self {
        Self {
            keys: keys.clone(),
            relays: Vec::new(),
            options,
        }
    }

    /// Create a new `Client` with the given `Keys` and `Options` (alias for `new`).
    pub fn with_opts(keys: &Keys, options: Options) -> Self {
        Self::new(keys, options)
    }

    /// Add a list of relay URLs to the client.
    pub async fn add_relays(&mut self, relays: Vec<String>) -> Result<(), Error> {
        for relay_str in relays {
            match RelayUrl::try_from_str(&relay_str) {
                Ok(url) => self.relays.push(url),
                Err(e) => return Err(Error::Custom(e.into())),
            }
        }
        Ok(())
    }

    /// Connect to the configured relays.
    pub async fn connect(&self) {
        info!("Client connecting to {} relays", self.relays.len());
        // In a real implementation, this would establish WebSocket connections
        // For now, just log connection attempt
    }

    /// Get events from relays based on filters and options.
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

    /// Get events from relays based on filters.
    pub async fn get_events_of(
        &self,
        filters: Vec<Filter>,
        timeout: Option<Duration>,
    ) -> Result<Vec<Event>, Error> {
        self.get_events_of_with_opts(filters, timeout, FilterOptions::ExitOnEOSE)
            .await
    }

    /// Publish a reaction event.
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

    /// Delete an event.
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

    /// Set client metadata.
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

    /// Hide a channel message.
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

    /// Mute a user in a channel.
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

    /// Publish a text note event.
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

    /// Set the contact list (EventKind::ContactList).
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

    /// Send a NIP-44 direct message.
    pub async fn nip44_direct_message(
        &self,
        recipient_pubkey: PublicKey,
        content: String,
    ) -> Result<Id, Error> {
        // Use the proper NIP-44 v2 encryption provided by PrivateKey
        let encrypted_content = self.keys.secret_key()?.encrypt(
            &recipient_pubkey,
            &content,
            ContentEncryptionAlgorithm::Nip44v2,
        )?;

        // 5. Create EventKind::EncryptedDirectMessage (kind 4) event
        let direct_message_event = EventBuilder::new(
            EventKind::EncryptedDirectMessage,
            encrypted_content,
            vec![Tag::new_pubkey(recipient_pubkey, None, None)],
        )
        .to_event(
            &self
                .keys
                .secret_key()
                .map_err(|e| Error::Custom(e.into()))?,
        )
        .map_err(|e| Error::Custom(e.into()))?;

        // 6. Send the event
        self.send_event(direct_message_event).await
    }

    /// Send an event to the configured relays.
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

        let mut success = false;

        // REAL IMPLEMENTATION: Connect to relays and send event
        for relay_url in self.relays.iter() {
            let ws_url = relay_url.as_str().to_string();
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
                        // Do not set success to true, continue to next relay
                    } else {
                        info!("Event {} sent to relay {}", event.id, relay_url);
                        success = true; // Event sent successfully to at least one relay
                    }

                    // Keep connection open briefly for response
                    tokio::time::sleep(Duration::from_secs(2)).await;
                }
                Err(e) => {
                    warn!("Failed to connect to relay {}: {}", relay_url, e);
                    // Do not set success to true, continue to next relay
                }
            }
        }

        if success {
            Ok(event.id)
        } else {
            Err(Error::Custom(
                "Failed to send event to any configured relay.".into(),
            ))
        }
    }
}

impl fmt::Display for Client {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Client {{ pubkey: {}, relays: {} }}",
            self.keys.public_key().as_hex_string(),
            self.relays.len()
        )
    }
}
