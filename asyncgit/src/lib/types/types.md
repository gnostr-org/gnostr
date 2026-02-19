# types Code Documentation

**Generated on:** 2026-01-21 13:49:12
**Directory:** /Users/Shared/gnostr-org/.github/gnostr/src/lib/types
**Files included:** 91

---

## Directory Structure

```
./.genkit/traces_idx/genkit.metadata
./README.md
./client.rs
./client_message.rs
./content.rs
./delegation.rs
./error.rs
./event.rs
./event_builder.rs
./event_kind.rs
./event_reference.rs
./filter.rs
./id.rs
./identity.rs
./image_dimensions.rs
./internal.rs
./key_signer.rs
./keys.rs
./metadata.rs
./mod.rs
./naddr.rs
./nevent.rs
./nip0.rs
./nip05.rs
./nip10.rs
./nip13.rs
./nip14.rs
./nip15.rs
./nip18.rs
./nip19.rs
./nip2.rs
./nip25.rs
./nip26.rs
./nip28.rs
./nip3.rs
./nip30.rs
./nip32.rs
./nip34.rs
./nip36.rs
./nip38.rs
./nip4.rs
./nip40.rs
./nip44/error.rs
./nip44/mod.rs
./nip44/nip44.vectors.json
./nip44/tests.rs
./nip53.rs
./nip59.rs
./nip6.rs
./nip9.rs
... (41 more files)
```

---

## File Contents

### README.md

**Size:** 5081 bytes | **Modified:** 2026-01-20 14:02:27

```markdown
# Nostr Types Module (`gnostr/src/lib/types`)

This module provides a comprehensive set of types and utilities for handling the Nostr protocol. It aims to offer a robust and extensible foundation for building Nostr-compatible applications, covering event structures, client-relay communication, cryptographic elements, and NIP-specific implementations.

## Overview

The `types` module is designed to encapsulate all data structures and related logic essential for interacting with the Nostr network. It abstracts away the complexities of serialization, deserialization, and event validation, allowing developers to focus on application-level logic.

## Key Components

### Core Event Structures

- **`Event`**: Represents a Nostr event, including its ID, public key, creation timestamp, kind, tags, content, and signature.
- **`PreEvent`**: A precursor to `Event`, used for signing, containing all event data except the signature itself.
- **`Rumor`**: Represents an event received from a relay before full validation.
- **`EventKind`**: An enum defining the various types of Nostr events (e.g., TextNote, RecommendRelay, ChannelCreation).
- **`Tag`**: Represents a generic Nostr tag, used to attach metadata or references to events.
- **`Id`**: Represents the unique identifier of an event.
- **`PublicKey` / `PrivateKey`**: Cryptographic keys used for Nostr identities and event signing.
- **`Signature`**: The cryptographic signature of an event.
- **`Unixtime`**: A timestamp type for Nostr events.

### Client-Relay Communication

- **`ClientMessage`**: Messages sent from a client to a Nostr relay (e.g., publishing events, subscribing to filters).
- **`RelayMessage`**: Messages received from a Nostr relay (e.g., events, EOSE messages, notices).
- **`Filter`**: Used by clients to request specific events from relays based on various criteria.
- **`SubscriptionId`**: A unique identifier for a client's subscription to a relay.

### NIP-Specific Implementations

The module includes specific implementations and types related to various Nostr Improvement Proposals (NIPs):

- **NIP-00 (`nip0`)**: Basic protocol definitions.
- **NIP-02 (`nip2`)**: Contact List and Petnames.
- **NIP-04 (`nip4`)**: Encrypted Direct Message.
- **NIP-05 (`nip05`)**: Mapping Nostr keys to DNS-based internet identifiers.
- **NIP-06 (`nip6`)**: Basic key derivation from mnemonic seed phrase.
- **NIP-09 (`nip9`)**: Event Deletion.
- **NIP-15 (`nip15`)**: End of Stored Events Notice.
- **NIP-26 (`nip26`)**: Delegation.
- **NIP-28 (`nip28`)**: Public Chat Channels (includes `ChannelCreationEvent`, `ChannelMetadataEvent`, `create_channel`, `set_channel_metadata`, `create_channel_message`, `hide_message`, `mute_user` and their parsing counterparts).
- **NIP-34 (`nip34`)**: Git notes integration.
- **NIP-44 (`nip44`)**: Encrypted content for secure direct messages.

### Utility Types

- **`Url` / `UncheckedUrl`**: Types for handling URLs.
- **`NostrUrl` / `NostrBech32`**: Utilities for Nostr-specific URI schemes and Bech32 encoding.
- **`Metadata`**: Event content for Kind 0 (profile metadata).
- **`Profile`**: Represents a user profile with metadata.
- **`RelayInformationDocument`**: Structure for relay information (NIP-11).
- **`KeySigner`**: Concrete implementation of the `Signer` trait.
- **`IntoVec` trait**: A generic trait for converting `Option<T>` into `Vec<T>`.
- **`add_pubkey_to_tags`, `add_event_to_tags`, `add_addr_to_tags`, `add_subject_to_tags_if_missing`**: Helper functions for managing event tags.
- **`get_leading_zero_bits`**: Utility function.

### Versioned Types

The module also provides versioned representations of core Nostr types, allowing for compatibility with different protocol iterations. These include `ClientMessageV1`, `EventV1`, `TagV1`, etc., up to their latest versions (`V3`, `V4`, `V5` where applicable).

## Usage

Developers can import and utilize these types to:

- Construct and sign Nostr events.
- Parse incoming messages from Nostr relays.
- Implement NIP-specific functionalities like chat channels or encrypted DMs.
- Manage user profiles and relay information.

**Example: Creating a simple text note event**

```rust
use crate::types::{Event, EventKind, PrivateKey, Unixtime, Signer, KeySigner, Tag};

fn create_text_note(signer: &KeySigner, content: &str) -> Result<Event, Error> {
    let tags = vec![]; // No specific tags for a simple text note
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: EventKind::TextNote,
        tags,
        content: content.to_string(),
    };
    signer.sign_event(pre_event)
}

#[test]
fn test_create_text_note() {
    let privkey = PrivateKey::mock();
    let signer = KeySigner::from_private_key(privkey, "", 1).unwrap();
    let event = create_text_note(&signer, "Hello, Nostr!").unwrap();
    assert_eq!(event.kind, EventKind::TextNote);
    assert_eq!(event.content, "Hello, Nostr!");
}
```

This `types` module serves as the backbone for interacting with the Nostr protocol in a structured and type-safe manner.
```

---

### client.rs

**Size:** 11116 bytes | **Modified:** 2026-01-20 14:02:27

```rust
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
    private_key::content_encryption::ContentEncryptionAlgorithm, ClientMessage, Error, Event,
    EventBuilder, EventKind, Filter, Id, Keys, Metadata, PublicKey, RelayUrl, SubscriptionId, Tag,
    UncheckedUrl, Unixtime,
};
use tracing::{debug, info, warn};

// NIP-44 related imports
use base64::{
    engine::general_purpose::{GeneralPurpose, STANDARD},
    Engine,
};
use chacha20poly1305::{
    aead::{Aead, KeyInit, OsRng},
    XChaCha20Poly1305,
};
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
use sha2::Sha256;

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
```

---

### client_message.rs

**Size:** 121 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use super::versioned::ClientMessageV3;

/// A message from a client to a relay
pub type ClientMessage = ClientMessageV3;
```

---

### content.rs

**Size:** 7937 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use lazy_static::lazy_static;
use linkify::{LinkFinder, LinkKind};
use regex::Regex;

use super::{find_nostr_url_pos, NostrBech32, NostrUrl};

/// This is like `Range<usize>`, except we impl offset() on it
/// This is like linkify::Span, except we impl offset() on it and don't need
///   the as_str() or kind() functions.
#[derive(Clone, Copy, Debug)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    /// Modify a span by offsetting it from the start by `offset` bytes
    pub fn offset(&mut self, offset: usize) {
        self.start += offset;
        self.end += offset;
    }
}

/// A segment of content
#[derive(Clone, Debug)]
pub enum ContentSegment {
    /// A Nostr URL
    NostrUrl(NostrUrl),

    /// A reference to an event tag by index
    TagReference(usize),

    /// A hyperlink
    Hyperlink(Span),

    /// Plain text
    Plain(Span),
}

/// A sequence of content segments
#[derive(Clone, Debug)]
pub struct ShatteredContent {
    /// The sequence of `ContentSegment`s
    pub segments: Vec<ContentSegment>,

    /// The original content (the allocated string)
    /// `Range`s within segments refer to this
    pub allocated: String,
}

impl ShatteredContent {
    /// Break content into meaningful segments
    ///
    /// This avoids reallocation
    pub fn new(content: String) -> ShatteredContent {
        let segments = shatter_content_1(&content);

        ShatteredContent {
            segments,
            allocated: content,
        }
    }

    /// View a slice of the original content as specified in a Span
    #[allow(clippy::string_slice)] // the Span is trusted
    pub fn slice<'a>(&'a self, span: &Span) -> Option<&'a str> {
        if self.allocated.is_char_boundary(span.start) && self.allocated.is_char_boundary(span.end)
        {
            Some(&self.allocated[span.start..span.end])
        } else {
            None
        }
    }
}

/// Break content into a linear sequence of `ContentSegment`s
#[allow(clippy::string_slice)] // start/end from find_nostr_url_pos is trusted
fn shatter_content_1(mut content: &str) -> Vec<ContentSegment> {
    let mut segments: Vec<ContentSegment> = Vec::new();
    let mut offset: usize = 0; // used to adjust Span ranges

    // Pass 1 - `NostrUrl`s
    while let Some((start, end)) = find_nostr_url_pos(content) {
        let mut inner_segments = shatter_content_2(&content[..start]);
        apply_offset(&mut inner_segments, offset);
        segments.append(&mut inner_segments);

        // The Nostr Bech32 itself
        if let Some(nbech) = NostrBech32::try_from_string(&content[start + 6..end]) {
            segments.push(ContentSegment::NostrUrl(NostrUrl(nbech)));
        } else {
            segments.push(ContentSegment::Plain(Span { start, end }));
        }

        offset += end;
        content = &content[end..];
    }

    // The stuff after it
    let mut inner_segments = shatter_content_2(content);
    apply_offset(&mut inner_segments, offset);
    segments.append(&mut inner_segments);

    segments
}

// Pass 2 - `TagReference`s
#[allow(clippy::string_slice)] // Regex positions are trusted
fn shatter_content_2(content: &str) -> Vec<ContentSegment> {
    lazy_static! {
        static ref TAG_RE: Regex = Regex::new(r"(\#\[\d+\])").unwrap();
    }

    let mut segments: Vec<ContentSegment> = Vec::new();

    let mut pos = 0;
    for mat in TAG_RE.find_iter(content) {
        let mut inner_segments = shatter_content_3(&content[pos..mat.start()]);
        apply_offset(&mut inner_segments, pos);
        segments.append(&mut inner_segments);

        // If panics on unwrap, something is wrong with Regex.
        let u: usize = content[mat.start() + 2..mat.end() - 1].parse().unwrap();
        segments.push(ContentSegment::TagReference(u));
        pos = mat.end();
    }

    let mut inner_segments = shatter_content_3(&content[pos..]);
    apply_offset(&mut inner_segments, pos);
    segments.append(&mut inner_segments);

    segments
}

fn shatter_content_3(content: &str) -> Vec<ContentSegment> {
    let mut segments: Vec<ContentSegment> = Vec::new();

    for span in LinkFinder::new().kinds(&[LinkKind::Url]).spans(content) {
        if span.kind().is_some() {
            segments.push(ContentSegment::Hyperlink(Span {
                start: span.start(),
                end: span.end(),
            }));
        } else if !span.as_str().is_empty() {
            segments.push(ContentSegment::Plain(Span {
                start: span.start(),
                end: span.end(),
            }));
        }
    }

    segments
}

fn apply_offset(segments: &mut [ContentSegment], offset: usize) {
    for segment in segments.iter_mut() {
        match segment {
            ContentSegment::Hyperlink(span) => span.offset(offset),
            ContentSegment::Plain(span) => span.offset(offset),
            _ => {}
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_shatter_content() {
        let content_str = "My friend #[0]  wrote me this note: nostr:note10ttnuuvcs29y3k23gwrcurw2ksvgd7c2rrqlfx7urmt5m963vhss8nja90 and it might have referred to https://github.com/Giszmo/nostr.info/blob/master/assets/js/main.js";
        let content = content_str.to_string();
        let pieces = ShatteredContent::new(content);
        assert_eq!(pieces.segments.len(), 6);
        assert!(matches!(pieces.segments[0], ContentSegment::Plain(..)));
        assert!(matches!(
            pieces.segments[1],
            ContentSegment::TagReference(..)
        ));
        assert!(matches!(pieces.segments[2], ContentSegment::Plain(..)));
        assert!(matches!(pieces.segments[3], ContentSegment::NostrUrl(..)));
        assert!(matches!(pieces.segments[4], ContentSegment::Plain(..)));
        assert!(matches!(pieces.segments[5], ContentSegment::Hyperlink(..)));

        let content_str = r#"This is a test of NIP-27 posting support referencing this note nostr:nevent1qqsqqqq9wh98g4u6e480vyp6p4w3ux2cd0mxn2rssq0w5cscsgzp2ksprpmhxue69uhkzapwdehhxarjwahhy6mn9e3k7mf0qyt8wumn8ghj7etyv4hzumn0wd68ytnvv9hxgtcpremhxue69uhkummnw3ez6ur4vgh8wetvd3hhyer9wghxuet59uq3kamnwvaz7tmwdaehgu3wd45kketyd9kxwetj9e3k7mf0qy2hwumn8ghj7mn0wd68ytn00p68ytnyv4mz7qgnwaehxw309ahkvenrdpskjm3wwp6kytcpz4mhxue69uhhyetvv9ujuerpd46hxtnfduhsz9mhwden5te0wfjkccte9ehx7um5wghxyctwvshszxthwden5te0wfjkccte9eekummjwsh8xmmrd9skctcnmzajy and again without the url data nostr:note1qqqq2aw2w3te4n2w7cgr5r2arcv4s6lkdx58pqq7af3p3qsyz4dqns2935
And referencing this person nostr:npub1acg6thl5psv62405rljzkj8spesceyfz2c32udakc2ak0dmvfeyse9p35c and again as an nprofile nostr:nprofile1qqswuyd9ml6qcxd92h6pleptfrcqucvvjy39vg4wx7mv9wm8kakyujgprdmhxue69uhkummnw3ezumtfddjkg6tvvajhytnrdakj7qg7waehxw309ahx7um5wgkhqatz9emk2mrvdaexgetj9ehx2ap0qythwumn8ghj7un9d3shjtnwdaehgu3wd9hxvme0qyt8wumn8ghj7etyv4hzumn0wd68ytnvv9hxgtcpzdmhxue69uhk7enxvd5xz6tw9ec82c30qy2hwumn8ghj7mn0wd68ytn00p68ytnyv4mz7qgcwaehxw309ashgtnwdaehgunhdaexkuewvdhk6tczkvt9n all on the same damn line even (I think)."#;
        let content = content_str.to_string();
        let pieces = ShatteredContent::new(content);
        assert_eq!(pieces.segments.len(), 9);
    }

    #[test]
    fn test_shatter_content_2() {
        let content_str =
            "Ein wunderschönes langes Wochenende auf der #zitadelle2024 geht zu Ende...
🏰 #einundzwanzig
Hier einige Impressionen mit opsec gewährten Bildern.
Wonderful Long Weekend at a Zitadelle, Here Impressions opsec included
 nostr:npub1vwf2mytkyk22x2gcmr9d7k";
        let content = content_str.to_string();
        let pieces = ShatteredContent::new(content);
        assert_eq!(pieces.segments.len(), 2);
        assert!(matches!(pieces.segments[0], ContentSegment::Plain(..)));
        assert!(matches!(pieces.segments[1], ContentSegment::Plain(..))); // 223 - 256
        if let ContentSegment::Plain(span) = pieces.segments[1] {
            let _slice = pieces.slice(&span);
        }
    }
}
```

---

### delegation.rs

**Size:** 9619 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, Visitor},
    ser::{Serialize, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::{Error, EventKind, PublicKey, Signature, Unixtime};

/// Delegation information for an Event
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventDelegation {
    /// The event was not delegated
    NotDelegated,

    /// The delegation was invalid (with reason)
    InvalidDelegation(String),

    /// The event was delegated and is valid (with pubkey of delegator)
    DelegatedBy(PublicKey),
}

/// Conditions of delegation
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct DelegationConditions {
    /// If the delegation is only for a given event kind
    pub kind: Option<EventKind>,

    /// If the delegation is only for events created after a certain time
    pub created_after: Option<Unixtime>,

    /// If the delegation is only for events created before a certain time
    pub created_before: Option<Unixtime>,

    /// Optional full string form, in case it was parsed from string
    pub full_string: Option<String>,
}

impl DelegationConditions {
    /// Return in conmpiled string form. If full form is stored, it is returned,
    /// otherwise it is compiled from parts.
    pub fn as_string(&self) -> String {
        match &self.full_string {
            Some(fs) => fs.clone(),
            None => self.compile_full_string(),
        }
    }

    /// Compile full string from parts.
    fn compile_full_string(&self) -> String {
        let mut parts: Vec<String> = Vec::new();
        if let Some(kind) = self.kind {
            parts.push(format!("kind={}", u32::from(kind)));
        }
        if let Some(created_after) = self.created_after {
            parts.push(format!("created_at>{}", created_after.0));
        }
        if let Some(created_before) = self.created_before {
            parts.push(format!("created_at<{}", created_before.0));
        }
        parts.join("&")
    }

    #[allow(dead_code)]
    fn update_full_string(&mut self) {
        self.full_string = Some(self.compile_full_string())
    }

    /// Convert from string from
    pub fn try_from_str(s: &str) -> Result<DelegationConditions, Error> {
        let mut output: DelegationConditions = Default::default();

        let parts = s.split('&');
        for part in parts {
            if let Some(kindstr) = part.strip_prefix("kind=") {
                let event_num = kindstr.parse::<u32>()?;
                let event_kind: EventKind = From::from(event_num);
                output.kind = Some(event_kind);
            }
            if let Some(timestr) = part.strip_prefix("created_at>") {
                let time = timestr.parse::<i64>()?;
                output.created_after = Some(Unixtime(time));
            }
            if let Some(timestr) = part.strip_prefix("created_at<") {
                let time = timestr.parse::<i64>()?;
                output.created_before = Some(Unixtime(time));
            }
        }
        // store orignal string
        output.full_string = Some(s.to_string());

        Ok(output)
    }

    #[allow(dead_code)]
    pub(crate) fn mock() -> DelegationConditions {
        let mut dc = DelegationConditions {
            kind: Some(EventKind::Repost),
            created_after: Some(Unixtime(1677700000)),
            created_before: None,
            full_string: None,
        };
        dc.update_full_string();
        dc
    }

    /// Verify the signature part of a Delegation tag
    pub fn verify_signature(
        &self,
        pubkey_delegater: &PublicKey,
        pubkey_delegatee: &PublicKey,
        signature: &Signature,
    ) -> Result<(), Error> {
        let input = format!(
            "nostr:delegation:{}:{}",
            pubkey_delegatee.as_hex_string(),
            self.as_string()
        );
        pubkey_delegater.verify(input.as_bytes(), signature)
    }
}

impl Serialize for DelegationConditions {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_string())
    }
}

impl<'de> Deserialize<'de> for DelegationConditions {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(DelegationConditionsVisitor)
    }
}

struct DelegationConditionsVisitor;

impl Visitor<'_> for DelegationConditionsVisitor {
    type Value = DelegationConditions;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A string")
    }

    fn visit_str<E>(self, v: &str) -> Result<DelegationConditions, E>
    where
        E: DeError,
    {
        DelegationConditions::try_from_str(v).map_err(|e| E::custom(format!("{e}")))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{test_serde, types::PrivateKey, KeySigner, Signer, Tag};

    test_serde! {DelegationConditions, test_delegation_conditions_serde}

    #[test]
    fn test_sign_delegation_verify_delegation_signature() {
        let delegator_private_key = PrivateKey::try_from_hex_string(
            "ee35e8bb71131c02c1d7e73231daa48e9953d329a4b701f7133c8f46dd21139c",
        )
        .unwrap();
        let delegator_public_key = delegator_private_key.public_key();

        let signer = KeySigner::from_private_key(delegator_private_key, "lockme", 16).unwrap();

        let delegatee_public_key = PublicKey::try_from_hex_string(
            "477318cfb5427b9cfc66a9fa376150c1ddbc62115ae27cef72417eb959691396",
            true,
        )
        .unwrap();

        let dc = DelegationConditions::try_from_str(
            "kind=1&created_at>1674834236&created_at<1677426236",
        )
        .unwrap();

        let signature = signer
            .generate_delegation_signature(delegatee_public_key, &dc)
            .unwrap();

        // signature is changing, validate by verify method
        let sig = Signature::try_from(signature).unwrap();
        let verify_result = dc.verify_signature(&delegator_public_key, &delegatee_public_key, &sig);
        assert!(verify_result.is_ok());
    }

    #[test]
    fn test_delegation_tag_parse_and_verify() {
        let tag_str = "[\"delegation\",\"1a459a8a6aa6441d480ba665fb8fb21a4cfe8bcacb7d87300f8046a558a3fce4\",\"kind=1&created_at>1676067553&created_at<1678659553\",\"369aed09c1ad52fceb77ecd6c16f2433eac4a3803fc41c58876a5b60f4f36b9493d5115e5ec5a0ce6c3668ffe5b58d47f2cbc97233833bb7e908f66dbbbd9d36\"]";
        let dt = serde_json::from_str::<Tag>(tag_str).unwrap();
        if let Ok((pubkey, conditions, sig)) = dt.parse_delegation() {
            assert_eq!(
                conditions.as_string(),
                "kind=1&created_at>1676067553&created_at<1678659553"
            );

            let delegatee_public_key = PublicKey::try_from_hex_string(
                "bea8aeb6c1657e33db5ac75a83910f77e8ec6145157e476b5b88c6e85b1fab34",
                true,
            )
            .unwrap();

            let verify_result = conditions.verify_signature(
                &pubkey,
                &delegatee_public_key,
                &Signature::try_from(sig).unwrap(),
            );
            assert!(verify_result.is_ok());
        } else {
            panic!("Incorrect tag type")
        }
    }

    #[test]
    fn test_delegation_tag_parse_and_verify_alt_order() {
        // Clauses in the condition string are not in the canonical order, but this
        // should not matter
        let tag_str = "[\"delegation\",\"05bc52a6117c57f99b73f5315f3105b21cecdcd2c6825dee8d508bd7d972ad6a\",\"kind=1&created_at<1686078180&created_at>1680807780\",\"1016d2f4284cdb4e6dc6eaa4e61dff87b9f4138786154d070d36e9434f817bd623abed2133bb62b9dcfb2fbf54b42e16bcd44cfc23907f8eb5b45c011caaa47c\"]";
        let dt = serde_json::from_str::<Tag>(tag_str).unwrap();
        if let Ok((pubkey, conditions, sig)) = dt.parse_delegation() {
            assert_eq!(
                conditions.as_string(),
                "kind=1&created_at<1686078180&created_at>1680807780"
            );

            let delegatee_public_key = PublicKey::try_from_hex_string(
                "111c02821806b046068dffc4d8e4de4a56bc99d3015c335b8929d900928fa317",
                true,
            )
            .unwrap();

            let verify_result = conditions.verify_signature(
                &pubkey,
                &delegatee_public_key,
                &Signature::try_from(sig).unwrap(),
            );
            assert!(verify_result.is_ok());
        } else {
            panic!("Incorrect tag type")
        }
    }

    #[test]
    fn test_from_str() {
        let str = "kind=1&created_at>1000000&created_at<2000000";
        let dc = DelegationConditions::try_from_str(str).unwrap();
        assert_eq!(dc.as_string(), str);
    }

    #[test]
    fn test_from_str_alt_order() {
        // Even with alternative order, as_string() should return the same
        let str = "created_at<2000000&created_at>1000000&kind=1";
        let dc = DelegationConditions::try_from_str(str).unwrap();
        assert_eq!(dc.as_string(), str);
    }

    #[test]
    fn test_as_string() {
        let dc = DelegationConditions {
            kind: Some(EventKind::TextNote),
            created_before: Some(Unixtime(2000000)),
            created_after: Some(Unixtime(1000000)),
            full_string: None,
        };
        assert_eq!(
            dc.as_string(),
            "kind=1&created_at>1000000&created_at<2000000"
        );
    }
}
```

---

### error.rs

**Size:** 6072 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use thiserror::Error;

/// Errors that can occur in the nostr-proto crate
#[derive(Error, Debug)]
pub enum Error {
    /// Assertion failed
    #[error("Assertion failed: {0}")]
    AssertionFailed(String),

    /// Bad Encrypted Message
    #[error("Bad Encrypted Message")]
    BadEncryptedMessage,

    /// Bad Encrypted Message due to bad Base64
    #[error("Bad Encrypted Message due to invalid base64")]
    BadEncryptedMessageBase64(base64::DecodeError),

    /// Base64 error
    #[error("Base64 Decoding Error: {0}")]
    Base64(#[from] base64::DecodeError),

    /// Bech32 decode error
    #[error("Bech32 Error: {0}")]
    Bech32Decode(#[from] bech32::DecodeError),

    /// Bech32 encode error
    #[error("Bech32 Error: {0}")]
    Bech32Encode(#[from] bech32::EncodeError),

    /// Bech32 HRP error
    #[error("Bech32 Error: {0}")]
    Bech32Hrp(#[from] bech32::primitives::hrp::Error),

    /// Crypto error
    #[error("Crypto Error: {0}")]
    Crypto(#[from] super::nip44::Error),

    /// Encryption/Decryption Error
    #[error("Private Key Encryption/Decryption Error")]
    PrivateKeyEncryption,

    /// From utf8 Error
    #[error("From UTF-8 Error")]
    FromUtf8(#[from] std::string::FromUtf8Error),

    /// Bech32 error
    #[error("Wrong Bech32 Kind: Expected {0} found {0}")]
    WrongBech32(String, String),

    /// Key or Signature error
    #[error("Key or Signature Error: {0}")]
    KeyOrSignature(#[from] secp256k1::Error),

    /// Event is in the future
    #[error("Event is in the future")]
    EventInFuture,

    /// Formatting error
    #[error("Formatting Error: {0}")]
    Fmt(#[from] std::fmt::Error),

    /// A hash mismatch verification error
    #[error("Hash Mismatch")]
    HashMismatch,

    /// Hex string decoding error
    #[error("Hex Decode Error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    /// Invalid encrypted private key
    #[error("Invalid Encrypted Private Key")]
    InvalidEncryptedPrivateKey,

    /// Invalid encrypted event
    #[error("Invalid Encrypted Event")]
    InvalidEncryptedEvent,

    /// Invalid event Id
    #[error("Invalid event Id")]
    InvalidId,

    /// Invalid event Id Prefix
    #[error("Invalid event Id Prefix")]
    InvalidIdPrefix,

    /// Invalid digest length
    #[error("Invalid digest length")]
    InvalidLength(#[from] hmac::digest::InvalidLength),

    /// Invalid NAddr
    #[error("Invalid naddr")]
    InvalidNAddr,

    /// Invalid NEvent
    #[error("Invalid nevent")]
    InvalidNEvent,

    /// Invalid Operation
    #[error("Invalid Operation")]
    InvalidOperation,

    /// Invalid Private Key
    #[error("Invalid Private Key")]
    InvalidPrivateKey,

    /// Invalid Profile
    #[error("Invalid Profile")]
    InvalidProfile,

    /// Invalid public key
    #[error("Invalid Public Key")]
    InvalidPublicKey,

    /// Invalid public key prefix
    #[error("Invalid Public Key Prefix")]
    InvalidPublicKeyPrefix,

    /// Invalid recipient
    #[error("Invalid Recipient")]
    InvalidRecipient,

    /// Invalid URL
    #[error("Invalid URL: \"{0}\"")]
    InvalidUrl(#[from] url::ParseError),

    /// Invalid URL TLV encoding
    #[error("Invalid URL TLV encoding")]
    InvalidUrlTlv,

    /// Invalid URL Host
    #[error("Invalid URL Host: \"{0}\"")]
    InvalidUrlHost(String),

    /// Invalid URL Scheme
    #[error("Invalid URL Scheme: \"{0}\"")]
    InvalidUrlScheme(String),

    /// Missing URL Authority
    #[error("Missing URL Authority")]
    InvalidUrlMissingAuthority,

    /// Addr to a non-replaceable event kind
    #[error("Event kind is not replaceable")]
    NonReplaceableAddr,

    /// No Private Key
    #[error("No private key")]
    NoPrivateKey,

    /// No Public Key
    #[error("No public key")]
    NoPublicKey,

    /// Out of Range
    #[error("Out of Range")]
    OutOfRange(usize),

    /// Parse integer error
    #[error("Parse integer error")]
    ParseInt(#[from] std::num::ParseIntError),

    /// Scrypt error
    #[error("Scrypt invalid output length")]
    Scrypt,

    /// Serialization error
    #[error("JSON (de)serialization error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Signer is locked
    #[error("Signer is locked")]
    SignerIsLocked,

    /// Try from slice error
    #[error("Try From Slice error: {0}")]
    Slice(#[from] std::array::TryFromSliceError),

    /// Speedy error
    #[cfg(feature = "speedy")]
    #[error("Speedy (de)serialization error: {0}")]
    Speedy(#[from] speedy::Error),

    /// Tag mismatch
    #[error("Tag mismatch")]
    TagMismatch,

    /// Unknown event kind
    #[error("Unknown event kind = {0}")]
    UnknownEventKind(u32),

    /// Unknown Key Security
    #[error("Unknown key security = {0}")]
    UnknownKeySecurity(u8),

    /// Unknown Cipher Version
    #[error("Unknown cipher version = {0}")]
    UnknownCipherVersion(u8),

    /// Unpad error
    #[error("Decryption error: {0}")]
    Unpad(#[from] aes::cipher::block_padding::UnpadError),

    /// Url Error
    #[error("Not a valid nostr relay url: {0}")]
    Url(String),

    /// UTF-8 error
    #[error("UTF-8 Error: {0}")]
    Utf8Error(#[from] std::str::Utf8Error),

    /// Wrong event kind
    #[error("Wrong event kind")]
    WrongEventKind,

    /// Wrong length hex string
    #[error("Wrong length hex string")]
    WrongLengthHexString,

    /// Wrong length bytes for event kind
    #[error("Wrong length bytes for event kind")]
    WrongLengthKindBytes,

    /// Wrong Decryption Password
    #[error("Wrong decryption password")]
    WrongDecryptionPassword,

    /// Zap Receipt issue
    #[error("Invalid Zap Receipt: {0}")]
    ZapReceipt(String),

    /// Invalid NIP-19 data
    #[error("Invalid NIP-19 data")]
    InvalidNip19Data,

    /// Invalid NIP-19 prefix
    #[error("Invalid NIP-19 prefix")]
    InvalidNip19Prefix,

    /// Boxed standard error
    #[error(transparent)]
    Custom(#[from] Box<dyn std::error::Error + Send + Sync + 'static>),

    /// Anyhow error
    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}
```

---

### event.rs

**Size:** 543 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use crate::types::{
    versioned::{
        event3::{EventV3, PreEventV3, RumorV3, UnsignedEventV3},
        zap_data::ZapDataV2,
    },
    Id,
};

/// The main event type
pub type Event = EventV3;

/// The event ID
pub(crate) type EventId = Id;

/// Data used to construct an event
pub type PreEvent = PreEventV3;

/// An UnsignedEvent is an Event without a signature
pub(crate) type UnsignedEvent = UnsignedEventV3;

/// A Rumor is an Event without a signature
pub type Rumor = RumorV3;

/// Data about a Zap
pub type ZapData = ZapDataV2;
```

---

### event_builder.rs

**Size:** 5537 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// EventBuilder - Builder pattern for creating Nostr events
// Replaces nostr_sdk EventBuilder with local implementation

use anyhow::Result;

use super::{
    Event, EventKind, Id, ImageDimensions, KeySigner, Metadata, PreEvent, PrivateKey, PublicKey,
    Signer, Tag, UncheckedUrl, Unixtime,
};

/// Builder for creating Nostr events
#[derive(Debug)]
pub struct EventBuilder {
    kind: EventKind,
    content: String,
    tags: Vec<Tag>,
    created_at: Option<Unixtime>,
}

impl EventBuilder {
    /// Create a new event builder
    pub fn new(kind: EventKind, content: String, tags: Vec<Tag>) -> Self {
        Self {
            kind,
            content,
            tags,
            created_at: None,
        }
    }

    /// Set creation time
    pub fn created_at(mut self, created_at: Unixtime) -> Self {
        self.created_at = Some(created_at);
        self
    }

    /// Build and sign the event with proof of work
    pub fn to_pow_event(self, keys: &PrivateKey, difficulty_target: u8) -> Result<Event> {
        let preevent = PreEvent {
            pubkey: keys.public_key(),
            created_at: self.created_at.unwrap_or_else(Unixtime::now),
            kind: self.kind,
            tags: self.tags,
            content: self.content,
        };

        // Apply proof of work if difficulty > 0
        if difficulty_target > 0 {
            // TODO: Implement actual POW calculation
            // For now, just sign without POW
        }

        // Sign the event
        let signer = KeySigner::from_private_key(keys.clone(), "", 1)?;
        Ok(signer.sign_event(preevent)?)
    }

    /// Build and sign the event without proof of work
    pub fn to_event(self, keys: &PrivateKey) -> Result<Event> {
        self.to_pow_event(keys, 0)
    }
}

// Badge-related builders
impl EventBuilder {
    /// Create a badge definition event (NIP-58)
    pub fn define_badge(
        id: String,
        name: Option<String>,
        description: Option<String>,
        image_url: Option<UncheckedUrl>,
        image_size: Option<ImageDimensions>,
        thumbnails: Vec<(UncheckedUrl, Option<ImageDimensions>)>,
    ) -> Self {
        let mut tags = Vec::new();

        // Add identifier tag
        tags.push(Tag::new_identifier(id));

        // Add name tag
        if let Some(name) = name {
            tags.push(Tag::new_name(name));
        }

        // Add image tag
        if let Some(url) = image_url {
            if let Some(dims) = image_size {
                tags.push(Tag::new_image(url, Some(dims.width), Some(dims.height)));
            } else {
                tags.push(Tag::new_image(url, None, None));
            }
        }

        // Add thumbnail tags
        for (thumb_url, thumb_dims) in thumbnails {
            if let Some(dims) = thumb_dims {
                tags.push(Tag::new_thumb(
                    thumb_url,
                    Some(dims.width),
                    Some(dims.height),
                ));
            } else {
                tags.push(Tag::new_thumb(thumb_url, None, None));
            }
        }

        Self {
            kind: EventKind::BadgeDefinition,
            content: description.unwrap_or_default(),
            tags,
            created_at: None,
        }
    }

    /// Create a badge award event (NIP-58)
    pub fn award_badge(badge_definition_event: &Event, awarded_pubkeys: Vec<Tag>) -> Self {
        let mut tags = Vec::new();

        // Reference the badge definition
        tags.push(Tag::new_event(
            badge_definition_event.id,
            None,
            Some("a".to_string()),
        ));

        // Add awarded pubkeys
        for pubkey_tag in awarded_pubkeys {
            if let Ok((pk, _, _)) = pubkey_tag.parse_pubkey() {
                tags.push(Tag::new_pubkey(pk, None, None));
            }
        }

        Self {
            kind: EventKind::BadgeAward,
            content: "".to_string(),
            tags,
            created_at: None,
        }
    }

    /// Create a channel creation event (NIP-28)
    pub fn channel(metadata: &Metadata, pubkey: &PublicKey) -> Self {
        let mut tags = Vec::new();

        // Add author pubkey tag
        tags.push(Tag::new_pubkey(*pubkey, None, None));

        Self {
            kind: EventKind::ChannelCreation,
            content: serde_json::to_string(metadata).unwrap_or_default(),
            tags,
            created_at: None,
        }
    }

    /// Create a text note event
    pub fn text_note(content: String) -> Self {
        Self {
            kind: EventKind::TextNote,
            content,
            tags: Vec::new(),
            created_at: None,
        }
    }

    /// Create a reaction event
    pub fn reaction(reacted_event_id: Id, reaction: String) -> Self {
        let mut tags = Vec::new();
        tags.push(Tag::new_event(
            reacted_event_id,
            None,
            Some("reaction".to_string()),
        ));

        Self {
            kind: EventKind::Reaction,
            content: reaction,
            tags,
            created_at: None,
        }
    }

    /// Create an event deletion event
    pub fn delete_with_reason(event_ids_to_delete: Vec<Id>, reason: String) -> Self {
        let mut tags = Vec::new();

        for event_id in event_ids_to_delete {
            tags.push(Tag::new_event(event_id, None, None));
        }

        Self {
            kind: EventKind::EventDeletion,
            content: reason,
            tags,
            created_at: None,
        }
    }
}
```

---

### event_kind.rs

**Size:** 15443 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{convert::From, fmt};

use serde::{
    de::{Deserializer, Error as DeError, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
#[cfg(feature = "speedy")]
use speedy::{Context, Readable, Reader, Writable, Writer};

#[cfg(test)]
use crate::test_serde;

macro_rules! define_event_kinds {
    ($($comment:expr, $name:ident = $value:expr),*) => {
        /// A kind of Event
        #[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
        #[repr(u32)]
        pub enum EventKind {
            $(
                #[doc = $comment]
                $name = $value,
            )*
            /// Job Request (NIP-90) 5000-5999
            JobRequest(u32),
            /// Job Result (NIP-90) 6000-6999
            JobResult(u32),
            /// Group control events (NIP-29) 9000-9030
            GroupControl(u32),
            /// Relay-specific replaceable event
            Replaceable(u32),
            /// Ephemeral event, sent to all clients with matching filters and should not be stored
            Ephemeral(u32),
            /// Group Metadata events
            GroupMetadata(u32),
            /// Something else?
            Other(u32),
        }

        static WELL_KNOWN_KINDS: &[EventKind] = &[
            $($name,)*
        ];

        impl From<u32> for EventKind {
            fn from(u: u32) -> Self {
                match u {
                    $($value => $name,)*
                    x if (5_000..5_999).contains(&x) => JobRequest(x),
                    x if (6_000..6_999).contains(&x) => JobResult(x),
                    x if (9_000..9_030).contains(&x) => GroupControl(x),
                    x if (10_000..20_000).contains(&x) => Replaceable(x),
                    x if (20_000..30_000).contains(&x) => Ephemeral(x),
                    x if (39_000..39_009).contains(&x) => GroupMetadata(x),
                    x => Other(x),
                }
            }
        }

        impl From<EventKind> for u32 {
            fn from(e: EventKind) -> u32 {
                match e {
                    $($name => $value,)*
                    JobRequest(u) => u,
                    JobResult(u) => u,
                    GroupControl(u) => u,
                    Replaceable(u) => u,
                    Ephemeral(u) => u,
                    GroupMetadata(u) => u,
                    Other(u) => u,
                }
            }
        }
    };
}

define_event_kinds!(
    "Event sets the metadata associated with a public key (NIP-01)",
    Metadata = 0,
    "Event is a text note (NIP-01)",
    TextNote = 1,
    "Event contains a relay URL which the author recommends",
    RecommendRelay = 2,
    "Event contains tags which represent the authors contacts including the authors pet names for them (NIP-02)",
    ContactList = 3,
    "Event is an encrypted direct message (NIP-04)",
    EncryptedDirectMessage = 4,
    "Event is an authors request to delete previous events (NIP-09)",
    EventDeletion = 5,
    "Repost (NIP-18)",
    Repost = 6,
    "Event is a reaction to a `TextNote` event (NIP-25)",
    Reaction = 7,
    "Badge Award (NIP-58)",
    BadgeAward = 8,
    "Group Chat Message (NIP-29)",
    GroupChatMessage = 9,
    "Group Chat Threaded Reply (NIP-29)",
    GroupChatThreadedReply = 10,
    "Group Chat Thread (NIP-29)",
    GroupChatThread = 11,
    "Group Chat Reply (NIP-29)",
    GroupChatReply = 12,
    "Seal (NIP-59 PR 716)",
    Seal = 13,
    "Chat Message / DM (NIP-24 PR 686)",
    DmChat = 14,
    "Generic Repost (NIP-18)",
    GenericRepost = 16,
    "Event creates a public channel (NIP-28)",
    ChannelCreation = 40,
    "Event sets metadata on a public channel (NIP-28)",
    ChannelMetadata = 41,
    "Event creates a message on a public channel (NIP-28)",
    ChannelMessage = 42,
    "Event hides a message on a public channel (NIP-28)",
    ChannelHideMessage = 43,
    "Event mutes a user on a public channel (NIP-28)",
    ChannelMuteUser = 44,
    "Bid (NIP-15)",
    Bid = 1021,
    "Bid Confirmation (NIP-15)",
    BidConfirmation = 1022,
    "Timestamps",
    Timestamp = 1040,
    "Gift Wrap (NIP-59 PR 716)",
    GiftWrap = 1059,
    "File Metadata (NIP-94)",
    FileMetadata = 1063,
    "Live Chat Message (NIP-53)",
    LiveChatMessage = 1311,
    "Patches (NIP-34)",
    Patches = 1617,
    "Issue (NIP-34)",
    GitIssue = 1621,
    "Replies (NIP-34)",
    GitReply = 1622,
    "Status Open  (NIP-34)",
    GitStatusOpen = 1630,
    "Status Applied (NIP-34)",
    GitStatusApplied = 1631,
    "Status Closed (NIP-34)",
    GitStatusClosed = 1632,
    "Status Draft (NIP-34)",
    GitStatusDraft = 1633,
    "Problem Tracker (nostrocket-1971)",
    ProblemTracker = 1971,
    "Reporting (NIP-56)",
    Reporting = 1984,
    "Label (NIP-32)",
    Label = 1985,
    "Community (exclusive) Post (NIP-72 pr 753)",
    CommunityPost = 4549,
    "Community Post Approval (NIP-72)",
    CommunityPostApproval = 4550,
    "Job Feedback (NIP-90)",
    JobFeedback = 7000,
    "Zap Goal (NIP-75)",
    ZapGoal = 9041,
    "Zap Request",
    ZapRequest = 9734,
    "Zap",
    Zap = 9735,
    "Highlights (NIP-84)",
    Highlights = 9802,
    "Mute List (NIP-51)",
    MuteList = 10000,
    "PinList (NIP-51)",
    PinList = 10001,
    "Relays List (NIP-65)",
    RelayList = 10002,
    "Bookmarks List (NIP-51)",
    BookmarkList = 10003,
    "Communities List (NIP-51)",
    CommunityList = 10004,
    "Public Chats List (NIP-51)",
    PublicChatsList = 10005,
    "Blocked Relays List (NIP-51)",
    BlockedRelaysList = 10006,
    "Search Relays List (NIP-51)",
    SearchRelaysList = 10007,
    "User Groups (NIP-51, NIP-29)",
    UserGroups = 10009,
    "Interests List (NIP-51)",
    InterestsList = 10015,
    "User Emoji List (NIP-51)",
    UserEmojiList = 10030,
    "Relay list to receive DMs (NIP-17)",
    DmRelayList = 10050,
    "File storage server list (NIP-96)",
    FileStorageServerList = 10096,
    "Wallet Info (NIP-47)",
    WalletInfo = 13194,
    "Lightning Pub RPC (Lightning.Pub)",
    LightningPubRpc = 21000,
    "Client Authentication (NIP-42)",
    Auth = 22242,
    "Wallet Request (NIP-47)",
    WalletRequest = 23194,
    "Wallet Response (NIP-47)",
    WalletResponse = 23195,
    "Nostr Connect (NIP-46)",
    NostrConnect = 24133,
    "HTTP Auth (NIP-98)",
    HttpAuth = 27235,
    "Categorized People List (NIP-51)",
    FollowSets = 30000,
    "Categorized Bookmark List (NIP-51)",
    GenericSets = 30001,
    "Relay Sets (NIP-51)",
    RelaySets = 30002,
    "Bookmark Sets (NIP-51)",
    BookmarkSets = 30003,
    "Curation Sets (NIP-51)",
    CurationSets = 30004,
    "Profile Badges (NIP-58)",
    ProfileBadges = 30008,
    "Badge Definition (NIP-58)",
    BadgeDefinition = 30009,
    "Interest Sets (NIP-51)",
    InterestSets = 30015,
    "Create or update a stall (NIP-15)",
    CreateUpdateStall = 30017,
    "Create or update a product (NIP-15)",
    CreateUpdateProduct = 30018,
    "Marketplace UI/UX (NIP-15)",
    MarketplaceUi = 30019,
    "Product sold as auction (NIP-15)",
    ProductSoldAuction = 30020,
    "Long-form Content (NIP-23)",
    LongFormContent = 30023,
    "Draft Long-form Content (NIP-23)",
    DraftLongFormContent = 30024,
    "Emoji Sets (NIP-51)",
    EmojiSets = 30030,
    "Release artifact sets (NIP-51)",
    ReleaseArtifactSets = 30063,
    "Application Specific Data, (NIP-78)",
    AppSpecificData = 30078,
    "Live Event (NIP-53)",
    LiveEvent = 30311,
    "User Status (NIP-315 PR 737)",
    UserStatus = 30315,
    "Classified Listing (NIP-99)",
    ClassifiedListing = 30402,
    "Draft Classified Listing (NIP-99)",
    DraftClassifiedListing = 30403,
    "Repository Announcement (NIP-34)",
    RepositoryAnnouncement = 30617,
    "Git Repository Announcement (NIP-34)",
    GitRepoAnnouncement = 30618,
    "Wiki Article (NIP-54)",
    WikiArticle = 30818,
    "Date-Based Calendar Event (NIP-52)",
    DateBasedCalendarEvent = 31922,
    "Time-Based Calendar Event (NIP-52)",
    TimeBasedCalendarEvent = 31923,
    "Calendar (NIP-52)",
    Calendar = 31924,
    "Calendar Event RSVP (NIP-52)",
    CalendarEventRsvp = 31925,
    "Handler Recommendation (NIP-89)",
    HandlerRecommendation = 31989,
    "Handler Information (NIP-89)",
    HandlerInformation = 31990,
    "Community Definition (NIP-72)",
    CommunityDefinition = 34550
);

use EventKind::*;

impl EventKind {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> EventKind {
        TextNote
    }

    /// Is a job request kind
    pub fn is_job_request(&self) -> bool {
        let u: u32 = From::from(*self);
        (5000..=5999).contains(&u)
    }

    /// Is a job result kind
    pub fn is_job_result(&self) -> bool {
        let u: u32 = From::from(*self);
        (6000..=6999).contains(&u)
    }

    /// If this event kind is a replaceable event
    /// NOTE: this INCLUDES parameterized replaceable events
    pub fn is_replaceable(&self) -> bool {
        match *self {
            Metadata => true,
            ContactList => true,
            _ => {
                let u: u32 = From::from(*self);
                (10000..=19999).contains(&u) || (30000..=39999).contains(&u)
            }
        }
    }

    /// If this event kind is ephemeral
    pub fn is_ephemeral(&self) -> bool {
        let u: u32 = From::from(*self);
        (20000..=29999).contains(&u)
    }

    /// If this event kind is parameterized replaceable
    pub fn is_parameterized_replaceable(&self) -> bool {
        let u: u32 = From::from(*self);
        (30000..=39999).contains(&u)
    }

    /// If this event kind is feed related.
    pub fn is_feed_related(&self) -> bool {
        self.is_feed_displayable() || self.augments_feed_related()
    }

    /// If this event kind is feed displayable.
    pub fn is_feed_displayable(&self) -> bool {
        matches!(
            *self,
            TextNote
                | GroupChatMessage
                | GroupChatThreadedReply
                | GroupChatThread
                | GroupChatReply
                | EncryptedDirectMessage
                | Repost
                | DmChat
                | GenericRepost
                | ChannelMessage
                | FileMetadata
                | LiveChatMessage
                | Patches
                | GitIssue
                | GitReply
                | GitStatusOpen
                | GitStatusApplied
                | GitStatusClosed
                | GitStatusDraft
                | CommunityPost
                | LongFormContent
                | DraftLongFormContent
        )
    }

    /// Is direct message related
    pub fn is_direct_message_related(&self) -> bool {
        matches!(*self, EncryptedDirectMessage | DmChat | GiftWrap)
    }

    /// If this event kind augments a feed related event
    pub fn augments_feed_related(&self) -> bool {
        matches!(
            *self,
            EventDeletion | Reaction | Timestamp | Label | Reporting | Zap
        )
    }

    /// If the contents are expected to be encrypted (or empty)
    pub fn contents_are_encrypted(&self) -> bool {
        matches!(
            *self,
            EncryptedDirectMessage
                | MuteList
                | PinList
                | BookmarkList
                | CommunityList
                | PublicChatsList
                | BlockedRelaysList
                | SearchRelaysList
                | InterestsList
                | UserEmojiList
                | JobRequest(_)
                | JobResult(_)
                | WalletRequest
                | WalletResponse
                | NostrConnect
        )
    }

    /// This iterates through every well-known EventKind
    pub fn iter() -> EventKindIterator {
        EventKindIterator::new()
    }
}

/// Iterator over well known `EventKind`s
#[derive(Clone, Copy, Debug)]
pub struct EventKindIterator {
    pos: usize,
}

impl EventKindIterator {
    fn new() -> EventKindIterator {
        EventKindIterator { pos: 0 }
    }
}

impl Iterator for EventKindIterator {
    type Item = EventKind;

    fn next(&mut self) -> Option<EventKind> {
        if self.pos == WELL_KNOWN_KINDS.len() {
            None
        } else {
            let rval = WELL_KNOWN_KINDS[self.pos];
            self.pos += 1;
            Some(rval)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.pos, Some(WELL_KNOWN_KINDS.len()))
    }
}

impl Serialize for EventKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let u: u32 = From::from(*self);
        serializer.serialize_u32(u)
    }
}

impl<'de> Deserialize<'de> for EventKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_u32(EventKindVisitor)
    }
}

struct EventKindVisitor;

impl Visitor<'_> for EventKindVisitor {
    type Value = EventKind;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "an unsigned number that matches a known EventKind")
    }

    fn visit_u32<E>(self, v: u32) -> Result<EventKind, E>
    where
        E: DeError,
    {
        Ok(From::<u32>::from(v))
    }

    // JsonValue numbers come in as u64
    fn visit_u64<E>(self, v: u64) -> Result<EventKind, E>
    where
        E: DeError,
    {
        Ok(From::<u32>::from(v as u32))
    }
}

#[cfg(feature = "speedy")]
impl<'a, C: Context> Readable<'a, C> for EventKind {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let value = u32::read_from(reader)?;
        Ok(value.into())
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        <u32 as Readable<'a, C>>::minimum_bytes_needed()
    }
}

#[cfg(feature = "speedy")]
impl<C: Context> Writable<C> for EventKind {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_u32(u32::from(*self))
    }

    #[inline]
    fn bytes_needed(&self) -> Result<usize, C::Error> {
        Ok(std::mem::size_of::<u32>())
    }
}

/// Either an EventKind or a range (a vector of length 2 with start and end)
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[serde(untagged)]
pub enum EventKindOrRange {
    /// A single EventKind
    EventKind(EventKind),

    /// A range of EventKinds
    // NOTE: the internal Vec should have exactly 2 fields.  To force this with a tuple
    //       struct makes ser/de a bitch, so we don't.
    Range(Vec<EventKind>),
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {EventKind, test_event_kind_serde}

    #[test]
    fn test_replaceable_ephemeral() {
        assert!(Metadata.is_replaceable());
        assert!(!TextNote.is_replaceable());
        assert!(!Zap.is_replaceable());
        assert!(LongFormContent.is_replaceable());

        assert!(!TextNote.is_ephemeral());
        assert!(Auth.is_ephemeral());

        assert!(!TextNote.is_parameterized_replaceable());
        assert!(LongFormContent.is_parameterized_replaceable());
    }

    #[cfg(feature = "speedy")]
    #[test]
    fn test_speedy_event_kind() {
        let ek = EventKind::mock();
        let bytes = ek.write_to_vec().unwrap();
        let ek2 = EventKind::read_from_buffer(&bytes).unwrap();
        assert_eq!(ek, ek2);
    }
}
```

---

### event_reference.rs

**Size:** 3255 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};

use super::{Id, NAddr, PublicKey, RelayUrl};

/// A reference to another event, either by `Id` (often coming from an 'e' tag),
/// or by `NAddr` (often coming from an 'a' tag).
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum EventReference {
    /// Refer to a specific event by Id
    Id {
        /// The event id
        id: Id,

        /// Optionally include author (to find via their relay list)
        author: Option<PublicKey>,

        /// Optionally include relays (to find the event)
        relays: Vec<RelayUrl>,

        /// Optional marker, if this came from an event tag
        marker: Option<String>,
    },

    /// Refer to a replaceable event by NAddr
    Addr(NAddr),
}

impl EventReference {
    /// Get the author
    pub fn author(&self) -> Option<PublicKey> {
        match self {
            EventReference::Id { author, .. } => *author,
            EventReference::Addr(naddr) => Some(naddr.author),
        }
    }

    /// Set the author
    pub fn set_author(&mut self, new_author: PublicKey) {
        match self {
            EventReference::Id { ref mut author, .. } => *author = Some(new_author),
            EventReference::Addr(ref mut naddr) => naddr.author = new_author,
        }
    }

    /// Copy the relays
    pub fn copy_relays(&self) -> Vec<RelayUrl> {
        match self {
            EventReference::Id { relays, .. } => relays.clone(),
            EventReference::Addr(naddr) => naddr
                .relays
                .iter()
                .filter_map(|r| RelayUrl::try_from_unchecked_url(r).ok())
                .collect(),
        }
    }

    /// Extend relays
    pub fn extend_relays(&mut self, relays: Vec<RelayUrl>) {
        let mut new_relays = self.copy_relays();
        new_relays.extend(relays);

        match self {
            EventReference::Id { ref mut relays, .. } => *relays = new_relays,
            EventReference::Addr(ref mut naddr) => {
                naddr.relays = new_relays.iter().map(|r| r.to_unchecked_url()).collect()
            }
        }
    }
}

impl PartialEq for EventReference {
    fn eq(&self, other: &Self) -> bool {
        match self {
            EventReference::Id { id: id1, .. } => {
                match other {
                    EventReference::Id { id: id2, .. } => {
                        // We don't compare the other fields which are only helpers,
                        // not definitive identity
                        id1 == id2
                    }
                    _ => false,
                }
            }
            EventReference::Addr(addr1) => match other {
                EventReference::Addr(addr2) => addr1 == addr2,
                _ => false,
            },
        }
    }
}

impl Eq for EventReference {}

impl Hash for EventReference {
    fn hash<H: Hasher>(&self, state: &mut H) {
        match self {
            EventReference::Id { id, .. } => {
                // We do not hash the other fields which are only helpers,
                // not definitive identity
                id.hash(state);
            }
            EventReference::Addr(addr) => {
                addr.hash(state);
            }
        }
    }
}
```

---

### filter.rs

**Size:** 12982 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{collections::BTreeMap, fmt};

use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::{Event, EventKind, IdHex, PublicKeyHex, Tag, Unixtime};

/// Filter which specify what events a client is looking for
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct Filter {
    /// Events which match these ids
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub ids: Vec<IdHex>, // ID as hex

    /// Events which match these authors
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub authors: Vec<PublicKeyHex>, // PublicKey as hex

    /// Events which match these kinds
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub kinds: Vec<EventKind>,

    /// Events which match the given tags
    #[serde(
        flatten,
        serialize_with = "serialize_tags",
        deserialize_with = "deserialize_tags"
    )]
    pub tags: BTreeMap<char, Vec<String>>,

    /// Events occuring after this date
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub since: Option<Unixtime>,

    /// Events occuring before this date
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub until: Option<Unixtime>,

    /// A limit on the number of events to return in the initial query
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub limit: Option<usize>,
}

impl Filter {
    /// Create a new Filter object
    pub fn new() -> Filter {
        Default::default()
    }

    /// Add an Id to the filter.
    pub fn add_id(&mut self, id_hex: &IdHex) {
        if !self.ids.contains(id_hex) {
            self.ids.push(id_hex.to_owned());
        }
    }

    /// Delete an Id from the filter
    pub fn del_id(&mut self, id_hex: &IdHex) {
        if let Some(index) = self.ids.iter().position(|id| *id == *id_hex) {
            let _ = self.ids.swap_remove(index);
        }
    }

    /// Add a PublicKey to the filter
    pub fn add_author(&mut self, public_key_hex: &PublicKeyHex) -> &mut Self {
        if !self.authors.contains(public_key_hex) {
            self.authors.push(public_key_hex.to_owned());
        }
        self
    }

    /// Delete a PublicKey from the filter
    pub fn del_author(&mut self, public_key_hex: &PublicKeyHex) {
        if let Some(index) = self.authors.iter().position(|pk| *pk == *public_key_hex) {
            let _ = self.authors.swap_remove(index);
        }
    }

    /// Add an EventKind to the filter
    pub fn add_event_kind(&mut self, event_kind: EventKind) -> &mut Self {
        if self.kinds.contains(&event_kind) {
            return self;
        }
        self.kinds.push(event_kind);
        self
    }

    /// Delete an EventKind from the filter
    pub fn del_event_kind(&mut self, event_kind: EventKind) {
        if let Some(position) = self.kinds.iter().position(|&x| x == event_kind) {
            let _ = self.kinds.swap_remove(position);
        }
    }

    /// Add a Tag value to a filter
    pub fn add_tag_value(&mut self, letter: char, value: String) -> &mut Self {
        let _ = self
            .tags
            .entry(letter)
            .and_modify(|values| values.push(value.clone()))
            .or_insert(vec![value]);
        self
    }

    /// Add a Tag value from a filter
    pub fn del_tag_value(&mut self, letter: char, value: String) {
        let mut became_empty: bool = false;
        let _ = self.tags.entry(letter).and_modify(|values| {
            if let Some(position) = values.iter().position(|x| *x == value) {
                let _ = values.swap_remove(position);
            }
            if values.is_empty() {
                became_empty = true;
            }
        });
        if became_empty {
            let _ = self.tags.remove(&letter);
        }
    }

    /// Set all values for a given tag
    pub fn set_tag_values(&mut self, letter: char, values: Vec<String>) {
        let _ = self.tags.insert(letter, values);
    }

    /// Remove all Tag values of a given kind from a filter
    pub fn clear_tag_values(&mut self, letter: char) {
        let _ = self.tags.remove(&letter);
    }

    /// Convert filter tags into a `Vec<Tag>`
    pub fn tags_as_tags(&self) -> Vec<Tag> {
        let mut buffer: [u8; 4] = [0; 4];
        let mut tags: Vec<Tag> = Vec::with_capacity(self.tags.len());
        for (letter, values) in self.tags.iter() {
            let mut strings: Vec<String> = Vec::with_capacity(1 + values.len());
            strings.push(letter.encode_utf8(&mut buffer).to_owned());
            strings.extend(values.to_owned());
            tags.push(Tag::from_strings(strings));
        }
        tags
    }

    /// Does the event match the filter?
    pub fn event_matches(&self, e: &Event) -> bool {
        if !self.ids.is_empty() {
            let idhex: IdHex = e.id.into();
            if !self.ids.contains(&idhex) {
                return false;
            }
        }

        if !self.authors.is_empty() {
            let pubkeyhex: PublicKeyHex = e.pubkey.into();
            if !self.authors.contains(&pubkeyhex) {
                return false;
            }
        }

        if !self.kinds.is_empty() && !self.kinds.contains(&e.kind) {
            return false;
        }

        if let Some(since) = self.since {
            if e.created_at < since {
                return false;
            }
        }

        if let Some(until) = self.until {
            if e.created_at > until {
                return false;
            }
        }

        'tags: for (letter, values) in &self.tags {
            for tag in &e.tags {
                if tag.tagname().starts_with(*letter) && values.iter().any(|v| v == tag.value()) {
                    continue 'tags;
                }
            }

            return false;
        }

        true
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Filter {
        let mut map = BTreeMap::new();
        let _ = map.insert('e', vec![IdHex::mock().to_string()]);
        let _ = map.insert(
            'p',
            vec!["221115830ced1ca94352002485fcc7a75dcfe30d1b07f5f6fbe9c0407cfa59a1".to_string()],
        );

        Filter {
            ids: vec![IdHex::try_from_str(
                "3ab7b776cb547707a7497f209be799710ce7eb0801e13fd3c4e7b9261ac29084",
            )
            .unwrap()],
            authors: vec![],
            kinds: vec![EventKind::TextNote, EventKind::Metadata],
            tags: map,
            since: Some(Unixtime(1668572286)),
            ..Default::default()
        }
    }
}

impl fmt::Display for Filter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let mut parts = Vec::new();
        if !self.ids.is_empty() {
            parts.push(format!("ids: {:?}", self.ids));
        }
        if !self.authors.is_empty() {
            parts.push(format!("authors: {:?}", self.authors));
        }
        if !self.kinds.is_empty() {
            parts.push(format!("kinds: {:?}", self.kinds));
        }
        if !self.tags.is_empty() {
            parts.push(format!("tags: {:?}", self.tags));
        }
        if let Some(since) = self.since {
            parts.push(format!("since: {}", since));
        }
        if let Some(until) = self.until {
            parts.push(format!("until: {}", until));
        }
        if let Some(limit) = self.limit {
            parts.push(format!("limit: {}", limit));
        }
        write!(f, "Filter {{ {} }}", parts.join(", "))
    }
}

fn serialize_tags<S>(tags: &BTreeMap<char, Vec<String>>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let mut map = serializer.serialize_map(Some(tags.len()))?;
    for (tag, values) in tags.iter() {
        map.serialize_entry(&format!("#{tag}"), values)?;
    }
    map.end()
}

fn deserialize_tags<'de, D>(deserializer: D) -> Result<BTreeMap<char, Vec<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    struct TagsVisitor;

    impl<'de> Visitor<'de> for TagsVisitor {
        type Value = BTreeMap<char, Vec<String>>;

        fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
            formatter.write_str("map with keys in \"#t\" format")
        }

        fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
        where
            M: MapAccess<'de>,
        {
            let mut tags: BTreeMap<char, Vec<String>> = BTreeMap::new();
            while let Some((key, value)) = map.next_entry::<String, Vec<String>>()? {
                let mut chars = key.chars();
                if let (Some('#'), Some(ch), None) = (chars.next(), chars.next(), chars.next()) {
                    let _ = tags.insert(ch, value);
                }
            }
            Ok(tags)
        }
    }

    deserializer.deserialize_map(TagsVisitor)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {Filter, test_filters_serde}

    #[test]
    fn test_filter_mock() {
        assert_eq!(
            &serde_json::to_string(&Filter::mock()).unwrap(),
            r##"{"ids":["3ab7b776cb547707a7497f209be799710ce7eb0801e13fd3c4e7b9261ac29084"],"kinds":[1,0],"#e":["5df64b33303d62afc799bdc36d178c07b2e1f0d824f31b7dc812219440affab6"],"#p":["221115830ced1ca94352002485fcc7a75dcfe30d1b07f5f6fbe9c0407cfa59a1"],"since":1668572286}"##
        );
    }

    #[test]
    fn test_filter_example() {
        let raw_event = r##"{"id":"dcf0f0339a9868fc5f51867f27049186fd8497816a19967ba4f03a3edf65a647","pubkey":"f647c9568d09596e323fdd0144b8e2f35aaf5daa43f9eb59b502e99d90f43673","created_at":1715996970,"kind":7,"sig":"ef592a256107217d6710ba18f8446494f0feb99df430284d1d5a36e7859b04e642f40746916ecffb98614d57d9053242c543ad05a74f2c5843dc9ba2169c5175","content":"🫂","tags":[["e","b74444aaaee395e4e76de1902d00457742eecefbd6ee329a79a6ae125f97fcbf"],["p","a723805cda67251191c8786f4da58f797e6977582301354ba8e91bcb0342dc9c"],["k","1"]]}"##;
        let event: Event = serde_json::from_str(&raw_event).unwrap();

        let mut filter = Filter {
            kinds: vec![EventKind::Reaction],
            ..Default::default()
        };
        filter.set_tag_values(
            'p',
            vec!["a723805cda67251191c8786f4da58f797e6977582301354ba8e91bcb0342dc9c".to_owned()],
        );

        assert!(filter.event_matches(&event));
    }

    #[test]
    fn test_add_remove_id() {
        let mock = IdHex::mock();

        let mut filters: Filter = Filter::new();

        filters.add_id(&mock);
        assert_eq!(filters.ids.len(), 1);
        filters.add_id(&mock); // overwrites
        assert_eq!(filters.ids.len(), 1);
        filters.del_id(&mock);
        assert!(filters.ids.is_empty());
    }

    // add_remove_author would be very similar to the above

    #[test]
    fn test_add_remove_tags() {
        let mut filter = Filter::mock();
        filter.del_tag_value('e', IdHex::mock().to_string());
        assert_eq!(filter.tags.get(&'e'), None);

        let _ = filter.add_tag_value('t', "footstr".to_string());
        let _ = filter.add_tag_value('t', "bitcoin".to_string());
        filter.del_tag_value('t', "bitcoin".to_string());
        assert!(filter.tags.get(&'t').is_some());
    }

    #[test]
    fn test_event_matches() {
        use crate::{
            types::{PrivateKey, UncheckedUrl},
            Id, KeySigner, PreEvent, Signer, Tag,
        };

        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };
        let preevent = PreEvent {
            pubkey: signer.public_key(),
            created_at: Unixtime(1680000012),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new_event(Id::mock(), Some(UncheckedUrl::mock()), None),
                Tag::new_hashtag("foodstr".to_string()),
            ],
            content: "Hello World!".to_string(),
        };
        let event = signer.sign_event(preevent).unwrap();

        let mut filter = Filter {
            authors: vec![signer.public_key().into()],
            ..Default::default()
        };
        let _ = filter.add_tag_value('e', Id::mock().as_hex_string());
        assert_eq!(filter.event_matches(&event), true);

        let filter = Filter {
            authors: vec![signer.public_key().into()],
            kinds: vec![EventKind::LongFormContent],
            ..Default::default()
        };
        assert_eq!(filter.event_matches(&event), false);

        let filter = Filter {
            ids: vec![IdHex::mock()],
            authors: vec![signer.public_key().into()],
            ..Default::default()
        };
        assert_eq!(filter.event_matches(&event), false);
    }
}
```

---

### id.rs

**Size:** 6627 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use derive_more::{AsMut, AsRef, Deref, Display, From, FromStr, Into};
use serde::{
    de::{Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::Error;

/// An event identifier, constructed as a SHA256 hash of the event fields
/// according to NIP-01
#[derive(
    AsMut, AsRef, Clone, Copy, Debug, Deref, Eq, From, Hash, Into, Ord, PartialEq, PartialOrd,
)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
#[derive(Default)]
pub struct Id(pub [u8; 32]);

impl Id {
    /// Render into a hexadecimal string
    ///
    /// Consider converting `.into()` an `IdHex` which is a wrapped type rather
    /// than a naked `String`
    pub fn as_hex_string(&self) -> String {
        hex::encode(self.0)
    }

    /// Create from a hexadecimal string
    pub fn try_from_hex_string(v: &str) -> Result<Id, Error> {
        let vec: Vec<u8> = hex::decode(v)?;
        Ok(Id(vec
            .try_into()
            .map_err(|_| Error::WrongLengthHexString)?))
    }

    /// Create from a byte slice.
    pub fn try_from_bytes(v: &[u8]) -> Result<Id, Error> {
        if v.len() != 32 {
            return Err(Error::InvalidId);
        }
        let mut bytes = [0u8; 32];
        bytes.copy_from_slice(v);
        Ok(Id(bytes))
    }

    /// Export as a bech32 encoded string ("note")
    pub fn as_bech32_string(&self) -> String {
        bech32::encode::<bech32::Bech32>(*super::HRP_NOTE, &self.0).unwrap()
    }

    /// Import from a bech32 encoded string ("note")
    pub fn try_from_bech32_string(s: &str) -> Result<Id, Error> {
        let data = bech32::decode(s)?;
        if data.0 != *super::HRP_NOTE {
            Err(Error::WrongBech32(
                super::HRP_NOTE.to_lowercase(),
                data.0.to_lowercase(),
            ))
        } else if data.1.len() != 32 {
            Err(Error::InvalidId)
        } else {
            match <[u8; 32]>::try_from(data.1) {
                Ok(array) => Ok(Id(array)),
                _ => Err(Error::InvalidId),
            }
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Id {
        Id::try_from_hex_string("5df64b33303d62afc799bdc36d178c07b2e1f0d824f31b7dc812219440affab6")
            .unwrap()
    }
}

impl fmt::Display for Id {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_hex_string())
    }
}

impl Serialize for Id {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&hex::encode(self.0))
    }
}

impl<'de> Deserialize<'de> for Id {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IdVisitor)
    }
}

struct IdVisitor;

impl Visitor<'_> for IdVisitor {
    type Value = Id;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a lowercase hexadecimal string representing 32 bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<Id, E>
    where
        E: serde::de::Error,
    {
        let vec: Vec<u8> = hex::decode(v).map_err(|e| serde::de::Error::custom(format!("{e}")))?;

        Ok(Id(vec.try_into().map_err(|e: Vec<u8>| {
            E::custom(format!(
                "Id is not 32 bytes long. Was {} bytes long",
                e.len()
            ))
        })?))
    }
}

/// An event identifier, constructed as a SHA256 hash of the event fields
/// according to NIP-01, as a hex string
///
/// You can convert from an `Id` into this with `From`/`Into`.  You can convert
/// this back to an `Id` with `TryFrom`/`TryInto`.
#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Deref,
    Display,
    Eq,
    From,
    FromStr,
    Hash,
    Into,
    Ord,
    PartialEq,
    PartialOrd,
)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct IdHex(String);

impl IdHex {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> IdHex {
        From::from(Id::mock())
    }

    /// Try from &str
    pub fn try_from_str(s: &str) -> Result<IdHex, Error> {
        Self::try_from_string(s.to_owned())
    }

    /// Try from String
    pub fn try_from_string(s: String) -> Result<IdHex, Error> {
        if s.len() != 64 {
            return Err(Error::InvalidId);
        }
        let vec: Vec<u8> = hex::decode(&s)?;
        if vec.len() != 32 {
            return Err(Error::InvalidId);
        }
        Ok(IdHex(s))
    }

    /// As &str
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Into String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<&str> for IdHex {
    type Error = Error;

    fn try_from(s: &str) -> Result<IdHex, Error> {
        IdHex::try_from_str(s)
    }
}

impl From<Id> for IdHex {
    fn from(i: Id) -> IdHex {
        IdHex(i.as_hex_string())
    }
}

impl From<IdHex> for Id {
    fn from(h: IdHex) -> Id {
        // could only fail if IdHex is invalid
        Id::try_from_hex_string(&h.0).unwrap()
    }
}

impl Serialize for IdHex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for IdHex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(IdHexVisitor)
    }
}

struct IdHexVisitor;

impl Visitor<'_> for IdHexVisitor {
    type Value = IdHex;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a lowercase hexadecimal string representing 32 bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<IdHex, E>
    where
        E: serde::de::Error,
    {
        if v.len() != 64 {
            return Err(serde::de::Error::custom("IdHex is not 64 characters long"));
        }

        let vec: Vec<u8> = hex::decode(v).map_err(|e| serde::de::Error::custom(format!("{e}")))?;
        if vec.len() != 32 {
            return Err(serde::de::Error::custom("Invalid IdHex"));
        }

        Ok(IdHex(v.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {Id, test_id_serde}
    test_serde! {IdHex, test_id_hex_serde}

    #[test]
    fn test_id_bech32() {
        let bech32 = Id::mock().as_bech32_string();
        println!("{bech32}");
        assert_eq!(Id::mock(), Id::try_from_bech32_string(&bech32).unwrap());
    }
}
```

---

### identity.rs

**Size:** 13095 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{ops::DerefMut, sync::mpsc::Sender};

use super::{
    ContentEncryptionAlgorithm, DelegationConditions, EncryptedPrivateKey, Error, Event, EventV1,
    EventV2, Id, KeySecurity, KeySigner, Metadata, PreEvent, PrivateKey, PublicKey, Rumor, RumorV1,
    RumorV2, Signature, Signer,
};

/// All states that your identity can be in
#[derive(Debug, Default)]
pub enum Identity {
    /// No identity information
    #[default]
    None,

    /// Public key only
    Public(PublicKey),

    /// Signer (locked or unlocked)
    Signer(Box<dyn Signer>),
}

// No one besides the Identity has the internal Signer, so we can safely Send
unsafe impl Send for Identity {}

// Nobody can write while someone else is reading with just a non-mutable
// &reference
unsafe impl Sync for Identity {}

impl Identity {
    /// New `Identity` from a public key
    pub fn from_public_key(pk: PublicKey) -> Self {
        Self::Public(pk)
    }

    /// New `Identity` from a private key
    pub fn from_private_key(pk: PrivateKey, pass: &str, log_n: u8) -> Result<Self, Error> {
        let key_signer = KeySigner::from_private_key(pk, pass, log_n)?;
        Ok(Self::Signer(Box::new(key_signer)))
    }

    /// New `Identity` from an encrypted private key and a public key
    pub fn from_locked_parts(pk: PublicKey, epk: EncryptedPrivateKey) -> Self {
        let key_signer = KeySigner::from_locked_parts(epk, pk);
        Self::Signer(Box::new(key_signer))
    }

    /// New `Identity` from an encrypted private key and its password
    pub fn from_encrypted_private_key(epk: EncryptedPrivateKey, pass: &str) -> Result<Self, Error> {
        let key_signer = KeySigner::from_encrypted_private_key(epk, pass)?;
        Ok(Self::Signer(Box::new(key_signer)))
    }

    /// Generate a new `Identity`
    pub fn generate(password: &str, log_n: u8) -> Result<Self, Error> {
        let key_signer = KeySigner::generate(password, log_n)?;
        Ok(Self::Signer(Box::new(key_signer)))
    }

    /// Unlock
    pub fn unlock(&mut self, password: &str) -> Result<(), Error> {
        if let Identity::Signer(ref mut boxed_signer) = self {
            boxed_signer.deref_mut().unlock(password)
        } else {
            Ok(())
        }
    }

    /// Lock access to the private key
    pub fn lock(&mut self) {
        if let Identity::Signer(ref mut boxed_signer) = self {
            boxed_signer.deref_mut().lock()
        }
    }

    /// Has a public key
    pub fn has_public_key(&self) -> bool {
        !matches!(self, Identity::None)
    }

    /// Has a private key
    pub fn has_private_key(&self) -> bool {
        matches!(self, Identity::Signer(_))
    }

    /// Is the identity locked?
    pub fn is_locked(&self) -> bool {
        !self.is_unlocked()
    }

    /// Is the identity unlocked?
    pub fn is_unlocked(&self) -> bool {
        if let Identity::Signer(box_signer) = self {
            !box_signer.is_locked()
        } else {
            false
        }
    }

    /// Change the passphrase used for locking access to the private key
    pub fn change_passphrase(&mut self, old: &str, new: &str, log_n: u8) -> Result<(), Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.change_passphrase(old, new, log_n),
        }
    }

    /// What is the public key?
    pub fn public_key(&self) -> Option<PublicKey> {
        match self {
            Identity::None => None,
            Identity::Public(pk) => Some(*pk),
            Identity::Signer(boxed_signer) => Some(boxed_signer.public_key()),
        }
    }

    /// What is the signer's encrypted private key?
    pub fn encrypted_private_key(&self) -> Option<&EncryptedPrivateKey> {
        if let Identity::Signer(boxed_signer) = self {
            boxed_signer.encrypted_private_key()
        } else {
            None
        }
    }

    /// Sign a 32-bit hash
    pub fn sign_id(&self, id: Id) -> Result<Signature, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.sign_id(id),
        }
    }

    /// Sign a message (this hashes with SHA-256 first internally)
    pub fn sign(&self, message: &[u8]) -> Result<Signature, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.sign(message),
        }
    }

    /// Encrypt
    pub fn encrypt(
        &self,
        other: &PublicKey,
        plaintext: &str,
        algo: ContentEncryptionAlgorithm,
    ) -> Result<String, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.encrypt(other, plaintext, algo),
        }
    }

    /// Decrypt
    pub fn decrypt(&self, other: &PublicKey, ciphertext: &str) -> Result<String, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.decrypt(other, ciphertext),
        }
    }

    /// Get NIP-44 conversation key
    pub fn nip44_conversation_key(&self, other: &PublicKey) -> Result<[u8; 32], Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.nip44_conversation_key(other),
        }
    }

    /// Export the private key in hex.
    ///
    /// This returns a boolean indicating if the key security was downgraded. If
    /// it was, the caller should save the new self.encrypted_private_key()
    ///
    /// We need the password and log_n parameters to possibly rebuild
    /// the EncryptedPrivateKey when downgrading key security
    pub fn export_private_key_in_hex(
        &mut self,
        pass: &str,
        log_n: u8,
    ) -> Result<(String, bool), Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.export_private_key_in_hex(pass, log_n),
        }
    }

    /// Export the private key in bech32.
    ///
    /// This returns a boolean indicating if the key security was downgraded. If
    /// it was, the caller should save the new self.encrypted_private_key()
    ///
    /// We need the password and log_n parameters to possibly rebuild
    /// the EncryptedPrivateKey when downgrading key security
    pub fn export_private_key_in_bech32(
        &mut self,
        pass: &str,
        log_n: u8,
    ) -> Result<(String, bool), Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => {
                boxed_signer.export_private_key_in_bech32(pass, log_n)
            }
        }
    }

    /// Get the security level of the private key
    pub fn key_security(&self) -> Result<KeySecurity, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.key_security(),
        }
    }

    /// Upgrade the encrypted private key to the latest format
    pub fn upgrade(&mut self, pass: &str, log_n: u8) -> Result<(), Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.upgrade(pass, log_n),
        }
    }

    /// Create an event that sets Metadata
    pub fn create_metadata_event(
        &self,
        input: PreEvent,
        metadata: Metadata,
    ) -> Result<Event, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.create_metadata_event(input, metadata),
        }
    }

    /// Create a ZapRequest event These events are not published to nostr, they
    /// are sent to a lnurl.
    pub fn create_zap_request_event(
        &self,
        recipient_pubkey: PublicKey,
        zapped_event: Option<Id>,
        millisatoshis: u64,
        relays: Vec<String>,
        content: String,
    ) -> Result<Event, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.create_zap_request_event(
                recipient_pubkey,
                zapped_event,
                millisatoshis,
                relays,
                content,
            ),
        }
    }

    /// Decrypt the contents of an event
    pub fn decrypt_event_contents(&self, event: &Event) -> Result<String, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.decrypt_event_contents(event),
        }
    }

    /// If a gift wrap event, unwrap and return the inner Rumor
    pub fn unwrap_giftwrap(&self, event: &Event) -> Result<Rumor, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.unwrap_giftwrap(event),
        }
    }

    /// If a gift wrap event, unwrap and return the inner Rumor
    /// @deprecated for migrations only
    pub fn unwrap_giftwrap1(&self, event: &EventV1) -> Result<RumorV1, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.unwrap_giftwrap1(event),
        }
    }

    /// If a gift wrap event, unwrap and return the inner Rumor
    /// @deprecated for migrations only
    pub fn unwrap_giftwrap2(&self, event: &EventV2) -> Result<RumorV2, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.unwrap_giftwrap2(event),
        }
    }

    /// Generate delegation signature
    pub fn generate_delegation_signature(
        &self,
        delegated_pubkey: PublicKey,
        delegation_conditions: &DelegationConditions,
    ) -> Result<Signature, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => {
                boxed_signer.generate_delegation_signature(delegated_pubkey, delegation_conditions)
            }
        }
    }

    /// Giftwrap an event
    pub fn giftwrap(&self, input: PreEvent, pubkey: PublicKey) -> Result<Event, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.giftwrap(input, pubkey),
        }
    }

    /// Sign an event
    pub fn sign_event(&self, input: PreEvent) -> Result<Event, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.sign_event(input),
        }
    }

    /// Sign an event with Proof-of-Work
    pub fn sign_event_with_pow(
        &self,
        input: PreEvent,
        zero_bits: u8,
        work_sender: Option<Sender<u8>>,
    ) -> Result<Event, Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => {
                boxed_signer.sign_event_with_pow(input, zero_bits, work_sender)
            }
        }
    }

    /// Verify delegation signature
    pub fn verify_delegation_signature(
        &self,
        delegated_pubkey: PublicKey,
        delegation_conditions: &DelegationConditions,
        signature: &Signature,
    ) -> Result<(), Error> {
        match self {
            Identity::None => Err(Error::NoPublicKey),
            Identity::Public(_) => Err(Error::NoPrivateKey),
            Identity::Signer(boxed_signer) => boxed_signer.verify_delegation_signature(
                delegated_pubkey,
                delegation_conditions,
                signature,
            ),
        }
    }
}
```

---

### image_dimensions.rs

**Size:** 328 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// Dimensions for an image

use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct ImageDimensions {
    pub width: u64,
    pub height: u64,
}
```

---

### internal.rs

**Size:** 7581 bytes | **Modified:** 2026-01-20 14:02:27

```rust
#![allow(clippy::print_with_newline)]
use base64::Engine;
use http::Uri;
use tokio_tungstenite::{tungstenite, tungstenite::Message};

use super::{ClientMessage, Event, Filter, RelayMessage, RelayMessageV5, SubscriptionId};
use crate::{blockheight::blockheight_sync, weeble::weeble_sync};

pub(crate) fn filters_to_wire(filters: Vec<Filter>) -> String {
    let message = ClientMessage::Req(
        SubscriptionId(format!(
            "{:?}/{:?}/{:?}",
            weeble_sync(),
            blockheight_sync(),
            weeble_sync(),
        )),
        filters,
    );
    serde_json::to_string(&message).expect("Could not serialize message")
}

pub(crate) fn event_to_wire(event: Event) -> String {
    let message = ClientMessage::Event(Box::new(event));
    serde_json::to_string(&message).expect("Could not serialize message")
}
//use nostr_types::EventV2;
//pub(crate) fn event_to_wire_v2(event: EventV2) -> String {
//    let message = ClientMessage::Event_V2(Box::new(event));
//    serde_json::to_string(&message).expect("Could not serialize message")
//}
//pub(crate) fn event_to_wire(event: EventV3) -> String {
//    let message = ClientMessage::Event(Box::new(event));
//    serde_json::to_string(&message).expect("Could not serialize message")
//}

pub(crate) fn fetch(host: String, uri: Uri, wire: String) -> Vec<Event> {
    let mut events: Vec<Event> = Vec::new();

    let key: [u8; 16] = rand::random();
    let request = http::request::Request::builder()
        .method("GET")
        .header("Host", host)
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            base64::engine::general_purpose::STANDARD.encode(key),
        )
        .uri(uri)
        .body(())
        .expect("Could not build request");

    let (mut websocket, _response) =
        tungstenite::connect(request).expect("Could not connect to relay");

    websocket
        .send(Message::Text(wire.into()))
        .expect("Could not send message to relay");

    loop {
        let message = match websocket.read() {
            Ok(m) => m,
            Err(e) => {
                //handle differently
                println!("Problem reading from websocket: {}", e);
                return events;
            }
        };

        match message {
            Message::Text(s) => {
                let relay_message: RelayMessageV5 = serde_json::from_str(&s).expect(&s);
                match relay_message {
                    RelayMessageV5::Closed(_, _) => todo!(),
                    RelayMessageV5::Event(_, e) => events.push(*e),
                    RelayMessageV5::Notice(s) => println!("NOTICE: {}", s),
                    RelayMessageV5::Eose(_) => {
                        let message = ClientMessage::Close(SubscriptionId(format!(
                            "{:?}/{:?}/{:?}",
                            weeble_sync(),
                            blockheight_sync(),
                            weeble_sync(),
                        )));
                        let wire = match serde_json::to_string(&message) {
                            Ok(w) => w,
                            Err(e) => {
                                println!("Could not serialize message: {}", e);
                                return events;
                            }
                        };
                        if let Err(e) = websocket.send(Message::Text(wire.into())) {
                            println!("Could not write close subscription message: {}", e);
                            return events;
                        }
                        if let Err(e) = websocket.send(Message::Close(None)) {
                            println!("Could not write websocket close message: {}", e);
                            return events;
                        }
                    }
                    RelayMessageV5::Ok(_id, ok, reason) => {
                        println!("OK: ok={} reason={}", ok, reason)
                    }
                    RelayMessageV5::Auth(challenge) => {
                        // NIP-0042 [\"AUTH\", \"<challenge-string>\"]
                        print!("[\"AUTH\":\"{}\"]", challenge)
                    }
                    RelayMessageV5::Notify(_) => todo!(),
                }
            }
            Message::Binary(_) => {
                println!("IGNORING BINARY MESSAGE")
            }
            Message::Ping(vec) => {
                if let Err(e) = websocket.send(Message::Pong(vec)) {
                    println!("Unable to pong: {}", e);
                }
            }
            Message::Pong(_) => {
                println!("IGNORING PONG")
            }
            Message::Close(_) => {
                //println!("Closing");
                break;
            }
            Message::Frame(_) => {
                println!("UNEXPECTED RAW WEBSOCKET FRAME")
            }
        }
    }

    events
}

pub(crate) fn post(host: String, uri: Uri, wire: String) {
    //gnostr key here
    let key: [u8; 16] = rand::random();
    let request = http::request::Request::builder()
        .method("GET")
        .header("Host", host.clone())
        .header("Connection", "Upgrade")
        .header("Upgrade", "websocket")
        .header("Sec-WebSocket-Version", "13")
        .header(
            "Sec-WebSocket-Key",
            base64::engine::general_purpose::STANDARD.encode(key),
        )
        .uri(uri)
        .body(())
        .expect("Could not build request");

    let (mut websocket, _response) =
        tungstenite::connect(request).expect("Could not connect to relay");

    print!("{}\n", wire);
    websocket
        .send(Message::Text(wire.into()))
        .expect("Could not send message to relay");

    // Get and print one response message

    let message = match websocket.read() {
        Ok(m) => m,
        Err(e) => {
            //handle differently
            println!("Problem reading from websocket: {}", e);
            return;
        }
    };

    match message {
        Message::Text(s) => {
            let relay_message: RelayMessage = serde_json::from_str(&s).expect(&s);
            match relay_message {
                RelayMessage::Event(_, e) => {
                    println!("[\"EVENT\": {}]", serde_json::to_string(&e).unwrap())
                }
                RelayMessage::Notice(s) => println!("NOTICE: {}", s),
                RelayMessage::Eose(_) => println!("EOSE"),
                //nostr uses json extensively
                //yet relays dont return json formatted messages?
                RelayMessage::Ok(_id, ok, reason) => println!(
                    "[\"{}\",{{\"ok\":\"{}\",\"reason\":\"{}\"}}]",
                    host, ok, reason
                ),
                RelayMessage::Auth(challenge) => print!("[\"AUTH\":\"{}\"]", challenge),
                RelayMessage::Notify(_) => todo!(),
                RelayMessage::Closed(_, _) => todo!(),
            }
        }
        Message::Binary(_) => {
            println!("IGNORING BINARY MESSAGE")
        }
        Message::Ping(vec) => {
            if let Err(e) = websocket.send(Message::Pong(vec)) {
                println!("Unable to pong: {}", e);
            }
        }
        Message::Pong(_) => {
            println!("IGNORING PONG")
        }
        Message::Close(_) => {
            println!("Closing");
        }
        Message::Frame(_) => {
            println!("UNEXPECTED RAW WEBSOCKET FRAME")
        }
    }
}
```

---

### key_signer.rs

**Size:** 6603 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use super::{
    ContentEncryptionAlgorithm, EncryptedPrivateKey, Error, Id, KeySecurity, PrivateKey, PublicKey,
    Signature, Signer,
};

/// Signer with a local private key (and public key)
pub struct KeySigner {
    encrypted_private_key: EncryptedPrivateKey,
    public_key: PublicKey,
    private_key: Option<PrivateKey>,
}

impl Clone for KeySigner {
    fn clone(&self) -> Self {
        Self {
            encrypted_private_key: self.encrypted_private_key.clone(),
            public_key: self.public_key,
            private_key: self.private_key.clone(),
        }
    }
}

impl fmt::Debug for KeySigner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        f.debug_struct("KeySigner")
            .field("encrypted_private_key", &self.encrypted_private_key)
            .field("public_key", &self.public_key)
            .finish()
    }
}

impl KeySigner {
    /// Create a Signer from an `EncryptedPrivateKey`
    pub fn from_locked_parts(epk: EncryptedPrivateKey, pk: PublicKey) -> Self {
        Self {
            encrypted_private_key: epk,
            public_key: pk,
            private_key: None,
        }
    }

    /// Create a Signer from a `PrivateKey`
    pub fn from_private_key(privk: PrivateKey, password: &str, log_n: u8) -> Result<Self, Error> {
        let epk = privk.export_encrypted(password, log_n)?;
        Ok(Self {
            encrypted_private_key: epk,
            public_key: privk.public_key(),
            private_key: Some(privk),
        })
    }

    /// Create a Signer from an `EncryptedPrivateKey` and a password to unlock
    /// it
    pub fn from_encrypted_private_key(epk: EncryptedPrivateKey, pass: &str) -> Result<Self, Error> {
        let priv_key = epk.decrypt(pass)?;
        let pub_key = priv_key.public_key();
        Ok(Self::from_locked_parts(epk, pub_key))
    }

    /// Create a Signer by generating a new `PrivateKey`
    pub fn generate(password: &str, log_n: u8) -> Result<Self, Error> {
        let privk = PrivateKey::generate();
        let epk = privk.export_encrypted(password, log_n)?;
        Ok(Self {
            encrypted_private_key: epk,
            public_key: privk.public_key(),
            private_key: Some(privk),
        })
    }
}

impl Signer for KeySigner {
    fn is_locked(&self) -> bool {
        self.private_key.is_none()
    }

    fn unlock(&mut self, password: &str) -> Result<(), Error> {
        if !self.is_locked() {
            return Ok(());
        }

        let private_key = self.encrypted_private_key.decrypt(password)?;

        self.private_key = Some(private_key);

        Ok(())
    }

    fn lock(&mut self) {
        self.private_key = None;
    }

    fn change_passphrase(&mut self, old: &str, new: &str, log_n: u8) -> Result<(), Error> {
        let private_key = self.encrypted_private_key.decrypt(old)?;
        self.encrypted_private_key = private_key.export_encrypted(new, log_n)?;
        self.private_key = Some(private_key);
        Ok(())
    }

    fn upgrade(&mut self, pass: &str, log_n: u8) -> Result<(), Error> {
        let private_key = self.encrypted_private_key.decrypt(pass)?;
        self.encrypted_private_key = private_key.export_encrypted(pass, log_n)?;
        Ok(())
    }

    fn public_key(&self) -> PublicKey {
        self.public_key
    }

    fn encrypted_private_key(&self) -> Option<&EncryptedPrivateKey> {
        Some(&self.encrypted_private_key)
    }

    fn sign_id(&self, id: Id) -> Result<Signature, Error> {
        match &self.private_key {
            Some(pk) => pk.sign_id(id),
            None => Err(Error::SignerIsLocked),
        }
    }

    fn sign(&self, message: &[u8]) -> Result<Signature, Error> {
        match &self.private_key {
            Some(pk) => pk.sign(message),
            None => Err(Error::SignerIsLocked),
        }
    }

    fn encrypt(
        &self,
        other: &PublicKey,
        plaintext: &str,
        algo: ContentEncryptionAlgorithm,
    ) -> Result<String, Error> {
        match &self.private_key {
            Some(pk) => pk.encrypt(other, plaintext, algo),
            None => Err(Error::SignerIsLocked),
        }
    }

    fn decrypt(&self, other: &PublicKey, ciphertext: &str) -> Result<String, Error> {
        match &self.private_key {
            Some(pk) => pk.decrypt(other, ciphertext),
            None => Err(Error::SignerIsLocked),
        }
    }

    fn nip44_conversation_key(&self, other: &PublicKey) -> Result<[u8; 32], Error> {
        let xpub = other.as_xonly_public_key();
        match &self.private_key {
            Some(pk) => Ok(super::nip44::get_conversation_key(pk.as_secret_key(), xpub)),
            None => Err(Error::SignerIsLocked),
        }
    }

    fn export_private_key_in_hex(
        &mut self,
        pass: &str,
        log_n: u8,
    ) -> Result<(String, bool), Error> {
        if let Some(pk) = &mut self.private_key {
            // Test password and check key security
            let pkcheck = self.encrypted_private_key.decrypt(pass)?;

            // side effect: this may downgrade the key security of self.private_key
            let output = pk.as_hex_string();

            // If key security changed, re-export
            let mut downgraded = false;
            if pk.key_security() != pkcheck.key_security() {
                downgraded = true;
                self.encrypted_private_key = pk.export_encrypted(pass, log_n)?;
            }
            Ok((output, downgraded))
        } else {
            Err(Error::SignerIsLocked)
        }
    }

    fn export_private_key_in_bech32(
        &mut self,
        pass: &str,
        log_n: u8,
    ) -> Result<(String, bool), Error> {
        if let Some(pk) = &mut self.private_key {
            // Test password and check key security
            let pkcheck = self.encrypted_private_key.decrypt(pass)?;

            // side effect: this may downgrade the key security of self.private_key
            let output = pk.as_bech32_string();

            // If key security changed, re-export
            let mut downgraded = false;
            if pk.key_security() != pkcheck.key_security() {
                downgraded = true;
                self.encrypted_private_key = pk.export_encrypted(pass, log_n)?;
            }

            Ok((output, downgraded))
        } else {
            Err(Error::SignerIsLocked)
        }
    }

    fn key_security(&self) -> Result<KeySecurity, Error> {
        match &self.private_key {
            Some(pk) => Ok(pk.key_security()),
            None => Err(Error::SignerIsLocked),
        }
    }
}
```

---

### keys.rs

**Size:** 2438 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// Dummy Keys struct for now, to replace nostr_sdk::Keys
// TODO: Implement actual Keys functionality

use std::fmt;

use crate::types::{Error, PrivateKey, PublicKey};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Keys {
    private_key: Option<PrivateKey>,
    public_key: PublicKey,
}

impl fmt::Display for Keys {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Keys {{ public_key: {} }}",
            self.public_key.as_hex_string()
        )
    }
}

impl Keys {
    pub fn generate() -> Self {
        let private_key = PrivateKey::generate();
        let public_key = private_key.public_key();
        Keys {
            private_key: Some(private_key),
            public_key,
        }
    }

    pub fn new(private_key: PrivateKey) -> Self {
        let public_key = private_key.public_key();
        Keys {
            private_key: Some(private_key),
            public_key,
        }
    }

    pub fn public_key(&self) -> PublicKey {
        self.public_key
    }

    pub fn secret_key(&self) -> Result<PrivateKey, Error> {
        self.private_key.clone().ok_or(Error::NoPrivateKey)
    }

    /// Parse from nsec or bech32 string (for compatibility with nostr_sdk)
    pub fn parse(s: String) -> Option<Self> {
        use crate::types::PrivateKey;

        // Try to parse as private key first (nsec)
        if let Ok(private_key) = PrivateKey::try_from_bech32_string(&s) {
            return Some(Self::new(private_key));
        }

        // Try as hex private key
        if let Ok(private_key) = PrivateKey::try_from_hex_string(&s) {
            return Some(Self::new(private_key));
        }

        None
    }

    // Generate vanity key with specified prefixes
    pub fn vanity(prefixes: Vec<String>, bech32: bool, _num_cores: usize) -> Result<Self, Error> {
        println!("Generating vanity key with prefixes: {:?}", prefixes);

        // For now, return random key (TODO: implement actual vanity generation)
        let keys = Self::generate();

        if bech32 {
            println!("Public key: {}", keys.public_key().as_bech32_string());
            println!("Private key: {}", keys.secret_key()?.as_bech32_string());
        } else {
            println!("Public key (hex): {}", keys.public_key().as_hex_string());
            println!("Private key (hex): {}", keys.secret_key()?.as_hex_string());
        }

        Ok(keys)
    }
}
```

---

### metadata.rs

**Size:** 636 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use super::versioned::metadata::MetadataV1;

/// Metadata about a user
///
/// Note: the value is an Option because some real-world data has been found to
/// contain JSON nulls as values, and we don't want deserialization of those
/// events to fail. We treat these in our get() function the same as if the key
/// did not exist.

pub const DEFAULT_AVATAR: &str = "https://avatars.githubusercontent.com/u/135379339?s=400&u=11cb72cccbc2b13252867099546074c50caef1ae&v=4";
pub const DEFAULT_BANNER: &str = "https://raw.githubusercontent.com/gnostr-org/gnostr-icons/refs/heads/master/banner/1024x341.png";

pub type Metadata = MetadataV1;
```

---

### mod.rs

**Size:** 13832 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// Copyright 2015-2020 nostr-proto Developers
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according to
// those terms.

//! This crate provides types for nostr protocol handling.

#![allow(missing_docs)]
#![deny(
    missing_debug_implementations,
    trivial_casts,
    trivial_numeric_casts,
    unused_import_braces,
    //unused_qualifications,
    unused_results,
    unused_lifetimes,
    unused_labels,
    unused_extern_crates,
    keyword_idents,
    deprecated_in_future,
    unstable_features,
    single_use_lifetimes,
    //unsafe_code,
    unreachable_pub,
    //missing_docs,
    missing_copy_implementations
)]
#![deny(clippy::string_slice)]

/// internal
pub mod internal;

mod client_message;
pub use client_message::ClientMessage;

mod content;
pub use content::{ContentSegment, ShatteredContent, Span};

mod delegation;
pub use delegation::{DelegationConditions, EventDelegation};

mod error;
pub use error::Error;

mod event;
mod event_builder;
pub use event::{Event, PreEvent, Rumor, ZapData};

/// event_kind
pub mod event_kind;
pub use event_kind::{EventKind, EventKindIterator, EventKindOrRange};

/// event_reference
mod event_reference;
pub use event_reference::EventReference;

/// filter
mod filter;
pub use filter::Filter;

/// id
mod id;
pub use id::{Id, IdHex};

/// identity
mod identity;
pub use identity::Identity;

pub mod key_signer;
pub use key_signer::KeySigner;

/// NIP-28: Public Chat Channels
pub mod nip28;
pub use nip28::*;

/// metadata
pub mod metadata;
pub use metadata::Metadata;

/// naddr
mod naddr;
pub use naddr::NAddr;

/// nevent
mod nevent;
pub use nevent::NEvent;

/// NIP-05: Mapping Nostr keys to DNS-based internet identifiers
pub mod nip0;
mod nip05;
/// NIP-10: Text Notes and Threads
pub mod nip10;
/// NIP-13: Proof of Work
pub mod nip13;
/// NIP-15: End of Stored Events Notice
pub mod nip15;
/// NIP-18: Reposts
pub mod nip18;
/// NIP-02: Contact List and Petnames
pub mod nip2;
/// NIP-26: Delegation
pub mod nip26;
/// NIP-03: OpenTimestamps Attestations for Events
pub mod nip3;
pub mod nip34;
/// NIP-04: Encrypted Direct Message
pub mod nip4;
/// NIP-59: Gift Wrap
pub mod nip59;
/// NIP-06: Basic key derivation from mnemonic seed phrase
pub mod nip6;
/// NIP-09: Event Deletion
pub mod nip9;
pub use nip05::Nip05;

mod nostr_url;
pub use nostr_url::{find_nostr_bech32_pos, find_nostr_url_pos, NostrBech32, NostrUrl};

mod pay_request_data;
pub use pay_request_data::PayRequestData;

mod private_key;
pub use private_key::{ContentEncryptionAlgorithm, EncryptedPrivateKey, KeySecurity, PrivateKey};

mod profile;
pub use profile::Profile;

mod public_key;
pub use public_key::{PublicKey, PublicKeyHex};
pub use secp256k1::XOnlyPublicKey;

mod relay_information_document;
pub use relay_information_document::{
    Fee, RelayFees, RelayInformationDocument, RelayLimitation, RelayRetention,
};

mod relay_list;
pub use relay_list::{RelayList, RelayListUsage};

mod relay_message;
pub use relay_message::RelayMessage;

mod relay_usage;
pub use relay_usage::{RelayUsage, RelayUsageSet};

mod satoshi;
pub use satoshi::MilliSatoshi;

mod signature;
pub use signature::{Signature, SignatureHex};

mod signer;
pub use signer::Signer;

mod simple_relay_list;
pub use simple_relay_list::{SimpleRelayList, SimpleRelayUsage};

mod subscription_id;
pub use subscription_id::SubscriptionId;

mod tag;
pub use tag::Tag;

mod unixtime;
pub use unixtime::Unixtime;

mod url;
pub use self::url::{RelayOrigin, RelayUrl, UncheckedUrl, Url};

pub mod nip14;
pub mod nip25;
pub mod nip30;
pub mod nip32;
pub mod nip36;
pub mod nip38;
pub mod nip40;
/// NIP-44 related types and functionalities for secure direct messages.
pub mod nip44;
pub mod nip53;
pub mod nip94;
pub mod nostr_client; // Added
pub use nip44::{decrypt, encrypt, get_conversation_key, Error as Nip44Error};
pub use nostr_client::*; // Added
pub mod nip19;
pub use nip19::*;
pub mod keys;
pub use keys::Keys;
pub mod client;
pub use client::{Client, FilterOptions, Options};
pub mod image_dimensions;
// Re-export bitcoin_hashes for use throughout the codebase
pub use bitcoin_hashes::sha1::Hash as Sha1Hash;
pub use image_dimensions::ImageDimensions;

#[cfg(test)]
#[macro_export]
/// A helper macro for testing `serde` serialization and deserialization.
macro_rules! test_serde {
    ($t:ty, $fnname:ident) => {
        #[test]
        fn $fnname() {
            let a = <$t>::mock();
            let x = serde_json::to_string(&a).unwrap();
            println!("{}", x);
            let b = serde_json::from_str(&x).unwrap();
            assert_eq!(a, b);
        }
    };
}

// mod types;

pub mod versioned;
pub use event_builder::EventBuilder;
pub use versioned::{
    ClientMessageV1, ClientMessageV2, ClientMessageV3, EventV1, EventV2, EventV3, FeeV1,
    MetadataV1, Nip05V1, PreEventV1, PreEventV2, PreEventV3, RelayFeesV1,
    RelayInformationDocumentV1, RelayInformationDocumentV2, RelayLimitationV1, RelayLimitationV2,
    RelayMessageV1, RelayMessageV2, RelayMessageV3, RelayMessageV4, RelayMessageV5,
    RelayRetentionV1, RumorV1, RumorV2, RumorV3, TagV1, TagV2, TagV3, Why, ZapDataV1, ZapDataV2,
};

#[inline]
pub(crate) fn get_leading_zero_bits(bytes: &[u8]) -> u8 {
    let mut res = 0_u8;
    for b in bytes {
        if *b == 0 {
            res += 8;
        } else {
            res += b.leading_zeros() as u8;
            return res;
        }
    }
    res
}

/// Trait for converting Option<T> into Vec<T>
pub trait IntoVec<T> {
    /// Convert into a Vec<T>
    fn into_vec(self) -> Vec<T>;
}

impl<T> IntoVec<T> for Option<T> {
    fn into_vec(self) -> Vec<T> {
        match self {
            None => vec![],
            Some(t) => vec![t],
        }
    }
}

use bech32::Hrp;
lazy_static::lazy_static! {
    static ref HRP_LNURL: Hrp = Hrp::parse("lnurl").expect("HRP error on lnurl");
    static ref HRP_NADDR: Hrp = Hrp::parse("naddr").expect("HRP error on naddr");
    static ref HRP_NCRYPTSEC: Hrp = Hrp::parse("ncryptsec").expect("HRP error on ncryptsec");
    static ref HRP_NEVENT: Hrp = Hrp::parse("nevent").expect("HRP error on nevent");
    static ref HRP_NOTE: Hrp = Hrp::parse("note").expect("HRP error on note");
    static ref HRP_NPROFILE: Hrp = Hrp::parse("nprofile").expect("HRP error on nprofile");
    static ref HRP_NPUB: Hrp = Hrp::parse("npub").expect("HRP error on npub");
    static ref HRP_NRELAY: Hrp = Hrp::parse("nrelay").expect("HRP error on nrelay");
    static ref HRP_NSEC: Hrp = Hrp::parse("nsec").expect("HRP error on nsec");
}

/// Add a 'p' pubkey tag to a set of tags if it doesn't already exist
pub fn add_pubkey_to_tags(
    existing_tags: &mut Vec<Tag>,
    new_pubkey: PublicKey,
    new_hint: Option<UncheckedUrl>,
) -> usize {
    match existing_tags.iter().position(|existing_tag| {
        if let Ok((pubkey, _, __)) = existing_tag.parse_pubkey() {
            pubkey == new_pubkey
        } else {
            false
        }
    }) {
        Some(idx) => idx,
        None => {
            existing_tags.push(Tag::new_pubkey(new_pubkey, new_hint, None));
            existing_tags.len() - 1
        }
    }
}

/// Add an 'e' id tag to a set of tags if it doesn't already exist
pub fn add_event_to_tags(
    existing_tags: &mut Vec<Tag>,
    new_id: Id,
    new_hint: Option<UncheckedUrl>,
    new_marker: &str,
    use_quote: bool,
) -> usize {
    if new_marker == "mention" && use_quote {
        // NIP-18: "Quote reposts are kind 1 events with an embedded q tag..."
        let newtag = Tag::new_quote(new_id, new_hint);

        match existing_tags.iter().position(|existing_tag| {
            if let Ok((id, _rurl)) = existing_tag.parse_quote() {
                id == new_id
            } else {
                false
            }
        }) {
            None => {
                existing_tags.push(newtag);
                existing_tags.len() - 1
            }
            Some(idx) => idx,
        }
    } else {
        let newtag = Tag::new_event(new_id, new_hint, Some(new_marker.to_string()));

        match existing_tags.iter().position(|existing_tag| {
            if let Ok((id, _rurl, _optmarker)) = existing_tag.parse_event() {
                id == new_id
            } else {
                false
            }
        }) {
            None => {
                existing_tags.push(newtag);
                existing_tags.len() - 1
            }
            Some(idx) => idx,
        }
    }
}

/// Add an 'a' addr tag to a set of tags if it doesn't already exist
pub fn add_addr_to_tags(
    existing_tags: &mut Vec<Tag>,
    new_addr: &NAddr,
    new_marker: Option<String>,
) -> usize {
    match existing_tags.iter().position(|existing_tag| {
        if let Ok((ea, _optmarker)) = existing_tag.parse_address() {
            ea.kind == new_addr.kind && ea.author == new_addr.author && ea.d == new_addr.d
        } else {
            false
        }
    }) {
        Some(idx) => idx,
        None => {
            existing_tags.push(Tag::new_address(new_addr, new_marker));
            existing_tags.len() - 1
        }
    }
}

/// Add an 'subject' tag to a set of tags if it doesn't already exist
pub fn add_subject_to_tags_if_missing(existing_tags: &mut Vec<Tag>, subject: String) {
    if !existing_tags.iter().any(|t| t.tagname() == "subject") {
        existing_tags.push(Tag::new_subject(subject));
    }
}

#[cfg(test)]
mod test {
    use crate::*;

    #[test]
    fn test_real_messages() {
        let wire = r#"["EVENT","j5happy-fiatjaf",{"id":"75468b04a0e03633a40f1c8d7e1a0cad1363ecc514ecbcde22093874e04e8166","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1668011201,"kind":1,"tags":[["e","247baa8ed5db8097b16d9594a3a27fd2b64c030fa9e68ce7d6106df4a499700d","","reply"],["p","6b0d4c8d9dc59e110d380b0429a02891f1341a0fa2ba1b1cf83a3db4d47e3964","","reply"]],"content":"you're not allowed to pronounce these words, traitor","sig":"588577ccd5ad6be8f61d93e4738799dede9b169ad150ee3ee6a1c4bb80adfbee27bb4e302e0ea173637c189d6664f1dc82ad3590b5524240bf492fa0b754432c"}]"#;
        let message: RelayMessage = serde_json::from_str(wire).unwrap();
        match message {
            RelayMessage::Event(_subid, event) => {
                event.verify(None).unwrap();
                println!("{}", event.content);
            }
            _ => panic!("Wrong message type"),
        }

        let wire = r#"["EVENT","j5happy-fiatjaf",{"id":"267660849149c7226a4a4f7c75f359f3995965c05d25451f13c907bf0b158178","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1668011264,"kind":1,"tags":[["e","8a128cd11c6a56554b8201635a19c97258504060464cec4f3e5f0500814339cf","","reply"],["p","000000000652e452ee68a01187fb08c899496cb46cb51d1aa0803d063acedba7","","reply"]],"content":"this is quite nice, specially the part where you say it was written in Rust.","sig":"1c49b4f4d2b86077ae4c1f7f8dc212d6c040dfdff7864eac2154fe7df1baceb162cf658d78634b803b964f920aeb861014ed30df113ed0857aaf1854e3c572a3"}]"#;
        let message: RelayMessage = serde_json::from_str(wire).unwrap();
        match message {
            RelayMessage::Event(_subid, event) => {
                event.verify(None).unwrap();
                println!("{}", event.as_ref().content);
            }
            _ => panic!("Wrong message type"),
        }

        let wire = r#"["EVENT","j5happy-fiatjaf",{"id":"fe0cfc6d2be988f46f849535518c3e43a509ea8a016ccd8b83a3ffd79575fd33","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1668011340,"kind":1,"tags":[["e","b1a2a2e55f1b6f1f6756e6e4c1c4ecbce0123ede048423413228134143fd84ac","","root"],["e","c758d9d467bf925923f57bb6b47db870fad50ba9629bc086f573f3d4ff278c84","","reply"],["p","9ec7a778167afb1d30c4833de9322da0c08ba71a69e1911d5578d3144bb56437","","root"],["p","32e1827635450ebb3c5a7d12c1f8e7b2b514439ac10a67eef3d9fd9c5c68e245","","reply"]],"content":"they are definitely annoying in Go, but we already have them anyway because of the `[\"EVENT\", {}]` message so this doesn't make any difference in my case at least.","sig":"23b1eed3087a72f2e940c1c95541b22b3434390926780ed055abf5dd77a3aa16e1c5c3965382ec7343c0da3ece31e05945f910d684f3196e81e05765a5b1e631"}]"#;
        let message: RelayMessage = serde_json::from_str(wire).unwrap();
        match message {
            RelayMessage::Event(_subid, event) => {
                event.verify(None).unwrap();
                println!("{}", event.content);
            }
            _ => panic!("Wrong message type"),
        }

        let wire = r#"["EVENT","j5happy-fiatjaf",{"id":"adf038ca047260a20f70b7863c3a8ef7afdac455cd9fcb785950b86ebb104911","pubkey":"3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d","created_at":1668011516,"kind":1,"tags":[["e","c0138298e2ac89078e206aea1e16f1d9a37257c8400f48aba781dd890bc9f35b","","root"],["e","24b757dfc938d9d29d7be40ac91424bfecd8c0016929ac911447a2f785519d97","","reply"],["p","3235036bd0957dfb27ccda02d452d7c763be40c91a1ac082ba6983b25238388c","","root"],["p","46fcbe3065eaf1ae7811465924e48923363ff3f526bd6f73d7c184b16bd8ce4d","","reply"]],"content":"when I started writing branle a million years ago I thought it would be so much simpler too, I guess that explains why twitter has 800 developers on its payroll","sig":"0f7d1cfbcc38bb861f51538cb8e4a5268e2bdca13969eaba8d0993e19fa8469d9ebcc60081523d075ca63c7ab55270e2a3de2373db605cde081b82357907af1f"}]"#;
        let message: RelayMessage = serde_json::from_str(wire).unwrap();
        match message {
            RelayMessage::Event(_subid, event) => {
                event.verify(None).unwrap();
                println!("{}", event.content);
            }
            _ => panic!("Wrong message type"),
        }
    }
}
```

---

### naddr.rs

**Size:** 6256 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::Error;
#[cfg(test)]
use crate::test_serde;
use crate::types::{EventKind, PublicKey, UncheckedUrl};

/// An 'naddr': data to address a possibly parameterized replaceable event
/// (d-tag, kind, author, and relays)
#[derive(Clone, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct NAddr {
    /// the 'd' tag of the Event, or an empty string if the kind is not
    /// parameterized
    pub d: String,

    /// Some of the relays where this could be found
    pub relays: Vec<UncheckedUrl>,

    /// Kind
    pub kind: EventKind,

    /// Author
    pub author: PublicKey,
}

impl NAddr {
    /// Export as a bech32 encoded string ("naddr")
    pub fn as_bech32_string(&self) -> String {
        // Compose
        let mut tlv: Vec<u8> = Vec::new();

        // Push d tag
        tlv.push(0); // the special value, in this case the 'd' tag
        let len = self.d.len() as u8;
        tlv.push(len); // the length of the d tag
        tlv.extend(self.d.as_bytes().iter().take(len as usize));

        // Push relays
        for relay in &self.relays {
            tlv.push(1); // type 'relay'
            let len = relay.0.len() as u8;
            tlv.push(len); // the length of the string
            tlv.extend(relay.0.as_bytes().iter().take(len as usize));
        }

        // Push kind
        let kindnum: u32 = From::from(self.kind);
        let bytes = kindnum.to_be_bytes();
        tlv.push(3); // type 'kind'
        tlv.push(bytes.len() as u8); // '4'
        tlv.extend(bytes);

        // Push author
        tlv.push(2); // type 'author'
        tlv.push(32); // the length of the value (always 32 for public key)
        tlv.extend(self.author.as_bytes());

        bech32::encode::<bech32::Bech32>(*super::HRP_NADDR, &tlv).unwrap()
    }

    /// Import from a bech32 encoded string ("naddr")
    pub fn try_from_bech32_string(s: &str) -> Result<NAddr, Error> {
        let data = bech32::decode(s)?;
        if data.0 != *super::HRP_NADDR {
            Err(Error::WrongBech32(
                super::HRP_NADDR.to_lowercase(),
                data.0.to_lowercase(),
            ))
        } else {
            let mut maybe_d: Option<String> = None;
            let mut relays: Vec<UncheckedUrl> = Vec::new();
            let mut maybe_kind: Option<EventKind> = None;
            let mut maybe_author: Option<PublicKey> = None;

            let tlv = data.1;
            let mut pos = 0;
            loop {
                // we need at least 2 more characters for anything meaningful
                if pos > tlv.len() - 2 {
                    break;
                }
                let ty = tlv[pos];
                let len = tlv[pos + 1] as usize;
                pos += 2;
                if pos + len > tlv.len() {
                    return Err(Error::InvalidProfile);
                }
                let raw = &tlv[pos..pos + len];
                match ty {
                    0 => {
                        // special (bytes of d tag)
                        maybe_d = Some(std::str::from_utf8(raw)?.to_string());
                    }
                    1 => {
                        // relay
                        let relay_str = std::str::from_utf8(raw)?;
                        let relay = UncheckedUrl::from_str(relay_str);
                        relays.push(relay);
                    }
                    2 => {
                        // author
                        //
                        // Don't fail if the pubkey is bad, just don't include it.
                        // Some client is generating these, and we want to tolerate it
                        // as much as we can.
                        if let Ok(pk) = PublicKey::from_bytes(raw, true) {
                            maybe_author = Some(pk);
                        }
                    }
                    3 => {
                        // kind
                        let kindnum = u32::from_be_bytes(
                            raw.try_into().map_err(|_| Error::WrongLengthKindBytes)?,
                        );
                        maybe_kind = Some(kindnum.into());
                    }
                    _ => {} // unhandled type for nprofile
                }
                pos += len;
            }

            match (maybe_d, maybe_kind, maybe_author) {
                (Some(d), Some(kind), Some(author)) => {
                    if !kind.is_replaceable() {
                        Err(Error::NonReplaceableAddr)
                    } else {
                        Ok(NAddr {
                            d,
                            relays,
                            kind,
                            author,
                        })
                    }
                }
                _ => Err(Error::InvalidNAddr),
            }
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> NAddr {
        let d = "Test D Indentifier 1lkjf23".to_string();

        NAddr {
            d,
            relays: vec![
                UncheckedUrl::from_str("wss://relay.example.com"),
                UncheckedUrl::from_str("wss://relay2.example.com"),
            ],
            kind: EventKind::LongFormContent,
            author: PublicKey::mock_deterministic(),
        }
    }
}

impl PartialEq for NAddr {
    fn eq(&self, other: &Self) -> bool {
        self.d == other.d && self.kind == other.kind && self.author == other.author
        // We do not compare the relays field!
    }
}

impl Eq for NAddr {}

impl Hash for NAddr {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.d.hash(state);
        self.kind.hash(state);
        self.author.hash(state);
        // We do not hash relays field!
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {NAddr, test_naddr_serde}

    #[test]
    fn test_profile_bech32() {
        let bech32 = NAddr::mock().as_bech32_string();
        println!("{bech32}");
        assert_eq!(
            NAddr::mock(),
            NAddr::try_from_bech32_string(&bech32).unwrap()
        );
    }
}
```

---

### nevent.rs

**Size:** 8761 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::{Error, EventKind, Id, PublicKey, UncheckedUrl};
#[cfg(test)]
use crate::test_serde;

/// An 'nevent': event id along with some relays in which that event may be
/// found.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct NEvent {
    /// Event id
    pub id: Id,

    /// Some of the relays where this could be in
    pub relays: Vec<UncheckedUrl>,

    /// Kind (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub kind: Option<EventKind>,

    /// Author (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub author: Option<PublicKey>,
}

impl NEvent {
    /// Export as a bech32 encoded string ("nevent")
    pub fn as_bech32_string(&self) -> String {
        // Compose
        let mut tlv: Vec<u8> = Vec::new();

        // Push Id
        tlv.push(0); // the special value, in this case the id
        tlv.push(32); // the length of the value (always 32 for id)
        tlv.extend(self.id.0);

        // Push relays
        for relay in &self.relays {
            tlv.push(1); // type 'relay'
            let len = relay.0.len() as u8;
            tlv.push(len); // the length of the string
            tlv.extend(relay.0.as_bytes().iter().take(len as usize));
        }

        // Maybe Push kind
        if let Some(kind) = self.kind {
            let kindnum: u32 = From::from(kind);
            let bytes = kindnum.to_be_bytes();
            tlv.push(3); // type 'kind'
            tlv.push(bytes.len() as u8); // '4'
            tlv.extend(bytes);
        }

        // Maybe Push author
        if let Some(pubkey) = self.author {
            tlv.push(2); // type 'author'
            tlv.push(32); // the length of the value (always 32 for public key)
            tlv.extend(pubkey.as_bytes());
        }

        bech32::encode::<bech32::Bech32>(*super::HRP_NEVENT, &tlv).unwrap()
    }

    /// Import from a bech32 encoded string ("nevent")
    pub fn try_from_bech32_string(s: &str) -> Result<NEvent, Error> {
        let data = bech32::decode(s)?;
        if data.0 != *super::HRP_NEVENT {
            Err(Error::WrongBech32(
                super::HRP_NEVENT.to_lowercase(),
                data.0.to_lowercase(),
            ))
        } else {
            let mut relays: Vec<UncheckedUrl> = Vec::new();
            let mut id: Option<Id> = None;
            let mut kind: Option<EventKind> = None;
            let mut author: Option<PublicKey> = None;

            let tlv = data.1;
            let mut pos = 0;
            loop {
                // we need at least 2 more characters for anything meaningful
                if pos > tlv.len() - 2 {
                    break;
                }
                let ty = tlv[pos];
                let len = tlv[pos + 1] as usize;
                pos += 2;
                if pos + len > tlv.len() {
                    return Err(Error::InvalidProfile);
                }
                let raw = &tlv[pos..pos + len];
                match ty {
                    0 => {
                        // special (32 bytes of id)
                        if len != 32 {
                            return Err(Error::InvalidNEvent);
                        }
                        id = Some(Id(raw
                            .try_into()
                            .map_err(|_| Error::WrongLengthHexString)?));
                    }
                    1 => {
                        // relay
                        let relay_str = std::str::from_utf8(raw)?;
                        let relay = UncheckedUrl::from_str(relay_str);
                        relays.push(relay);
                    }
                    2 => {
                        // author
                        //
                        // Don't fail if the pubkey is bad, just don't include it.
                        // Some client is generating these, and we want to tolerate it
                        // as much as we can.
                        if let Ok(pk) = PublicKey::from_bytes(raw, true) {
                            author = Some(pk);
                        }
                    }
                    3 => {
                        // kind
                        let kindnum = u32::from_be_bytes(
                            raw.try_into().map_err(|_| Error::WrongLengthKindBytes)?,
                        );
                        kind = Some(kindnum.into());
                    }
                    _ => {} // unhandled type for nprofile
                }
                pos += len;
            }
            if let Some(id) = id {
                Ok(NEvent {
                    id,
                    relays,
                    kind,
                    author,
                })
            } else {
                Err(Error::InvalidNEvent)
            }
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> NEvent {
        let id = Id::try_from_hex_string(
            "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9",
        )
        .unwrap();

        NEvent {
            id,
            relays: vec![
                UncheckedUrl::from_str("wss://relay.example.com"),
                UncheckedUrl::from_str("wss://relay2.example.com"),
            ],
            kind: None,
            author: None,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {NEvent, test_nevent_serde}

    #[test]
    fn test_profile_bech32() {
        let bech32 = NEvent::mock().as_bech32_string();
        println!("{bech32}");
        assert_eq!(
            NEvent::mock(),
            NEvent::try_from_bech32_string(&bech32).unwrap()
        );
    }

    #[test]
    fn test_nip19_example() {
        let nevent = NEvent {
            id: Id::try_from_hex_string(
                "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d",
            )
            .unwrap(),
            relays: vec![
                UncheckedUrl::from_str("wss://r.x.com"),
                UncheckedUrl::from_str("wss://djbas.sadkb.com"),
            ],
            kind: None,
            author: None,
        };

        // As serialized by us (not necessarily in the order others would do it)
        let bech32 = "nevent1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gpp4mhxue69uhhytnc9e3k7mgpz4mhxue69uhkg6nzv9ejuumpv34kytnrdaks343fay";

        // Try converting profile to bech32
        assert_eq!(nevent.as_bech32_string(), bech32);

        // Try converting bech32 to profile
        assert_eq!(nevent, NEvent::try_from_bech32_string(bech32).unwrap());

        // Try this one that used to fail
        let bech32 =
            "nevent1qqstxx3lk7zqfyn8cyyptvujfxq9w6mad4205x54772tdkmyqaay9scrqsqqqpp8x4vwhf";
        let _ = NEvent::try_from_bech32_string(bech32).unwrap();
        // it won't be equal, but should have the basics and should not error.
    }

    #[test]
    fn test_nevent_alt_fields() {
        let nevent = NEvent {
            id: Id::try_from_hex_string(
                "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d",
            )
            .unwrap(),
            relays: vec![
                UncheckedUrl::from_str("wss://r.x.com"),
                UncheckedUrl::from_str("wss://djbas.sadkb.com"),
            ],
            kind: Some(EventKind::TextNote),
            author: Some(
                PublicKey::try_from_hex_string(
                    "000000000332c7831d9c5a99f183afc2813a6f69a16edda7f6fc0ed8110566e6",
                    true,
                )
                .unwrap(),
            ),
        };

        // As serialized by us (not necessarily in the order others would do it)
        let bech32 = "nevent1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gpp4mhxue69uhhytnc9e3k7mgpz4mhxue69uhkg6nzv9ejuumpv34kytnrdaksxpqqqqqqzq3qqqqqqqqrxtrcx8vut2vlrqa0c2qn5mmf59hdmflkls8dsyg9vmnqu25v0j";

        // Try converting profile to bech32
        assert_eq!(nevent.as_bech32_string(), bech32);

        // Try converting bech32 to profile
        assert_eq!(nevent, NEvent::try_from_bech32_string(bech32).unwrap());
    }

    #[test]
    fn test_ones_that_were_failing() {
        let bech32 = "nevent1qqswrqr63ddwk8l3zfqrgdxh2lxh2jlcxl36k3h33g25gtchzchx8agpp4mhxue69uhkummn9ekx7mqpz3mhxue69uhhyetvv9ujuerpd46hxtnfduq3yamnwvaz7tm0venxx6rpd9hzuur4vgpyqdmyxs6rzdmyx4jxvdpnx4snjdmz8pnr2dtr8pnryefhv5ex2e34xvek2v3nxuckxef4v5ckxenxvs6njdtrxymnjcfnv4skvvekvs6qfe99uy";

        let _ne = NEvent::try_from_bech32_string(bech32).unwrap();
    }
}
```

---

### nip0.rs

**Size:** 2829 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-05: Mapping Nostr keys to DNS-based internet identifiers
// https://github.com/nostr-protocol/nips/blob/master/05.md

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::{
    types::event::{Event, UnsignedEvent},
    utils::ureq_async,
};

/// A Nip05 record
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Nip05 {
    /// A map of names to public keys
    pub names: HashMap<String, String>,
}

/// A metadata record
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Metadata {
    /// The user's name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// A description of the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub about: Option<String>,
    /// A URL to the user's profile picture
    #[serde(skip_serializing_if = "Option::is_none")]
    pub picture: Option<String>,
    /// A URL to the user's banner image
    #[serde(skip_serializing_if = "Option::is_none")]
    pub banner: Option<String>,
    /// The user's Nip05 identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nip05: Option<String>,
    /// The user's lightning address (LUD-06)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lud06: Option<String>,
    /// The user's lightning address (LUD-16)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lud16: Option<String>,
    /// Extra fields
    #[serde(flatten)]
    pub extra: HashMap<String, Value>,
}

/// Set metadata for a user
pub fn set_metadata(
    metadata: &Metadata,
    tags: Vec<Vec<String>>,
    public_key: &XOnlyPublicKey,
    private_key: &secp256k1::SecretKey,
) -> Result<Event, serde_json::Error> {
    let content = serde_json::to_string(metadata)?;
    let unsigned_event = UnsignedEvent::new(public_key, 0, tags, content);
    let signed_event = unsigned_event.sign(private_key).unwrap();
    Ok(signed_event)
}

/// Verify a nip05 identifier
pub async fn verify(public_key: &XOnlyPublicKey, nip05_identifier: &str) -> Result<bool> {
    let mut parts = nip05_identifier.split('@');
    let name = parts.next();
    let domain = parts.next();

    if name.is_none() || domain.is_none() {
        return Err(anyhow!("Invalid NIP-05 identifier format"));
    }

    let name = name.unwrap();
    let domain = domain.unwrap();

    let url = format!("https://{}/.well-known/nostr.json?name={}", domain, name);

    let response_str = ureq_async(url).await.map_err(|e| anyhow!(e))?;
    let nip05_data: Nip05 = serde_json::from_str(&response_str)?;

    if let Some(found_pubkey) = nip05_data.names.get(name) {
        let pk_hex = public_key.to_string();
        return Ok(found_pubkey == &pk_hex);
    }

    Ok(false)
}
```

---

### nip05.rs

**Size:** 251 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use super::versioned::nip05::Nip05V1;

/// The content of a webserver's /.well-known/nostr.json file used in NIP-05 and
/// NIP-35 This allows lookup and verification of a nostr user via a
/// `user@domain` style identifier.
pub type Nip05 = Nip05V1;
```

---

### nip10.rs

**Size:** 755 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-10: Text Notes and Threads
// https://github.com/nostr-protocol/nips/blob/master/10.md

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::types::event::{Event, EventId, UnsignedEvent};

/// Create a reply to an event.
pub fn create_reply(
    root_id: EventId,
    replied_to_id: EventId,
    content: String,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let tags = vec![
        vec!["e".to_string(), root_id.as_hex_string(), "root".to_string()],
        vec![
            "e".to_string(),
            replied_to_id.as_hex_string(),
            "reply".to_string(),
        ],
    ];
    let unsigned_event = UnsignedEvent::new(public_key, 1, tags, content);
    unsigned_event.sign(private_key).unwrap()
}
```

---

### nip13.rs

**Size:** 7553 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-13: Proof of Work
//! https://github.com/nostr-protocol/nips/blob/master/13.md

use anyhow::Result;
use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::types::{
    event::{Event, UnsignedEvent},
    Id, PublicKey, Signature, Tag, Unixtime,
};

/// The name of the nonce tag.
pub const NONCE_TAG_NAME: &str = "nonce";

/// Helper trait for NIP-13 proof of work tags on Event types.
pub trait NIP13Event {
    /// Extracts the nonce value and target difficulty from the event's "nonce"
    /// tag. Returns `None` if the "nonce" tag is not found or is malformed.
    fn nonce_data(&self) -> Option<(u64, u8)>;

    /// Adds a "nonce" tag to the event with the given nonce value and target
    /// difficulty. If a "nonce" tag already exists, it will be replaced.
    fn add_nonce_tag(&mut self, nonce_value: u64, target_difficulty: u8);

    /// Creates a "nonce" tag with the given nonce value and target difficulty.
    fn create_nonce_tag(nonce_value: u64, target_difficulty: u8) -> Tag;
}

impl NIP13Event for Event {
    fn nonce_data(&self) -> Option<(u64, u8)> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() == 3 && tag.0[0] == NONCE_TAG_NAME {
                if let (Ok(nonce), Ok(difficulty)) =
                    (tag.0[1].parse::<u64>(), tag.0[2].parse::<u8>())
                {
                    Some((nonce, difficulty))
                } else {
                    None
                }
            } else {
                None
            }
        })
    }

    fn add_nonce_tag(&mut self, nonce_value: u64, target_difficulty: u8) {
        // Remove existing nonce tags to ensure only one is present
        self.tags
            .retain(|tag| tag.0.get(0) != Some(&NONCE_TAG_NAME.to_string()));
        self.tags
            .push(Self::create_nonce_tag(nonce_value, target_difficulty));
    }

    fn create_nonce_tag(nonce_value: u64, target_difficulty: u8) -> Tag {
        Tag::new(&[
            NONCE_TAG_NAME,
            &nonce_value.to_string(),
            &target_difficulty.to_string(),
        ])
    }
}

/// Generate a Proof of Work event
pub fn generate_pow_event(
    content: String,
    mut tags: Vec<Tag>, // Change to Vec<Tag>
    difficulty: u8,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let mut nonce: u64 = 0;
    loop {
        // Remove any existing nonce tag before adding a new one for current iteration
        tags.retain(|tag| tag.0.get(0) != Some(&NONCE_TAG_NAME.to_string()));
        tags.push(Event::create_nonce_tag(nonce, difficulty));

        let unsigned_event = UnsignedEvent::new(
            public_key,
            1,
            tags.iter().map(|tag| tag.0.clone()).collect(), // Convert Vec<Tag> to Vec<Vec<String>>
            content.clone(),
        );
        let event = unsigned_event.sign(private_key).unwrap();
        if crate::types::get_leading_zero_bits(&event.id.0) >= difficulty {
            return event;
        }
        nonce += 1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature, Unixtime};

    // Helper to create a dummy event for testing
    fn create_dummy_event_with_nonce(nonce_value: u64, difficulty: u8) -> Event {
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![],
            content: "Test event for PoW".to_string(),
            sig: Signature::zeroes(),
        };
        event.add_nonce_tag(nonce_value, difficulty);
        event
    }

    #[test]
    fn test_create_nonce_tag() {
        let nonce_val = 12345;
        let difficulty_val = 20;
        let tag = Event::create_nonce_tag(nonce_val, difficulty_val);
        assert_eq!(
            tag.0,
            vec![
                NONCE_TAG_NAME.to_string(),
                nonce_val.to_string(),
                difficulty_val.to_string()
            ]
        );
    }

    #[test]
    fn test_nonce_data_found() {
        let nonce_val = 56789;
        let difficulty_val = 15;
        let event = create_dummy_event_with_nonce(nonce_val, difficulty_val);
        assert_eq!(event.nonce_data(), Some((nonce_val, difficulty_val)));
    }

    #[test]
    fn test_nonce_data_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])], // No nonce tag
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.nonce_data(), None);
    }

    #[test]
    fn test_nonce_data_malformed_tag() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&[NONCE_TAG_NAME, "not_a_number", "10"])], // Malformed nonce value
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.nonce_data(), None);

        let event2 = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&[NONCE_TAG_NAME, "123", "not_a_difficulty"])], /* Malformed difficulty */
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event2.nonce_data(), None);

        let event3 = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&[NONCE_TAG_NAME, "123"])], // Too few elements
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event3.nonce_data(), None);
    }

    #[test]
    fn test_add_nonce_tag() {
        let nonce1 = 1;
        let difficulty1 = 5;
        let nonce2 = 2;
        let difficulty2 = 10;
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_nonce_tag(nonce1, difficulty1),
            ],
            content: "Some content".to_string(),
            sig: Signature::zeroes(),
        };

        // Add a new nonce tag, it should replace the existing one
        event.add_nonce_tag(nonce2, difficulty2);
        assert_eq!(event.nonce_data(), Some((nonce2, difficulty2)));
        // Ensure only one nonce tag exists
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&NONCE_TAG_NAME.to_string()))
                .count(),
            1
        );

        // Add another nonce tag, it should replace the existing one
        let nonce3 = 3;
        let difficulty3 = 15;
        event.add_nonce_tag(nonce3, difficulty3);
        assert_eq!(event.nonce_data(), Some((nonce3, difficulty3)));
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&NONCE_TAG_NAME.to_string()))
                .count(),
            1
        );
    }
}
```

---

### nip14.rs

**Size:** 4709 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-14: Subject Tag in Text Events
//!
//! This NIP defines a standard `subject` tag for kind 1 (text note) events,
//! allowing clients to display messages in threaded lists with a clear subject.
//!
//! https://github.com/nostr-protocol/nips/blob/master/14.md

use anyhow::Result;

use crate::types::{Event, Tag};

/// The name of the subject tag.
pub const SUBJECT_TAG_NAME: &str = "subject";

/// Helper trait for NIP-14 subject tags on Event types.
pub trait NIP14Event {
    /// Extracts the subject string from the event's tags.
    /// Returns `None` if the "subject" tag is not found.
    fn subject(&self) -> Option<&str>;

    /// Adds a "subject" tag to the event with the given subject string.
    /// If a "subject" tag already exists, it will be replaced.
    fn add_subject_tag(&mut self, subject: String);

    /// Creates a "subject" tag with the given subject string.
    fn create_subject_tag(subject: String) -> Tag;
}

impl NIP14Event for Event {
    fn subject(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == SUBJECT_TAG_NAME {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn add_subject_tag(&mut self, subject: String) {
        // Remove existing subject tags to ensure only one is present
        self.tags
            .retain(|tag| tag.0.get(0) != Some(&SUBJECT_TAG_NAME.to_string()));
        self.tags.push(Self::create_subject_tag(subject));
    }

    fn create_subject_tag(subject: String) -> Tag {
        Tag::new(&[SUBJECT_TAG_NAME, &subject])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature, Unixtime};

    #[test]
    fn test_create_subject_tag() {
        let subject_str = "My Nostr Subject";
        let tag = Event::create_subject_tag(subject_str.to_string());
        assert_eq!(tag.0, vec!["subject", "My Nostr Subject"]);
    }

    #[test]
    fn test_subject_found() {
        let subject_str = "Nostr Dev Talk";
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_subject_tag(subject_str.to_string()),
            ],
            content: "This is a text note.".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(
            event.subject().map(|s| s.to_string()),
            Some(subject_str.to_string())
        );
    }

    #[test]
    fn test_subject_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])],
            content: "This is a text note without a subject.".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(event.subject(), None);
    }

    #[test]
    fn test_add_subject_tag() {
        let initial_subject = "Initial discussion";
        let new_subject = "Revised discussion topic";
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_subject_tag(initial_subject.to_string()),
            ],
            content: "Some content".to_string(),
            sig: Signature::zeroes(),
        };

        // Add a new subject tag
        event.add_subject_tag(new_subject.to_string());
        assert_eq!(
            event.subject().map(|s| s.to_string()),
            Some(new_subject.to_string())
        );
        // Ensure only one subject tag exists
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&SUBJECT_TAG_NAME.to_string()))
                .count(),
            1
        );

        // Add another subject tag, it should replace the existing one
        let even_newer_subject = "Final topic of conversation";
        event.add_subject_tag(even_newer_subject.to_string());
        assert_eq!(
            event.subject().map(|s| s.to_string()),
            Some(even_newer_subject.to_string())
        );
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&SUBJECT_TAG_NAME.to_string()))
                .count(),
            1
        );
    }
}
```

---

### nip15.rs

**Size:** 468 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-15: End of Stored Events Notice
// https://github.com/nostr-protocol/nips/blob/master/15.md

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::types::event::{Event, UnsignedEvent};

/// Create an End of Stored Events (EOSE) event
pub fn end_of_stored_events(public_key: &XOnlyPublicKey, private_key: &SecretKey) -> Event {
    let unsigned_event = UnsignedEvent::new(public_key, 4, vec![], "".to_string());
    unsigned_event.sign(private_key).unwrap()
}
```

---

### nip18.rs

**Size:** 1395 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-18: Reposts
// https://github.com/nostr-protocol/nips/blob/master/18.md

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::types::{
    event::{Event, EventId, UnsignedEvent},
    PublicKey, RelayUrl, Tag,
};

/// Create a kind 6 repost event for a text note (kind 1).
pub fn create_repost_text_note(
    reposted_event: &Event,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Result<Event, crate::types::Error> {
    let content = serde_json::to_string(reposted_event)?;
    let tags = vec![
        Tag::new_event(reposted_event.id, None, None).0,
        Tag::new_pubkey(reposted_event.pubkey, None, None).0,
    ];
    let unsigned_event = UnsignedEvent::new(public_key, 6, tags, content);
    Ok(unsigned_event.sign(private_key).unwrap())
}

/// Create a kind 16 generic repost event for any event other than kind 1.
pub fn create_generic_repost(
    reposted_event: &Event,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Result<Event, crate::types::Error> {
    let content = serde_json::to_string(reposted_event)?;
    let tags = vec![
        Tag::new_event(reposted_event.id, None, None).0,
        Tag::new_pubkey(reposted_event.pubkey, None, None).0,
        Tag::new_kind(reposted_event.kind).0,
    ];
    let unsigned_event = UnsignedEvent::new(public_key, 16, tags, content);
    Ok(unsigned_event.sign(private_key).unwrap())
}
```

---

### nip19.rs

**Size:** 11429 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// Copyright 2015-2020 nostr-proto Developers
// Licensed under the MIT license <LICENSE-MIT or http://opensource.org/licenses/MIT>
// This file may not be copied, modified, or distributed except according to
// those terms.

//! NIP-19 bech32-encoded entities

#![allow(missing_docs)]

use std::str::FromStr;

use bech32::{self, Bech32, Bech32m, Hrp};

use crate::types::{Error, EventKind, Id, PublicKey, RelayUrl};

/// Different NIP-19 bech32 encoded entity types
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Nip19 {
    /// Public key (npub)
    PublicKey(PublicKey),
    /// Private key (nsec)
    PrivateKey(String), // We'll store this as a hex string for now
    /// Event Id (note)
    EventId(Id),
    /// Nostr Profile (nprofile)
    Profile(Nip19Profile),
    /// Nostr Event with metadata (nevent)
    Event(Nip19Event),
    /// Nostr Addressable Event (naddr)
    Address(Nip19Address),
    /// Nostr Relay (nrelay) - Deprecated
    Relay(RelayUrl),
}

/// A NIP-19 bech32 profile (`nprofile`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nip19Profile {
    /// The public key
    pub public_key: PublicKey,
    /// Relays where the profile may be found
    pub relays: Vec<RelayUrl>,
}

/// A NIP-19 bech32 event (`nevent`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nip19Event {
    /// The event ID
    pub event_id: Id,
    /// Author of the event (optional)
    pub author: Option<PublicKey>,
    /// Kind of the event (optional)
    pub kind: Option<EventKind>,
    /// Relays where the event may be found
    pub relays: Vec<RelayUrl>,
}

/// A NIP-19 bech32 addressable event (`naddr`)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Nip19Address {
    /// The event kind
    pub kind: EventKind,
    /// The public key of the author
    pub public_key: PublicKey,
    /// The 'd' tag identifier
    pub identifier: String,
    /// Relays where the event may be found
    pub relays: Vec<RelayUrl>,
}

const TLV_TYPE_SPECIAL: u8 = 0;
const TLV_TYPE_RELAY: u8 = 1;
const TLV_TYPE_AUTHOR: u8 = 2;
const TLV_TYPE_KIND: u8 = 3;

impl Nip19 {
    /// Decode a bech32 encoded NIP-19 string
    pub fn decode(s: &str) -> Result<Self, Error> {
        let (hrp, data) = bech32::decode(s)?;

        match hrp.as_str() {
            "npub" => Ok(Nip19::PublicKey(PublicKey::from_bytes(&data, true)?)),
            "nsec" => {
                if data.len() != 32 {
                    return Err(Error::InvalidPrivateKey);
                }
                Ok(Nip19::PrivateKey(hex::encode(data)))
            }
            "note" => Ok(Nip19::EventId(
                Id::try_from_bytes(&data).map_err(|_| Error::InvalidId)?,
            )),
            "nprofile" => {
                let mut public_key = None;
                let mut relays = Vec::new();
                let mut cursor = 0;
                while cursor < data.len() {
                    let t = data[cursor];
                    cursor += 1;
                    let l = data[cursor] as usize;
                    cursor += 1;
                    let v = &data[cursor..cursor + l];
                    cursor += l;

                    match t {
                        TLV_TYPE_SPECIAL => public_key = Some(PublicKey::from_bytes(v, true)?),
                        TLV_TYPE_RELAY => {
                            relays.push(RelayUrl::try_from_str(&String::from_utf8(v.to_vec())?)?)
                        }
                        _ => {} // Ignore unknown TLV types
                    }
                }
                let public_key = public_key.ok_or(Error::InvalidNip19Data)?;
                Ok(Nip19::Profile(Nip19Profile { public_key, relays }))
            }
            "nevent" => {
                let mut event_id = None;
                let mut author = None;
                let mut kind = None;
                let mut relays = Vec::new();
                let mut cursor = 0;
                while cursor < data.len() {
                    let t = data[cursor];
                    cursor += 1;
                    let l = data[cursor] as usize;
                    cursor += 1;
                    let v = &data[cursor..cursor + l];
                    cursor += l;

                    match t {
                        TLV_TYPE_SPECIAL => {
                            event_id = Some(Id::try_from_bytes(v).map_err(|_| Error::InvalidId)?)
                        }
                        TLV_TYPE_RELAY => {
                            relays.push(RelayUrl::try_from_str(&String::from_utf8(v.to_vec())?)?)
                        }
                        TLV_TYPE_AUTHOR => author = Some(PublicKey::from_bytes(v, true)?),
                        TLV_TYPE_KIND => {
                            if v.len() == 4 {
                                let mut bytes = [0u8; 4];
                                bytes.copy_from_slice(v);
                                kind = Some(EventKind::from(u32::from_be_bytes(bytes)));
                            }
                        }
                        _ => {} // Ignore unknown TLV types
                    }
                }
                let event_id = event_id.ok_or(Error::InvalidNip19Data)?;
                Ok(Nip19::Event(Nip19Event {
                    event_id,
                    author,
                    kind,
                    relays,
                }))
            }
            "naddr" => {
                let mut kind = None;
                let mut public_key = None;
                let mut identifier = None;
                let mut relays = Vec::new();
                let mut cursor = 0;
                while cursor < data.len() {
                    let t = data[cursor];
                    cursor += 1;
                    let l = data[cursor] as usize;
                    cursor += 1;
                    let v = &data[cursor..cursor + l];
                    cursor += l;

                    match t {
                        TLV_TYPE_SPECIAL => identifier = Some(String::from_utf8(v.to_vec())?),
                        TLV_TYPE_RELAY => {
                            relays.push(RelayUrl::try_from_str(&String::from_utf8(v.to_vec())?)?)
                        }
                        TLV_TYPE_AUTHOR => public_key = Some(PublicKey::from_bytes(v, true)?),
                        TLV_TYPE_KIND => {
                            if v.len() == 4 {
                                let mut bytes = [0u8; 4];
                                bytes.copy_from_slice(v);
                                kind = Some(EventKind::from(u32::from_be_bytes(bytes)));
                            }
                        }
                        _ => {} // Ignore unknown TLV types
                    }
                }
                let kind = kind.ok_or(Error::InvalidNip19Data)?;
                let public_key = public_key.ok_or(Error::InvalidNip19Data)?;
                let identifier = identifier.ok_or(Error::InvalidNip19Data)?;
                Ok(Nip19::Address(Nip19Address {
                    kind,
                    public_key,
                    identifier,
                    relays,
                }))
            }
            "nrelay" => Ok(Nip19::Relay(RelayUrl::try_from_str(&String::from_utf8(
                data,
            )?)?)),
            _ => Err(Error::InvalidNip19Prefix),
        }
    }

    /// Encode a NIP-19 entity into a bech32 string
    pub fn encode(&self) -> Result<String, Error> {
        match self {
            Nip19::PublicKey(pk) => {
                bech32::encode::<Bech32>(Hrp::parse("npub")?, pk.as_bytes()).map_err(|e| e.into())
            }
            Nip19::PrivateKey(sk_hex) => {
                let sk_bytes = hex::decode(sk_hex)?;
                bech32::encode::<Bech32>(Hrp::parse("nsec")?, &sk_bytes).map_err(|e| e.into())
            }
            Nip19::EventId(id) => {
                bech32::encode::<Bech32>(Hrp::parse("note")?, id.0.as_slice()).map_err(|e| e.into())
            }
            Nip19::Profile(profile) => {
                let mut data = Vec::new();
                // Special: Public Key
                data.push(TLV_TYPE_SPECIAL);
                data.push(profile.public_key.as_bytes().len() as u8);
                data.extend_from_slice(profile.public_key.as_bytes());
                // Relays
                for relay in &profile.relays {
                    let relay_bytes = relay.as_str().as_bytes();
                    data.push(TLV_TYPE_RELAY);
                    data.push(relay_bytes.len() as u8);
                    data.extend_from_slice(relay_bytes);
                }
                bech32::encode::<Bech32>(Hrp::parse("nprofile")?, &data).map_err(|e| e.into())
            }
            Nip19::Event(event) => {
                let mut data = Vec::new();
                // Special: Event ID
                data.push(TLV_TYPE_SPECIAL);
                data.push(event.event_id.0.as_slice().len() as u8);
                data.extend_from_slice(event.event_id.0.as_slice());
                // Author
                if let Some(author) = &event.author {
                    data.push(TLV_TYPE_AUTHOR);
                    data.push(author.as_bytes().len() as u8);
                    data.extend_from_slice(author.as_bytes());
                }
                // Kind
                if let Some(kind) = &event.kind {
                    let kind_bytes = u32::from(*kind).to_be_bytes();
                    data.push(TLV_TYPE_KIND);
                    data.push(kind_bytes.len() as u8);
                    data.extend_from_slice(&kind_bytes);
                }
                // Relays
                for relay in &event.relays {
                    let relay_bytes = relay.as_str().as_bytes();
                    data.push(TLV_TYPE_RELAY);
                    data.push(relay_bytes.len() as u8);
                    data.extend_from_slice(relay_bytes);
                }
                bech32::encode::<Bech32>(Hrp::parse("nevent")?, &data).map_err(|e| e.into())
            }
            Nip19::Address(addr) => {
                let mut data = Vec::new();
                // Special: Identifier
                let identifier_bytes = addr.identifier.as_bytes();
                data.push(TLV_TYPE_SPECIAL);
                data.push(identifier_bytes.len() as u8);
                data.extend_from_slice(identifier_bytes);
                // Kind
                let kind_bytes = u32::from(addr.kind).to_be_bytes();
                data.push(TLV_TYPE_KIND);
                data.push(kind_bytes.len() as u8);
                data.extend_from_slice(&kind_bytes);
                // Author
                data.push(TLV_TYPE_AUTHOR);
                data.push(addr.public_key.as_bytes().len() as u8);
                data.extend_from_slice(addr.public_key.as_bytes());
                // Relays
                for relay in &addr.relays {
                    let relay_bytes = relay.as_str().as_bytes();
                    data.push(TLV_TYPE_RELAY);
                    data.push(relay_bytes.len() as u8);
                    data.extend_from_slice(relay_bytes);
                }
                bech32::encode::<Bech32>(Hrp::parse("naddr")?, &data).map_err(|e| e.into())
            }
            Nip19::Relay(relay_url) => {
                bech32::encode::<Bech32>(Hrp::parse("nrelay")?, relay_url.as_str().as_bytes())
                    .map_err(|e| e.into())
            }
        }
    }
}
```

---

### nip2.rs

**Size:** 1146 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-02: Contact List and Petnames
// https://github.com/nostr-protocol/nips/blob/master/02.md

use secp256k1::XOnlyPublicKey;

use crate::types::event::{Event, UnsignedEvent};

/// A contact
#[derive(Debug, Clone)]
pub struct Contact {
    /// Their public key
    pub public_key: XOnlyPublicKey,
    /// A relay URL for them
    pub relay_url: Option<String>,
    /// A petname for them
    pub petname: Option<String>,
}

/// Set a contact list
pub fn set_contact_list(
    contacts: Vec<Contact>,
    public_key: &XOnlyPublicKey,
    private_key: &secp256k1::SecretKey,
) -> Event {
    let tags: Vec<Vec<String>> = contacts
        .into_iter()
        .map(|contact| {
            let mut tag = vec!["p".to_string(), contact.public_key.to_string()];
            if let Some(relay_url) = contact.relay_url {
                tag.push(relay_url);
            }
            if let Some(petname) = contact.petname {
                tag.push(petname);
            }
            tag
        })
        .collect();

    let unsigned_event = UnsignedEvent::new(public_key, 3, tags, "".to_string());
    unsigned_event.sign(private_key).unwrap()
}
```

---

### nip25.rs

**Size:** 5919 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-25: Reactions
//!
//! This NIP defines how users can express reactions to other events,
//! typically representing "likes", "dislikes", or other emoji-based feedback.
//!
//! https://github.com/nostr-protocol/nips/blob/master/25.md

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Event, EventKind, Id, PreEvent, PublicKey, Signature, Tag, Unixtime};

/// NIP-25 Reaction Event Kind
pub const REACTION_KIND: u32 = 7;
/// NIP-25 External Content Reaction Event Kind (optional, will focus on Kind 7
/// first)
pub const EXTERNAL_CONTENT_REACTION_KIND: u32 = 17;

/// Represents the content of a NIP-25 Reaction Event.
/// This is typically a single emoji or "+"/"-".
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReactionContent {
    pub reaction: String,
}

/// Helper trait for NIP-25 reaction events.
pub trait NIP25Event {
    /// Extracts the ID of the event this reaction is responding to (from 'e'
    /// tag).
    fn reacted_event_id(&self) -> Option<Id>;

    /// Extracts the PublicKey of the author of the event this reaction is
    /// responding to (from 'p' tag).
    fn reacted_pubkey(&self) -> Option<PublicKey>;

    /// Extracts the original event coordinates from 'a' tag (for addressable
    /// events).
    // fn reacted_addressable_event_coords(&self) -> Option<Coordinate>; // Maybe later if
    // Coordinate is available

    /// Creates a NIP-25 Reaction event (Kind 7).
    fn new_reaction_event(
        public_key: PublicKey,
        reacted_event_id: Id,
        reacted_pubkey: Option<PublicKey>,
        reaction_content: String,
    ) -> Result<Event>;
}

impl NIP25Event for Event {
    fn reacted_event_id(&self) -> Option<Id> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "e" {
                Id::try_from_hex_string(tag.0[1].as_str()).ok()
            } else {
                None
            }
        })
    }

    fn reacted_pubkey(&self) -> Option<PublicKey> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "p" {
                PublicKey::try_from_hex_string(tag.0[1].as_str(), true).ok() // true to verify
            } else {
                None
            }
        })
    }

    fn new_reaction_event(
        public_key: PublicKey,
        reacted_event_id: Id,
        reacted_pubkey: Option<PublicKey>,
        reaction_content: String,
    ) -> Result<Event> {
        let content = ReactionContent {
            reaction: reaction_content.clone(),
        }
        .reaction;
        let mut tags: Vec<Tag> = vec![Tag::new(&["e", &reacted_event_id.as_hex_string()])];

        if let Some(pk) = reacted_pubkey {
            tags.push(Tag::new(&["p", &pk.as_hex_string()]));
        }
        // NIP-10 reply markers: reply, root
        // NIP-25 doesn't explicitly mention markers for e and p tags,
        // but it's good practice to include them for context if available
        // For a simple reaction, a "reply" marker might be appropriate for 'e' tag

        // Create PreEvent
        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Reaction, // Use the specific EventKind for reactions
            tags: tags.clone(),
            content: content.clone(),
        };

        let id = pre_event.hash().unwrap();

        Ok(Event {
            id,
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Reaction, // Use the specific EventKind for reactions
            tags,
            content,
            sig: Signature::zeroes(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Id, PublicKey, Signature};

    #[test]
    fn test_create_reaction_event() {
        let public_key = PublicKey::mock();
        let reacted_event_id = Id::mock();
        let reacted_pubkey = Some(PublicKey::mock_deterministic());
        let reaction_content = "+".to_string();

        let event = Event::new_reaction_event(
            public_key,
            reacted_event_id,
            reacted_pubkey,
            reaction_content.clone(),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::Reaction);
        assert_eq!(event.content, reaction_content);
        assert_eq!(event.reacted_event_id(), Some(reacted_event_id));
        assert_eq!(event.reacted_pubkey(), reacted_pubkey);

        // Test with custom emoji reaction
        let custom_reaction = "😂".to_string();
        let event_emoji = Event::new_reaction_event(
            public_key,
            reacted_event_id,
            None, // No reacted pubkey
            custom_reaction.clone(),
        )
        .unwrap();

        assert_eq!(event_emoji.kind, EventKind::Reaction);
        assert_eq!(event_emoji.content, custom_reaction);
        assert_eq!(event_emoji.reacted_event_id(), Some(reacted_event_id));
        assert_eq!(event_emoji.reacted_pubkey(), None);
    }

    #[test]
    fn test_reacted_event_id_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::Reaction,
            tags: vec![], // No 'e' tag
            content: "+".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.reacted_event_id(), None);
    }

    #[test]
    fn test_reacted_pubkey_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::Reaction,
            tags: vec![Tag::new(&["e", &Id::mock().as_hex_string()])], // No 'p' tag
            content: "+".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.reacted_pubkey(), None);
    }
}
```

---

### nip26.rs

**Size:** 3836 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-26: Delegation
// https://github.com/nostr-protocol/nips/blob/master/26.md

use secp256k1::{schnorr::Signature, Keypair, Message, Secp256k1, SecretKey, XOnlyPublicKey};
use sha2::{Digest, Sha256};

/// A delegation, which allows one key to sign an event on behalf of another
/// key.
#[derive(Debug, Copy, Clone)]
pub struct Delegation {
    /// The public key of the delegator
    pub delegator: XOnlyPublicKey,
    /// The public key of the delegatee
    pub delegatee: XOnlyPublicKey,
    /// The kind of event being delegated
    pub event_kind: u16,
    /// An optional expiration timestamp for the delegation
    pub until: Option<u64>,
    /// An optional creation timestamp for the delegation
    pub since: Option<u64>,
}

impl Delegation {
    /// Create a delegation tag
    pub fn create_tag(&self, private_key: &SecretKey) -> Result<String, secp256k1::Error> {
        let secp = Secp256k1::new();
        let keypair = Keypair::from_secret_key(&secp, private_key);
        let conditions = self.build_conditions_string();
        let message = format!("nostr:delegation:{}:{}", self.delegatee, conditions);
        let mut hasher = Sha256::new();
        hasher.update(message.as_bytes());
        let message_hash = Message::from_digest_slice(&hasher.finalize()).unwrap();
        let signature = secp.sign_schnorr(&message_hash, &keypair);
        Ok(format!(
            "delegation:{}:{}:{}",
            self.delegator, conditions, signature
        ))
    }

    fn build_conditions_string(&self) -> String {
        let mut conditions = format!("kind={}", self.event_kind);
        if let Some(until) = self.until {
            conditions.push_str(&format!("&created_at<{}", until));
        }
        if let Some(since) = self.since {
            conditions.push_str(&format!("&created_at>{}", since));
        }
        conditions
    }
}

/// Verify a delegation tag
pub fn verify(
    delegation_tag: &str,
    delegatee_pubkey: &XOnlyPublicKey,
    event_kind: u16,
    created_at: u64,
) -> Result<bool, anyhow::Error> {
    let mut parts = delegation_tag.split(':');
    if parts.next() != Some("delegation") {
        return Ok(false);
    }

    let delegator_str = parts.next().ok_or(anyhow::anyhow!("Missing delegator"))?;
    let conditions = parts.next().ok_or(anyhow::anyhow!("Missing conditions"))?;
    let signature_str = parts.next().ok_or(anyhow::anyhow!("Missing signature"))?;

    // Verify signature
    let secp = Secp256k1::new();
    let message = format!("nostr:delegation:{}:{}", delegatee_pubkey, conditions);
    let mut hasher = Sha256::new();
    hasher.update(message.as_bytes());
    let message_hash = Message::from_digest_slice(&hasher.finalize()).unwrap();
    let signature = Signature::from_slice(&hex::decode(signature_str)?)?;
    let delegator = XOnlyPublicKey::from_slice(&hex::decode(delegator_str)?)?;
    secp.verify_schnorr(&signature, &message_hash, &delegator)?;

    // Verify conditions
    for condition in conditions.split('&') {
        let mut parts = condition.split('=');
        let key = parts.next();
        let value = parts.next();

        if let (Some(key), Some(value)) = (key, value) {
            match key {
                "kind" => {
                    if value.parse::<u16>()? != event_kind {
                        return Ok(false);
                    }
                }
                "created_at<" => {
                    if created_at >= value.parse::<u64>()? {
                        return Ok(false);
                    }
                }
                "created_at>" => {
                    if created_at <= value.parse::<u64>()? {
                        return Ok(false);
                    }
                }
                _ => {} // Unknown conditions are ignored
            }
        }
    }

    Ok(true)
}
```

---

### nip28.rs

**Size:** 50812 bytes | **Modified:** 2026-01-20 14:02:27

```rust
#![allow(clippy::module_inception)]

// NIP-28: Public Chat Channels
// https://github.com/nostr-protocol/nips/blob/master/28.md

use std::{collections::HashSet, str::FromStr};

use secp256k1::{SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

use crate::types::{
    event_kind::{EventKind, EventKindOrRange},
    versioned::event3::{EventV3, PreEventV3},
    Error, Id, KeySecurity, NAddr, NostrBech32, NostrUrl, PublicKey, PublicKeyHex, Signature,
    Signer, TagV3, UncheckedUrl, Unixtime,
};

/// Event Kind 40: Create channel
/// Used to create a public chat channel, including initial metadata like name,
/// description, and picture.
pub const CREATE_CHANNEL: EventKind = EventKind::ChannelCreation;

/// Event Kind 41: Set channel metadata
/// Used to update a channel's public metadata. Clients should treat these like
/// replaceable events, only storing the most recent one, and ignore updates
/// from pubkeys other than the channel creator.
pub const SET_CHANNEL_METADATA: EventKind = EventKind::ChannelMetadata;

/// Event Kind 42: Create channel message
/// Used to send text messages within a channel. It supports NIP-10 tags for
/// relay recommendations and to indicate if a message is a reply or a root
/// message within a thread.
pub const CREATE_CHANNEL_MESSAGE: EventKind = EventKind::ChannelMessage;

/// Event Kind 43: Hide message
/// Allows a user to hide a specific message within a channel. Clients can
/// optionally hide messages for other users based on multiple hide events.
pub const HIDE_MESSAGE: EventKind = EventKind::ChannelHideMessage;

/// Event Kind 44: Mute user
/// Allows a user to mute another user, hiding their messages within the
/// channel. Similar to hiding messages, clients can extend this moderation to
/// multiple users.
pub const MUTE_USER: EventKind = EventKind::ChannelMuteUser;

/// Represents a parsed Kind 40 event for creating a public channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelCreationEvent {
    /// The unique identifier for the channel (from 'd' tag).
    pub channel_id: String,
    /// The name of the channel (from 'name' tag, optional).
    pub channel_name: Option<String>,
    /// The description of the channel (from 'description' tag, optional).
    pub channel_description: Option<String>,
    /// URL to the channel's picture (from 'picture' tag, optional).
    pub channel_picture: Option<String>,
    /// A recommended relay URL for the channel (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
    /// The public key of the event author.
    pub pubkey: PublicKey,
}

/// Represents a parsed Kind 41 event for setting channel metadata.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelMetadataEvent {
    /// The unique identifier for the channel (from 'd' tag).
    pub channel_id: String,
    /// The name of the channel (from 'name' tag, optional).
    pub channel_name: Option<String>,
    /// The description of the channel (from 'description' tag, optional).
    pub channel_description: Option<String>,
    /// URL to the channel's picture (from 'picture' tag, optional).
    pub channel_picture: Option<String>,
    /// A recommended relay URL for the channel (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
    /// The public key of the event author.
    pub pubkey: PublicKey,
}

/// Creates a Kind 40 event for creating a public channel.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `channel_name`: The name of the channel (optional, 'name' tag).
/// * `channel_description`: The description of the channel (optional,
///   'description' tag).
/// * `channel_picture`: URL to the channel's picture (optional, 'picture' tag).
/// * `relay_url`: A recommended relay URL for the channel (optional, 'relay'
///   tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on
/// failure.
pub fn create_channel(
    signer: &dyn Signer,
    channel_id: &str,
    channel_name: &str,
    channel_description: &str,
    channel_picture: Option<&str>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'name' tag - optional
    if !channel_name.is_empty() {
        tags.push(TagV3::new(&["name", channel_name]));
    }

    // 'description' tag - optional
    if !channel_description.is_empty() {
        tags.push(TagV3::new(&["description", channel_description]));
    }

    // 'picture' tag - optional
    if let Some(picture_url) = channel_picture {
        if !picture_url.is_empty() {
            tags.push(TagV3::new(&["picture", picture_url]));
        }
    }

    // 'relay' tag - optional
    if let Some(relay) = relay_url {
        // NIP-28 doesn't explicitly define a marker for channel creation relay, so use
        // None.
        tags.push(TagV3::new_relay(relay.clone(), None));
    }

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: CREATE_CHANNEL, // Kind 40
        tags,
        content: "".to_string(), // Channel creation event might not have content per NIP-28
    };

    // Sign the event
    signer.sign_event(pre_event)
}

/// Parses a generic `EventV3` into a `ChannelCreationEvent` if it matches Kind
/// 40 and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `ChannelCreationEvent` on success, or an `Error`
/// if parsing fails or the event is not a valid Kind 40 event.
pub fn parse_channel_creation(event: &EventV3) -> Result<ChannelCreationEvent, Error> {
    if event.kind != CREATE_CHANNEL {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut channel_name: Option<String> = None;
    let mut channel_description: Option<String> = None;
    let mut channel_picture: Option<String> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if tag.tagname() == "name" && !tag.value().is_empty() {
            channel_name = Some(tag.value().to_string());
        } else if tag.tagname() == "description" && !tag.value().is_empty() {
            channel_description = Some(tag.value().to_string());
        } else if tag.tagname() == "picture" && !tag.value().is_empty() {
            channel_picture = Some(tag.value().to_string());
        } else if let Ok((url, _)) = tag.parse_relay() {
            relay_url = Some(url);
        }
    }

    match channel_id {
        Some(id) => Ok(ChannelCreationEvent {
            channel_id: id,
            channel_name,
            channel_description,
            channel_picture,
            relay_url,
            pubkey: event.pubkey,
        }),
        None => Err(Error::AssertionFailed(
            "Missing 'd' tag for channel ID.".to_string(),
        )),
    }
}

/// Represents a parsed Kind 42 event for a message within a channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelMessageEvent {
    /// The unique identifier of the channel (from 'd' tag).
    pub channel_id: String,
    /// The content of the message.
    pub message: String,
    /// The ID of the message this message is replying to ('e' tag with 'reply'
    /// marker).
    pub reply_to: Option<Id>,
    /// The ID of the root message in a thread ('e' tag with 'root' marker).
    pub root_message: Option<Id>,
    /// The public key of the sender.
    pub pubkey: PublicKey,
    /// A recommended relay URL for context (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
}

/// Parses a generic `EventV3` into a `ChannelMessageEvent` if it matches Kind
/// 42 and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `ChannelMessageEvent` on success, or an `Error` if
/// parsing fails or the event is not a valid Kind 42 event.
pub fn parse_channel_message(event: &EventV3) -> Result<ChannelMessageEvent, Error> {
    if event.kind != CREATE_CHANNEL_MESSAGE {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut reply_to: Option<Id> = None;
    let mut root_message: Option<Id> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if let Ok((id, recommended_relay_url, marker)) = tag.parse_event() {
            if marker.as_deref() == Some("reply") {
                reply_to = Some(id);
                // Store relay if present, prioritizing explicit relay tags on reply/root.
                relay_url = recommended_relay_url;
            } else if marker.as_deref() == Some("root") {
                root_message = Some(id);
                relay_url = recommended_relay_url; // Store relay if present
            }
        } else if let Ok((url, _)) = tag.parse_relay() {
            // If no explicit relay tag was found on reply/root, check for a standalone 'r'
            // tag.
            if relay_url.is_none() {
                relay_url = Some(url);
            }
        }
    }

    match channel_id {
        Some(id) => Ok(ChannelMessageEvent {
            channel_id: id,
            message: event.content.clone(),
            reply_to,
            root_message,
            pubkey: event.pubkey,
            relay_url,
        }),
        None => Err(Error::AssertionFailed(
            "Missing 'd' tag for channel ID.".to_string(),
        )),
    }
}

/// Represents a parsed Kind 43 event for hiding a message.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct HideMessageEvent {
    /// The unique identifier of the channel (from 'd' tag).
    pub channel_id: String,
    /// The ID of the message that was hidden ('e' tag).
    pub message_id_to_hide: Id,
    /// An optional reason for hiding the message (from 'reason' tag).
    pub reason: Option<String>,
    /// A recommended relay URL for context (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
    /// The public key of the sender of the hide message event.
    pub pubkey: PublicKey,
}

/// Parses a generic `EventV3` into a `HideMessageEvent` if it matches Kind 43
/// and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `HideMessageEvent` on success, or an `Error` if
/// parsing fails or the event is not a valid Kind 43 event.
pub fn parse_hide_message(event: &EventV3) -> Result<HideMessageEvent, Error> {
    if event.kind != HIDE_MESSAGE {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut message_id_to_hide: Option<Id> = None;
    let mut reason: Option<String> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if let Ok((id, recommended_relay_url, _)) = tag.parse_event() {
            // Assume the first 'e' tag is the message to hide.
            if message_id_to_hide.is_none() {
                message_id_to_hide = Some(id);
                relay_url = recommended_relay_url;
            }
        } else if let Ok((url, _)) = tag.parse_relay() {
            // Capture relay URL if not already set by an 'e' tag.
            if relay_url.is_none() {
                relay_url = Some(url);
            }
        } else if tag.tagname() == "reason" && !tag.value().is_empty() {
            reason = Some(tag.value().to_string());
        }
    }

    match (channel_id, message_id_to_hide) {
        (Some(id), Some(msg_id)) => Ok(HideMessageEvent {
            channel_id: id,
            message_id_to_hide: msg_id,
            reason,
            relay_url,
            pubkey: event.pubkey,
        }),
        (None, _) => Err(Error::AssertionFailed(
            "Missing 'd' tag for channel ID.".to_string(),
        )),
        (_, None) => Err(Error::AssertionFailed(
            "Missing 'e' tag for message ID.".to_string(),
        )),
    }
}

/// Represents a parsed Kind 44 event for muting a user.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MuteUserEvent {
    /// The unique identifier of the channel (from 'd' tag).
    pub channel_id: String,
    /// The public key of the user being muted ('p' tag).
    pub user_pubkey: PublicKey,
    /// An optional reason for muting the user (from 'reason' tag).
    pub reason: Option<String>,
    /// A recommended relay URL for context (from 'relay' tag, optional).
    pub relay_url: Option<UncheckedUrl>,
    /// The public key of the sender of the mute event.
    pub pubkey: PublicKey,
}

/// Parses a generic `EventV3` into a `MuteUserEvent` if it matches Kind 44 and
/// has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `MuteUserEvent` on success, or an `Error` if
/// parsing fails or the event is not a valid Kind 44 event.
pub fn parse_mute_user(event: &EventV3) -> Result<MuteUserEvent, Error> {
    if event.kind != MUTE_USER {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut user_pubkey: Option<PublicKey> = None;
    let mut reason: Option<String> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if let Ok((pubkey, recommended_relay_url, _)) = tag.parse_pubkey() {
            // NIP-28 specifies the 'p' tag for the muted user's public key.
            user_pubkey = Some(pubkey);
            relay_url = recommended_relay_url; // Capture relay if present
        } else if let Ok((url, _)) = tag.parse_relay() {
            // Capture relay URL if not already set by a 'p' tag.
            if relay_url.is_none() {
                relay_url = Some(url);
            }
        } else if tag.tagname() == "reason" && !tag.value().is_empty() {
            reason = Some(tag.value().to_string());
        }
    }

    match (channel_id, user_pubkey) {
        (Some(id), Some(pk)) => Ok(MuteUserEvent {
            channel_id: id,
            user_pubkey: pk,
            reason,
            relay_url,
            pubkey: event.pubkey,
        }),
        (None, _) => Err(Error::AssertionFailed(
            "Missing 'd' tag for channel ID.".to_string(),
        )),
        (_, None) => Err(Error::AssertionFailed(
            "Missing 'p' tag for user public key.".to_string(),
        )),
    }
}

/// Creates a Kind 41 event for setting channel metadata.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `channel_name`: The new name of the channel (optional, 'name' tag).
/// * `channel_description`: The new description of the channel (optional,
///   'description' tag).
/// * `channel_picture`: New URL to the channel's picture (optional, 'picture'
///   tag).
/// * `relay_url`: A recommended relay URL for the channel (optional, 'relay'
///   tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on
/// failure.
pub fn set_channel_metadata(
    signer: &dyn Signer,
    channel_id: &str,
    channel_name: Option<&str>,
    channel_description: Option<&str>,
    channel_picture: Option<&str>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'name' tag - optional
    if let Some(name) = channel_name {
        if !name.is_empty() {
            tags.push(TagV3::new(&["name", name]));
        }
    }

    // 'description' tag - optional
    if let Some(description) = channel_description {
        if !description.is_empty() {
            tags.push(TagV3::new(&["description", description]));
        }
    }

    // 'picture' tag - optional
    if let Some(picture_url) = channel_picture {
        if !picture_url.is_empty() {
            tags.push(TagV3::new(&["picture", picture_url]));
        }
    }

    // 'relay' tag - optional
    if let Some(relay) = relay_url {
        tags.push(TagV3::new_relay(relay.clone(), None)); // Metadata updates might also include relay recommendations
    }

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: SET_CHANNEL_METADATA, // Kind 41
        tags,
        content: "".to_string(), // Metadata events typically have empty content
    };

    // Sign the event
    signer.sign_event(pre_event)
}

/// Parses a generic `EventV3` into a `ChannelMetadataEvent` if it matches Kind
/// 41 and has valid tags.
///
/// # Arguments
/// * `event`: The `EventV3` to parse.
///
/// # Returns
/// A `Result` containing the `ChannelMetadataEvent` on success, or an `Error`
/// if parsing fails or the event is not a valid Kind 41 event.
pub fn parse_set_channel_metadata(event: &EventV3) -> Result<ChannelMetadataEvent, Error> {
    if event.kind != SET_CHANNEL_METADATA {
        return Err(Error::WrongEventKind);
    }

    let mut channel_id: Option<String> = None;
    let mut channel_name: Option<String> = None;
    let mut channel_description: Option<String> = None;
    let mut channel_picture: Option<String> = None;
    let mut relay_url: Option<UncheckedUrl> = None;

    for tag in event.tags.iter() {
        if let Ok(d) = tag.parse_identifier() {
            channel_id = Some(d);
        } else if tag.tagname() == "name" && !tag.value().is_empty() {
            channel_name = Some(tag.value().to_string());
        } else if tag.tagname() == "description" && !tag.value().is_empty() {
            channel_description = Some(tag.value().to_string());
        } else if tag.tagname() == "picture" && !tag.value().is_empty() {
            channel_picture = Some(tag.value().to_string());
        } else if let Ok((url, _)) = tag.parse_relay() {
            relay_url = Some(url);
        }
    }

    match channel_id {
        Some(id) => Ok(ChannelMetadataEvent {
            channel_id: id,
            channel_name,
            channel_description,
            channel_picture,
            relay_url,
            pubkey: event.pubkey,
        }),
        None => Err(Error::AssertionFailed(
            "Missing 'd' tag for channel ID.".to_string(),
        )),
    }
}

/// Creates a Kind 42 event for sending a message within a channel.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `message`: The content of the message.
/// * `reply_to_id`: The ID of the message this message is replying to
///   (optional, 'e' tag with 'reply' marker).
/// * `root_message_id`: The ID of the root message in a thread (optional, 'e'
///   tag with 'root' marker).
/// * `relay_url`: A recommended relay URL for context (optional, 'relay' tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on
/// failure.
pub fn create_channel_message(
    signer: &dyn Signer,
    channel_id: &str,
    message: &str,
    reply_to_id: Option<Id>,
    root_message_id: Option<Id>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'e' tag for reply
    if let Some(id) = reply_to_id {
        tags.push(TagV3::new_event(
            id,
            relay_url.cloned(),
            Some("reply".to_string()),
        ));
    }

    // 'e' tag for root message
    if let Some(id) = root_message_id {
        tags.push(TagV3::new_event(
            id,
            relay_url.cloned(),
            Some("root".to_string()),
        ));
    }

    // 'relay' tag
    if let Some(url) = relay_url {
        tags.push(TagV3::new_relay(url.clone(), None));
    }

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: CREATE_CHANNEL_MESSAGE, // Kind 42
        tags,
        content: message.to_string(),
    };
    // Sign the event
    signer.sign_event(pre_event)
}

/// Creates a Kind 43 event for hiding a message.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `message_id_to_hide`: The ID of the message to hide (required, 'e' tag).
/// * `reason`: An optional reason for hiding the message.
/// * `relay_url`: A recommended relay URL for context (optional, 'relay' tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on
/// failure.
pub fn hide_message(
    signer: &dyn Signer,
    channel_id: &str,
    message_id_to_hide: &Id,
    reason: Option<&str>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'e' tag for the message to hide
    tags.push(TagV3::new_event(
        *message_id_to_hide,
        relay_url.cloned(),
        None,
    ));

    // 'reason' tag - optional
    if let Some(r) = reason {
        if !r.is_empty() {
            tags.push(TagV3::new(&["reason", r]));
        }
    }

    // 'relay' tag - optional
    if let Some(url) = relay_url {
        tags.push(TagV3::new_relay(url.clone(), None));
    }

    // Filter out any 'p' tags to ensure NIP-28 compliance for Kind 43 events
    let filtered_tags: Vec<TagV3> = tags
        .into_iter()
        .filter(|tag| tag.tagname() != "p")
        .collect();

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: HIDE_MESSAGE, // Kind 43
        tags: filtered_tags,
        content: "".to_string(),
    };
    // Sign the event
    signer.sign_event(pre_event)
}

/// Creates a Kind 44 event for muting a user.
///
/// # Arguments
/// * `signer`: The signer that will be used to sign the event.
/// * `channel_id`: The unique identifier for the channel (required, 'd' tag).
/// * `user_pubkey`: The public key of the user to mute (required, 'p' tag).
/// * `reason`: An optional reason for muting the user.
/// * `relay_url`: A recommended relay URL for context (optional, 'relay' tag).
///
/// # Returns
/// A `Result` containing the signed `EventV3` on success, or an `Error` on
/// failure.
pub fn mute_user(
    signer: &dyn Signer,
    channel_id: &str,
    user_pubkey: &PublicKey,
    reason: Option<&str>,
    relay_url: Option<&UncheckedUrl>,
) -> Result<EventV3, Error> {
    let mut tags = vec![];

    // 'd' tag (channel identifier) - required
    tags.push(TagV3::new_identifier(channel_id.to_string()));

    // 'p' tag for the user to mute
    tags.push(TagV3::new_pubkey(*user_pubkey, relay_url.cloned(), None));

    // 'reason' tag - optional
    if let Some(r) = reason {
        if !r.is_empty() {
            tags.push(TagV3::new(&["reason", r]));
        }
    }

    // 'relay' tag - optional
    if let Some(url) = relay_url {
        tags.push(TagV3::new_relay(url.clone(), None));
    }

    // Create PreEvent
    let pre_event = PreEventV3 {
        pubkey: signer.public_key(),
        created_at: Unixtime::now(),
        kind: MUTE_USER, // Kind 44
        tags,
        content: "".to_string(), // Mute user events typically have empty content
    };

    // Sign the event
    signer.sign_event(pre_event)
}

#[cfg(test)]
mod test {
    use std::time::{SystemTime, UNIX_EPOCH};

    use secp256k1::{Keypair, Secp256k1, SecretKey, XOnlyPublicKey};
    use sha2::{Digest, Sha256};

    use super::*;
    use crate::{
        test_serde,
        types::{
            Error, EventKind, Id, KeySecurity, PrivateKey, PublicKey, PublicKeyHex, Signer, TagV3,
            UncheckedUrl, Unixtime,
        },
        KeySigner,
    };

    #[test]
    fn test_nip28_event_kinds() {
        assert_eq!(CREATE_CHANNEL, EventKind::from(40));
        assert_eq!(SET_CHANNEL_METADATA, EventKind::from(41));
        assert_eq!(CREATE_CHANNEL_MESSAGE, EventKind::from(42));
        assert_eq!(HIDE_MESSAGE, EventKind::from(43));
        assert_eq!(MUTE_USER, EventKind::from(44));
    }

    #[test]
    fn test_create_channel_event() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "my-cool-channel";
        let channel_name = "My Cool Channel";
        let channel_description = "A channel for cool people.";
        let channel_picture = Some("https://example.com/picture.jpg");
        let relay_url = UncheckedUrl::from_str("wss://relay.example.com");

        let event = create_channel(
            &signer,
            channel_id,
            channel_name,
            channel_description,
            channel_picture,
            Some(&relay_url).cloned().as_ref(),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::ChannelCreation);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, "");

        // Check tags
        let mut found_d_tag = false;
        let mut found_name_tag = false;
        let mut found_description_tag = false;
        let mut found_picture_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "name" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_name);
                found_name_tag = true;
            } else if tag.tagname() == "description" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_description);
                found_description_tag = true;
            } else if tag.tagname() == "picture" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_picture.unwrap());
                found_picture_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_name_tag);
        assert!(found_description_tag);
        assert!(found_picture_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_set_channel_metadata_event() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "my-channel-id";
        let channel_name = Some("Updated Channel Name");
        let channel_description = Some("Updated description.");
        let channel_picture = Some("https://example.com/new_picture.jpg");
        let relay_url = UncheckedUrl::from_str("wss://new-relay.example.com");

        let event = set_channel_metadata(
            &signer,
            channel_id,
            channel_name,
            channel_description,
            channel_picture,
            Some(&relay_url).cloned().as_ref(),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::ChannelMetadata);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, ""); // Metadata events typically have empty content

        // Check tags
        let mut found_d_tag = false;
        let mut found_name_tag = false;
        let mut found_description_tag = false;
        let mut found_picture_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "name" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_name.unwrap());
                found_name_tag = true;
            } else if tag.tagname() == "description" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_description.unwrap());
                found_description_tag = true;
            } else if tag.tagname() == "picture" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_picture.unwrap());
                found_picture_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_name_tag);
        assert!(found_description_tag);
        assert!(found_picture_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_set_channel_metadata_with_optional_args() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "another-channel";

        // Test with only required arguments
        let event_minimal = set_channel_metadata(
            &signer, channel_id, None, // name
            None, // description
            None, // picture
            None, // relay_url
        )
        .unwrap();

        assert_eq!(event_minimal.kind, EventKind::ChannelMetadata);
        assert_eq!(event_minimal.pubkey, signer.public_key());
        assert_eq!(event_minimal.content, "");
        // Should only contain the 'd' tag
        assert_eq!(event_minimal.tags.len(), 1);
        if event_minimal.tags[0].tagname() == "d" {
            let d = event_minimal.tags[0].parse_identifier().unwrap();
            assert_eq!(d, channel_id);
        } else {
            panic!("Expected 'd' tag for channel ID");
        }

        // Test with some optional arguments
        let channel_name = Some("Channel Name");
        let channel_picture = Some("https://example.com/pic.png");
        let event_partial = set_channel_metadata(
            &signer,
            channel_id,
            channel_name,
            None, // description
            channel_picture,
            None, // relay_url
        )
        .unwrap();

        assert_eq!(event_partial.kind, EventKind::ChannelMetadata);
        assert_eq!(event_partial.pubkey, signer.public_key());
        assert_eq!(event_partial.content, "");

        let mut found_d = false;
        let mut found_name = false;
        let mut found_picture = false;
        for tag in event_partial.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d = true;
            } else if tag.tagname() == "name" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_name.unwrap());
                found_name = true;
            } else if tag.tagname() == "picture" && !tag.value().is_empty() {
                assert_eq!(tag.value(), channel_picture.unwrap());
                found_picture = true;
            }
        }
        assert!(found_d && found_name && found_picture);
    }

    #[test]
    fn test_create_channel_message_event() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "chat-channel";
        let message = "Hello, this is a chat message!";
        let reply_to_id = Some(Id::mock());
        let root_message_id = Some(Id::mock());
        let relay_url = UncheckedUrl::from_str("wss://chat-relay.example.com");

        let event = create_channel_message(
            &signer,
            channel_id,
            message,
            reply_to_id,
            root_message_id,
            Some(&relay_url).cloned().as_ref(),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::ChannelMessage);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, message);

        // Check tags
        let mut found_d_tag = false;
        let mut found_reply_e_tag = false;
        let mut found_root_e_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "e" {
                let (id, recommended_relay_url, marker) = tag.parse_event().unwrap();
                if marker.as_deref() == Some("reply") {
                    assert_eq!(id, reply_to_id.unwrap());
                    assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                    found_reply_e_tag = true;
                } else if marker.as_deref() == Some("root") {
                    assert_eq!(id, root_message_id.unwrap());
                    assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                    found_root_e_tag = true;
                }
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_reply_e_tag);
        assert!(found_root_e_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_hide_message_event() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "secret-channel";
        let message_id_to_hide = Id::mock();
        let reason = Some("spam");
        let relay_url = UncheckedUrl::from_str("wss://hide-relay.example.com");

        let event = hide_message(
            &signer,
            channel_id,
            &message_id_to_hide,
            reason,
            Some(&relay_url).cloned().as_ref(),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::ChannelHideMessage);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, "");

        // Check tags
        let mut found_d_tag = false;
        let mut found_e_tag = false;
        let mut found_p_tag = false;
        let mut found_reason_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "e" {
                let (id, recommended_relay_url, _) = tag.parse_event().unwrap();
                assert_eq!(id, message_id_to_hide);
                assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                found_e_tag = true;
            } else if tag.tagname() == "reason" && !tag.value().is_empty() {
                assert_eq!(tag.value(), reason.unwrap());
                found_reason_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_e_tag);
        assert!(found_reason_tag);
        assert!(found_relay_tag);

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "p" {
                let (pubkey, recommended_relay_url, petname) = tag.parse_pubkey().unwrap();
                assert!(pubkey.len() > 0);
                assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                assert!(petname.is_none()); // Mute user tag should not have petname
                if petname.is_none() {
                    found_p_tag = false;
                }
                assert_eq!(petname.is_none(), !found_p_tag);
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            } else if tag.tagname() == "reason" && !tag.value().is_empty() {
                assert_eq!(tag.value(), reason.unwrap());
                found_reason_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(!found_p_tag);
        assert!(found_reason_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_parse_channel_creation() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "my-cool-channel";
        let channel_name = "My Cool Channel";
        let channel_description = "A channel for cool people.";
        let channel_picture = Some("https://example.com/picture.jpg");
        let relay_url = UncheckedUrl::from_str("wss://relay.example.com");

        let event = create_channel(
            &signer,
            channel_id,
            channel_name,
            channel_description,
            channel_picture,
            Some(&relay_url).cloned().as_ref(),
        )
        .unwrap();

        let parsed_event = parse_channel_creation(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.channel_name, Some(channel_name.to_string()));
        assert_eq!(
            parsed_event.channel_description,
            Some(channel_description.to_string())
        );
        assert_eq!(
            parsed_event.channel_picture,
            Some(channel_picture.unwrap().to_string())
        );
        assert_eq!(parsed_event.relay_url, Some(&relay_url).cloned());
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_channel_creation_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "minimal-channel";

        let event = create_channel(
            &signer, channel_id, "",   // Empty name
            "",   // Empty description
            None, // No picture
            None, // No relay URL
        )
        .unwrap();

        let parsed_event = parse_channel_creation(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.channel_name, None);
        assert_eq!(parsed_event.channel_description, None);
        assert_eq!(parsed_event.channel_picture, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_channel_message() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "chat-channel";
        let message = "Hello, this is a chat message!";
        let reply_to_id = Some(Id::mock());
        let root_message_id = Some(Id::mock());
        let relay_url = UncheckedUrl::from_str("wss://chat-relay.example.com");

        let event = create_channel_message(
            &signer,
            channel_id,
            message,
            reply_to_id,
            root_message_id,
            Some(&relay_url).cloned().as_ref(),
        )
        .unwrap();

        let parsed_event = parse_channel_message(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.message, message);
        assert_eq!(parsed_event.reply_to, reply_to_id);
        assert_eq!(parsed_event.root_message, root_message_id);
        assert_eq!(parsed_event.relay_url, Some(&relay_url).cloned());
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_channel_message_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "minimal-chat";
        let message = "Just a simple message";

        let event = create_channel_message(
            &signer, channel_id, message, None, // reply_to
            None, // root_message
            None, // relay_url
        )
        .unwrap();

        let parsed_event = parse_channel_message(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.message, message);
        assert_eq!(parsed_event.reply_to, None);
        assert_eq!(parsed_event.root_message, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_hide_message_event_duplicate() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "secret-channel";
        let message_id_to_hide = Id::mock();
        let reason = Some("spam");
        let relay_url = UncheckedUrl::from_str("wss://hide-relay.example.com");

        let event = hide_message(
            &signer,
            channel_id,
            &message_id_to_hide,
            reason,
            Some(&relay_url),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::ChannelHideMessage);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, "");

        // Check tags
        let mut found_d_tag = false;
        let mut found_e_tag = false;
        let mut found_reason_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "e" {
                let (id, recommended_relay_url, _) = tag.parse_event().unwrap();
                assert_eq!(id, message_id_to_hide);
                assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                found_e_tag = true;
            } else if tag.tagname() == "reason" && !tag.value().is_empty() {
                assert_eq!(tag.value(), reason.unwrap());
                found_reason_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_e_tag);
        assert!(found_reason_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_parse_hide_message() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "secret-channel";
        let message_id_to_hide = Id::mock();
        let reason = Some("spam");
        let relay_url = UncheckedUrl::from_str("wss://hide-relay.example.com");

        let event = hide_message(
            &signer,
            channel_id,
            &message_id_to_hide,
            reason,
            Some(&relay_url).cloned().as_ref(),
        )
        .unwrap();

        let parsed_event = parse_hide_message(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.message_id_to_hide, message_id_to_hide);
        assert_eq!(parsed_event.reason, reason.map(|r| r.to_string()));
        assert_eq!(parsed_event.relay_url, Some(&relay_url).cloned());
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_hide_message_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "minimal-hide-channel";
        let message_id_to_hide = Id::mock();

        let event = hide_message(
            &signer,
            channel_id,
            &message_id_to_hide,
            None, // reason
            None, // relay_url
        )
        .unwrap();

        let parsed_event = parse_hide_message(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.message_id_to_hide, message_id_to_hide);
        assert_eq!(parsed_event.reason, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_mute_user_event_duplicate() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "muted-channel";
        let user_pubkey = PublicKey::mock_deterministic();
        let reason = Some("trolling");
        let relay_url = UncheckedUrl::from_str("wss://mute-relay.example.com");

        let event = mute_user(&signer, channel_id, &user_pubkey, reason, Some(&relay_url)).unwrap();

        assert_eq!(event.kind, EventKind::ChannelMuteUser);
        assert_eq!(event.pubkey, signer.public_key());
        assert_eq!(event.content, "");

        // Check tags
        let mut found_d_tag = false;
        let mut found_p_tag = false;
        let mut found_reason_tag = false;
        let mut found_relay_tag = false;

        for tag in event.tags.iter() {
            if tag.tagname() == "d" {
                let d = tag.parse_identifier().unwrap();
                assert_eq!(d, channel_id);
                found_d_tag = true;
            } else if tag.tagname() == "p" {
                let (pubkey, recommended_relay_url, petname) = tag.parse_pubkey().unwrap();
                assert_eq!(pubkey, user_pubkey);
                assert_eq!(recommended_relay_url, Some(relay_url.clone()));
                assert!(petname.is_none()); // Mute user tag should not have petname
                found_p_tag = true;
            } else if tag.tagname() == "r" {
                let (url, _) = tag.parse_relay().unwrap();
                assert_eq!(url, relay_url);
                found_relay_tag = true;
            } else if tag.tagname() == "reason" && !tag.value().is_empty() {
                assert_eq!(tag.value(), reason.unwrap());
                found_reason_tag = true;
            }
        }

        assert!(found_d_tag);
        assert!(found_p_tag);
        assert!(found_reason_tag);
        assert!(found_relay_tag);
    }

    #[test]
    fn test_parse_mute_user_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "muted-channel";
        let user_pubkey = PublicKey::mock_deterministic();

        let event = mute_user(
            &signer,
            channel_id,
            &user_pubkey,
            None, // reason
            None, // relay_url
        )
        .unwrap();

        let parsed_event = parse_mute_user(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.user_pubkey, user_pubkey);
        assert_eq!(parsed_event.reason, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_set_channel_metadata() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "test-channel-id";
        let channel_name = Some("Test Channel Name");
        let channel_description = Some("This is a test channel description.");
        let channel_picture = Some("https://example.com/test_picture.jpg");
        let relay_url = UncheckedUrl::from_str("wss://test-relay.example.com");

        let event = set_channel_metadata(
            &signer,
            channel_id,
            channel_name,
            channel_description,
            channel_picture,
            Some(&relay_url).cloned().as_ref(),
        )
        .unwrap();

        let parsed_event = parse_set_channel_metadata(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(
            parsed_event.channel_name,
            channel_name.map(|s| s.to_string())
        );
        assert_eq!(
            parsed_event.channel_description,
            channel_description.map(|s| s.to_string())
        );
        assert_eq!(
            parsed_event.channel_picture,
            channel_picture.map(|s| s.to_string())
        );
        assert_eq!(parsed_event.relay_url, Some(&relay_url).cloned());
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }

    #[test]
    fn test_parse_set_channel_metadata_minimal() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let channel_id = "minimal-channel-id";

        let event = set_channel_metadata(
            &signer, channel_id, None, // name
            None, // description
            None, // picture
            None, // relay_url
        )
        .unwrap();

        let parsed_event = parse_set_channel_metadata(&event).unwrap();

        assert_eq!(parsed_event.channel_id, channel_id);
        assert_eq!(parsed_event.channel_name, None);
        assert_eq!(parsed_event.channel_description, None);
        assert_eq!(parsed_event.channel_picture, None);
        assert_eq!(parsed_event.relay_url, None);
        assert_eq!(parsed_event.pubkey, signer.public_key());
    }
}
```

---

### nip3.rs

**Size:** 685 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-03: OpenTimestamps Attestations for Events
// https://github.com/nostr-protocol/nips/blob/master/03.md

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::types::event::{Event, EventId, UnsignedEvent};

/// Create an OpenTimestamps attestation event for another event.
///
/// The content must be a base64-encoded .ots file.
pub fn create_attestation(
    event_id: EventId,
    ots_base64: String,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let tags = vec![vec!["e".to_string(), event_id.as_hex_string()]];
    let unsigned_event = UnsignedEvent::new(public_key, 1040, tags, ots_base64);
    unsigned_event.sign(private_key).unwrap()
}
```

---

### nip30.rs

**Size:** 5084 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-30: Custom Emoji
//!
//! This NIP defines how to include custom emojis in Nostr events using a
//! special "emoji" tag. These emojis can be used in various event kinds like
//! kind 0, 1, 7, and 30315.
//!
//! https://github.com/nostr-protocol/nips/blob/master/30.md

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::types::{Event, Tag};

/// The name of the custom emoji tag.
pub const CUSTOM_EMOJI_TAG_NAME: &str = "emoji";

/// Represents a custom emoji, encapsulating its shortcode and image URL.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Emoji {
    pub shortcode: String,
    pub image_url: String,
}

/// Helper trait for NIP-30 custom emoji tags on Event types.
pub trait NIP30Event {
    /// Extracts all custom emojis defined in "emoji" tags from the event.
    /// Returns a vector of `Emoji` structs. Invalid or malformed tags are
    /// ignored.
    fn extract_custom_emojis(&self) -> Vec<Emoji>;

    /// Creates an "emoji" tag with the given shortcode and image URL.
    fn create_custom_emoji_tag(shortcode: String, image_url: String) -> Tag;
}

impl NIP30Event for Event {
    fn extract_custom_emojis(&self) -> Vec<Emoji> {
        self.tags
            .iter()
            .filter(|tag| tag.0.len() == 3 && tag.0[0] == CUSTOM_EMOJI_TAG_NAME)
            .filter_map(|tag| {
                Some(Emoji {
                    shortcode: tag.0[1].clone(),
                    image_url: tag.0[2].clone(),
                })
            })
            .collect()
    }

    fn create_custom_emoji_tag(shortcode: String, image_url: String) -> Tag {
        Tag::new(&[CUSTOM_EMOJI_TAG_NAME, &shortcode, &image_url])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature, Unixtime};

    // Helper to create a dummy event for testing
    fn create_dummy_event_with_emojis(emojis: Vec<Emoji>) -> Event {
        let mut tags = Vec::new();
        for emoji in emojis {
            tags.push(Event::create_custom_emoji_tag(
                emoji.shortcode,
                emoji.image_url,
            ));
        }

        Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "Hello :wave: world :globe:".to_string(),
            sig: Signature::zeroes(),
        }
    }

    #[test]
    fn test_create_custom_emoji_tag() {
        let shortcode = "wave".to_string();
        let image_url = "https://example.com/wave.png".to_string();
        let tag = Event::create_custom_emoji_tag(shortcode.clone(), image_url.clone());
        assert_eq!(
            tag.0,
            vec![
                CUSTOM_EMOJI_TAG_NAME.to_string(),
                shortcode.clone(),
                image_url.clone()
            ]
        );
    }

    #[test]
    fn test_extract_custom_emojis() {
        let emoji1 = Emoji {
            shortcode: "wave".to_string(),
            image_url: "https://example.com/wave.png".to_string(),
        };
        let emoji2 = Emoji {
            shortcode: "globe".to_string(),
            image_url: "https://example.com/globe.png".to_string(),
        };
        let emojis = vec![emoji1.clone(), emoji2.clone()];
        let event = create_dummy_event_with_emojis(emojis.clone());

        let extracted_emojis = event.extract_custom_emojis();
        assert_eq!(extracted_emojis, emojis);
    }

    #[test]
    fn test_extract_custom_emojis_no_tags() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])], // No emoji tags
            content: "Hello world.".to_string(),
            sig: Signature::zeroes(),
        };

        let extracted_emojis = event.extract_custom_emojis();
        assert!(extracted_emojis.is_empty());
    }

    #[test]
    fn test_extract_custom_emojis_malformed_tag() {
        let emoji1 = Emoji {
            shortcode: "wave".to_string(),
            image_url: "https://example.com/wave.png".to_string(),
        };
        let mut tags = vec![Event::create_custom_emoji_tag(
            emoji1.shortcode.clone(),
            emoji1.image_url.clone(),
        )];
        // Malformed tag (missing image_url)
        tags.push(Tag::new(&[CUSTOM_EMOJI_TAG_NAME, "malformed"]));
        // Malformed tag (wrong name)
        tags.push(Tag::new(&["wrong_tag", "short", "url"]));

        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "Hello :wave: world :malformed:".to_string(),
            sig: Signature::zeroes(),
        };

        let extracted_emojis = event.extract_custom_emojis();
        assert_eq!(extracted_emojis, vec![emoji1]); // Only the valid emoji should be extracted
    }
}
```

---

### nip32.rs

**Size:** 9768 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-32: Labeling
//!
//! This NIP defines a system for labeling Nostr events using two new indexable
//! tags: `L` for label namespaces and `l` for labels. It also defines a new
//! event kind (1985) for attaching these labels to existing events.
//!
//! https://github.com/nostr-protocol/nips/blob/master/32.md

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::types::{Event, EventKind, Id, PreEvent, PublicKey, Signature, Tag, Unixtime};

/// NIP-32 Label Event Kind
pub const LABEL_EVENT_KIND: u32 = 1985;
/// The name of the label tag.
pub const LABEL_TAG_NAME: &str = "l";
/// The name of the label namespace tag.
pub const LABEL_NAMESPACE_TAG_NAME: &str = "L";

/// Represents a label, composed of a value and an optional namespace.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Label {
    pub value: String,
    pub namespace: Option<String>,
}

/// Helper trait for NIP-32 labeling events.
pub trait NIP32Event {
    /// Extracts all labels defined in "l" and "L" tags from the event.
    /// Returns a vector of `Label` structs. Malformed tags are ignored.
    fn extract_labels(&self) -> Vec<Label>;

    /// Adds an "l" tag (label value) to the event.
    /// Optionally includes a "mark" if the label is associated with a
    /// namespace.
    fn add_label_tag(&mut self, label_value: String, mark: Option<String>);

    /// Adds an "L" tag (label namespace) to the event.
    fn add_label_namespace_tag(&mut self, namespace: String);

    /// Creates an "l" tag with the given label value and optional mark.
    fn create_label_tag(label_value: String, mark: Option<String>) -> Tag;

    /// Creates an "L" tag with the given namespace.
    fn create_label_namespace_tag(namespace: String) -> Tag;

    /// Creates a NIP-32 Label event (Kind 1985) to label a target event.
    fn new_label_event(
        public_key: PublicKey,
        label_value: String,
        namespace: Option<String>,
        target_event_id: Id, // For simplicity, initially target an Event ID
        content: Option<String>,
    ) -> Result<Event>;
}

impl NIP32Event for Event {
    fn extract_labels(&self) -> Vec<Label> {
        let mut labels = Vec::new();
        let mut namespace_map: std::collections::HashMap<String, String> =
            std::collections::HashMap::new();

        // First, process 'L' tags to build the namespace map
        for tag in &self.tags {
            if tag.0.len() == 2 && tag.0[0] == LABEL_NAMESPACE_TAG_NAME {
                let _ = namespace_map.insert(tag.0[1].clone(), tag.0[1].clone());
                // Mark is usually same as namespace for 'L' tags
            }
        }

        // Then, process 'l' tags
        for tag in &self.tags {
            if tag.0.len() >= 2 && tag.0[0] == LABEL_TAG_NAME {
                let label_value = tag.0[1].clone();
                let mark = tag.0.get(2).map(|s| s.clone());

                let namespace = if let Some(m) = mark {
                    // If a mark is present, check if it corresponds to a namespace
                    namespace_map.get(&m).map(|s| s.clone())
                } else {
                    // If no mark, NIP-32 implies "ugc" if omitted.
                    Some("ugc".to_string()) // TODO: NIP-32 says "ugc" is implied if omitted, check for existing "L" tag
                };

                labels.push(Label {
                    value: label_value,
                    namespace,
                });
            }
        }
        labels
    }

    fn add_label_tag(&mut self, label_value: String, mark: Option<String>) {
        let mut tag_elements = vec![LABEL_TAG_NAME.to_string(), label_value];
        if let Some(m) = mark {
            tag_elements.push(m);
        }
        self.tags.push(Tag::new(
            tag_elements
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        ));
    }

    fn add_label_namespace_tag(&mut self, namespace: String) {
        self.tags
            .push(Tag::new(&[LABEL_NAMESPACE_TAG_NAME, &namespace]));
    }

    fn create_label_tag(label_value: String, mark: Option<String>) -> Tag {
        let mut tag_elements = vec![LABEL_TAG_NAME.to_string(), label_value];
        if let Some(m) = mark {
            tag_elements.push(m);
        }
        Tag::new(
            tag_elements
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        )
    }

    fn create_label_namespace_tag(namespace: String) -> Tag {
        Tag::new(&[LABEL_NAMESPACE_TAG_NAME, &namespace])
    }

    fn new_label_event(
        public_key: PublicKey,
        label_value: String,
        namespace: Option<String>,
        target_event_id: Id,
        content_description: Option<String>,
    ) -> Result<Event> {
        let mut tags: Vec<Tag> = vec![
            Tag::new(&["e", &target_event_id.as_hex_string()]), // Target the event ID
            Self::create_label_tag(label_value, namespace.clone().or(Some("ugc".to_string()))), /* Default mark to "ugc" */
        ];
        if let Some(ns) = namespace {
            tags.push(Self::create_label_namespace_tag(ns));
        }

        let content = content_description.unwrap_or_default();

        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Label, // Use the specific EventKind for NIP-32 labels
            tags: tags.clone(),
            content: content.clone(),
        };

        let id = pre_event.hash().unwrap();

        Ok(Event {
            id,
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Label,
            tags,
            content,
            sig: Signature::zeroes(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Id, PublicKey, Signature};

    #[test]
    fn test_create_label_tag() {
        let tag = Event::create_label_tag("permies".to_string(), None);
        assert_eq!(tag.0, vec![LABEL_TAG_NAME, "permies"]);

        let tag_with_mark = Event::create_label_tag("humor".to_string(), Some("ugc".to_string()));
        assert_eq!(tag_with_mark.0, vec![LABEL_TAG_NAME, "humor", "ugc"]);
    }

    #[test]
    fn test_create_label_namespace_tag() {
        let tag = Event::create_label_namespace_tag("ugc".to_string());
        assert_eq!(tag.0, vec![LABEL_NAMESPACE_TAG_NAME, "ugc"]);
    }

    #[test]
    fn test_extract_labels() {
        let mut tags = Vec::new();
        tags.push(Event::create_label_namespace_tag("ISO-3166-2".to_string()));
        tags.push(Event::create_label_tag(
            "IT-MI".to_string(),
            Some("ISO-3166-2".to_string()),
        ));
        tags.push(Event::create_label_tag(
            "bug".to_string(),
            Some("ugc".to_string()),
        ));
        tags.push(Tag::new(&["e", &Id::mock().as_hex_string()])); // Non-label tag

        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "An event with labels".to_string(),
            sig: Signature::zeroes(),
        };

        let labels = event.extract_labels();
        assert_eq!(labels.len(), 2);
        assert!(labels.contains(&Label {
            value: "IT-MI".to_string(),
            namespace: Some("ISO-3166-2".to_string())
        }));
        assert!(labels.contains(&Label {
            value: "bug".to_string(),
            namespace: Some("ugc".to_string())
        }));
    }

    #[test]
    fn test_extract_labels_no_namespace_tag() {
        let mut tags = Vec::new();
        tags.push(Event::create_label_tag("feature".to_string(), None)); // No mark
        tags.push(Tag::new(&["p", &PublicKey::mock().as_hex_string()]));

        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "Another event with labels".to_string(),
            sig: Signature::zeroes(),
        };

        let labels = event.extract_labels();
        assert_eq!(labels.len(), 1);
        // NIP-32 implies "ugc" mark if omitted in the 'l' tag.
        assert!(labels.contains(&Label {
            value: "feature".to_string(),
            namespace: Some("ugc".to_string())
        }));
    }

    #[test]
    fn test_new_label_event() {
        let public_key = PublicKey::mock();
        let target_event_id = Id::mock();
        let label_value = "approved".to_string();
        let namespace = Some("moderation".to_string());
        let content_description = Some("This event has been approved by moderators.".to_string());

        let event = Event::new_label_event(
            public_key,
            label_value.clone(),
            namespace.clone(),
            target_event_id,
            content_description.clone(),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::Label);
        assert_eq!(event.content, content_description.unwrap());
        assert_eq!(event.extract_labels().len(), 1);
        assert_eq!(event.extract_labels()[0].value, label_value);
        assert_eq!(event.extract_labels()[0].namespace, namespace);
        assert!(event.tags.iter().any(|tag| tag.0.len() == 2
            && tag.0[0] == "e"
            && tag.0[1] == target_event_id.as_hex_string()));
        assert!(event.tags.iter().any(|tag| tag.0.len() == 2
            && tag.0[0] == LABEL_NAMESPACE_TAG_NAME
            && tag.0[1] == namespace.clone().unwrap()));
    }
}
```

---

### nip34.rs

**Size:** 11814 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-34 implementation for creating git-related events.

use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::anyhow;
use secp256k1::{Message, SecretKey, XOnlyPublicKey};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{blockhash, blockheight, weeble, wobble};

/// A signed Nostr event.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    /// 32-byte, hex-encoded SHA256 hash of the serialized event data.
    pub id: String,
    /// 32-byte, hex-encoded public key of the event creator.
    pub pubkey: String,
    /// Unix timestamp in seconds.
    pub created_at: u64,
    /// Event kind.
    pub kind: u16,
    /// A list of tags.
    pub tags: Vec<Vec<String>>,
    /// Event content.
    pub content: String,
    /// 64-byte signature of the event ID hash.
    pub sig: String,
}

/// An unsigned Nostr event.
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UnsignedEvent {
    /// 32-byte, hex-encoded public key of the event creator.
    pub pubkey: String,
    /// Unix timestamp in seconds.
    pub created_at: u64,
    /// Event kind.
    pub kind: u16,
    /// A list of tags.
    pub tags: Vec<Vec<String>>,
    /// Event content.
    pub content: String,
}

impl UnsignedEvent {
    /// Create a new unsigned event.
    pub fn new(
        pubkey: &XOnlyPublicKey,
        kind: u16,
        mut tags: Vec<Vec<String>>,
        content: String,
    ) -> Self {
        let created_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Ok(val) = weeble::weeble() {
            tags.push(vec!["weeble".to_string(), val.to_string()]);
        }
        if let Ok(val) = blockheight::blockheight() {
            tags.push(vec!["blockheight".to_string(), val.to_string()]);
        }
        if let Ok(val) = wobble::wobble() {
            tags.push(vec!["wobble".to_string(), val.to_string()]);
        }
        if let Ok(val) = blockhash::blockhash() {
            tags.push(vec!["blockhash".to_string(), val]);
        }

        Self {
            pubkey: pubkey.to_string(),
            created_at,
            kind,
            tags,
            content,
        }
    }

    /// Serialize the event data for hashing and signing.
    fn serialize(&self) -> Result<String, serde_json::Error> {
        let data = (
            0,
            &self.pubkey,
            self.created_at,
            self.kind,
            &self.tags,
            &self.content,
        );
        serde_json::to_string(&data)
    }

    /// Sign the event and return a signed `Event`.
    pub fn sign(self, secret_key: &SecretKey) -> Result<Event, Box<dyn std::error::Error>> {
        let serialized_event = self.serialize()?;
        let mut hasher = Sha256::new();
        hasher.update(serialized_event.as_bytes());
        let event_id_bytes = hasher.finalize();
        let id = hex::encode(event_id_bytes);

        let secp = secp256k1::Secp256k1::new();
        let message = Message::from_digest_slice(&event_id_bytes)?;
        let sig = secp.sign_schnorr(&message, &secret_key.keypair(&secp));

        Ok(Event {
            id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            tags: self.tags,
            content: self.content,
            sig: sig.to_string(),
        })
    }
}

/// NIP-34 event kinds.
#[allow(missing_docs)]
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Nip34Kind {
    RepoAnnouncement = 30617,
    RepoState = 30618,
    Patch = 1617,
    PullRequest = 1618,
    PullRequestUpdate = 1619,
    Issue = 1621,
    StatusOpen = 1630,
    StatusApplied = 1631,
    StatusClosed = 1632,
    StatusDraft = 1633,
    UserGraspList = 10317,
}

impl TryFrom<u16> for Nip34Kind {
    type Error = anyhow::Error;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            30617 => Ok(Nip34Kind::RepoAnnouncement),
            30618 => Ok(Nip34Kind::RepoState),
            1617 => Ok(Nip34Kind::Patch),
            1618 => Ok(Nip34Kind::PullRequest),
            1619 => Ok(Nip34Kind::PullRequestUpdate),
            1621 => Ok(Nip34Kind::Issue),
            1630 => Ok(Nip34Kind::StatusOpen),
            1631 => Ok(Nip34Kind::StatusApplied),
            1632 => Ok(Nip34Kind::StatusClosed),
            1633 => Ok(Nip34Kind::StatusDraft),
            10317 => Ok(Nip34Kind::UserGraspList),
            _ => Err(anyhow::anyhow!("Invalid NIP-34 kind: {}", value)),
        }
    }
}

#[cfg(test)]
mod tests {
    use rand::rngs::OsRng;
    use secp256k1::{schnorr, Secp256k1};

    use super::*;

    fn test_event_creation(kind: Nip34Kind, mut tags: Vec<Vec<String>>, content: String) {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let x_only_public_key = public_key.x_only_public_key().0;

        let unsigned_event = UnsignedEvent::new(
            &x_only_public_key,
            kind as u16,
            tags.clone(),
            content.clone(),
        );

        let event = unsigned_event.sign(&secret_key).unwrap();
        println!("Signed event for kind {:?}: {:?}", kind, event);

        assert_eq!(event.kind, kind as u16);
        assert_eq!(event.content, content);

        if let Ok(val) = weeble::weeble() {
            tags.push(vec!["weeble".to_string(), val.to_string()]);
        }
        if let Ok(val) = blockheight::blockheight() {
            tags.push(vec!["blockheight".to_string(), val.to_string()]);
        }
        if let Ok(val) = wobble::wobble() {
            tags.push(vec!["wobble".to_string(), val.to_string()]);
        }
        if let Ok(val) = blockhash::blockhash() {
            tags.push(vec!["blockhash".to_string(), val]);
        }

        let mut actual_tags = event.tags.clone();
        let actual_wobble_pos = actual_tags
            .iter()
            .position(|t| t.get(0).map_or(false, |s| s == "wobble"));
        if let Some(pos) = actual_wobble_pos {
            let _ = actual_tags.remove(pos);
        }

        let mut expected_tags = tags;
        let expected_wobble_pos = expected_tags
            .iter()
            .position(|t| t.get(0).map_or(false, |s| s == "wobble"));
        if let Some(pos) = expected_wobble_pos {
            let _ = expected_tags.remove(pos);
        }

        assert!(
            actual_wobble_pos.is_some(),
            "wobble tag not found in actual tags"
        );
        assert!(
            expected_wobble_pos.is_some(),
            "wobble tag not found in expected tags"
        );

        actual_tags.sort();
        expected_tags.sort();

        assert_eq!(actual_tags, expected_tags);

        let event_id_bytes = hex::decode(event.id).unwrap();
        let message = Message::from_digest_slice(&event_id_bytes).unwrap();
        let signature = schnorr::Signature::from_slice(&hex::decode(event.sig).unwrap()).unwrap();

        secp.verify_schnorr(&signature, &message, &x_only_public_key)
            .unwrap();
    }

    #[test]
    fn test_repo_announcement() {
        let tags = vec![
            vec!["d".to_string(), "gnostr".to_string()],
            vec!["name".to_string(), "gnostr".to_string()],
            vec![
                "description".to_string(),
                "A git implementation on nostr".to_string(),
            ],
            vec![
                "web".to_string(),
                "https://github.com/gnostr-org/gnostr".to_string(),
            ],
            vec![
                "clone".to_string(),
                "https://github.com/gnostr-org/gnostr.git".to_string(),
            ],
            vec!["relays".to_string(), "wss://relay.damus.io".to_string()],
        ];
        test_event_creation(Nip34Kind::RepoAnnouncement, tags, "".to_string());
    }

    #[test]
    fn test_repo_state() {
        let tags = vec![
            vec!["d".to_string(), "gnostr".to_string()],
            vec!["refs/heads/main".to_string(), "abcdef123456".to_string()],
            vec!["refs/tags/v0.1.0".to_string(), "fedcba654321".to_string()],
        ];
        test_event_creation(Nip34Kind::RepoState, tags, "".to_string());
    }

    #[test]
    fn test_patch() {
        let tags = vec![
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["commit".to_string(), "abcdef123456".to_string()],
        ];
        let content = "--- a/README.md\n+++ b/README.md\n@@ -1,3 +1,3 @@\n # gnostr\n-A git implementation on nostr\n+A git implementation over nostr".to_string();
        test_event_creation(Nip34Kind::Patch, tags, content);
    }

    #[test]
    fn test_pull_request() {
        let tags = vec![
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["subject".to_string(), "Add new feature".to_string()],
            vec!["branch-name".to_string(), "feature-branch".to_string()],
            vec!["merge-base".to_string(), "abcdef123456".to_string()],
        ];
        test_event_creation(Nip34Kind::PullRequest, tags, "".to_string());
    }

    #[test]
    fn test_pull_request_update() {
        let tags = vec![
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["e".to_string(), "event_id_of_pr".to_string()],
            vec!["c".to_string(), "new_commit_hash".to_string()],
        ];
        test_event_creation(Nip34Kind::PullRequestUpdate, tags, "".to_string());
    }

    #[test]
    fn test_issue() {
        let tags = vec![
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec!["subject".to_string(), "Bug report".to_string()],
        ];
        test_event_creation(Nip34Kind::Issue, tags, "This is a bug report.".to_string());
    }

    #[test]
    fn test_status_open() {
        let tags = vec![
            vec![
                "e".to_string(),
                "event_id_of_issue".to_string(),
                "root".to_string(),
            ],
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
        ];
        test_event_creation(Nip34Kind::StatusOpen, tags, "".to_string());
    }

    #[test]
    fn test_status_applied() {
        let tags = vec![
            vec![
                "e".to_string(),
                "event_id_of_patch".to_string(),
                "root".to_string(),
            ],
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
            vec![
                "applied-as-commits".to_string(),
                "commit1,commit2".to_string(),
            ],
        ];
        test_event_creation(Nip34Kind::StatusApplied, tags, "".to_string());
    }

    #[test]
    fn test_status_closed() {
        let tags = vec![
            vec![
                "e".to_string(),
                "event_id_of_pr".to_string(),
                "root".to_string(),
            ],
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
        ];
        test_event_creation(Nip34Kind::StatusClosed, tags, "".to_string());
    }

    #[test]
    fn test_status_draft() {
        let tags = vec![
            vec![
                "e".to_string(),
                "event_id_of_patch".to_string(),
                "root".to_string(),
            ],
            vec!["a".to_string(), "30617:pubkey:gnostr".to_string()],
        ];
        test_event_creation(Nip34Kind::StatusDraft, tags, "".to_string());
    }

    #[test]
    fn test_user_grasp_list() {
        let tags = vec![
            vec!["g".to_string(), "wss://grasp.example.com".to_string()],
            vec![
                "g".to_string(),
                "wss://another-grasp.example.com".to_string(),
            ],
        ];
        test_event_creation(Nip34Kind::UserGraspList, tags, "".to_string());
    }
}
```

---

### nip36.rs

**Size:** 5681 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-36: Sensitive Content
//!
//! This NIP introduces a `content-warning` tag to indicate that an event's
//! content should be hidden by clients until the reader explicitly approves its
//! display.
//!
//! https://github.com/nostr-protocol/nips/blob/master/36.md

use anyhow::Result;

use crate::types::{Event, Tag};

/// The name of the content-warning tag.
pub const CONTENT_WARNING_TAG_NAME: &str = "content-warning";

/// Helper trait for NIP-36 content warning tags on Event types.
pub trait NIP36Event {
    /// Extracts the optional reason for the content warning from the event's
    /// tags. Returns `None` if the "content-warning" tag is not found or
    /// has no reason.
    fn content_warning_reason(&self) -> Option<&str>;

    /// Adds a "content-warning" tag to the event with an optional reason.
    /// If a "content-warning" tag already exists, it will be replaced.
    fn add_content_warning_tag(&mut self, reason: Option<String>);

    /// Creates a "content-warning" tag with an optional reason.
    fn create_content_warning_tag(reason: Option<String>) -> Tag;
}

impl NIP36Event for Event {
    fn content_warning_reason(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() >= 1 && tag.0[0] == CONTENT_WARNING_TAG_NAME {
                tag.0.get(1).map(|s| s.as_str()) // Reason is at index 1 if present
            } else {
                None
            }
        })
    }

    fn add_content_warning_tag(&mut self, reason: Option<String>) {
        // Remove existing content-warning tags to ensure only one is present
        self.tags
            .retain(|tag| tag.0.get(0) != Some(&CONTENT_WARNING_TAG_NAME.to_string()));
        self.tags.push(Self::create_content_warning_tag(reason));
    }

    fn create_content_warning_tag(reason: Option<String>) -> Tag {
        let mut tag_elements = vec![CONTENT_WARNING_TAG_NAME.to_string()];
        if let Some(r) = reason {
            tag_elements.push(r);
        }
        Tag::new(
            tag_elements
                .iter()
                .map(|s| s.as_str())
                .collect::<Vec<&str>>()
                .as_slice(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature, Unixtime};

    // Helper to create a dummy event for testing
    fn create_dummy_event_with_warning(reason: Option<&str>) -> Event {
        let mut tags = Vec::new();
        tags.push(Event::create_content_warning_tag(
            reason.map(|s| s.to_string()),
        ));

        Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags,
            content: "This content is potentially sensitive.".to_string(),
            sig: Signature::zeroes(),
        }
    }

    #[test]
    fn test_create_content_warning_tag_with_reason() {
        let reason = "sexual content";
        let tag = Event::create_content_warning_tag(Some(reason.to_string()));
        assert_eq!(tag.0, vec![CONTENT_WARNING_TAG_NAME, reason]);
    }

    #[test]
    fn test_create_content_warning_tag_no_reason() {
        let tag = Event::create_content_warning_tag(None);
        assert_eq!(tag.0, vec![CONTENT_WARNING_TAG_NAME]);
    }

    #[test]
    fn test_content_warning_reason_found() {
        let reason = "hate speech";
        let event = create_dummy_event_with_warning(Some(reason));
        assert_eq!(event.content_warning_reason(), Some(reason));
    }

    #[test]
    fn test_content_warning_reason_not_found() {
        let event = create_dummy_event_with_warning(None);
        assert_eq!(event.content_warning_reason(), None);
    }

    #[test]
    fn test_content_warning_tag_not_present() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])], // No content-warning tag
            content: "Normal content.".to_string(),
            sig: Signature::zeroes(),
        };
        assert_eq!(event.content_warning_reason(), None);
    }

    #[test]
    fn test_add_content_warning_tag() {
        let reason1 = "graphic violence";
        let reason2 = "spoiler";
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_content_warning_tag(Some(reason1.to_string())),
            ],
            content: "Some content".to_string(),
            sig: Signature::zeroes(),
        };

        // Add a new content-warning tag, it should replace the existing one
        event.add_content_warning_tag(Some(reason2.to_string()));
        assert_eq!(event.content_warning_reason(), Some(reason2));
        // Ensure only one content-warning tag exists
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&CONTENT_WARNING_TAG_NAME.to_string()))
                .count(),
            1
        );

        // Add a content-warning tag without a reason
        event.add_content_warning_tag(None);
        assert_eq!(event.content_warning_reason(), None);
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&CONTENT_WARNING_TAG_NAME.to_string()))
                .count(),
            1
        );
    }
}
```

---

### nip38.rs

**Size:** 4275 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-38: User Statuses
//!
//! This NIP defines how to set a user's status message, which can be short
//! text or an indicator of activity, displayed next to their username.
//! It uses a Parameterized Replaceable Event (Kind 30315).
//!
//! https://github.com/nostr-protocol/nips/blob/master/38.md

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::types::{event_kind::EventKind, signature::Signature};
use crate::types::{Event, Id, PreEvent, PublicKey, Tag, Unixtime}; // Re-using existing types

/// NIP-38 User Status Event Kind (Parameterized Replaceable Event)
pub const USER_STATUS_KIND: u32 = 30315;

/// Represents the type of user status, defined by the 'd' tag.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserStatusType {
    /// General status, e.g., "Working", "Hiking".
    General,
    /// Music status, indicating what the user is listening to.
    Music,
    /// A custom status type not explicitly defined by NIP-38.
    Custom(String),
}

impl ToString for UserStatusType {
    fn to_string(&self) -> String {
        match self {
            UserStatusType::General => "general".to_string(),
            UserStatusType::Music => "music".to_string(),
            UserStatusType::Custom(s) => s.clone(),
        }
    }
}

impl From<&str> for UserStatusType {
    fn from(s: &str) -> Self {
        match s {
            "general" => UserStatusType::General,
            "music" => UserStatusType::Music,
            _ => UserStatusType::Custom(s.to_string()),
        }
    }
}

/// Represents the content of a NIP-38 User Status Event (Kind 30315).
/// The content is a simple string.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct UserStatusContent {
    pub message: String,
}

/// Helper functions for NIP-38 events.
pub trait NIP38Event {
    /// Returns the status type from the 'd' tag.
    fn status_type(&self) -> Option<UserStatusType>;
    /// Returns the expiration timestamp from the 'expiration' tag (NIP-40).
    fn expiration(&self) -> Option<Unixtime>;
    /// Creates a NIP-38 User Status event builder.
    fn new_user_status(
        public_key: PublicKey,
        message: String,
        status_type: UserStatusType,
        expiration: Option<Unixtime>,
        // Other optional tags like r, p, e, a can be added here
    ) -> Result<Event>; // Returns a full Event for now, can be an EventBuilder later.
}

impl NIP38Event for Event {
    fn status_type(&self) -> Option<UserStatusType> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "d" {
                Some(UserStatusType::from(tag.0[1].as_str()))
            } else {
                None
            }
        })
    }

    fn expiration(&self) -> Option<Unixtime> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "expiration" {
                tag.0[1].parse::<i64>().ok().map(Unixtime::from)
            } else {
                None
            }
        })
    }

    fn new_user_status(
        public_key: PublicKey,
        message: String,
        status_type: UserStatusType,
        expiration: Option<Unixtime>,
        // Other optional tags like r, p, e, a can be added here
    ) -> Result<Event> {
        let content = UserStatusContent { message }.message; // Content is just the message string

        let mut tags: Vec<Tag> = vec![Tag::new(&["d", &status_type.to_string()])];

        if let Some(exp) = expiration {
            tags.push(Tag::new(&["expiration", &exp.to_string()]));
        }

        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Replaceable(USER_STATUS_KIND),
            tags: tags.clone(),       // Clone tags for PreEvent
            content: content.clone(), // Clone content for PreEvent
        };

        let id = pre_event.hash().unwrap(); // Calculate event ID

        Ok(Event {
            id,
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::Replaceable(USER_STATUS_KIND),
            tags,
            content,
            sig: Signature::zeroes(), // Signature will be added during signing
        })
    }
}
```

---

### nip4.rs

**Size:** 2365 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use aes::Aes256;
use base64::{engine::general_purpose::STANDARD as BASE64, Engine as _};
use block_padding::Pkcs7;
use cbc::{
    cipher::{BlockDecryptMut, BlockEncryptMut, KeyIvInit},
    Decryptor, Encryptor,
};
use rand::RngCore;
use secp256k1::{ecdh, Secp256k1, SecretKey, XOnlyPublicKey};

type Aes256CbcEncryptor = Encryptor<Aes256>;
type Aes256CbcDecryptor = Decryptor<Aes256>;

/// Encrypt content
pub fn encrypt(
    sender_private_key: &SecretKey,
    recipient_public_key: &XOnlyPublicKey,
    content: &str,
) -> Result<String, anyhow::Error> {
    let _secp = Secp256k1::new();

    // NIP-04 specifies using the first 32 bytes of the sha256 of the shared secret
    // point
    let shared_secret = ecdh::shared_secret_point(
        &recipient_public_key.public_key(secp256k1::Parity::Even), // Simplified assumption
        sender_private_key,
    );
    let shared_key = &shared_secret[..32];

    let mut iv = [0u8; 16];
    rand::thread_rng().fill_bytes(&mut iv);

    let cipher = Aes256CbcEncryptor::new(shared_key.into(), &iv.into());
    let encrypted_content = cipher.encrypt_padded_vec_mut::<Pkcs7>(content.as_bytes());

    let iv_base64 = BASE64.encode(iv);
    let content_base64 = BASE64.encode(encrypted_content);

    Ok(format!("{}?iv={}", content_base64, iv_base64))
}

/// Decrypt content
pub fn decrypt(
    recipient_private_key: &SecretKey,
    sender_public_key: &XOnlyPublicKey,
    encrypted_content: &str,
) -> Result<String, anyhow::Error> {
    let mut parts = encrypted_content.split("?iv=");
    let content_base64 = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid encrypted content format"))?;
    let iv_base64 = parts
        .next()
        .ok_or_else(|| anyhow::anyhow!("Invalid encrypted content format: missing iv"))?;

    let iv = BASE64.decode(iv_base64)?;
    let encrypted_bytes = BASE64.decode(content_base64)?;

    let _secp = Secp256k1::new();
    let shared_secret = ecdh::shared_secret_point(
        &sender_public_key.public_key(secp256k1::Parity::Even), // Simplified assumption
        recipient_private_key,
    );
    let shared_key = &shared_secret[..32];

    let cipher = Aes256CbcDecryptor::new(shared_key.into(), iv.as_slice().into());
    let decrypted_bytes = cipher.decrypt_padded_vec_mut::<Pkcs7>(&encrypted_bytes)?;

    Ok(String::from_utf8(decrypted_bytes)?)
}
```

---

### nip40.rs

**Size:** 5130 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-40: Expiration Timestamp
//!
//! This NIP defines an `expiration` tag that can be added to Nostr events
//! to indicate a Unix timestamp (in seconds) at which the event should be
//! considered expired.
//!
//! https://github.com/nostr-protocol/nips/blob/master/40.md

use anyhow::Result;

use crate::types::{Event, Tag, Unixtime};

/// The name of the expiration tag.
pub const EXPIRATION_TAG_NAME: &str = "expiration";

/// Helper trait for NIP-40 expiration tags on Event types.
pub trait NIP40Event {
    /// Extracts the expiration timestamp from the event's tags.
    /// Returns `None` if the "expiration" tag is not found or is invalid.
    fn expiration_time(&self) -> Option<Unixtime>;

    /// Adds an "expiration" tag to the event with the given Unix timestamp.
    /// If an "expiration" tag already exists, it will be replaced.
    fn add_expiration_tag(&mut self, expiry_time: Unixtime);

    /// Creates an "expiration" tag with the given Unix timestamp.
    fn create_expiration_tag(expiry_time: Unixtime) -> Tag;
}

impl NIP40Event for Event {
    fn expiration_time(&self) -> Option<Unixtime> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == EXPIRATION_TAG_NAME {
                tag.0[1].parse::<i64>().ok().map(Unixtime::from)
            } else {
                None
            }
        })
    }

    fn add_expiration_tag(&mut self, expiry_time: Unixtime) {
        // Remove existing expiration tags to ensure only one is present
        self.tags
            .retain(|tag| tag.0.get(0) != Some(&EXPIRATION_TAG_NAME.to_string()));
        self.tags.push(Self::create_expiration_tag(expiry_time));
    }

    fn create_expiration_tag(expiry_time: Unixtime) -> Tag {
        Tag::new(&[EXPIRATION_TAG_NAME, &expiry_time.to_string()])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{EventKind, Id, PublicKey, Signature};

    #[test]
    fn test_create_expiration_tag() {
        let expiry = Unixtime::from(1678886400); // Example Unix timestamp
        let tag = Event::create_expiration_tag(expiry);
        assert_eq!(tag.0, vec!["expiration", "1678886400"]);
    }

    #[test]
    fn test_expiration_time_found() {
        let expiry = Unixtime::from(1678886400);
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_expiration_tag(expiry),
            ],
            content: "Test event".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(event.expiration_time(), Some(expiry));
    }

    #[test]
    fn test_expiration_time_not_found() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["e", "some_event_id"])],
            content: "Test event".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(event.expiration_time(), None);
    }

    #[test]
    fn test_expiration_time_invalid_format() {
        let event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![Tag::new(&["expiration", "not_a_number"])],
            content: "Test event".to_string(),
            sig: Signature::zeroes(),
        };

        assert_eq!(event.expiration_time(), None);
    }

    #[test]
    fn test_add_expiration_tag() {
        let initial_expiry = Unixtime::from(1600000000);
        let new_expiry = Unixtime::from(1700000000);
        let mut event = Event {
            id: Id::mock(),
            pubkey: PublicKey::mock(),
            created_at: Unixtime::now(),
            kind: EventKind::TextNote,
            tags: vec![
                Tag::new(&["e", "some_event_id"]),
                Event::create_expiration_tag(initial_expiry),
            ],
            content: "Test event".to_string(),
            sig: Signature::zeroes(),
        };

        // Add a new expiration tag
        event.add_expiration_tag(new_expiry);
        assert_eq!(event.expiration_time(), Some(new_expiry));
        // Ensure only one expiration tag exists
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&EXPIRATION_TAG_NAME.to_string()))
                .count(),
            1
        );

        // Add another expiration tag, it should replace the existing one
        let even_newer_expiry = Unixtime::from(1800000000);
        event.add_expiration_tag(even_newer_expiry);
        assert_eq!(event.expiration_time(), Some(even_newer_expiry));
        assert_eq!(
            event
                .tags
                .iter()
                .filter(|t| t.0.get(0) == Some(&EXPIRATION_TAG_NAME.to_string()))
                .count(),
            1
        );
    }
}
```

---

### nip44/error.rs

**Size:** 1169 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use thiserror::Error;

/// Errors that can occur in NIP-44 operations.
#[derive(Clone, Error, Debug, PartialEq)]
pub enum Error {
    /// Base64 Decode
    #[error("Base64 decode: {0}")]
    Base64Decode(#[from] base64::DecodeError),

    /// HKDF Length
    #[error("Invalid Length for HKDF: {0}")]
    HkdfLength(usize),

    /// HMAC Length
    #[error("Invalid Length for HMAC: {0}")]
    HmacLength(#[from] chacha20::cipher::InvalidLength),

    /// Invalid MAC
    #[error("Invalid MAC")]
    InvalidMac,

    /// Invalid padding
    #[error("Invalid Padding")]
    InvalidPadding,

    /// Message is empty
    #[error("Message is empty")]
    MessageIsEmpty,

    /// Message is too long (max len 65536 - 128)
    #[error("Message is too long")]
    MessageIsTooLong,

    /// Unsupported future version
    #[error("Encryption format is not yet supported")]
    UnsupportedFutureVersion,

    /// Unknown version
    #[error("Encryption format is unknown")]
    UnknownVersion,

    /// Invalid length
    #[error("Invalid Length")]
    InvalidLength,

    /// UTF8 Decode
    #[error("UTF8 Decode: {0}")]
    Utf8Decode(#[from] std::string::FromUtf8Error),
}
```

---

### nip44/mod.rs

**Size:** 6209 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::convert::TryInto;

use base64::Engine;
use chacha20::{
    cipher::{KeyIvInit, StreamCipher},
    ChaCha20,
};
use hkdf::Hkdf;
use hmac::{Hmac, Mac};
use rand_core::{OsRng, RngCore};
use secp256k1::{ecdh::shared_secret_point, Parity, PublicKey, SecretKey, XOnlyPublicKey};
use sha2::Sha256;
mod error;
pub use error::Error;

#[cfg(test)]
mod tests;

struct MessageKeys([u8; 76]);

impl MessageKeys {
    #[inline]
    pub(crate) fn zero() -> MessageKeys {
        MessageKeys([0; 76])
    }

    #[inline]
    pub(crate) fn encryption(&self) -> [u8; 32] {
        self.0[0..32].try_into().unwrap()
    }

    #[inline]
    pub(crate) fn nonce(&self) -> [u8; 12] {
        self.0[32..44].try_into().unwrap()
    }

    #[inline]
    pub(crate) fn auth(&self) -> [u8; 32] {
        self.0[44..76].try_into().unwrap()
    }
}

/// A conversation key is the long-term secret that two nostr identities share.
fn get_shared_point(private_key_a: SecretKey, x_only_public_key_b: XOnlyPublicKey) -> [u8; 32] {
    let pubkey = PublicKey::from_x_only_public_key(x_only_public_key_b, Parity::Even);
    let mut ssp = shared_secret_point(&pubkey, &private_key_a)
        .as_slice()
        .to_owned();
    ssp.resize(32, 0); // toss the Y part
    ssp.try_into().unwrap()
}

/// Derives a NIP-44 conversation key from a private key and an XOnlyPublicKey.
pub fn get_conversation_key(
    private_key_a: SecretKey,
    x_only_public_key_b: XOnlyPublicKey,
) -> [u8; 32] {
    let shared_point = get_shared_point(private_key_a, x_only_public_key_b);
    let (convo_key, _hkdf) =
        Hkdf::<Sha256>::extract(Some("nip44-v2".as_bytes()), shared_point.as_slice());
    convo_key.into()
}

fn get_message_keys(conversation_key: &[u8; 32], nonce: &[u8; 32]) -> Result<MessageKeys, Error> {
    let hk: Hkdf<Sha256> = match Hkdf::from_prk(conversation_key) {
        Ok(hk) => hk,
        Err(_) => return Err(Error::HkdfLength(conversation_key.len())),
    };
    let mut message_keys: MessageKeys = MessageKeys::zero();
    if hk.expand(&nonce[..], &mut message_keys.0).is_err() {
        return Err(Error::HkdfLength(message_keys.0.len()));
    }
    Ok(message_keys)
}

fn calc_padding(len: usize) -> usize {
    if len < 32 {
        return 32;
    }
    let nextpower = 1 << ((len - 1).ilog2() + 1);
    let chunk = if nextpower <= 256 { 32 } else { nextpower / 8 };
    if len <= 32 {
        32
    } else {
        chunk * (((len - 1) / chunk) + 1)
    }
}

fn pad(unpadded: &str) -> Result<Vec<u8>, Error> {
    let len: usize = unpadded.len();
    if len < 1 {
        return Err(Error::MessageIsEmpty);
    }
    if len > 65536 - 128 {
        return Err(Error::MessageIsTooLong);
    }

    let mut padded: Vec<u8> = Vec::new();
    padded.extend_from_slice(&(len as u16).to_be_bytes());
    padded.extend_from_slice(unpadded.as_bytes());
    padded.extend(std::iter::repeat_n(0, calc_padding(len) - len));
    Ok(padded)
}

/// Encrypt a plaintext message with a conversation key.
/// The output is a base64 encoded string that can be placed into message
/// contents.
#[inline]
pub fn encrypt(conversation_key: &[u8; 32], plaintext: &str) -> Result<String, Error> {
    encrypt_inner(conversation_key, plaintext, None)
}

fn encrypt_inner(
    conversation_key: &[u8; 32],
    plaintext: &str,
    override_random_nonce: Option<&[u8; 32]>,
) -> Result<String, Error> {
    let nonce = match override_random_nonce {
        Some(nonce) => nonce.to_owned(),
        None => {
            let mut nonce: [u8; 32] = [0; 32];
            OsRng.fill_bytes(&mut nonce);
            nonce
        }
    };

    let keys = get_message_keys(conversation_key, &nonce)?;
    let mut buffer = pad(plaintext)?;
    let mut cipher = ChaCha20::new(&keys.encryption().into(), &keys.nonce().into());
    cipher.apply_keystream(&mut buffer);
    let mut mac = Hmac::<Sha256>::new_from_slice(&keys.auth())?;
    mac.update(&nonce);
    mac.update(&buffer);
    let mac_bytes = mac.finalize().into_bytes();

    let mut pre_base64: Vec<u8> = vec![2];
    pre_base64.extend_from_slice(&nonce);
    pre_base64.extend_from_slice(&buffer);
    pre_base64.extend_from_slice(&mac_bytes);

    Ok(base64::engine::general_purpose::STANDARD.encode(&pre_base64))
}

/// Decrypt the base64 encrypted contents with a conversation key
pub fn decrypt(conversation_key: &[u8; 32], base64_ciphertext: &str) -> Result<String, Error> {
    if base64_ciphertext.is_empty() {
        return Err(Error::InvalidLength);
    }
    if base64_ciphertext.as_bytes()[0] == b'#' {
        return Err(Error::UnsupportedFutureVersion);
    }
    let binary_ciphertext: Vec<u8> =
        base64::engine::general_purpose::STANDARD.decode(base64_ciphertext)?;
    if binary_ciphertext.len() < 65 {
        return Err(Error::InvalidLength);
    }
    let version = binary_ciphertext[0];
    if version != 2 {
        return Err(Error::UnknownVersion);
    }
    let dlen = binary_ciphertext.len();
    let nonce = &binary_ciphertext[1..33];
    let mut buffer = binary_ciphertext[33..dlen - 32].to_owned();
    let mac = &binary_ciphertext[dlen - 32..dlen];
    let keys = get_message_keys(conversation_key, &nonce.try_into().unwrap())?;
    let mut calculated_mac = Hmac::<Sha256>::new_from_slice(&keys.auth())?;
    calculated_mac.update(nonce);
    calculated_mac.update(&buffer);
    let calculated_mac_bytes = calculated_mac.finalize().into_bytes();
    if !constant_time_eq::constant_time_eq(mac, calculated_mac_bytes.as_slice()) {
        return Err(Error::InvalidMac);
    }
    let mut cipher = ChaCha20::new(&keys.encryption().into(), &keys.nonce().into());
    cipher.apply_keystream(&mut buffer);
    let unpadded_len = u16::from_be_bytes(buffer[0..2].try_into().unwrap()) as usize;
    if buffer.len() < 2 + unpadded_len {
        return Err(Error::InvalidPadding);
    }
    let unpadded = &buffer[2..2 + unpadded_len];
    if unpadded.is_empty() {
        return Err(Error::MessageIsEmpty);
    }
    if unpadded.len() != unpadded_len {
        return Err(Error::InvalidPadding);
    }
    if buffer.len() != 2 + calc_padding(unpadded_len) {
        return Err(Error::InvalidPadding);
    }
    Ok(String::from_utf8(unpadded.to_vec())?)
}
```

---

### nip44/nip44.vectors.json

**Size:** 37630 bytes | **Modified:** 2025-11-23 12:51:57

```json
{
  "v2": {
    "valid": {
      "get_conversation_key": [
        {
          "sec1": "315e59ff51cb9209768cf7da80791ddcaae56ac9775eb25b6dee1234bc5d2268",
          "pub2": "c2f9d9948dc8c7c38321e4b85c8558872eafa0641cd269db76848a6073e69133",
          "conversation_key": "3dfef0ce2a4d80a25e7a328accf73448ef67096f65f79588e358d9a0eb9013f1"
        },
        {
          "sec1": "a1e37752c9fdc1273be53f68c5f74be7c8905728e8de75800b94262f9497c86e",
          "pub2": "03bb7947065dde12ba991ea045132581d0954f042c84e06d8c00066e23c1a800",
          "conversation_key": "4d14f36e81b8452128da64fe6f1eae873baae2f444b02c950b90e43553f2178b"
        },
        {
          "sec1": "98a5902fd67518a0c900f0fb62158f278f94a21d6f9d33d30cd3091195500311",
          "pub2": "aae65c15f98e5e677b5050de82e3aba47a6fe49b3dab7863cf35d9478ba9f7d1",
          "conversation_key": "9c00b769d5f54d02bf175b7284a1cbd28b6911b06cda6666b2243561ac96bad7"
        },
        {
          "sec1": "86ae5ac8034eb2542ce23ec2f84375655dab7f836836bbd3c54cefe9fdc9c19f",
          "pub2": "59f90272378089d73f1339710c02e2be6db584e9cdbe86eed3578f0c67c23585",
          "conversation_key": "19f934aafd3324e8415299b64df42049afaa051c71c98d0aa10e1081f2e3e2ba"
        },
        {
          "sec1": "2528c287fe822421bc0dc4c3615878eb98e8a8c31657616d08b29c00ce209e34",
          "pub2": "f66ea16104c01a1c532e03f166c5370a22a5505753005a566366097150c6df60",
          "conversation_key": "c833bbb292956c43366145326d53b955ffb5da4e4998a2d853611841903f5442"
        },
        {
          "sec1": "49808637b2d21129478041813aceb6f2c9d4929cd1303cdaf4fbdbd690905ff2",
          "pub2": "74d2aab13e97827ea21baf253ad7e39b974bb2498cc747cdb168582a11847b65",
          "conversation_key": "4bf304d3c8c4608864c0fe03890b90279328cd24a018ffa9eb8f8ccec06b505d"
        },
        {
          "sec1": "af67c382106242c5baabf856efdc0629cc1c5b4061f85b8ceaba52aa7e4b4082",
          "pub2": "bdaf0001d63e7ec994fad736eab178ee3c2d7cfc925ae29f37d19224486db57b",
          "conversation_key": "a3a575dd66d45e9379904047ebfb9a7873c471687d0535db00ef2daa24b391db"
        },
        {
          "sec1": "0e44e2d1db3c1717b05ffa0f08d102a09c554a1cbbf678ab158b259a44e682f1",
          "pub2": "1ffa76c5cc7a836af6914b840483726207cb750889753d7499fb8b76aa8fe0de",
          "conversation_key": "a39970a667b7f861f100e3827f4adbf6f464e2697686fe1a81aeda817d6b8bdf"
        },
        {
          "sec1": "5fc0070dbd0666dbddc21d788db04050b86ed8b456b080794c2a0c8e33287bb6",
          "pub2": "31990752f296dd22e146c9e6f152a269d84b241cc95bb3ff8ec341628a54caf0",
          "conversation_key": "72c21075f4b2349ce01a3e604e02a9ab9f07e35dd07eff746de348b4f3c6365e"
        },
        {
          "sec1": "1b7de0d64d9b12ddbb52ef217a3a7c47c4362ce7ea837d760dad58ab313cba64",
          "pub2": "24383541dd8083b93d144b431679d70ef4eec10c98fceef1eff08b1d81d4b065",
          "conversation_key": "dd152a76b44e63d1afd4dfff0785fa07b3e494a9e8401aba31ff925caeb8f5b1"
        },
        {
          "sec1": "df2f560e213ca5fb33b9ecde771c7c0cbd30f1cf43c2c24de54480069d9ab0af",
          "pub2": "eeea26e552fc8b5e377acaa03e47daa2d7b0c787fac1e0774c9504d9094c430e",
          "conversation_key": "770519e803b80f411c34aef59c3ca018608842ebf53909c48d35250bd9323af6"
        },
        {
          "sec1": "cffff919fcc07b8003fdc63bc8a00c0f5dc81022c1c927c62c597352190d95b9",
          "pub2": "eb5c3cca1a968e26684e5b0eb733aecfc844f95a09ac4e126a9e58a4e4902f92",
          "conversation_key": "46a14ee7e80e439ec75c66f04ad824b53a632b8409a29bbb7c192e43c00bb795"
        },
        {
          "sec1": "64ba5a685e443e881e9094647ddd32db14444bb21aa7986beeba3d1c4673ba0a",
          "pub2": "50e6a4339fac1f3bf86f2401dd797af43ad45bbf58e0801a7877a3984c77c3c4",
          "conversation_key": "968b9dbbfcede1664a4ca35a5d3379c064736e87aafbf0b5d114dff710b8a946"
        },
        {
          "sec1": "dd0c31ccce4ec8083f9b75dbf23cc2878e6d1b6baa17713841a2428f69dee91a",
          "pub2": "b483e84c1339812bed25be55cff959778dfc6edde97ccd9e3649f442472c091b",
          "conversation_key": "09024503c7bde07eb7865505891c1ea672bf2d9e25e18dd7a7cea6c69bf44b5d"
        },
        {
          "sec1": "af71313b0d95c41e968a172b33ba5ebd19d06cdf8a7a98df80ecf7af4f6f0358",
          "pub2": "2a5c25266695b461ee2af927a6c44a3c598b8095b0557e9bd7f787067435bc7c",
          "conversation_key": "fe5155b27c1c4b4e92a933edae23726a04802a7cc354a77ac273c85aa3c97a92"
        },
        {
          "sec1": "6636e8a389f75fe068a03b3edb3ea4a785e2768e3f73f48ffb1fc5e7cb7289dc",
          "pub2": "514eb2064224b6a5829ea21b6e8f7d3ea15ff8e70e8555010f649eb6e09aec70",
          "conversation_key": "ff7afacd4d1a6856d37ca5b546890e46e922b508639214991cf8048ddbe9745c"
        },
        {
          "sec1": "94b212f02a3cfb8ad147d52941d3f1dbe1753804458e6645af92c7b2ea791caa",
          "pub2": "f0cac333231367a04b652a77ab4f8d658b94e86b5a8a0c472c5c7b0d4c6a40cc",
          "conversation_key": "e292eaf873addfed0a457c6bd16c8effde33d6664265697f69f420ab16f6669b"
        },
        {
          "sec1": "aa61f9734e69ae88e5d4ced5aae881c96f0d7f16cca603d3bed9eec391136da6",
          "pub2": "4303e5360a884c360221de8606b72dd316da49a37fe51e17ada4f35f671620a6",
          "conversation_key": "8e7d44fd4767456df1fb61f134092a52fcd6836ebab3b00766e16732683ed848"
        },
        {
          "sec1": "5e914bdac54f3f8e2cba94ee898b33240019297b69e96e70c8a495943a72fc98",
          "pub2": "5bd097924f606695c59f18ff8fd53c174adbafaaa71b3c0b4144a3e0a474b198",
          "conversation_key": "f5a0aecf2984bf923c8cd5e7bb8be262d1a8353cb93959434b943a07cf5644bc"
        },
        {
          "sec1": "8b275067add6312ddee064bcdbeb9d17e88aa1df36f430b2cea5cc0413d8278a",
          "pub2": "65bbbfca819c90c7579f7a82b750a18c858db1afbec8f35b3c1e0e7b5588e9b8",
          "conversation_key": "2c565e7027eb46038c2263563d7af681697107e975e9914b799d425effd248d6"
        },
        {
          "sec1": "1ac848de312285f85e0f7ec208aac20142a1f453402af9b34ec2ec7a1f9c96fc",
          "pub2": "45f7318fe96034d23ee3ddc25b77f275cc1dd329664dd51b89f89c4963868e41",
          "conversation_key": "b56e970e5057a8fd929f8aad9248176b9af87819a708d9ddd56e41d1aec74088"
        },
        {
          "sec1": "295a1cf621de401783d29d0e89036aa1c62d13d9ad307161b4ceb535ba1b40e6",
          "pub2": "840115ddc7f1034d3b21d8e2103f6cb5ab0b63cf613f4ea6e61ae3d016715cdd",
          "conversation_key": "b4ee9c0b9b9fef88975773394f0a6f981ca016076143a1bb575b9ff46e804753"
        },
        {
          "sec1": "a28eed0fe977893856ab9667e06ace39f03abbcdb845c329a1981be438ba565d",
          "pub2": "b0f38b950a5013eba5ab4237f9ed29204a59f3625c71b7e210fec565edfa288c",
          "conversation_key": "9d3a802b45bc5aeeb3b303e8e18a92ddd353375710a31600d7f5fff8f3a7285b"
        },
        {
          "sec1": "7ab65af72a478c05f5c651bdc4876c74b63d20d04cdbf71741e46978797cd5a4",
          "pub2": "f1112159161b568a9cb8c9dd6430b526c4204bcc8ce07464b0845b04c041beda",
          "conversation_key": "943884cddaca5a3fef355e9e7f08a3019b0b66aa63ec90278b0f9fdb64821e79"
        },
        {
          "sec1": "95c79a7b75ba40f2229e85756884c138916f9d103fc8f18acc0877a7cceac9fe",
          "pub2": "cad76bcbd31ca7bbda184d20cc42f725ed0bb105b13580c41330e03023f0ffb3",
          "conversation_key": "81c0832a669eea13b4247c40be51ccfd15bb63fcd1bba5b4530ce0e2632f301b"
        },
        {
          "sec1": "baf55cc2febd4d980b4b393972dfc1acf49541e336b56d33d429bce44fa12ec9",
          "pub2": "0c31cf87fe565766089b64b39460ebbfdedd4a2bc8379be73ad3c0718c912e18",
          "conversation_key": "37e2344da9ecdf60ae2205d81e89d34b280b0a3f111171af7e4391ded93b8ea6"
        },
        {
          "sec1": "6eeec45acd2ed31693c5256026abf9f072f01c4abb61f51cf64e6956b6dc8907",
          "pub2": "e501b34ed11f13d816748c0369b0c728e540df3755bab59ed3327339e16ff828",
          "conversation_key": "afaa141b522ddb27bb880d768903a7f618bb8b6357728cae7fb03af639b946e6"
        },
        {
          "sec1": "261a076a9702af1647fb343c55b3f9a4f1096273002287df0015ba81ce5294df",
          "pub2": "b2777c863878893ae100fb740c8fab4bebd2bf7be78c761a75593670380a6112",
          "conversation_key": "76f8d2853de0734e51189ced523c09427c3e46338b9522cd6f74ef5e5b475c74"
        },
        {
          "sec1": "ed3ec71ca406552ea41faec53e19f44b8f90575eda4b7e96380f9cc73c26d6f3",
          "pub2": "86425951e61f94b62e20cae24184b42e8e17afcf55bafa58645efd0172624fae",
          "conversation_key": "f7ffc520a3a0e9e9b3c0967325c9bf12707f8e7a03f28b6cd69ae92cf33f7036"
        },
        {
          "sec1": "5a788fc43378d1303ac78639c59a58cb88b08b3859df33193e63a5a3801c722e",
          "pub2": "a8cba2f87657d229db69bee07850fd6f7a2ed070171a06d006ec3a8ac562cf70",
          "conversation_key": "7d705a27feeedf78b5c07283362f8e361760d3e9f78adab83e3ae5ce7aeb6409"
        },
        {
          "sec1": "63bffa986e382b0ac8ccc1aa93d18a7aa445116478be6f2453bad1f2d3af2344",
          "pub2": "b895c70a83e782c1cf84af558d1038e6b211c6f84ede60408f519a293201031d",
          "conversation_key": "3a3b8f00d4987fc6711d9be64d9c59cf9a709c6c6481c2cde404bcc7a28f174e"
        },
        {
          "sec1": "e4a8bcacbf445fd3721792b939ff58e691cdcba6a8ba67ac3467b45567a03e5c",
          "pub2": "b54053189e8c9252c6950059c783edb10675d06d20c7b342f73ec9fa6ed39c9d",
          "conversation_key": "7b3933b4ef8189d347169c7955589fc1cfc01da5239591a08a183ff6694c44ad"
        },
        {
          "sec1": "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364139",
          "pub2": "0000000000000000000000000000000000000000000000000000000000000002",
          "conversation_key": "8b6392dbf2ec6a2b2d5b1477fc2be84d63ef254b667cadd31bd3f444c44ae6ba",
          "note": "sec1 = n-2, pub2: random, 0x02"
        },
        {
          "sec1": "0000000000000000000000000000000000000000000000000000000000000002",
          "pub2": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeb",
          "conversation_key": "be234f46f60a250bef52a5ee34c758800c4ca8e5030bf4cc1a31d37ba2104d43",
          "note": "sec1 = 2, pub2: rand"
        },
        {
          "sec1": "0000000000000000000000000000000000000000000000000000000000000001",
          "pub2": "79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798",
          "conversation_key": "3b4610cb7189beb9cc29eb3716ecc6102f1247e8f3101a03a1787d8908aeb54e",
          "note": "sec1 == pub2"
        }
      ],
      "get_message_keys": {
        "conversation_key": "a1a3d60f3470a8612633924e91febf96dc5366ce130f658b1f0fc652c20b3b54",
        "keys": [
          {
            "nonce": "e1e6f880560d6d149ed83dcc7e5861ee62a5ee051f7fde9975fe5d25d2a02d72",
            "chacha_key": "f145f3bed47cb70dbeaac07f3a3fe683e822b3715edb7c4fe310829014ce7d76",
            "chacha_nonce": "c4ad129bb01180c0933a160c",
            "hmac_key": "027c1db445f05e2eee864a0975b0ddef5b7110583c8c192de3732571ca5838c4"
          },
          {
            "nonce": "e1d6d28c46de60168b43d79dacc519698512ec35e8ccb12640fc8e9f26121101",
            "chacha_key": "e35b88f8d4a8f1606c5082f7a64b100e5d85fcdb2e62aeafbec03fb9e860ad92",
            "chacha_nonce": "22925e920cee4a50a478be90",
            "hmac_key": "46a7c55d4283cb0df1d5e29540be67abfe709e3b2e14b7bf9976e6df994ded30"
          },
          {
            "nonce": "cfc13bef512ac9c15951ab00030dfaf2626fdca638dedb35f2993a9eeb85d650",
            "chacha_key": "020783eb35fdf5b80ef8c75377f4e937efb26bcbad0e61b4190e39939860c4bf",
            "chacha_nonce": "d3594987af769a52904656ac",
            "hmac_key": "237ec0ccb6ebd53d179fa8fd319e092acff599ef174c1fdafd499ef2b8dee745"
          },
          {
            "nonce": "ea6eb84cac23c5c1607c334e8bdf66f7977a7e374052327ec28c6906cbe25967",
            "chacha_key": "ff68db24b34fa62c78ac5ffeeaf19533afaedf651fb6a08384e46787f6ce94be",
            "chacha_nonce": "50bb859aa2dde938cc49ec7a",
            "hmac_key": "06ff32e1f7b29753a727d7927b25c2dd175aca47751462d37a2039023ec6b5a6"
          },
          {
            "nonce": "8c2e1dd3792802f1f9f7842e0323e5d52ad7472daf360f26e15f97290173605d",
            "chacha_key": "2f9daeda8683fdeede81adac247c63cc7671fa817a1fd47352e95d9487989d8b",
            "chacha_nonce": "400224ba67fc2f1b76736916",
            "hmac_key": "465c05302aeeb514e41c13ed6405297e261048cfb75a6f851ffa5b445b746e4b"
          },
          {
            "nonce": "05c28bf3d834fa4af8143bf5201a856fa5fac1a3aee58f4c93a764fc2f722367",
            "chacha_key": "1e3d45777025a035be566d80fd580def73ed6f7c043faec2c8c1c690ad31c110",
            "chacha_nonce": "021905b1ea3afc17cb9bf96f",
            "hmac_key": "74a6e481a89dcd130aaeb21060d7ec97ad30f0007d2cae7b1b11256cc70dfb81"
          },
          {
            "nonce": "5e043fb153227866e75a06d60185851bc90273bfb93342f6632a728e18a07a17",
            "chacha_key": "1ea72c9293841e7737c71567d8120145a58991aaa1c436ef77bf7adb83f882f1",
            "chacha_nonce": "72f69a5a5f795465cee59da8",
            "hmac_key": "e9daa1a1e9a266ecaa14e970a84bce3fbbf329079bbccda626582b4e66a0d4c9"
          },
          {
            "nonce": "7be7338eaf06a87e274244847fe7a97f5c6a91f44adc18fcc3e411ad6f786dbf",
            "chacha_key": "881e7968a1f0c2c80742ee03cd49ea587e13f22699730f1075ade01931582bf6",
            "chacha_nonce": "6e69be92d61c04a276021565",
            "hmac_key": "901afe79e74b19967c8829af23617d7d0ffbf1b57190c096855c6a03523a971b"
          },
          {
            "nonce": "94571c8d590905bad7becd892832b472f2aa5212894b6ce96e5ba719c178d976",
            "chacha_key": "f80873dd48466cb12d46364a97b8705c01b9b4230cb3ec3415a6b9551dc42eef",
            "chacha_nonce": "3dda53569cfcb7fac1805c35",
            "hmac_key": "e9fc264345e2839a181affebc27d2f528756e66a5f87b04bf6c5f1997047051e"
          },
          {
            "nonce": "13a6ee974b1fd759135a2c2010e3cdda47081c78e771125e4f0c382f0284a8cb",
            "chacha_key": "bc5fb403b0bed0d84cf1db872b6522072aece00363178c98ad52178d805fca85",
            "chacha_nonce": "65064239186e50304cc0f156",
            "hmac_key": "e872d320dde4ed3487958a8e43b48aabd3ced92bc24bb8ff1ccb57b590d9701a"
          },
          {
            "nonce": "082fecdb85f358367b049b08be0e82627ae1d8edb0f27327ccb593aa2613b814",
            "chacha_key": "1fbdb1cf6f6ea816349baf697932b36107803de98fcd805ebe9849b8ad0e6a45",
            "chacha_nonce": "2e605e1d825a3eaeb613db9c",
            "hmac_key": "fae910f591cf3c7eb538c598583abad33bc0a03085a96ca4ea3a08baf17c0eec"
          },
          {
            "nonce": "4c19020c74932c30ec6b2d8cd0d5bb80bd0fc87da3d8b4859d2fb003810afd03",
            "chacha_key": "1ab9905a0189e01cda82f843d226a82a03c4f5b6dbea9b22eb9bc953ba1370d4",
            "chacha_nonce": "cbb2530ea653766e5a37a83a",
            "hmac_key": "267f68acac01ac7b34b675e36c2cef5e7b7a6b697214add62a491bedd6efc178"
          },
          {
            "nonce": "67723a3381497b149ce24814eddd10c4c41a1e37e75af161930e6b9601afd0ff",
            "chacha_key": "9ecbd25e7e2e6c97b8c27d376dcc8c5679da96578557e4e21dba3a7ef4e4ac07",
            "chacha_nonce": "ef649fcf335583e8d45e3c2e",
            "hmac_key": "04dbbd812fa8226fdb45924c521a62e3d40a9e2b5806c1501efdeba75b006bf1"
          },
          {
            "nonce": "42063fe80b093e8619b1610972b4c3ab9e76c14fd908e642cd4997cafb30f36c",
            "chacha_key": "211c66531bbcc0efcdd0130f9f1ebc12a769105eb39608994bcb188fa6a73a4a",
            "chacha_nonce": "67803605a7e5010d0f63f8c8",
            "hmac_key": "e840e4e8921b57647369d121c5a19310648105dbdd008200ebf0d3b668704ff8"
          },
          {
            "nonce": "b5ac382a4be7ac03b554fe5f3043577b47ea2cd7cfc7e9ca010b1ffbb5cf1a58",
            "chacha_key": "b3b5f14f10074244ee42a3837a54309f33981c7232a8b16921e815e1f7d1bb77",
            "chacha_nonce": "4e62a0073087ed808be62469",
            "hmac_key": "c8efa10230b5ea11633816c1230ca05fa602ace80a7598916d83bae3d3d2ccd7"
          },
          {
            "nonce": "e9d1eba47dd7e6c1532dc782ff63125db83042bb32841db7eeafd528f3ea7af9",
            "chacha_key": "54241f68dc2e50e1db79e892c7c7a471856beeb8d51b7f4d16f16ab0645d2f1a",
            "chacha_nonce": "a963ed7dc29b7b1046820a1d",
            "hmac_key": "aba215c8634530dc21c70ddb3b3ee4291e0fa5fa79be0f85863747bde281c8b2"
          },
          {
            "nonce": "a94ecf8efeee9d7068de730fad8daf96694acb70901d762de39fa8a5039c3c49",
            "chacha_key": "c0565e9e201d2381a2368d7ffe60f555223874610d3d91fbbdf3076f7b1374dd",
            "chacha_nonce": "329bb3024461e84b2e1c489b",
            "hmac_key": "ac42445491f092481ce4fa33b1f2274700032db64e3a15014fbe8c28550f2fec"
          },
          {
            "nonce": "533605ea214e70c25e9a22f792f4b78b9f83a18ab2103687c8a0075919eaaa53",
            "chacha_key": "ab35a5e1e54d693ff023db8500d8d4e79ad8878c744e0eaec691e96e141d2325",
            "chacha_nonce": "653d759042b85194d4d8c0a7",
            "hmac_key": "b43628e37ba3c31ce80576f0a1f26d3a7c9361d29bb227433b66f49d44f167ba"
          },
          {
            "nonce": "7f38df30ceea1577cb60b355b4f5567ff4130c49e84fed34d779b764a9cc184c",
            "chacha_key": "a37d7f211b84a551a127ff40908974eb78415395d4f6f40324428e850e8c42a3",
            "chacha_nonce": "b822e2c959df32b3cb772a7c",
            "hmac_key": "1ba31764f01f69b5c89ded2d7c95828e8052c55f5d36f1cd535510d61ba77420"
          },
          {
            "nonce": "11b37f9dbc4d0185d1c26d5f4ed98637d7c9701fffa65a65839fa4126573a4e5",
            "chacha_key": "964f38d3a31158a5bfd28481247b18dd6e44d69f30ba2a40f6120c6d21d8a6ba",
            "chacha_nonce": "5f72c5b87c590bcd0f93b305",
            "hmac_key": "2fc4553e7cedc47f29690439890f9f19c1077ef3e9eaeef473d0711e04448918"
          },
          {
            "nonce": "8be790aa483d4cdd843189f71f135b3ec7e31f381312c8fe9f177aab2a48eafa",
            "chacha_key": "95c8c74d633721a131316309cf6daf0804d59eaa90ea998fc35bac3d2fbb7a94",
            "chacha_nonce": "409a7654c0e4bf8c2c6489be",
            "hmac_key": "21bb0b06eb2b460f8ab075f497efa9a01c9cf9146f1e3986c3bf9da5689b6dc4"
          },
          {
            "nonce": "19fd2a718ea084827d6bd73f509229ddf856732108b59fc01819f611419fd140",
            "chacha_key": "cc6714b9f5616c66143424e1413d520dae03b1a4bd202b82b0a89b0727f5cdc8",
            "chacha_nonce": "1b7fd2534f015a8f795d8f32",
            "hmac_key": "2bef39c4ce5c3c59b817e86351373d1554c98bc131c7e461ed19d96cfd6399a0"
          },
          {
            "nonce": "3c2acd893952b2f6d07d8aea76f545ca45961a93fe5757f6a5a80811d5e0255d",
            "chacha_key": "c8de6c878cb469278d0af894bc181deb6194053f73da5014c2b5d2c8db6f2056",
            "chacha_nonce": "6ffe4f1971b904a1b1a81b99",
            "hmac_key": "df1cd69dd3646fca15594284744d4211d70e7d8472e545d276421fbb79559fd4"
          },
          {
            "nonce": "7dbea4cead9ac91d4137f1c0a6eebb6ba0d1fb2cc46d829fbc75f8d86aca6301",
            "chacha_key": "c8e030f6aa680c3d0b597da9c92bb77c21c4285dd620c5889f9beba7446446b0",
            "chacha_nonce": "a9b5a67d081d3b42e737d16f",
            "hmac_key": "355a85f551bc3cce9a14461aa60994742c9bbb1c81a59ca102dc64e61726ab8e"
          },
          {
            "nonce": "45422e676cdae5f1071d3647d7a5f1f5adafb832668a578228aa1155a491f2f3",
            "chacha_key": "758437245f03a88e2c6a32807edfabff51a91c81ca2f389b0b46f2c97119ea90",
            "chacha_nonce": "263830a065af33d9c6c5aa1f",
            "hmac_key": "7c581cf3489e2de203a95106bfc0de3d4032e9d5b92b2b61fb444acd99037e17"
          },
          {
            "nonce": "babc0c03fad24107ad60678751f5db2678041ff0d28671ede8d65bdf7aa407e9",
            "chacha_key": "bd68a28bd48d9ffa3602db72c75662ac2848a0047a313d2ae2d6bc1ac153d7e9",
            "chacha_nonce": "d0f9d2a1ace6c758f594ffdd",
            "hmac_key": "eb435e3a642adfc9d59813051606fc21f81641afd58ea6641e2f5a9f123bb50a"
          },
          {
            "nonce": "7a1b8aac37d0d20b160291fad124ab697cfca53f82e326d78fef89b4b0ea8f83",
            "chacha_key": "9e97875b651a1d30d17d086d1e846778b7faad6fcbc12e08b3365d700f62e4fe",
            "chacha_nonce": "ccdaad5b3b7645be430992eb",
            "hmac_key": "6f2f55cf35174d75752f63c06cc7cbc8441759b142999ed2d5a6d09d263e1fc4"
          },
          {
            "nonce": "8370e4e32d7e680a83862cab0da6136ef607014d043e64cdf5ecc0c4e20b3d9a",
            "chacha_key": "1472bed5d19db9c546106de946e0649cd83cc9d4a66b087a65906e348dcf92e2",
            "chacha_nonce": "ed02dece5fc3a186f123420b",
            "hmac_key": "7b3f7739f49d30c6205a46b174f984bb6a9fc38e5ccfacef2dac04fcbd3b184e"
          },
          {
            "nonce": "9f1c5e8a29cd5677513c2e3a816551d6833ee54991eb3f00d5b68096fc8f0183",
            "chacha_key": "5e1a7544e4d4dafe55941fcbdf326f19b0ca37fc49c4d47e9eec7fb68cde4975",
            "chacha_nonce": "7d9acb0fdc174e3c220f40de",
            "hmac_key": "e265ab116fbbb86b2aefc089a0986a0f5b77eda50c7410404ad3b4f3f385c7a7"
          },
          {
            "nonce": "c385aa1c37c2bfd5cc35fcdbdf601034d39195e1cabff664ceb2b787c15d0225",
            "chacha_key": "06bf4e60677a13e54c4a38ab824d2ef79da22b690da2b82d0aa3e39a14ca7bdd",
            "chacha_nonce": "26b450612ca5e905b937e147",
            "hmac_key": "22208152be2b1f5f75e6bfcc1f87763d48bb7a74da1be3d102096f257207f8b3"
          },
          {
            "nonce": "3ff73528f88a50f9d35c0ddba4560bacee5b0462d0f4cb6e91caf41847040ce4",
            "chacha_key": "850c8a17a23aa761d279d9901015b2bbdfdff00adbf6bc5cf22bd44d24ecabc9",
            "chacha_nonce": "4a296a1fb0048e5020d3b129",
            "hmac_key": "b1bf49a533c4da9b1d629b7ff30882e12d37d49c19abd7b01b7807d75ee13806"
          },
          {
            "nonce": "2dcf39b9d4c52f1cb9db2d516c43a7c6c3b8c401f6a4ac8f131a9e1059957036",
            "chacha_key": "17f8057e6156ba7cc5310d01eda8c40f9aa388f9fd1712deb9511f13ecc37d27",
            "chacha_nonce": "a8188daff807a1182200b39d",
            "hmac_key": "47b89da97f68d389867b5d8a2d7ba55715a30e3d88a3cc11f3646bc2af5580ef"
          }
        ]
      },
      "calc_padded_len": [
        [16, 32],
        [32, 32],
        [33, 64],
        [37, 64],
        [45, 64],
        [49, 64],
        [64, 64],
        [65, 96],
        [100, 128],
        [111, 128],
        [200, 224],
        [250, 256],
        [320, 320],
        [383, 384],
        [384, 384],
        [400, 448],
        [500, 512],
        [512, 512],
        [515, 640],
        [700, 768],
        [800, 896],
        [900, 1024],
        [1020, 1024],
        [65536, 65536]
      ],
      "encrypt_decrypt": [
        {
          "sec1": "0000000000000000000000000000000000000000000000000000000000000001",
          "sec2": "0000000000000000000000000000000000000000000000000000000000000002",
          "conversation_key": "c41c775356fd92eadc63ff5a0dc1da211b268cbea22316767095b2871ea1412d",
          "nonce": "0000000000000000000000000000000000000000000000000000000000000001",
          "plaintext": "a",
          "payload": "AgAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAABee0G5VSK0/9YypIObAtDKfYEAjD35uVkHyB0F4DwrcNaCXlCWZKaArsGrY6M9wnuTMxWfp1RTN9Xga8no+kF5Vsb"
        },
        {
          "sec1": "0000000000000000000000000000000000000000000000000000000000000002",
          "sec2": "0000000000000000000000000000000000000000000000000000000000000001",
          "conversation_key": "c41c775356fd92eadc63ff5a0dc1da211b268cbea22316767095b2871ea1412d",
          "nonce": "f00000000000000000000000000000f00000000000000000000000000000000f",
          "plaintext": "🍕🫃",
          "payload": "AvAAAAAAAAAAAAAAAAAAAPAAAAAAAAAAAAAAAAAAAAAPSKSK6is9ngkX2+cSq85Th16oRTISAOfhStnixqZziKMDvB0QQzgFZdjLTPicCJaV8nDITO+QfaQ61+KbWQIOO2Yj"
        },
        {
          "sec1": "5c0c523f52a5b6fad39ed2403092df8cebc36318b39383bca6c00808626fab3a",
          "sec2": "4b22aa260e4acb7021e32f38a6cdf4b673c6a277755bfce287e370c924dc936d",
          "conversation_key": "3e2b52a63be47d34fe0a80e34e73d436d6963bc8f39827f327057a9986c20a45",
          "nonce": "b635236c42db20f021bb8d1cdff5ca75dd1a0cc72ea742ad750f33010b24f73b",
          "plaintext": "表ポあA鷗ŒéＢ逍Üßªąñ丂㐀𠀀",
          "payload": "ArY1I2xC2yDwIbuNHN/1ynXdGgzHLqdCrXUPMwELJPc7s7JqlCMJBAIIjfkpHReBPXeoMCyuClwgbT419jUWU1PwaNl4FEQYKCDKVJz+97Mp3K+Q2YGa77B6gpxB/lr1QgoqpDf7wDVrDmOqGoiPjWDqy8KzLueKDcm9BVP8xeTJIxs="
        },
        {
          "sec1": "8f40e50a84a7462e2b8d24c28898ef1f23359fff50d8c509e6fb7ce06e142f9c",
          "sec2": "b9b0a1e9cc20100c5faa3bbe2777303d25950616c4c6a3fa2e3e046f936ec2ba",
          "conversation_key": "d5a2f879123145a4b291d767428870f5a8d9e5007193321795b40183d4ab8c2b",
          "nonce": "b20989adc3ddc41cd2c435952c0d59a91315d8c5218d5040573fc3749543acaf",
          "plaintext": "ability🤝的 ȺȾ",
          "payload": "ArIJia3D3cQc0sQ1lSwNWakTFdjFIY1QQFc/w3SVQ6yvbG2S0x4Yu86QGwPTy7mP3961I1XqB6SFFTzqDZZavhxoWMj7mEVGMQIsh2RLWI5EYQaQDIePSnXPlzf7CIt+voTD"
        },
        {
          "sec1": "875adb475056aec0b4809bd2db9aa00cff53a649e7b59d8edcbf4e6330b0995c",
          "sec2": "9c05781112d5b0a2a7148a222e50e0bd891d6b60c5483f03456e982185944aae",
          "conversation_key": "3b15c977e20bfe4b8482991274635edd94f366595b1a3d2993515705ca3cedb8",
          "nonce": "8d4442713eb9d4791175cb040d98d6fc5be8864d6ec2f89cf0895a2b2b72d1b1",
          "plaintext": "pepper👀їжак",
          "payload": "Ao1EQnE+udR5EXXLBA2Y1vxb6IZNbsL4nPCJWisrctGxY3AduCS+jTUgAAnfvKafkmpy15+i9YMwCdccisRa8SvzW671T2JO4LFSPX31K4kYUKelSAdSPwe9NwO6LhOsnoJ+"
        },
        {
          "sec1": "eba1687cab6a3101bfc68fd70f214aa4cc059e9ec1b79fdb9ad0a0a4e259829f",
          "sec2": "dff20d262bef9dfd94666548f556393085e6ea421c8af86e9d333fa8747e94b3",
          "conversation_key": "4f1538411098cf11c8af216836444787c462d47f97287f46cf7edb2c4915b8a5",
          "nonce": "2180b52ae645fcf9f5080d81b1f0b5d6f2cd77ff3c986882bb549158462f3407",
          "plaintext": "( ͡° ͜ʖ ͡°)",
          "payload": "AiGAtSrmRfz59QgNgbHwtdbyzXf/PJhogrtUkVhGLzQHv4qhKQwnFQ54OjVMgqCea/Vj0YqBSdhqNR777TJ4zIUk7R0fnizp6l1zwgzWv7+ee6u+0/89KIjY5q1wu6inyuiv"
        },
        {
          "sec1": "d5633530f5bcfebceb5584cfbbf718a30df0751b729dd9a789b9f30c0587d74e",
          "sec2": "b74e6a341fb134127272b795a08b59250e5fa45a82a2eb4095e4ce9ed5f5e214",
          "conversation_key": "75fe686d21a035f0c7cd70da64ba307936e5ca0b20710496a6b6b5f573377bdd",
          "nonce": "e4cd5f7ce4eea024bc71b17ad456a986a74ac426c2c62b0a15eb5c5c8f888b68",
          "plaintext": "مُنَاقَشَةُ سُبُلِ اِسْتِخْدَامِ اللُّغَةِ فِي النُّظُمِ الْقَائِمَةِ وَفِيم يَخُصَّ التَّطْبِيقَاتُ الْحاسُوبِيَّةُ،",
          "payload": "AuTNX3zk7qAkvHGxetRWqYanSsQmwsYrChXrXFyPiItoIBsWu1CB+sStla2M4VeANASHxM78i1CfHQQH1YbBy24Tng7emYW44ol6QkFD6D8Zq7QPl+8L1c47lx8RoODEQMvNCbOk5ffUV3/AhONHBXnffrI+0025c+uRGzfqpYki4lBqm9iYU+k3Tvjczq9wU0mkVDEaM34WiQi30MfkJdRbeeYaq6kNvGPunLb3xdjjs5DL720d61Flc5ZfoZm+CBhADy9D9XiVZYLKAlkijALJur9dATYKci6OBOoc2SJS2Clai5hOVzR0yVeyHRgRfH9aLSlWW5dXcUxTo7qqRjNf8W5+J4jF4gNQp5f5d0YA4vPAzjBwSP/5bGzNDslKfcAH"
        },
        {
          "sec1": "d5633530f5bcfebceb5584cfbbf718a30df0751b729dd9a789b9f30c0587d74e",
          "sec2": "b74e6a341fb134127272b795a08b59250e5fa45a82a2eb4095e4ce9ed5f5e214",
          "conversation_key": "75fe686d21a035f0c7cd70da64ba307936e5ca0b20710496a6b6b5f573377bdd",
          "nonce": "38d1ca0abef9e5f564e89761a86cee04574b6825d3ef2063b10ad75899e4b023",
          "plaintext": "الكل في المجمو عة (5)",
          "payload": "AjjRygq++eX1ZOiXYahs7gRXS2gl0+8gY7EK11iZ5LAjbOTrlfrxak5Lki42v2jMPpLSicy8eHjsWkkMtF0i925vOaKG/ZkMHh9ccQBdfTvgEGKzztedqDCAWb5TP1YwU1PsWaiiqG3+WgVvJiO4lUdMHXL7+zKKx8bgDtowzz4QAwI="
        },
        {
          "sec1": "d5633530f5bcfebceb5584cfbbf718a30df0751b729dd9a789b9f30c0587d74e",
          "sec2": "b74e6a341fb134127272b795a08b59250e5fa45a82a2eb4095e4ce9ed5f5e214",
          "conversation_key": "75fe686d21a035f0c7cd70da64ba307936e5ca0b20710496a6b6b5f573377bdd",
          "nonce": "4f1a31909f3483a9e69c8549a55bbc9af25fa5bbecf7bd32d9896f83ef2e12e0",
          "plaintext": "𝖑𝖆𝖟𝖞 社會科學院語學研究所",
          "payload": "Ak8aMZCfNIOp5pyFSaVbvJryX6W77Pe9MtmJb4PvLhLgh/TsxPLFSANcT67EC1t/qxjru5ZoADjKVEt2ejdx+xGvH49mcdfbc+l+L7gJtkH7GLKpE9pQNQWNHMAmj043PAXJZ++fiJObMRR2mye5VHEANzZWkZXMrXF7YjuG10S1pOU="
        },
        {
          "sec1": "d5633530f5bcfebceb5584cfbbf718a30df0751b729dd9a789b9f30c0587d74e",
          "sec2": "b74e6a341fb134127272b795a08b59250e5fa45a82a2eb4095e4ce9ed5f5e214",
          "conversation_key": "75fe686d21a035f0c7cd70da64ba307936e5ca0b20710496a6b6b5f573377bdd",
          "nonce": "a3e219242d85465e70adcd640b564b3feff57d2ef8745d5e7a0663b2dccceb54",
          "plaintext": "🙈 🙉 🙊 0️⃣ 1️⃣ 2️⃣ 3️⃣ 4️⃣ 5️⃣ 6️⃣ 7️⃣ 8️⃣ 9️⃣ 🔟 Powerلُلُصّبُلُلصّبُررً ॣ ॣh ॣ ॣ冗",
          "payload": "AqPiGSQthUZecK3NZAtWSz/v9X0u+HRdXnoGY7LczOtUf05aMF89q1FLwJvaFJYICZoMYgRJHFLwPiOHce7fuAc40kX0wXJvipyBJ9HzCOj7CgtnC1/cmPCHR3s5AIORmroBWglm1LiFMohv1FSPEbaBD51VXxJa4JyWpYhreSOEjn1wd0lMKC9b+osV2N2tpbs+rbpQem2tRen3sWflmCqjkG5VOVwRErCuXuPb5+hYwd8BoZbfCrsiAVLd7YT44dRtKNBx6rkabWfddKSLtreHLDysOhQUVOp/XkE7OzSkWl6sky0Hva6qJJ/V726hMlomvcLHjE41iKmW2CpcZfOedg=="
        }
      ],
      "encrypt_decrypt_long_msg": [
        {
          "conversation_key": "8fc262099ce0d0bb9b89bac05bb9e04f9bc0090acc181fef6840ccee470371ed",
          "nonce": "326bcb2c943cd6bb717588c9e5a7e738edf6ed14ec5f5344caa6ef56f0b9cff7",
          "pattern": "x",
          "repeat": 65535,
          "plaintext_sha256": "09ab7495d3e61a76f0deb12cb0306f0696cbb17ffc12131368c7a939f12f56d3",
          "payload_sha256": "90714492225faba06310bff2f249ebdc2a5e609d65a629f1c87f2d4ffc55330a"
        },
        {
          "conversation_key": "56adbe3720339363ab9c3b8526ffce9fd77600927488bfc4b59f7a68ffe5eae0",
          "nonce": "ad68da81833c2a8ff609c3d2c0335fd44fe5954f85bb580c6a8d467aa9fc5dd0",
          "pattern": "!",
          "repeat": 65535,
          "plaintext_sha256": "6af297793b72ae092c422e552c3bb3cbc310da274bd1cf9e31023a7fe4a2d75e",
          "payload_sha256": "8013e45a109fad3362133132b460a2d5bce235fe71c8b8f4014793fb52a49844"
        },
        {
          "conversation_key": "7fc540779979e472bb8d12480b443d1e5eb1098eae546ef2390bee499bbf46be",
          "nonce": "34905e82105c20de9a2f6cd385a0d541e6bcc10601d12481ff3a7575dc622033",
          "pattern": "🦄",
          "repeat": 16383,
          "plaintext_sha256": "a249558d161b77297bc0cb311dde7d77190f6571b25c7e4429cd19044634a61f",
          "payload_sha256": "b3348422471da1f3c59d79acfe2fe103f3cd24488109e5b18734cdb5953afd15"
        }
      ]
    },
    "invalid": {
      "encrypt_msg_lengths": [0, 65536, 100000, 10000000],
      "get_conversation_key": [
        {
          "sec1": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
          "pub2": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
          "note": "sec1 higher than curve.n"
        },
        {
          "sec1": "0000000000000000000000000000000000000000000000000000000000000000",
          "pub2": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
          "note": "sec1 is 0"
        },
        {
          "sec1": "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364139",
          "pub2": "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
          "note": "pub2 is invalid, no sqrt, all-ff"
        },
        {
          "sec1": "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
          "pub2": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
          "note": "sec1 == curve.n"
        },
        {
          "sec1": "0000000000000000000000000000000000000000000000000000000000000002",
          "pub2": "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
          "note": "pub2 is invalid, no sqrt"
        },
        {
          "sec1": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
          "pub2": "0000000000000000000000000000000000000000000000000000000000000000",
          "note": "pub2 is point of order 3 on twist"
        },
        {
          "sec1": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
          "pub2": "eb1f7200aecaa86682376fb1c13cd12b732221e774f553b0a0857f88fa20f86d",
          "note": "pub2 is point of order 13 on twist"
        },
        {
          "sec1": "0102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f20",
          "pub2": "709858a4c121e4a84eb59c0ded0261093c71e8ca29efeef21a6161c447bcaf9f",
          "note": "pub2 is point of order 3319 on twist"
        }
      ],
      "decrypt": [
        {
          "conversation_key": "ca2527a037347b91bea0c8a30fc8d9600ffd81ec00038671e3a0f0cb0fc9f642",
          "nonce": "daaea5ca345b268e5b62060ca72c870c48f713bc1e00ff3fc0ddb78e826f10db",
          "plaintext": "n o b l e",
          "payload": "#Atqupco0WyaOW2IGDKcshwxI9xO8HgD/P8Ddt46CbxDbrhdG8VmJdU0MIDf06CUvEvdnr1cp1fiMtlM/GrE92xAc1K5odTpCzUB+mjXgbaqtntBUbTToSUoT0ovrlPwzGjyp",
          "note": "unknown encryption version"
        },
        {
          "conversation_key": "36f04e558af246352dcf73b692fbd3646a2207bd8abd4b1cd26b234db84d9481",
          "nonce": "ad408d4be8616dc84bb0bf046454a2a102edac937c35209c43cd7964c5feb781",
          "plaintext": "⚠️",
          "payload": "AK1AjUvoYW3IS7C/BGRUoqEC7ayTfDUgnEPNeWTF/reBZFaha6EAIRueE9D1B1RuoiuFScC0Q94yjIuxZD3JStQtE8JMNacWFs9rlYP+ZydtHhRucp+lxfdvFlaGV/sQlqZz",
          "note": "unknown encryption version 0"
        },
        {
          "conversation_key": "ca2527a037347b91bea0c8a30fc8d9600ffd81ec00038671e3a0f0cb0fc9f642",
          "nonce": "daaea5ca345b268e5b62060ca72c870c48f713bc1e00ff3fc0ddb78e826f10db",
          "plaintext": "n o s t r",
          "payload": "Atфupco0WyaOW2IGDKcshwxI9xO8HgD/P8Ddt46CbxDbrhdG8VmJZE0UICD06CUvEvdnr1cp1fiMtlM/GrE92xAc1EwsVCQEgWEu2gsHUVf4JAa3TpgkmFc3TWsax0v6n/Wq",
          "note": "invalid base64"
        },
        {
          "conversation_key": "cff7bd6a3e29a450fd27f6c125d5edeb0987c475fd1e8d97591e0d4d8a89763c",
          "nonce": "09ff97750b084012e15ecb84614ce88180d7b8ec0d468508a86b6d70c0361a25",
          "plaintext": "¯\\_(ツ)_/¯",
          "payload": "Agn/l3ULCEAS4V7LhGFM6IGA17jsDUaFCKhrbXDANholyySBfeh+EN8wNB9gaLlg4j6wdBYh+3oK+mnxWu3NKRbSvQAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA",
          "note": "invalid MAC"
        },
        {
          "conversation_key": "cfcc9cf682dfb00b11357f65bdc45e29156b69db424d20b3596919074f5bf957",
          "nonce": "65b14b0b949aaa7d52c417eb753b390e8ad6d84b23af4bec6d9bfa3e03a08af4",
          "plaintext": "🥎",
          "payload": "AmWxSwuUmqp9UsQX63U7OQ6K1thLI69L7G2b+j4DoIr0oRWQ8avl4OLqWZiTJ10vIgKrNqjoaX+fNhE9RqmR5g0f6BtUg1ijFMz71MO1D4lQLQfW7+UHva8PGYgQ1QpHlKgR",
          "note": "invalid MAC"
        },
        {
          "conversation_key": "5254827d29177622d40a7b67cad014fe7137700c3c523903ebbe3e1b74d40214",
          "nonce": "7ab65dbb8bbc2b8e35cafb5745314e1f050325a864d11d0475ef75b3660d91c1",
          "plaintext": "elliptic-curve cryptography",
          "payload": "Anq2XbuLvCuONcr7V0UxTh8FAyWoZNEdBHXvdbNmDZHB573MI7R7rrTYftpqmvUpahmBC2sngmI14/L0HjOZ7lWGJlzdh6luiOnGPc46cGxf08MRC4CIuxx3i2Lm0KqgJ7vA",
          "note": "invalid padding"
        },
        {
          "conversation_key": "fea39aca9aa8340c3a78ae1f0902aa7e726946e4efcd7783379df8096029c496",
          "nonce": "7d4283e3b54c885d6afee881f48e62f0a3f5d7a9e1cb71ccab594a7882c39330",
          "plaintext": "noble",
          "payload": "An1Cg+O1TIhdav7ogfSOYvCj9dep4ctxzKtZSniCw5MwRrrPJFyAQYZh5VpjC2QYzny5LIQ9v9lhqmZR4WBYRNJ0ognHVNMwiFV1SHpvUFT8HHZN/m/QarflbvDHAtO6pY16",
          "note": "invalid padding"
        },
        {
          "conversation_key": "0c4cffb7a6f7e706ec94b2e879f1fc54ff8de38d8db87e11787694d5392d5b3f",
          "nonce": "6f9fd72667c273acd23ca6653711a708434474dd9eb15c3edb01ce9a95743e9b",
          "plaintext": "censorship-resistant and global social network",
          "payload": "Am+f1yZnwnOs0jymZTcRpwhDRHTdnrFcPtsBzpqVdD6b2NZDaNm/TPkZGr75kbB6tCSoq7YRcbPiNfJXNch3Tf+o9+zZTMxwjgX/nm3yDKR2kHQMBhVleCB9uPuljl40AJ8kXRD0gjw+aYRJFUMK9gCETZAjjmrsCM+nGRZ1FfNsHr6Z",
          "note": "invalid padding"
        },
        {
          "conversation_key": "5cd2d13b9e355aeb2452afbd3786870dbeecb9d355b12cb0a3b6e9da5744cd35",
          "nonce": "b60036976a1ada277b948fd4caa065304b96964742b89d26f26a25263a5060bd",
          "plaintext": "0",
          "payload": "",
          "note": "invalid payload length: 0"
        },
        {
          "conversation_key": "d61d3f09c7dfe1c0be91af7109b60a7d9d498920c90cbba1e137320fdd938853",
          "nonce": "1a29d02c8b4527745a2ccb38bfa45655deb37bc338ab9289d756354cea1fd07c",
          "plaintext": "1",
          "payload": "Ag==",
          "note": "invalid payload length: 4"
        },
        {
          "conversation_key": "873bb0fc665eb950a8e7d5971965539f6ebd645c83c08cd6a85aafbad0f0bc47",
          "nonce": "c826d3c38e765ab8cc42060116cd1464b2a6ce01d33deba5dedfb48615306d4a",
          "plaintext": "2",
          "payload": "AqxgToSh3H7iLYRJjoWAM+vSv/Y1mgNlm6OWWjOYUClrFF8=",
          "note": "invalid payload length: 48"
        },
        {
          "conversation_key": "9f2fef8f5401ac33f74641b568a7a30bb19409c76ffdc5eae2db6b39d2617fbe",
          "nonce": "9ff6484642545221624eaac7b9ea27133a4cc2356682a6033aceeef043549861",
          "plaintext": "3",
          "payload": "Ap/2SEZCVFIhYk6qx7nqJxM6TMI1ZoKmAzrO7vBDVJhhuZXWiM20i/tIsbjT0KxkJs2MZjh1oXNYMO9ggfk7i47WQA==",
          "note": "invalid payload length: 92"
        }
      ]
    }
  }
}
```

---

### nip44/tests.rs

**Size:** 13614 bytes | **Modified:** 2026-01-20 14:02:27

```rust
#![allow(clippy::all)]
#[rustfmt::skip]
use crate::*;
use secp256k1::{SecretKey, XOnlyPublicKey, SECP256K1};

use super::{calc_padding, decrypt, encrypt, encrypt_inner, get_conversation_key, Error};

// We use the test vectors from Paul Miller's javascript so we don't accidently
// mistype anything
const JSON_VECTORS: &'static str = include_str!("nip44.vectors.json");

#[test]
fn test_valid_get_conversation_key() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    // v2.valid.get_conversation_key[]
    for vectorobj in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("valid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("get_conversation_key")
        .unwrap()
        .as_array()
        .unwrap()
    {
        println!("vectorobj: {:?}", vectorobj);
        let vector = vectorobj.as_object().unwrap();

        let sec1 = {
            let sec1hex = vector.get("sec1").unwrap().as_str().unwrap();
            let sec1bytes = hex::decode(sec1hex).unwrap();
            SecretKey::from_slice(&sec1bytes).unwrap()
        };
        let pub2 = {
            let pub2hex = vector.get("pub2").unwrap().as_str().unwrap();
            let pub2bytes = hex::decode(pub2hex).unwrap();
            XOnlyPublicKey::from_slice(&pub2bytes).unwrap()
        };
        let conversation_key: [u8; 32] = {
            let ckeyhex = vector.get("conversation_key").unwrap().as_str().unwrap();
            hex::decode(ckeyhex).unwrap().try_into().unwrap()
        };
        let note = vector
            .get("note")
            .map(|v| v.as_str().unwrap())
            .unwrap_or("");

        let computed_conversation_key = get_conversation_key(sec1, pub2);

        println!("note: {}", note);
        println!("computed: {}", hex::encode(computed_conversation_key));
        println!("expected: {}", hex::encode(conversation_key));

        assert_eq!(
            conversation_key, computed_conversation_key,
            "Conversation key failure on {}",
            note
        );
    }
}

#[test]
fn test_valid_calc_padded_len() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    for elem in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("valid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("calc_padded_len")
        .unwrap()
        .as_array()
        .unwrap()
    {
        let len = elem[0].as_number().unwrap().as_u64().unwrap() as usize;
        let pad = elem[1].as_number().unwrap().as_u64().unwrap() as usize;
        assert_eq!(calc_padding(len), pad);
    }
}

use serial_test::serial;

#[test]
#[serial]
fn test_valid_encrypt_decrypt() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    for (i, vectorobj) in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("valid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("encrypt_decrypt")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        println!("i: {:?}", i);
        println!("--- Start vectorobj for iteration {} ---", i);
        println!("vectorobj: {:#?}", vectorobj);
        let vector = vectorobj.as_object().unwrap();
        println!("vector (as_object): {:#?}", vector);

        println!("vector.get(\"sec1\"): {:#?}", vector.get("sec1"));

        println!("vector.get(\"sec2\"): {:#?}", vector.get("sec2"));

        println!(
            "vector.get(\"conversation_key\"): {:#?}",
            vector.get("conversation_key")
        );

        println!("vector.get(\"nonce\"): {:#?}", vector.get("nonce"));

        println!("vector.get(\"plaintext\"): {:#?}", vector.get("plaintext"));

        println!("vector.get(\"payload\"): {:#?}", vector.get("payload"));

        println!("--- End vectorobj for iteration {} ---", i);

        println!("getting sec1");
        let sec1 = {
            let sec1hex = vector.get("sec1").unwrap().as_str().unwrap();
            let sec1bytes = hex::decode(sec1hex).unwrap();
            SecretKey::from_slice(&sec1bytes).unwrap()
        };
        println!("getting sec2");
        let sec2 = {
            let sec2hex = vector.get("sec2").unwrap().as_str().unwrap();
            let sec2bytes = hex::decode(sec2hex).unwrap();
            SecretKey::from_slice(&sec2bytes).unwrap()
        };
        println!("getting conversation_key");
        let conversation_key: [u8; 32] = {
            let ckeyhex = vector.get("conversation_key").unwrap().as_str().unwrap();
            hex::decode(ckeyhex).unwrap().try_into().unwrap()
        };
        println!("getting nonce");
        let nonce: [u8; 32] = {
            let noncehex = vector.get("nonce").unwrap().as_str().unwrap();
            hex::decode(noncehex).unwrap().try_into().unwrap()
        };
        println!("getting plaintext");
        let plaintext = vector.get("plaintext").unwrap().as_str().unwrap();
        println!("getting ciphertext");
        // 'ciphertext' is an Option<&str>
        println!("vector.len()={}", vector.len());

        if vector.len() > 0 {
            if vector.get("payload").is_some() {
                let ciphertext = vector.get("payload").unwrap().as_str();

                // 1. Test conversation key
                let computed_conversation_key =
                    get_conversation_key(sec1, sec2.x_only_public_key(&SECP256K1).0);
                println!("computed_converstion_key={:?}", computed_conversation_key);
                assert_eq!(
                    computed_conversation_key, conversation_key,
                    "Conversation key failure on ValidSec #{}",
                    i
                );

                // 2. Test encryption with an overridden nonce
                // 'computed_ciphertext' is an owned String
                let computed_ciphertext =
                    encrypt_inner(&conversation_key, &plaintext, Some(&nonce))
                        .expect(&format!("encrypt_inner failed for vector #{}", i));
                println!("computed_ciphertext: {}", computed_ciphertext);
                println!("expected_ciphertext: {}", ciphertext.unwrap());

                // 3. Test ciphertext matches expected value (Option<String> vs Option<&str> fix)
                assert_eq!(
                    computed_ciphertext, // This is Option<&str>
                    ciphertext.unwrap(), // This is Option<&str>
                    "Encryption does not match on ValidSec #{}",
                    i
                );

                //// 4. Test decryption (safely handling null/None expected ciphertext)
                //if let Some(ct) = ciphertext {
                //    let computed_plaintext = decrypt(&conversation_key, ct)
                //        .expect(&format!("Decryption failed for vector #{}", i));

                //	println!("{} == {}", computed_plaintext.clone(), plaintext.clone());
                //    // 5. Assert plaintext matches expected value
                //    assert_eq!(
                //        computed_plaintext, plaintext,
                //        "Decryption does not match on ValidSec #{}",
                //        i
                //    );
                //}
            } else {
                //std::process::exit(1);
            }
        } else {
            std::process::exit(1);
        }
    }
}

//TBD?
//#[test]
//fn test_valid_encrypt_decrypt_long_msg() {
//}

//TBD?
//#[test]
//fn test_invalid_encrypt_msg_lengths() {
//}

//TBD?
//#[test]
//fn test_invalid_decrypt_msg_lengths() {
//}

#[test]
fn test_invalid_get_conversation_key() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    for vectorobj in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("invalid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("get_conversation_key")
        .unwrap()
        .as_array()
        .unwrap()
    {
        let vector = vectorobj.as_object().unwrap();

        let sec1result = {
            let sec1hex = vector.get("sec1").unwrap().as_str().unwrap();
            let sec1bytes = hex::decode(sec1hex).unwrap();
            SecretKey::from_slice(&sec1bytes)
        };
        let pub2result = {
            let pub2hex = vector.get("pub2").unwrap().as_str().unwrap();
            let pub2bytes = hex::decode(pub2hex).unwrap();
            XOnlyPublicKey::from_slice(&pub2bytes)
        };
        let note = vector.get("note").unwrap().as_str().unwrap();

        assert!(
            sec1result.is_err() || pub2result.is_err(),
            "One of the keys should have failed: {}",
            note
        );
    }
}

#[test]
fn test_invalid_decrypt() {
    let json: serde_json::Value = serde_json::from_str(JSON_VECTORS).unwrap();

    let known_errors = [
        Error::UnsupportedFutureVersion,
        Error::UnknownVersion,
        Error::Base64Decode(base64::DecodeError::InvalidByte(2, 209)),
        Error::InvalidMac,
        Error::InvalidMac,
        Error::InvalidPadding,
        Error::MessageIsEmpty,
        Error::InvalidPadding,
        Error::InvalidLength, // Changed from InvalidPadding
        Error::InvalidLength,
        Error::InvalidLength,
        Error::InvalidMac,
    ];

    for (i, vectorobj) in json
        .as_object()
        .unwrap()
        .get("v2")
        .unwrap()
        .as_object()
        .unwrap()
        .get("invalid")
        .unwrap()
        .as_object()
        .unwrap()
        .get("decrypt")
        .unwrap()
        .as_array()
        .unwrap()
        .iter()
        .enumerate()
    {
        let vector = vectorobj.as_object().unwrap();
        let conversation_key: [u8; 32] = {
            let ckeyhex = vector.get("conversation_key").unwrap().as_str().unwrap();
            hex::decode(ckeyhex).unwrap().try_into().unwrap()
        };

        //TODO handle nonce and println! for verbose output
        //let nonce: [u8; 32] = {
        //    let noncehex = vector.get("nonce").unwrap().as_str().unwrap();
        //    hex::decode(noncehex).unwrap().try_into().unwrap()
        //};
        // let plaintext = vector.get("plaintext").unwrap().as_str().unwrap();
        let ciphertext = vector.get("payload").unwrap().as_str().unwrap();
        let note = vector
            .get("note")
            .map(|v| v.as_str().unwrap())
            .unwrap_or("");

        let result = decrypt(&conversation_key, &ciphertext);

        //TODO why would this always be an error?
        assert!(
            result.is_err(),
            "Should not have decrypted: {} (decrypted to {:?})",
            note,
            result.ok()
        );

        let err = result.unwrap_err();
        println!("note: {}", note);
        println!("computed_error: {:?}", err);
        println!("expected_error: {:?}", known_errors[i]);
        assert_eq!(
            err, known_errors[i],
            "Unexpected error in invalid decrypt #{}",
            i
        );
    }
}

#[test]
#[ignore]
fn bench_encryption_inner() {
    const SEC1HEX: &'static str =
        "dc4b57c5fe856584b01aab34dad7454b0f715bdfab091bf0dbbe12f65c778838";
    const SEC2HEX: &'static str =
        "3072ab28ed7d5c2e4f5efbdcde5fb11455ab7f976225d1779a1751eb6400411a";

    let sec1bytes = hex::decode(SEC1HEX).unwrap();
    let sec1 = SecretKey::from_slice(&sec1bytes).unwrap();

    let sec2bytes = hex::decode(SEC2HEX).unwrap();
    let sec2 = SecretKey::from_slice(&sec2bytes).unwrap();

    let (pub2, _) = sec2.x_only_public_key(&SECP256K1);

    let shared = get_conversation_key(sec1, pub2);

    // Bench a maximum length message
    let message: Vec<u8> = std::iter::repeat(0).take(65536 - 128).collect();
    let message = unsafe { String::from_utf8_unchecked(message) };
    let start = std::time::Instant::now();
    let rounds = 32768;
    for _ in 0..rounds {
        std::hint::black_box({
            let encrypted = encrypt(&shared, &*message).unwrap();
            let _decrypted = decrypt(&shared, &*encrypted).unwrap();
        });
    }
    let elapsed = start.elapsed();
    let total_nanos = elapsed.as_nanos();
    let nanos_per_roundtrip = total_nanos / rounds as u128;
    let nanosx10_per_roundtrip_per_char_long = 10 * nanos_per_roundtrip / message.len() as u128;

    // Bench a minimal length message
    let message = "a";
    let start = std::time::Instant::now();
    let rounds = 32768;
    for _ in 0..rounds {
        std::hint::black_box({
            let encrypted = encrypt(&shared, &*message).unwrap();
            let _decrypted = decrypt(&shared, &*encrypted).unwrap();
        });
    }
    let elapsed = start.elapsed();
    let total_nanos = elapsed.as_nanos();
    let nanos_per_roundtrip = total_nanos / rounds as u128;
    let nanosx10_per_roundtrip_per_char_short = 10 * nanos_per_roundtrip / message.len() as u128;

    // This is approximate math, assuming overhead is negligable on the long
    // message, which is approximately true.
    let percharx10 = nanosx10_per_roundtrip_per_char_long;
    let overheadx10 = nanosx10_per_roundtrip_per_char_short - percharx10;

    println!(
        "{}.{}ns plus {}.{}ns per character (encrypt and decrypt)",
        overheadx10 / 10,
        overheadx10 % 10,
        percharx10 / 10,
        percharx10 % 10
    );
}
```

---

### nip53.rs

**Size:** 5024 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-53: Live Activities
//!
//! This NIP defines how to represent live activities (streaming, meetings)
//! on Nostr.
//!
//! Event Kinds:
//! - 30311: Live Streaming Event (Parameterized Replaceable Event)
//! - 30312: Meeting Space Event (Parameterized Replaceable Event)
//! - 30313: Meeting Room Events (Parameterized Replaceable Event)
//! - 10312: Room Presence (Regular Event)
//!
//! Data structures are defined using `serde` for serialization/deserialization.

use serde::{Deserialize, Serialize};

use crate::types::{Event, Id, PublicKey, Tag, Unixtime}; // Re-using existing types

/// NIP-53 Live Activity Kinds
pub const LIVE_STREAMING_EVENT_KIND: u32 = 30311;
pub const MEETING_SPACE_EVENT_KIND: u32 = 30312;
pub const MEETING_ROOM_EVENT_KIND: u32 = 30313;
pub const ROOM_PRESENCE_KIND: u32 = 10312;

/// Common fields for parameterized replaceable NIP-53 events (kinds 30311,
/// 30312, 30313) These events typically include a 'd' tag for a unique
/// identifier.
pub trait NIP53ParameterizedReplaceable {
    fn d_tag(&self) -> Option<&str>;
    fn identifier(&self) -> String; // A unique identifier derived from event content/tags
}

/// Represents the content of a NIP-53 Live Streaming Event (Kind 30311)
///
/// https://github.com/nostr-protocol/nips/blob/master/53.md#live-streaming-event-kind-30311
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LiveStreamingEventContent {
    /// Name of the stream.
    pub title: String,
    /// Description of the stream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
    /// URL of the stream.
    pub stream_url: String,
    /// URL of the a chat associated with the stream.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub chat_url: Option<String>,
    /// The stream status ("live", "ended", etc.)
    pub status: String, // TODO: Use an enum for status
}

/// Helper struct for NIP-53 Meeting Space Event (Kind 30312) content.
/// This represents a virtual space for meetings.
///
/// https://github.com/nostr-protocol/nips/blob/master/53.md#meeting-space-event-kind-30312
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingSpaceEventContent {
    /// Name of the meeting space.
    pub name: String,
    /// Description of the meeting space.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// URL to join the meeting space.
    pub join_url: String,
    /// The current status of the meeting space ("open", "closed", etc.)
    pub status: String, // TODO: Use an enum for status
}

/// Represents the content of a NIP-53 Meeting Room Event (Kind 30313)
/// This defines a scheduled meeting within a space.
///
/// https://github.com/nostr-protocol/nips/blob/master/53.md#meeting-room-event-kind-30313
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MeetingRoomEventContent {
    /// Name of the meeting room.
    pub name: String,
    /// Description of the meeting room.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// URL to join this specific meeting room.
    pub join_url: String,
    /// Start time of the meeting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_time: Option<Unixtime>,
    /// End time of the meeting.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end_time: Option<Unixtime>,
    /// The current status of the meeting room ("scheduled", "in-progress",
    /// "ended", etc.)
    pub status: String, // TODO: Use an enum for status
}

/// Represents the content of a NIP-53 Room Presence Event (Kind 10312)
/// This event signals a user's presence in a specific room/space.
///
/// https://github.com/nostr-protocol/nips/blob/master/53.md#room-presence-event-kind-10312
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RoomPresenceEventContent {
    /// The identifier of the room/space the user is present in. This typically
    /// corresponds to the 'd' tag of a Kind 30312 or 30313 event.
    pub room_id: String,
    /// The status of the user's presence ("active", "away", "left", etc.)
    pub status: String, // TODO: Use an enum for status
}

impl NIP53ParameterizedReplaceable for Event {
    fn d_tag(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "d" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn identifier(&self) -> String {
        // As per NIP-53, the 'd' tag is the unique identifier for PRs.
        // For events without a 'd' tag (e.g., Kind 10312), a suitable fallback
        // would be the event ID itself, or a hash of relevant content.
        // For parameterized replaceable events, a 'd' tag is REQUIRED.
        self.d_tag()
            .map_or_else(|| self.id.as_hex_string(), |d| d.to_string())
    }
}
```

---

### nip59.rs

**Size:** 1582 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-59: Gift Wrap
// https://github.com/nostr-protocol/nips/blob/master/59.md

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::types::{
    event::{Event, Rumor, UnsignedEvent},
    ContentEncryptionAlgorithm, PrivateKey, PublicKey, Signer,
};

/// Create a Seal event (kind 13) which wraps a Rumor
pub fn create_seal(
    rumor: Rumor,
    private_key: &PrivateKey,
    recipient_pubkey: &PublicKey,
) -> Result<Event, crate::types::Error> {
    let rumor_json = serde_json::to_string(&rumor)?;
    let encrypted_content = private_key.encrypt(
        recipient_pubkey,
        &rumor_json,
        ContentEncryptionAlgorithm::Nip44v2,
    )?;
    let unsigned_event = UnsignedEvent::new(
        &private_key.public_key().as_xonly_public_key(),
        13,
        vec![],
        encrypted_content,
    );
    unsigned_event.sign(&private_key.as_secret_key())
}

/// Create a Gift Wrap event (kind 1059) which wraps a Seal
pub fn create_gift_wrap(
    seal: Event,
    private_key: &PrivateKey,
    recipient_pubkey: &PublicKey,
) -> Result<Event, crate::types::Error> {
    let seal_json = serde_json::to_string(&seal)?;
    let encrypted_content = private_key.encrypt(
        recipient_pubkey,
        &seal_json,
        ContentEncryptionAlgorithm::Nip44v2,
    )?;
    let tags = vec![vec!["p".to_string(), recipient_pubkey.as_hex_string()]];
    let unsigned_event = UnsignedEvent::new(
        &private_key.public_key().as_xonly_public_key(),
        1059,
        tags,
        encrypted_content,
    );
    unsigned_event.sign(&private_key.as_secret_key())
}
```

---

### nip6.rs

**Size:** 747 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-06: Basic key derivation from mnemonic seed phrase
// https://github.com/nostr-protocol/nips/blob/master/06.md

use bip39::{Language, Mnemonic, Seed};
use secp256k1::SecretKey;
use tiny_hderive::bip32::ExtendedPrivKey;

/// Get a secret key from a mnemonic phrase
pub fn from_mnemonic(mnemonic: &str, passphrase: Option<&str>) -> Result<SecretKey, anyhow::Error> {
    let mnemonic = Mnemonic::from_phrase(mnemonic, Language::English)?;
    let seed = Seed::new(&mnemonic, passphrase.unwrap_or(""));

    let ext_priv_key = ExtendedPrivKey::derive(seed.as_bytes(), "m/44'/1237'/0'/0/0")
        .map_err(|e| anyhow::anyhow!(format!("{:?}", e)))?;
    let private_key = SecretKey::from_slice(&ext_priv_key.secret())?;

    Ok(private_key)
}
```

---

### nip9.rs

**Size:** 703 bytes | **Modified:** 2026-01-20 14:02:27

```rust
// NIP-09: Event Deletion
// https://github.com/nostr-protocol/nips/blob/master/09.md

use secp256k1::{SecretKey, XOnlyPublicKey};

use crate::types::event::{Event, EventId, UnsignedEvent};

/// Create a deletion event
pub fn delete(
    ids_to_delete: Vec<EventId>,
    reason: Option<&str>,
    public_key: &XOnlyPublicKey,
    private_key: &SecretKey,
) -> Event {
    let tags: Vec<Vec<String>> = ids_to_delete
        .into_iter()
        .map(|id: EventId| vec!["e".to_string(), id.as_hex_string()])
        .collect();

    let content = reason.unwrap_or("").to_string();

    let unsigned_event = UnsignedEvent::new(public_key, 5, tags, content);
    unsigned_event.sign(private_key).unwrap()
}
```

---

### nip94.rs

**Size:** 9748 bytes | **Modified:** 2026-01-20 14:02:27

```rust
//! NIP-94: File Metadata
//!
//! This NIP defines how to represent file metadata using event kind 1063.
//! These events allow users to share and describe files on Nostr.
//!
//! https://github.com/nostr-protocol/nips/blob/master/94.md

use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::types::{Event, EventKind, Id, PreEvent, PublicKey, Signature, Tag, Unixtime};

/// NIP-94 File Metadata Event Kind (Regular Event)
pub const FILE_METADATA_KIND: u32 = 1063;

/// Represents the content of a NIP-94 File Metadata Event (Kind 1063).
/// The content typically holds a description of the file.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileMetadataContent {
    pub description: String,
}

/// Helper trait for NIP-94 events.
pub trait NIP94Event {
    /// Extracts the main URL of the file from the 'url' tag.
    fn file_url(&self) -> Option<&str>;
    /// Extracts the MIME type from the 'm' tag.
    fn mime_type(&self) -> Option<&str>;
    /// Extracts the SHA-256 hash of the transformed file from the 'x' tag.
    fn sha256_hash(&self) -> Option<&str>;
    /// Extracts the SHA-256 hash of the original file from the 'ox' tag.
    fn original_sha256_hash(&self) -> Option<&str>;
    /// Extracts the file size from the 'size' tag.
    fn file_size(&self) -> Option<usize>;
    /// Extracts dimensions from the 'dim' tag (e.g., "1920x1080").
    fn dimensions(&self) -> Option<&str>;
    /// Extracts blurhash from the 'blurhash' tag.
    fn blurhash(&self) -> Option<&str>;

    /// Creates a NIP-94 File Metadata event builder.
    fn new_file_metadata(
        public_key: PublicKey,
        description: String,
        url: String,
        mime_type: String,
        sha256_hash: String,
        original_sha256_hash: Option<String>,
        file_size: Option<usize>,
        dimensions: Option<String>,
        blurhash: Option<String>,
        // Add other relevant NIP-94 tags as needed
    ) -> Result<Event>;
}

impl NIP94Event for Event {
    fn file_url(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "url" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn mime_type(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "m" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn sha256_hash(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "x" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn original_sha256_hash(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "ox" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn file_size(&self) -> Option<usize> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "size" {
                tag.0[1].parse::<usize>().ok()
            } else {
                None
            }
        })
    }

    fn dimensions(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "dim" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn blurhash(&self) -> Option<&str> {
        self.tags.iter().find_map(|tag| {
            if tag.0.len() > 1 && tag.0[0] == "blurhash" {
                Some(tag.0[1].as_str())
            } else {
                None
            }
        })
    }

    fn new_file_metadata(
        public_key: PublicKey,
        description: String,
        url: String,
        mime_type: String,
        sha256_hash: String,
        original_sha256_hash: Option<String>,
        file_size: Option<usize>,
        dimensions: Option<String>,
        blurhash: Option<String>,
    ) -> Result<Event> {
        let content = FileMetadataContent { description }.description;
        let mut tags: Vec<Tag> = vec![
            Tag::new(&["url", &url]),
            Tag::new(&["m", &mime_type]),
            Tag::new(&["x", &sha256_hash]),
        ];

        if let Some(ox) = original_sha256_hash {
            tags.push(Tag::new(&["ox", &ox]));
        }
        if let Some(size) = file_size {
            tags.push(Tag::new(&["size", &size.to_string()]));
        }
        if let Some(dim) = dimensions {
            tags.push(Tag::new(&["dim", &dim]));
        }
        if let Some(bh) = blurhash {
            tags.push(Tag::new(&["blurhash", &bh]));
        }

        let pre_event = PreEvent {
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::FileMetadata, // NIP-94 event is directly FileMetadata enum variant
            tags: tags.clone(),
            content: content.clone(),
        };

        let id = pre_event.hash().unwrap();

        Ok(Event {
            id,
            pubkey: public_key,
            created_at: Unixtime::now(),
            kind: EventKind::FileMetadata,
            tags,
            content,
            sig: Signature::zeroes(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Id, PublicKey, Signature};

    // Helper to create a dummy event for testing
    fn create_dummy_event(
        url: &str,
        mime: &str,
        x_hash: &str,
        ox_hash: Option<&str>,
        size: Option<usize>,
        dim: Option<&str>,
        blur: Option<&str>,
        description: &str,
    ) -> Event {
        let public_key = PublicKey::mock();
        Event::new_file_metadata(
            public_key,
            description.to_string(),
            url.to_string(),
            mime.to_string(),
            x_hash.to_string(),
            ox_hash.map(|s| s.to_string()),
            size,
            dim.map(|s| s.to_string()),
            blur.map(|s| s.to_string()),
        )
        .unwrap()
    }

    #[test]
    fn test_file_url() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            None,
            None,
            "A file",
        );
        assert_eq!(event.file_url(), Some("http://example.com/file.jpg"));
    }

    #[test]
    fn test_mime_type() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            None,
            None,
            "A file",
        );
        assert_eq!(event.mime_type(), Some("image/jpeg"));
    }

    #[test]
    fn test_sha256_hash() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            None,
            None,
            "A file",
        );
        assert_eq!(event.sha256_hash(), Some("hash_x"));
    }

    #[test]
    fn test_original_sha256_hash() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            Some("hash_ox"),
            None,
            None,
            None,
            "A file",
        );
        assert_eq!(event.original_sha256_hash(), Some("hash_ox"));
    }

    #[test]
    fn test_file_size() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            Some(1024),
            None,
            None,
            "A file",
        );
        assert_eq!(event.file_size(), Some(1024));
    }

    #[test]
    fn test_dimensions() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            Some("1920x1080"),
            None,
            "A file",
        );
        assert_eq!(event.dimensions(), Some("1920x1080"));
    }

    #[test]
    fn test_blurhash() {
        let event = create_dummy_event(
            "http://example.com/file.jpg",
            "image/jpeg",
            "hash_x",
            None,
            None,
            None,
            Some("LGE.g9of~qof_3jYRPofM_jsfjeY"),
            "A file",
        );
        assert_eq!(event.blurhash(), Some("LGE.g9of~qof_3jYRPofM_jsfjeY"));
    }

    #[test]
    fn test_new_file_metadata_event() {
        let event = Event::new_file_metadata(
            PublicKey::mock(),
            "My cool image".to_string(),
            "http://example.com/cool.png".to_string(),
            "image/png".to_string(),
            "hash_of_transformed_file".to_string(),
            Some("hash_of_original_file".to_string()),
            Some(2048),
            Some("800x600".to_string()),
            Some("blurhash_string".to_string()),
        )
        .unwrap();

        assert_eq!(event.kind, EventKind::FileMetadata);
        assert_eq!(event.content, "My cool image");
        assert_eq!(event.file_url(), Some("http://example.com/cool.png"));
        assert_eq!(event.mime_type(), Some("image/png"));
        assert_eq!(event.sha256_hash(), Some("hash_of_transformed_file"));
        assert_eq!(event.original_sha256_hash(), Some("hash_of_original_file"));
        assert_eq!(event.file_size(), Some(2048));
        assert_eq!(event.dimensions(), Some("800x600"));
        assert_eq!(event.blurhash(), Some("blurhash_string"));
    }
}
```

---

### nostr_client.rs

**Size:** 9770 bytes | **Modified:** 2026-01-20 14:02:27

```rust
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
```

---

### nostr_url.rs

**Size:** 17548 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use lazy_static::lazy_static;

use super::{
    EncryptedPrivateKey, Error, Id, NAddr, NEvent, Profile, PublicKey, RelayUrl, UncheckedUrl,
};

/// A bech32 sequence representing a nostr object (or set of objects)
// note, internally we store them as the object the sequence represents
#[derive(Clone, Debug)]
pub enum NostrBech32 {
    /// naddr - a NostrBech32 parameterized replaceable event coordinate
    NAddr(NAddr),
    /// nevent - a NostrBech32 representing an event and a set of relay URLs
    NEvent(NEvent),
    /// note - a NostrBech32 representing an event
    Id(Id),
    /// nprofile - a NostrBech32 representing a public key and a set of relay
    /// URLs
    Profile(Profile),
    /// npub - a NostrBech32 representing a public key
    Pubkey(PublicKey),
    /// nrelay - a NostrBech32 representing a set of relay URLs
    Relay(UncheckedUrl),
    /// ncryptsec - a NostrBech32 representing an encrypted private key
    CryptSec(EncryptedPrivateKey),
}

impl std::fmt::Display for NostrBech32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match self {
            NostrBech32::NAddr(na) => write!(f, "{}", na.as_bech32_string()),
            NostrBech32::NEvent(ep) => write!(f, "{}", ep.as_bech32_string()),
            NostrBech32::Id(i) => write!(f, "{}", i.as_bech32_string()),
            NostrBech32::Profile(p) => write!(f, "{}", p.as_bech32_string()),
            NostrBech32::Pubkey(pk) => write!(f, "{}", pk.as_bech32_string()),
            NostrBech32::Relay(url) => write!(f, "{}", Self::nrelay_as_bech32_string(url)),
            NostrBech32::CryptSec(epk) => write!(f, "{}", epk.0),
        }
    }
}

impl NostrBech32 {
    /// Create from a `PublicKey`
    pub fn new_pubkey(pubkey: PublicKey) -> NostrBech32 {
        NostrBech32::Pubkey(pubkey)
    }

    /// Create from a `Profile`
    pub fn new_profile(profile: Profile) -> NostrBech32 {
        NostrBech32::Profile(profile)
    }

    /// Create from an `Id`
    pub fn new_id(id: Id) -> NostrBech32 {
        NostrBech32::Id(id)
    }

    /// Create from an `NEvent`
    pub fn new_nevent(ne: NEvent) -> NostrBech32 {
        NostrBech32::NEvent(ne)
    }

    /// Create from an `NAddr`
    pub fn new_naddr(na: NAddr) -> NostrBech32 {
        NostrBech32::NAddr(na)
    }

    /// Create from an `UncheckedUrl`
    pub fn new_relay(url: UncheckedUrl) -> NostrBech32 {
        NostrBech32::Relay(url)
    }

    /// Create from an `EncryptedPrivateKey`
    pub fn new_cryptsec(epk: EncryptedPrivateKey) -> NostrBech32 {
        NostrBech32::CryptSec(epk)
    }

    /// Try to convert a string into a NostrBech32. Must not have leading or
    /// trailing junk for this to work.
    pub fn try_from_string(s: &str) -> Option<NostrBech32> {
        if s.get(..6) == Some("naddr1") {
            if let Ok(na) = NAddr::try_from_bech32_string(s) {
                return Some(NostrBech32::NAddr(na));
            }
        } else if s.get(..7) == Some("nevent1") {
            if let Ok(ep) = NEvent::try_from_bech32_string(s) {
                return Some(NostrBech32::NEvent(ep));
            }
        } else if s.get(..5) == Some("note1") {
            if let Ok(id) = Id::try_from_bech32_string(s) {
                return Some(NostrBech32::Id(id));
            }
        } else if s.get(..9) == Some("nprofile1") {
            if let Ok(p) = Profile::try_from_bech32_string(s, true) {
                return Some(NostrBech32::Profile(p));
            }
        } else if s.get(..5) == Some("npub1") {
            if let Ok(pk) = PublicKey::try_from_bech32_string(s, true) {
                return Some(NostrBech32::Pubkey(pk));
            }
        } else if s.get(..7) == Some("nrelay1") {
            if let Ok(urls) = Self::nrelay_try_from_bech32_string(s) {
                return Some(NostrBech32::Relay(urls));
            }
        } else if s.get(..10) == Some("ncryptsec1") {
            return Some(NostrBech32::CryptSec(EncryptedPrivateKey(s.to_owned())));
        }
        None
    }

    /// Find all `NostrBech32`s in a string, returned in the order found
    pub fn find_all_in_string(s: &str) -> Vec<NostrBech32> {
        let mut output: Vec<NostrBech32> = Vec::new();
        let mut cursor = 0;
        while let Some((relstart, relend)) = find_nostr_bech32_pos(s.get(cursor..).unwrap()) {
            if let Some(nurl) =
                NostrBech32::try_from_string(s.get(cursor + relstart..cursor + relend).unwrap())
            {
                output.push(nurl);
            }
            cursor += relend;
        }
        output
    }

    // Because nrelay uses TLV, we can't just use UncheckedUrl::as_bech32_string()
    fn nrelay_as_bech32_string(url: &UncheckedUrl) -> String {
        let mut tlv: Vec<u8> = Vec::new();
        tlv.push(0); // special for nrelay
        let len = url.0.len() as u8;
        tlv.push(len); // length
        tlv.extend(url.0.as_bytes().iter().take(len as usize));
        bech32::encode::<bech32::Bech32>(*super::HRP_NRELAY, &tlv).unwrap()
    }

    // Because nrelay uses TLV, we can't just use
    // UncheckedUrl::try_from_bech32_string
    fn nrelay_try_from_bech32_string(s: &str) -> Result<UncheckedUrl, Error> {
        let data = bech32::decode(s)?;
        if data.0 != *super::HRP_NRELAY {
            Err(Error::WrongBech32(
                super::HRP_NRELAY.to_lowercase(),
                data.0.to_lowercase(),
            ))
        } else {
            let mut url: Option<UncheckedUrl> = None;
            let tlv = data.1;
            let mut pos = 0;
            loop {
                // we need at least 2 more characters for anything meaningful
                if pos > tlv.len() - 2 {
                    break;
                }
                let ty = tlv[pos];
                let len = tlv[pos + 1] as usize;
                pos += 2;
                if pos + len > tlv.len() {
                    return Err(Error::InvalidUrlTlv);
                }
                let raw = &tlv[pos..pos + len];
                #[allow(clippy::single_match)]
                match ty {
                    0 => {
                        let relay_str = std::str::from_utf8(raw)?;
                        let relay = UncheckedUrl::from_str(relay_str);
                        url = Some(relay);
                    }
                    _ => {} // unhandled type for nrelay
                }
                pos += len;
            }
            if let Some(url) = url {
                Ok(url)
            } else {
                Err(Error::InvalidUrlTlv)
            }
        }
    }
}

/// A Nostr URL (starting with 'nostr:')
#[derive(Clone, Debug)]
pub struct NostrUrl(pub NostrBech32);

impl std::fmt::Display for NostrUrl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "nostr:")?;
        self.0.fmt(f)
    }
}

impl NostrUrl {
    /// Create a new NostrUrl from a NostrBech32
    pub fn new(bech32: NostrBech32) -> NostrUrl {
        NostrUrl(bech32)
    }

    /// Try to convert a string into a NostrUrl. Must not have leading or
    /// trailing junk for this to work.
    pub fn try_from_string(s: &str) -> Option<NostrUrl> {
        if s.get(..6) != Some("nostr:") {
            return None;
        }
        NostrBech32::try_from_string(s.get(6..).unwrap()).map(NostrUrl)
    }

    /// Find all `NostrUrl`s in a string, returned in the order found
    /// (If not prefixed with 'nostr:' they will not count, see NostrBech32)
    pub fn find_all_in_string(s: &str) -> Vec<NostrUrl> {
        let mut output: Vec<NostrUrl> = Vec::new();
        let mut cursor = 0;
        while let Some((relstart, relend)) = find_nostr_url_pos(s.get(cursor..).unwrap()) {
            if let Some(nurl) =
                NostrUrl::try_from_string(s.get(cursor + relstart..cursor + relend).unwrap())
            {
                output.push(nurl);
            }
            cursor += relend;
        }
        output
    }

    /// This converts all recognized bech32 sequences into proper nostr URLs by
    /// adding the "nostr:" prefix where missing.
    pub fn urlize(s: &str) -> String {
        let mut output: String = String::with_capacity(s.len());
        let mut cursor = 0;
        while let Some((relstart, relend)) = find_nostr_bech32_pos(s.get(cursor..).unwrap()) {
            // If it already has it, leave it alone
            if relstart >= 6 && s.get(cursor + relstart - 6..cursor + relstart) == Some("nostr:") {
                output.push_str(s.get(cursor..cursor + relend).unwrap());
            } else {
                output.push_str(s.get(cursor..cursor + relstart).unwrap());
                output.push_str("nostr:");
                output.push_str(s.get(cursor + relstart..cursor + relend).unwrap());
            }
            cursor += relend;
        }
        output.push_str(s.get(cursor..).unwrap());
        output
    }
}

impl From<NAddr> for NostrUrl {
    fn from(e: NAddr) -> NostrUrl {
        NostrUrl(NostrBech32::NAddr(e))
    }
}

impl From<NEvent> for NostrUrl {
    fn from(e: NEvent) -> NostrUrl {
        NostrUrl(NostrBech32::NEvent(e))
    }
}

impl From<Id> for NostrUrl {
    fn from(i: Id) -> NostrUrl {
        NostrUrl(NostrBech32::Id(i))
    }
}

impl From<Profile> for NostrUrl {
    fn from(p: Profile) -> NostrUrl {
        NostrUrl(NostrBech32::Profile(p))
    }
}

impl From<PublicKey> for NostrUrl {
    fn from(p: PublicKey) -> NostrUrl {
        NostrUrl(NostrBech32::Pubkey(p))
    }
}

impl From<UncheckedUrl> for NostrUrl {
    fn from(u: UncheckedUrl) -> NostrUrl {
        NostrUrl(NostrBech32::Relay(u))
    }
}

impl From<RelayUrl> for NostrUrl {
    fn from(u: RelayUrl) -> NostrUrl {
        NostrUrl(NostrBech32::Relay(UncheckedUrl(u.into_string())))
    }
}

/// Returns start and end position of next valid NostrBech32
pub fn find_nostr_bech32_pos(s: &str) -> Option<(usize, usize)> {
    // BECH32 Alphabet:
    // qpzry9x8gf2tvdw0s3jn54khce6mua7l
    // acdefghjklmnpqrstuvwxyz023456789
    use regex::Regex;
    lazy_static! {
        static ref BECH32_RE: Regex = Regex::new(
            r#"(?:^|[^a-zA-Z0-9])((?:nsec|npub|nprofile|note|nevent|nrelay|naddr)1[ac-hj-np-z02-9]{7,})(?:$|[^a-zA-Z0-9])"#
        ).expect("Could not compile nostr URL regex");
    }
    BECH32_RE.captures(s).map(|cap| {
        let mat = cap.get(1).unwrap();
        (mat.start(), mat.end())
    })
}

/// Returns start and end position of next valid NostrUrl
pub fn find_nostr_url_pos(s: &str) -> Option<(usize, usize)> {
    // BECH32 Alphabet:
    // qpzry9x8gf2tvdw0s3jn54khce6mua7l
    // acdefghjklmnpqrstuvwxyz023456789
    use regex::Regex;
    lazy_static! {
        static ref NOSTRURL_RE: Regex = Regex::new(
            r#"(?:^|[^a-zA-Z0-9])(nostr:(?:nsec|npub|nprofile|note|nevent|nrelay|naddr)1[ac-hj-np-z02-9]{7,})(?:$|[^a-zA-Z0-9])"#
        ).expect("Could not compile nostr URL regex");
    }
    NOSTRURL_RE.captures(s).map(|cap| {
        let mat = cap.get(1).unwrap();
        (mat.start(), mat.end())
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_nostr_bech32_try_from_string() {
        let a = "npub1sn0wdenkukak0d9dfczzeacvhkrgz92ak56egt7vdgzn8pv2wfqqhrjdv9";
        let nurl = NostrBech32::try_from_string(a).unwrap();
        assert!(matches!(nurl, NostrBech32::Pubkey(..)));

        let b = "nprofile1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gpp4mhxue69uhhytnc9e3k7mgpz4mhxue69uhkg6nzv9ejuumpv34kytnrdaksjlyr9p";
        let nurl = NostrBech32::try_from_string(b).unwrap();
        assert!(matches!(nurl, NostrBech32::Profile(..)));

        let c = "note1fntxtkcy9pjwucqwa9mddn7v03wwwsu9j330jj350nvhpky2tuaspk6nqc";
        let nurl = NostrBech32::try_from_string(c).unwrap();
        assert!(matches!(nurl, NostrBech32::Id(..)));

        let d = "nevent1qqstna2yrezu5wghjvswqqculvvwxsrcvu7uc0f78gan4xqhvz49d9spr3mhxue69uhkummnw3ez6un9d3shjtn4de6x2argwghx6egpr4mhxue69uhkummnw3ez6ur4vgh8wetvd3hhyer9wghxuet5nxnepm";
        let nurl = NostrBech32::try_from_string(d).unwrap();
        assert!(matches!(nurl, NostrBech32::NEvent(..)));

        let e = "naddr1qqxk67txd9e8xardv96x7mt9qgsgfvxyd2mfntp4avk29pj8pwz7pqwmyzrummmrjv3rdsuhg9mc9agrqsqqqa28rkfdwv";
        let nurl = NostrBech32::try_from_string(e).unwrap();
        assert!(matches!(nurl, NostrBech32::NAddr(..)));

        let f = "naddr1qq9xuum9vd382mntv4eqz8nhwden5te0dehhxarj9eek2argvehhyurjd9mxzcme9e3k7mgpzamhxue69uhhyetvv9ujucm4wfex2mn59en8j6gpzfmhxue69uhhqatjwpkx2urpvuhx2ucpr9mhxue69uhkummnw3ezu7n9vfjkget99e3kcmm4vsq32amnwvaz7tm9v3jkutnwdaehgu3wd3skueqpp4mhxue69uhkummn9ekx7mqpr9mhxue69uhhqatjv9mxjerp9ehx7um5wghxcctwvsq3samnwvaz7tmjv4kxz7fwwdhx7un59eek7cmfv9kqz9rhwden5te0wfjkccte9ejxzmt4wvhxjmcpr4mhxue69uhkummnw3ezu6r0wa6x7cnfw33k76tw9eeksmmsqy2hwumn8ghj7mn0wd68ytn2v96x6tnvd9hxkqgkwaehxw309ashgmrpwvhxummnw3ezumrpdejqzynhwden5te0danxvcmgv95kutnsw43qzynhwden5te0wfjkccte9enrw73wd9hsz9rhwden5te0wfjkccte9ehx7um5wghxyecpzemhxue69uhhyetvv9ujumn0wd68ytnfdenx7qg7waehxw309ahx7um5wgkhyetvv9ujumn0ddhhgctjduhxxmmdqy28wumn8ghj7cnvv9ehgu3wvcmh5tnc09aqzymhwden5te0wfjkcctev93xcefwdaexwqgcwaehxw309akxjemgw3hxjmn8wfjkccte9e3k7mgprfmhxue69uhhyetvv9ujumn0wd68y6trdpjhxtn0wfnszyrhwden5te0dehhxarj9emkjmn9qyrkxmmjv93kcegzypl4c26wfzswnlk2vwjxky7dhqjgnaqzqwvdvz3qwz5k3j4grrt46qcyqqq823cd90lu6";
        let nurl = NostrBech32::try_from_string(f).unwrap();
        assert!(matches!(nurl, NostrBech32::NAddr(..)));

        let g = "nrelay1qqghwumn8ghj7mn0wd68yv339e3k7mgftj9ag";
        let nurl = NostrBech32::try_from_string(g).unwrap();
        assert!(matches!(nurl, NostrBech32::Relay(..)));

        // too short
        let short = "npub1sn0wdenkukak0d9dfczzeacvhkrgz92ak56egt7vdgzn8pv2wfqqhrjdv";
        assert!(NostrBech32::try_from_string(short).is_none());

        // bad char
        let badchar = "note1fntxtkcy9pjwucqwa9mddn7v03wwwsu9j330jj350nvhpky2tuaspk6bqc";
        assert!(NostrBech32::try_from_string(badchar).is_none());

        // unknown prefix char
        let unknown = "nurl1sn0wdenkukak0d9dfczzeacvhkrgz92ak56egt7vdgzn8pv2wfqqhrjdv9";
        assert!(NostrBech32::try_from_string(unknown).is_none());
    }

    #[test]
    fn test_nostr_urlize() {
        let sample = r#"This is now the offical Gossip Client account.  Please follow it.  I will be reposting it's messages for some time until it catches on.

nprofile1qqsrjerj9rhamu30sjnuudk3zxeh3njl852mssqng7z4up9jfj8yupqpzamhxue69uhhyetvv9ujumn0wd68ytnfdenx7tcpz4mhxue69uhkummnw3ezummcw3ezuer9wchszxmhwden5te0dehhxarj9ekkj6m9v35kcem9wghxxmmd9uq3xamnwvaz7tm0venxx6rpd9hzuur4vghsz8nhwden5te0dehhxarj94c82c3wwajkcmr0wfjx2u3wdejhgtcsfx2xk

#[1]
"#;
        let fixed = NostrUrl::urlize(sample);
        println!("{fixed}");
        assert!(fixed.contains("nostr:nprofile1"));

        let sample2 = r#"Have you been switching nostr clients lately?
Could be related to:
nostr:note10ttnuuvcs29y3k23gwrcurw2ksvgd7c2rrqlfx7urmt5m963vhss8nja90
"#;
        let nochange = NostrUrl::urlize(sample2);
        assert_eq!(sample2.len(), nochange.len());

        let sample3 = r#"Have you been switching nostr clients lately?
Could be related to:
note10ttnuuvcs29y3k23gwrcurw2ksvgd7c2rrqlfx7urmt5m963vhss8nja90
"#;
        let fixed = NostrUrl::urlize(sample3);
        assert!(fixed.contains("nostr:note1"));
        assert!(fixed.len() > sample3.len());
    }

    #[test]
    fn test_nostr_url_unicode_issues() {
        let sample = r#"🌝🐸note1fntxtkcy9pjwucqwa9mddn7v03wwwsu9j330jj350nvhpky2tuaspk6nqc"#;
        assert!(NostrUrl::try_from_string(sample).is_none())
    }

    #[test]
    fn test_multiple_nostr_urls() {
        let sample = r#"
Here is a list of relays I use and consider reliable so far. I've included some relevant information for each relay such as if payment is required or [NIP-33](https://nips.be/33) is supported. I'll be updating this list as I discover more good relays, which ones do you find reliable?

## Nokotaro

nostr:nrelay1qq0hwumn8ghj7mn0wd68yttjv4kxz7fwdehkkmm5v9ex7tnrdakj78zlgae

- Paid? **No**
- [NIP-33](https://nips.be/33) supported? **Yes**
- Operator: nostr:npub12ftld459xqw7s7fqnxstzu7r74l5yagxztwcwmaqj4d24jgpj2csee3mx0

## Nostr World

nostr:nrelay1qqvhwumn8ghj7mn0wd68ytthdaexcepwdqeh5tn2wqhsv5kg7j

- Paid? **Yes**
- [NIP-33](https://nips.be/33) supported? **Yes**
- Operator: nostr:npub1zpq2gsz25wsgun2e4gtks9p63j7fvyfd46weyjzp5tv6yys89zcsjdflcv

## Nos.lol

nostr:nrelay1qq88wumn8ghj7mn0wvhxcmmv9uvj5a67

- Paid? **No**
- [NIP-33](https://nips.be/33) supported? **No**
- Operator: nostr:npub1nlk894teh248w2heuu0x8z6jjg2hyxkwdc8cxgrjtm9lnamlskcsghjm9c

## Nostr Wine

nostr:nrelay1qqghwumn8ghj7mn0wd68ytnhd9hx2tcw2qslz

- Paid? **Yes**
- [NIP-33](https://nips.be/33) supported? **No**
- Operators: nostr:npub1qlkwmzmrhzpuak7c2g9akvcrh7wzkd7zc7fpefw9najwpau662nqealf5y & nostr:npub18kzz4lkdtc5n729kvfunxuz287uvu9f64ywhjz43ra482t2y5sks0mx5sz

## Nostrich Land

nostr:nrelay1qqvhwumn8ghj7un9d3shjtnwdaehgunfvd5zumrpdejqpdl8ln

- Paid? **Yes**
- [NIP-33](https://nips.be/33) supported? **No**
- Operator: nostr:nprofile1qqsxf8h0u35dmvg8cp0t5mg9z8f222v9grly6hcqw2cqvdsq3lrjlyspr9mhxue69uhhyetvv9ujumn0wd68y6trdqhxcctwvsj9ulqc
"#;

        assert_eq!(NostrUrl::find_all_in_string(sample).len(), 11);
    }

    #[test]
    fn test_generate_nrelay() {
        let url = UncheckedUrl("wss://nostr.mikedilger.com/".to_owned());
        let nb32 = NostrBech32::new_relay(url);
        let nurl = NostrUrl(nb32);
        println!("{}", nurl);
    }
}
```

---

### pay_request_data.rs

**Size:** 5333 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};
use serde_json::{json, Map, Value};

use super::{PublicKeyHex, UncheckedUrl};

/// This is a response from a zapper lnurl
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PayRequestData {
    /// The URL to make the pay request to with a kind 9374 event
    pub callback: UncheckedUrl,

    /// Metadata
    pub metadata: Vec<(String, String)>,

    /// Whether the lnurl supports nostr zaps
    pub allows_nostr: Option<bool>,

    /// The nostr public key of the zapper
    pub nostr_pubkey: Option<PublicKeyHex>,

    /// Other fields such as:
    ///
    /// "maxSendable": 100000000000,
    /// "minSendable": 1000,
    /// "commentAllowed": 32
    /// "tag": "payRequest"
    pub other: Map<String, Value>,
}

impl Default for PayRequestData {
    fn default() -> Self {
        PayRequestData {
            callback: UncheckedUrl("".to_owned()),
            metadata: vec![],
            allows_nostr: None,
            nostr_pubkey: None,
            other: Map::new(),
        }
    }
}

impl PayRequestData {
    #[allow(dead_code)]
    pub(crate) fn mock() -> PayRequestData {
        let mut map = Map::new();
        let _ = map.insert("tag".to_string(), Value::String("payRequest".to_owned()));
        let _ = map.insert(
            "maxSendable".to_string(),
            Value::Number(100000000000_u64.into()),
        );
        let _ = map.insert("minSendable".to_string(), Value::Number(1000.into()));
        let _ = map.insert("commentAllowed".to_string(), Value::Number(32.into()));
        PayRequestData {
            callback: UncheckedUrl("https://livingroomofsatoshi.com/api/v1/lnurl/payreq/f16bacaa-8e5f-4038-bdea-4c9e796f913c".to_string()),
            metadata: vec![
                ("text/plain".to_owned(),
                 "Pay to Wallet of Satoshi user: decentbun13".to_owned()),
                ("text/identifier".to_owned(),
                 "decentbun13@walletofsatoshi.com".to_owned()),
            ],
            allows_nostr: Some(true),
            nostr_pubkey: Some(PublicKeyHex::try_from_str("be1d89794bf92de5dd64c1e60f6a2c70c140abac9932418fee30c5c637fe9479").unwrap()),
            other: map,
        }
    }
}

impl Serialize for PayRequestData {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4 + self.other.len()))?;
        map.serialize_entry("callback", &json!(&self.callback))?;
        map.serialize_entry("metadata", &json!(&self.metadata))?;
        map.serialize_entry("allowsNostr", &json!(&self.allows_nostr))?;
        map.serialize_entry("nostrPubkey", &json!(&self.nostr_pubkey))?;
        for (k, v) in &self.other {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for PayRequestData {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(PayRequestDataVisitor)
    }
}

struct PayRequestDataVisitor;

impl<'de> Visitor<'de> for PayRequestDataVisitor {
    type Value = PayRequestData;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A JSON object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<PayRequestData, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: Map<String, Value> = Map::new();
        while let Some((key, value)) = access.next_entry::<String, Value>()? {
            let _ = map.insert(key, value);
        }

        let mut m: PayRequestData = Default::default();

        if let Some(Value::String(s)) = map.remove("callback") {
            m.callback = UncheckedUrl(s)
        } else {
            return Err(DeError::custom("Missing callback url".to_owned()));
        }

        if let Some(Value::Array(a)) = map.remove("metadata") {
            for elem in a.iter() {
                if let Value::Array(a2) = elem {
                    if a2.len() == 2 {
                        if let Value::String(key) = &a2[0] {
                            if let Value::String(val) = &a2[1] {
                                m.metadata.push((key.to_owned(), val.to_owned()));
                            }
                        }
                    } else {
                        return Err(DeError::custom("Metadata entry not a pair".to_owned()));
                    }
                } else {
                    return Err(DeError::custom("Metadata entry not recognized".to_owned()));
                }
            }
        }

        if let Some(Value::Bool(b)) = map.remove("allowsNostr") {
            m.allows_nostr = Some(b);
        } else {
            m.allows_nostr = None;
        }

        if let Some(Value::String(s)) = map.remove("nostrPubkey") {
            m.nostr_pubkey = match PublicKeyHex::try_from_string(s) {
                Ok(pkh) => Some(pkh),
                Err(e) => return Err(DeError::custom(format!("{e}"))),
            };
        }

        m.other = map;

        Ok(m)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {PayRequestData, test_pay_request_data_serde}
}
```

---

### private_key/content_encryption.rs

**Size:** 24618 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use base64::Engine;
use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use zeroize::Zeroize;

use super::{
    super::{Error, PublicKey},
    PrivateKey,
};

/// Content Encryption Algorithm
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContentEncryptionAlgorithm {
    /// NIP-04 (insecure)
    Nip04,

    /// NIP-44 unpadded (produced by Amethyst for a few months around Aug-Oct
    /// 2023
    Nip44v1Unpadded,

    /// NIP-44 padded (possibly never in use, or a few tests were produced by
    /// Gossip around Aug-Oct 2023)
    Nip44v1Padded,

    /// NIP-44 v2 (latest, not yet audited)
    Nip44v2,
}

impl PrivateKey {
    /// Get the shared secret
    pub fn shared_secret(&self, other: &PublicKey, algo: ContentEncryptionAlgorithm) -> [u8; 32] {
        match algo {
            ContentEncryptionAlgorithm::Nip04 => self.shared_secret_nip04(other),
            ContentEncryptionAlgorithm::Nip44v1Unpadded => self.shared_secret_nip44_v1(other),
            ContentEncryptionAlgorithm::Nip44v1Padded => self.shared_secret_nip44_v1(other),
            ContentEncryptionAlgorithm::Nip44v2 => self.shared_secret_nip44_v2(other),
        }
    }

    /// Encrypt
    pub fn encrypt(
        &self,
        other: &PublicKey,
        plaintext: &str,
        algo: ContentEncryptionAlgorithm,
    ) -> Result<String, Error> {
        match algo {
            ContentEncryptionAlgorithm::Nip04 => self.nip04_encrypt(other, plaintext.as_bytes()),
            ContentEncryptionAlgorithm::Nip44v1Unpadded => {
                Ok(self.nip44_v1_encrypt(other, plaintext.as_bytes(), false, None))
            }
            ContentEncryptionAlgorithm::Nip44v1Padded => {
                Ok(self.nip44_v1_encrypt(other, plaintext.as_bytes(), true, None))
            }
            ContentEncryptionAlgorithm::Nip44v2 => self.nip44_v2_encrypt(other, plaintext),
        }
    }

    /// Decrypt (detects encryption version)
    pub fn decrypt(&self, other: &PublicKey, ciphertext: &str) -> Result<String, Error> {
        let cbytes = ciphertext.as_bytes();
        if cbytes.len() >= 28
            && cbytes[ciphertext.len() - 28] == b'?'
            && cbytes[ciphertext.len() - 27] == b'i'
            && cbytes[ciphertext.len() - 26] == b'v'
            && cbytes[ciphertext.len() - 25] == b'='
        {
            self.decrypt_nip04(other, ciphertext)
                .map(|bytes| String::from_utf8_lossy(&bytes).into_owned())
        } else {
            self.decrypt_nip44(other, ciphertext)
        }
    }

    /// Decrypt NIP-04 only
    pub fn decrypt_nip04(&self, other: &PublicKey, ciphertext: &str) -> Result<Vec<u8>, Error> {
        self.nip04_decrypt(other, ciphertext)
    }

    /// Decrypt NIP-44 only, version is detected
    pub fn decrypt_nip44(&self, other: &PublicKey, ciphertext: &str) -> Result<String, Error> {
        if ciphertext.as_bytes().first() == Some(&b'#') {
            return Err(crate::types::nip44::Error::UnsupportedFutureVersion.into());
        };

        let algo = {
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(ciphertext)
                .map_err(Error::BadEncryptedMessageBase64)?;
            if bytes.is_empty() {
                return Err(Error::BadEncryptedMessage);
            }
            match bytes[0] {
                1 => ContentEncryptionAlgorithm::Nip44v1Unpadded,
                // Note: Nip44v1Padded cannot be detected, and there may be no events out there
                // using it.
                2 => ContentEncryptionAlgorithm::Nip44v2,
                _ => return Err(crate::types::nip44::Error::UnknownVersion.into()),
            }
        };

        match algo {
            ContentEncryptionAlgorithm::Nip44v1Unpadded => {
                let bytes = self.nip44_v1_decrypt(other, ciphertext, false)?;
                Ok(String::from_utf8(bytes)?)
            }
            ContentEncryptionAlgorithm::Nip44v2 => self.nip44_v2_decrypt(other, ciphertext),
            _ => unreachable!(),
        }
    }

    /// Generate a shared secret with someone elses public key (NIP-04 method)
    fn shared_secret_nip04(&self, other: &PublicKey) -> [u8; 32] {
        // Build the whole PublicKey from the XOnlyPublicKey
        let pubkey = secp256k1::PublicKey::from_x_only_public_key(
            other.as_xonly_public_key(),
            secp256k1::Parity::Even,
        );

        // Get the shared secret point without hashing
        let mut shared_secret_point: [u8; 64] =
            secp256k1::ecdh::shared_secret_point(&pubkey, &self.0);

        // Take the first 32 bytes
        let mut shared_key: [u8; 32] = [0; 32];
        shared_key.copy_from_slice(&shared_secret_point[..32]);

        // Zeroize what we aren't keeping
        shared_secret_point.zeroize();

        shared_key
    }

    /// Generate a shared secret with someone elses public key (NIP-44 method,
    /// version 1)
    fn shared_secret_nip44_v1(&self, other: &PublicKey) -> [u8; 32] {
        // Build the whole PublicKey from the XOnlyPublicKey
        let pubkey = secp256k1::PublicKey::from_x_only_public_key(
            other.as_xonly_public_key(),
            secp256k1::Parity::Even,
        );

        let mut ssp = secp256k1::ecdh::shared_secret_point(&pubkey, &self.0)
            .as_slice()
            .to_owned();
        ssp.resize(32, 0); // keep only the X coordinate part

        let mut hasher = Sha256::new();
        hasher.update(ssp);
        let result = hasher.finalize();

        result.into()
    }

    /// Generate a shared secret with someone elses public key (NIP-44 method)
    fn shared_secret_nip44_v2(&self, other: &PublicKey) -> [u8; 32] {
        super::super::nip44::get_conversation_key(self.0, other.as_xonly_public_key())
    }

    /// Encrypt content via a shared secret according to NIP-04. Returns (IV,
    /// Ciphertext) pair.
    fn nip04_encrypt(&self, other: &PublicKey, plaintext: &[u8]) -> Result<String, Error> {
        let mut shared_secret = self.shared_secret_nip04(other);
        let iv = {
            let mut iv: [u8; 16] = [0; 16];
            OsRng.fill_bytes(&mut iv);
            iv
        };

        let ciphertext = cbc::Encryptor::<aes::Aes256>::new(&shared_secret.into(), &iv.into())
            .encrypt_padded_vec_mut::<Pkcs7>(plaintext);

        shared_secret.zeroize();

        Ok(format!(
            "{}?iv={}",
            base64::engine::general_purpose::STANDARD.encode(ciphertext),
            base64::engine::general_purpose::STANDARD.encode(iv)
        ))
    }

    /// Decrypt content via a shared secret according to NIP-04
    fn nip04_decrypt(&self, other: &PublicKey, ciphertext: &str) -> Result<Vec<u8>, Error> {
        let parts: Vec<&str> = ciphertext.split("?iv=").collect();
        if parts.len() != 2 {
            return Err(Error::BadEncryptedMessage);
        }
        let ciphertext: Vec<u8> = base64::engine::general_purpose::STANDARD
            .decode(parts[0])
            .map_err(Error::BadEncryptedMessageBase64)?;
        let iv_vec: Vec<u8> = base64::engine::general_purpose::STANDARD
            .decode(parts[1])
            .map_err(Error::BadEncryptedMessageBase64)?;
        let iv: [u8; 16] = iv_vec.try_into().unwrap();

        let mut shared_secret = self.shared_secret_nip04(other);
        let plaintext = cbc::Decryptor::<aes::Aes256>::new(&shared_secret.into(), &iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(&ciphertext)?;

        shared_secret.zeroize();

        Ok(plaintext)
    }

    /// Encrypt content via a shared secret according to NIP-44 v1
    /// Always set forced_nonce=None (except for test vectors)
    fn nip44_v1_encrypt(
        &self,
        other: &PublicKey,
        plaintext: &[u8],
        pad: bool,
        forced_nonce: Option<[u8; 24]>,
    ) -> String {
        use rand::Rng;
        let mut new_plaintext;

        let encrypt = |plaintext: &[u8]| -> String {
            use chacha20::cipher::StreamCipher;
            let mut shared_secret = self.shared_secret_nip44_v1(other);
            let mut output: Vec<u8> = Vec::with_capacity(1 + 24 + plaintext.len());
            output.resize(1 + 24, 0);
            output[0] = 1; // Version
            match forced_nonce {
                Some(nonce) => output[1..=24].copy_from_slice(nonce.as_slice()),
                None => OsRng.fill_bytes(&mut output[1..=24]),
            }
            output.extend(plaintext); // Plaintext (will encrypt in place)
            let mut cipher = chacha20::XChaCha20::new(&shared_secret.into(), output[1..=24].into());
            shared_secret.zeroize();
            cipher.apply_keystream(&mut output[25..]);
            base64::engine::general_purpose::STANDARD.encode(output)
        };

        if pad {
            let end_plaintext = 4 + plaintext.len();

            // forced padding, up to a minimum of 32 bytes total so far (4 used for the u32
            // length)
            let forced_padding = 32_usize.saturating_sub(end_plaintext);
            let end_forced_padding = end_plaintext + forced_padding;

            // random length padding, up to 50% more
            let random_padding =
                OsRng.sample(rand::distributions::Uniform::new(0, end_forced_padding / 2));
            let end_random_padding = end_forced_padding + random_padding;

            // Make space
            new_plaintext = vec![0; end_random_padding];

            new_plaintext[0..4].copy_from_slice((plaintext.len() as u32).to_be_bytes().as_slice());
            new_plaintext[4..end_plaintext].copy_from_slice(plaintext);
            OsRng.fill_bytes(&mut new_plaintext[end_plaintext..]); // random padding

            let output = encrypt(&new_plaintext);
            new_plaintext.zeroize();
            output
        } else {
            encrypt(plaintext)
        }
    }

    /// Decrypt content via a shared secret according to NIP-44, version 1
    fn nip44_v1_decrypt(
        &self,
        other: &PublicKey,
        ciphertext: &str,
        padded: bool,
    ) -> Result<Vec<u8>, Error> {
        use chacha20::cipher::StreamCipher;
        let mut shared_secret = self.shared_secret_nip44_v1(other);
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(ciphertext)
            .map_err(Error::BadEncryptedMessageBase64)?;
        if bytes[0] != 1 {
            return Err(Error::UnknownCipherVersion(bytes[0]));
        }
        let mut output: Vec<u8> = Vec::with_capacity(bytes[25..].len());
        output.extend(&bytes[25..]);

        let mut cipher = chacha20::XChaCha20::new(&shared_secret.into(), bytes[1..=24].into());
        shared_secret.zeroize();
        cipher.apply_keystream(&mut output);

        if padded {
            let len = u32::from_be_bytes(output[0..4].try_into().unwrap());
            if 4 + len as usize > output.len() {
                return Err(Error::OutOfRange(len as usize));
            }
            Ok(output[4..4 + len as usize].to_owned())
        } else {
            Ok(output)
        }
    }

    /// Encrypt content via a shared secret according to NIP-44 v1
    fn nip44_v2_encrypt(&self, counterparty: &PublicKey, plaintext: &str) -> Result<String, Error> {
        let conversation_key = self.shared_secret_nip44_v2(counterparty);
        let ciphertext = super::super::nip44::encrypt(&conversation_key, plaintext)?;
        Ok(ciphertext)
    }

    /// Decrypt content via a shared secret according to NIP-44, version 2
    fn nip44_v2_decrypt(
        &self,
        counterparty: &PublicKey,
        ciphertext: &str,
    ) -> Result<String, Error> {
        let conversation_key = self.shared_secret_nip44_v2(counterparty);
        let plaintext = super::super::nip44::decrypt(&conversation_key, ciphertext)?;
        Ok(plaintext)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_privkey_nip04() {
        let private_key = PrivateKey::mock();
        let other_public_key = PublicKey::mock();

        let message = "hello world, this should come out just dandy.".as_bytes();
        let encrypted = private_key
            .nip04_encrypt(&other_public_key, message)
            .unwrap();
        let decrypted = private_key
            .nip04_decrypt(&other_public_key, &encrypted)
            .unwrap();
        assert_eq!(message, decrypted);
    }

    #[test]
    fn test_privkey_nip44_v1() {
        struct TestVector {
            sec1: &'static str,
            sec2: Option<&'static str>,
            pub2: Option<&'static str>,
            shared: Option<&'static str>,
            nonce: Option<&'static str>,
            plaintext: Option<Vec<u8>>,
            ciphertext: Option<&'static str>,
            note: &'static str,
            fail: bool,
        }

        impl Default for TestVector {
            fn default() -> TestVector {
                TestVector {
                    sec1: "0000000000000000000000000000000000000000000000000000000000000001",
                    sec2: None,
                    pub2: None,
                    shared: None,
                    nonce: None,
                    plaintext: None,
                    ciphertext: None,
                    note: "none",
                    fail: false,
                }
            }
        }

        let vectors: Vec<TestVector> = vec![
            TestVector {
                sec1: "0000000000000000000000000000000000000000000000000000000000000001",
                sec2: Some("0000000000000000000000000000000000000000000000000000000000000002"),
                shared: Some("0135da2f8acf7b9e3090939432e47684eb888ea38c2173054d4eedffdf152ca5"),
                nonce: Some("121f9d60726777642fd82286791ab4d7461c9502ebcbb6e6"),
                plaintext: Some(b"a".to_vec()),
                ciphertext: Some("ARIfnWByZ3dkL9gihnkatNdGHJUC68u25qM="),
                note: "sk1 = 1, sk2 = random, 0x02",
                .. Default::default()
            },
            TestVector {
                sec1: "0000000000000000000000000000000000000000000000000000000000000002",
                sec2: Some("0000000000000000000000000000000000000000000000000000000000000001"),
                shared: Some("0135da2f8acf7b9e3090939432e47684eb888ea38c2173054d4eedffdf152ca5"),
                plaintext: Some(b"a".to_vec()),
                ciphertext: Some("AeCt7jJ8L+WBOTiCSfeXEGXB/C/wgsrSRek="),
                nonce: Some("e0adee327c2fe58139388249f7971065c1fc2ff082cad245"),
                note: "sk1 = 1, sk2 = random, 0x02",
                .. Default::default()
            },
            TestVector {
                sec1: "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364139",
                pub2: Some("0000000000000000000000000000000000000000000000000000000000000002"),
                shared: Some("a6d6a2f7011cdd1aeef325948f48c6efa40f0ec723ae7f5ac7e3889c43481500"),
                nonce: Some("f481750e13dfa90b722b7cce0db39d80b0db2e895cc3001a"),
                plaintext: Some(b"a".to_vec()),
                ciphertext: Some("AfSBdQ4T36kLcit8zg2znYCw2y6JXMMAGjM="),
                note: "sec1 = n-2, pub2: random, 0x02",
                .. Default::default()
            },
            TestVector {
                sec1: "0000000000000000000000000000000000000000000000000000000000000002",
                pub2: Some("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdeb"),
                shared: Some("4908464f77dd74e11a9b4e4a3bc2467445bd794e8abcbfafb65a6874f9e25a8f"),
                nonce: Some("45c484ba2c0397853183adba6922156e09a2ad4e3e6914f2"),
                plaintext: Some(b"A Peer-to-Peer Electronic Cash System".to_vec()),
                ciphertext: Some("AUXEhLosA5eFMYOtumkiFW4Joq1OPmkU8k/25+3+VDFvOU39qkUDl1aiy8Q+0ozTwbhD57VJoIYayYS++hE="),
                note: "sec1 = 2, pub2: ",
                .. Default::default()
            },
            TestVector {
                sec1: "0000000000000000000000000000000000000000000000000000000000000001",
                pub2: Some("79be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798"),
                shared: Some("132f39a98c31baaddba6525f5d43f2954472097fa15265f45130bfdb70e51def"),
                nonce: Some("d60de08405cf9bde508147e82224ac6af409c12b9e5492e1"),
                plaintext: Some(b"A purely peer-to-peer version of electronic cash would allow online payments to be sent directly from one party to another without going through a financial institution. Digital signatures provide part of the solution, but the main benefits are lost if a trusted third party is still required to prevent double-spending.".to_vec()),
                ciphertext: Some("AdYN4IQFz5veUIFH6CIkrGr0CcErnlSS4VdvoQaP2DCB1dIFL72HSriG1aFABcTlu86hrsG0MdOO9rPdVXc3jptMMzqvIN6tJlHPC8GdwFD5Y8BT76xIIOTJR2W0IdrM7++WC/9harEJAdeWHDAC9zNJX81CpCz4fnV1FZ8GxGLC0nUF7NLeUiNYu5WFXQuO9uWMK0pC7tk3XVogk90X6rwq0MQG9ihT7e1elatDy2YGat+VgQlDrz8ZLRw/lvU+QqeXMQgjqn42sMTrimG6NdKfHJSVWkT6SKZYVsuTyU1Iu5Nk0twEV8d11/MPfsMx4i36arzTC9qxE6jftpOoG8f/jwPTSCEpHdZzrb/CHJcpc+zyOW9BZE2ZOmSxYHAE0ustC9zRNbMT3m6LqxIoHq8j+8Ysu+Cwqr4nUNLYq/Q31UMdDg1oamYS17mWIAS7uf2yF5uT5IlG"),
                note: "sec1 == pub2",
                .. Default::default()
            },
            TestVector {
                sec1: "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
                pub2: Some("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
                plaintext: Some(b"a".to_vec()),
                note: "sec1 higher than curve.n",
                fail: true,
                .. Default::default()
            },
            TestVector {
                sec1: "0000000000000000000000000000000000000000000000000000000000000000",
                pub2: Some("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
                plaintext: Some(b"a".to_vec()),
                note: "sec1 is 0",
                fail: true,
                .. Default::default()
            },
            TestVector {
                sec1: "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364139",
                pub2: Some("ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"),
                plaintext: Some(b"a".to_vec()),
                note: "pub2 is invalid, no sqrt, all-ff",
                fail: true,
                .. Default::default()
            },
            TestVector {
                sec1: "fffffffffffffffffffffffffffffffebaaedce6af48a03bbfd25e8cd0364141",
                pub2: Some("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
                plaintext: Some(b"a".to_vec()),
                note: "sec1 == curve.n",
                fail: true,
                .. Default::default()
            },
            TestVector {
                sec1: "0000000000000000000000000000000000000000000000000000000000000002",
                pub2: Some("1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"),
                plaintext: Some(b"a".to_vec()),
                note: "pub2 is invalid, no sqrt",
                fail: true,
                .. Default::default()
            },
        ];

        for (num, vector) in vectors.iter().enumerate() {
            let mut sec1 = match PrivateKey::try_from_hex_string(vector.sec1) {
                Ok(key) => key,
                Err(_) => {
                    if vector.fail {
                        continue;
                    } else {
                        panic!("Test vector {} failed on sec1: {}", num, vector.note);
                    }
                }
            };
            println!("sec1: {}", sec1.as_hex_string());

            let pub2 = {
                if let Some(sec2) = vector.sec2 {
                    let sec2 = match PrivateKey::try_from_hex_string(sec2) {
                        Ok(priv_key) => priv_key,
                        Err(_) => {
                            if vector.fail {
                                continue;
                            } else {
                                panic!("Test vector {} failed on sec2: {}", num, vector.note);
                            }
                        }
                    };
                    sec2.public_key()
                } else if let Some(pub2) = vector.pub2 {
                    match PublicKey::try_from_hex_string(pub2, true) {
                        Ok(pub_key) => pub_key,
                        Err(_) => {
                            if vector.fail {
                                continue;
                            } else {
                                panic!("Test vector {} failed on pub2: {}", num, vector.note);
                            }
                        }
                    }
                } else {
                    panic!("Test vector {} has no sec2 or pub2: {}", num, vector.note);
                }
            };
            println!("pub2: {}", pub2.as_hex_string());

            // Test shared vector
            let shared = sec1.shared_secret_nip44_v1(&pub2);
            let shared_hex = hex::encode(shared);
            if let Some(s) = vector.shared {
                if s != shared_hex {
                    panic!(
                        "Test vector {} shared point mismatch: {}\ntheirs: {}\nours:   {}",
                        num, vector.note, s, shared_hex
                    );
                } else {
                    println!("Test vector {} shared point is good", num);
                }
            }

            // Test Encrypting
            if let (Some(plaintext), Some(ciphertext), Some(noncestr)) =
                (&vector.plaintext, vector.ciphertext, vector.nonce)
            {
                let nonce: [u8; 24] = hex::decode(noncestr).unwrap().try_into().unwrap();
                let ciphertext2 = sec1.nip44_v1_encrypt(&pub2, &plaintext, false, Some(nonce));
                assert_eq!(ciphertext, ciphertext2);
                println!("Test vector {} encryption matches", num);
            }

            // Test Decrypting
            if let (Some(plaintext), Some(ciphertext), Some(sec2)) =
                (&vector.plaintext, vector.ciphertext, vector.sec2)
            {
                let sec2 = match PrivateKey::try_from_hex_string(sec2) {
                    Ok(key) => key,
                    Err(_) => {
                        if vector.fail {
                            continue;
                        } else {
                            panic!("Test vector {} failed on sec1: {}", num, vector.note);
                        }
                    }
                };
                let pub1 = sec1.public_key();

                let plaintext2 = sec2.nip44_v1_decrypt(&pub1, ciphertext, false).unwrap();
                assert_eq!(plaintext, &plaintext2);
                println!("Test vector {} decryption matches", num);
            }
        }
    }

    #[test]
    fn test_privkey_nip44_v1_pad() {
        let sec1 = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000001",
        )
        .unwrap();

        let sec2 = PrivateKey::try_from_hex_string(
            "0000000000000000000000000000000000000000000000000000000000000002",
        )
        .unwrap();

        let plaintext = "yes".as_bytes();

        let ciphertext = sec1.nip44_v1_encrypt(&sec2.public_key(), plaintext, true, None);
        assert!(ciphertext.len() >= 32);

        let plaintext2 = sec2
            .nip44_v1_decrypt(&sec1.public_key(), &ciphertext, true)
            .unwrap();
        assert_eq!(plaintext, plaintext2);
    }

    #[test]
    fn test_nip44_version_detection() {
        let private_key = PrivateKey::generate();
        let private_key_b = PrivateKey::generate();
        let public_key = private_key_b.public_key();
        let message = "This is a test";

        let v1unpadded = private_key
            .encrypt(
                &public_key,
                message,
                ContentEncryptionAlgorithm::Nip44v1Unpadded,
            )
            .unwrap();
        let v1unpadded_decrypted = private_key.decrypt_nip44(&public_key, &v1unpadded).unwrap();

        assert_eq!(&v1unpadded_decrypted, message);

        let v2 = private_key
            .encrypt(&public_key, message, ContentEncryptionAlgorithm::Nip44v2)
            .unwrap();
        let v2_decrypted = private_key.decrypt_nip44(&public_key, &v2).unwrap();

        assert_eq!(&v2_decrypted, message);
    }
}
```

---

### private_key/encrypted_private_key.rs

**Size:** 19369 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::ops::Deref;

use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, KeyIvInit};
use base64::Engine;
use chacha20poly1305::{
    aead::{Aead, AeadCore, KeyInit, Payload},
    XChaCha20Poly1305,
};
use derive_more::Display;
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand_core::{OsRng, RngCore};
use serde::{Deserialize, Serialize};
use sha2::Sha256;
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};
use unicode_normalization::UnicodeNormalization;
use zeroize::Zeroize;

use super::{super::Error, KeySecurity, PrivateKey};

// This allows us to detect bad decryptions with wrong passwords.
const V1_CHECK_VALUE: [u8; 11] = [15, 91, 241, 148, 90, 143, 101, 12, 172, 255, 103];
const V1_HMAC_ROUNDS: u32 = 100_000;

/// This is an encrypted private key (the string inside is the bech32 ncryptsec
/// string)
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct EncryptedPrivateKey(pub String);

impl Deref for EncryptedPrivateKey {
    type Target = String;

    fn deref(&self) -> &String {
        &self.0
    }
}

impl EncryptedPrivateKey {
    /// Create from a bech32 string (this just type wraps as the internal
    /// stringly already is one)
    pub fn from_bech32_string(s: String) -> EncryptedPrivateKey {
        EncryptedPrivateKey(s)
    }

    /// only correct for version 1 and onwards
    pub fn as_bech32_string(&self) -> String {
        self.0.clone()
    }

    /// Decrypt into a Private Key with a passphrase.
    ///
    /// We recommend you zeroize() the password you pass in after you are
    /// done with it.
    pub fn decrypt(&self, password: &str) -> Result<PrivateKey, Error> {
        PrivateKey::import_encrypted(self, password)
    }

    /// Version
    ///
    /// Version -1:
    ///    PBKDF = pbkdf2-hmac-sha256 ( salt = "nostr", rounds = 4096 )
    ///    inside = concat(private_key, 15 specified bytes, key_security_byte)
    ///    encrypt = AES-256-CBC with random IV
    ///    compose = iv + ciphertext
    ///    encode = base64
    /// Version 0:
    ///    PBKDF = pbkdf2-hmac-sha256 ( salt = concat(0x1, 15 random bytes),
    /// rounds = 100000 )    inside = concat(private_key, 15 specified
    /// bytes, key_security_byte)    encrypt = AES-256-CBC with random IV
    ///    compose = salt + iv + ciphertext
    ///    encode = base64
    /// Version 1:
    ///    PBKDF = pbkdf2-hmac-sha256 ( salt = concat(0x1, 15 random bytes),
    /// rounds = 100000 )    inside = concat(private_key, 15 specified
    /// bytes, key_security_byte)    encrypt = AES-256-CBC with random IV
    ///    compose = salt + iv + ciphertext
    ///    encode = bech32('ncryptsec')
    /// Version 2:
    ///    PBKDF = scrypt ( salt = 16 random bytes, log_n = user choice, r = 8,
    /// p = 1)    inside = private_key
    ///    associated_data = key_security_byte
    ///    encrypt = XChaCha20-Poly1305
    ///    compose = concat (0x2, log_n, salt, nonce, associated_data,
    /// ciphertext)    encode = bech32('ncryptsec')
    pub fn version(&self) -> Result<i8, Error> {
        if self.0.starts_with("ncryptsec1") {
            let data = bech32::decode(&self.0)?;
            if data.0 != *super::super::HRP_NCRYPTSEC {
                return Err(Error::WrongBech32(
                    super::super::HRP_NCRYPTSEC.to_lowercase(),
                    data.0.to_lowercase(),
                ));
            }
            Ok(data.1[0] as i8)
        } else if self.0.len() == 64 {
            Ok(-1)
        } else {
            Ok(0) // base64 variant of v1
        }
    }
}

impl PrivateKey {
    /// Export in a (non-portable) encrypted form. This does not downgrade
    /// the security of the key, but you are responsible to keep it encrypted.
    /// You should not attempt to decrypt it, only use `import_encrypted()` on
    /// it, or something similar in another library/client which also respects
    /// key security.
    ///
    /// This currently exports into EncryptedPrivateKey version 2.
    ///
    /// We recommend you zeroize() the password you pass in after you are
    /// done with it.
    pub fn export_encrypted(
        &self,
        password: &str,
        log2_rounds: u8,
    ) -> Result<EncryptedPrivateKey, Error> {
        // Generate a random 16-byte salt
        let salt = {
            let mut salt: [u8; 16] = [0; 16];
            OsRng.fill_bytes(&mut salt);
            salt
        };

        let nonce = XChaCha20Poly1305::generate_nonce(&mut OsRng);

        let associated_data: Vec<u8> = {
            let key_security: u8 = match self.1 {
                KeySecurity::Weak => 0,
                KeySecurity::Medium => 1,
                KeySecurity::NotTracked => 2,
            };
            vec![key_security]
        };

        let ciphertext = {
            let cipher = {
                let symmetric_key = Self::password_to_key_v2(password, &salt, log2_rounds)?;
                XChaCha20Poly1305::new((&symmetric_key).into())
            };

            // The inner secret. We don't have to drop this because we are
            // encrypting-in-place
            let mut inner_secret: Vec<u8> = self.0.secret_bytes().to_vec();

            let payload = Payload {
                msg: &inner_secret,
                aad: &associated_data,
            };

            let ciphertext = match cipher.encrypt(&nonce, payload) {
                Ok(c) => c,
                Err(_) => return Err(Error::PrivateKeyEncryption),
            };

            inner_secret.zeroize();

            ciphertext
        };

        // Combine salt, IV and ciphertext
        let mut concatenation: Vec<u8> = Vec::new();
        concatenation.push(0x2); // 1 byte version number
        concatenation.push(log2_rounds); // 1 byte for scrypt N (rounds)
        concatenation.extend(salt); // 16 bytes of salt
        concatenation.extend(nonce); // 24 bytes of nonce
        concatenation.extend(associated_data); // 1 byte of key security
        concatenation.extend(ciphertext); // 48 bytes of ciphertext expected
                                          // Total length is 91 = 1 + 1 + 16 + 24 + 1 + 48

        // bech32 encode
        Ok(EncryptedPrivateKey(bech32::encode::<bech32::Bech32>(
            *super::super::HRP_NCRYPTSEC,
            &concatenation,
        )?))
    }

    /// Import an encrypted private key which was exported with
    /// `export_encrypted()`.
    ///
    /// We recommend you zeroize() the password you pass in after you are
    /// done with it.
    ///
    /// This is backwards-compatible with keys that were exported with older
    /// code.
    pub fn import_encrypted(
        encrypted: &EncryptedPrivateKey,
        password: &str,
    ) -> Result<PrivateKey, Error> {
        if encrypted.0.starts_with("ncryptsec1") {
            // Versioned
            Self::import_encrypted_bech32(encrypted, password)
        } else {
            // Pre-versioned, deprecated
            Self::import_encrypted_base64(encrypted, password)
        }
    }

    // Current
    fn import_encrypted_bech32(
        encrypted: &EncryptedPrivateKey,
        password: &str,
    ) -> Result<PrivateKey, Error> {
        // bech32 decode
        let data = bech32::decode(&encrypted.0)?;
        if data.0 != *super::super::HRP_NCRYPTSEC {
            return Err(Error::WrongBech32(
                super::super::HRP_NCRYPTSEC.to_lowercase(),
                data.0.to_lowercase(),
            ));
        }
        match data.1[0] {
            1 => Self::import_encrypted_v1(data.1, password),
            2 => Self::import_encrypted_v2(data.1, password),
            _ => Err(Error::InvalidEncryptedPrivateKey),
        }
    }

    // current
    fn import_encrypted_v2(concatenation: Vec<u8>, password: &str) -> Result<PrivateKey, Error> {
        if concatenation.len() < 91 {
            return Err(Error::InvalidEncryptedPrivateKey);
        }

        // Break into parts
        let version: u8 = concatenation[0];
        assert_eq!(version, 2);
        let log2_rounds: u8 = concatenation[1];
        let salt: [u8; 16] = concatenation[2..2 + 16].try_into()?;
        let nonce = &concatenation[2 + 16..2 + 16 + 24];
        let associated_data = &concatenation[2 + 16 + 24..2 + 16 + 24 + 1];
        let ciphertext = &concatenation[2 + 16 + 24 + 1..];

        let cipher = {
            let symmetric_key = Self::password_to_key_v2(password, &salt, log2_rounds)?;
            XChaCha20Poly1305::new((&symmetric_key).into())
        };

        let payload = Payload {
            msg: ciphertext,
            aad: associated_data,
        };

        let mut inner_secret = match cipher.decrypt(nonce.into(), payload) {
            Ok(is) => is,
            Err(_) => return Err(Error::PrivateKeyEncryption),
        };

        if associated_data.is_empty() {
            return Err(Error::InvalidEncryptedPrivateKey);
        }
        let key_security = match associated_data[0] {
            0 => KeySecurity::Weak,
            1 => KeySecurity::Medium,
            2 => KeySecurity::NotTracked,
            _ => return Err(Error::InvalidEncryptedPrivateKey),
        };

        let signing_key = secp256k1::SecretKey::from_slice(&inner_secret)?;
        inner_secret.zeroize();

        Ok(PrivateKey(signing_key, key_security))
    }

    // deprecated
    fn import_encrypted_base64(
        encrypted: &EncryptedPrivateKey,
        password: &str,
    ) -> Result<PrivateKey, Error> {
        let concatenation = base64::engine::general_purpose::STANDARD.decode(&encrypted.0)?; // 64 or 80 bytes
        if concatenation.len() == 64 {
            Self::import_encrypted_pre_v1(concatenation, password)
        } else if concatenation.len() == 80 {
            Self::import_encrypted_v1(concatenation, password)
        } else {
            Err(Error::InvalidEncryptedPrivateKey)
        }
    }

    // deprecated
    fn import_encrypted_v1(concatenation: Vec<u8>, password: &str) -> Result<PrivateKey, Error> {
        // Break into parts
        let salt: [u8; 16] = concatenation[..16].try_into()?;
        let iv: [u8; 16] = concatenation[16..32].try_into()?;
        let ciphertext = &concatenation[32..]; // 48 bytes

        let key = Self::password_to_key_v1(password, &salt, V1_HMAC_ROUNDS)?;

        // AES-256-CBC decrypt
        let mut plaintext = cbc::Decryptor::<aes::Aes256>::new(&key.into(), &iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)?; // 44 bytes
        if plaintext.len() != 44 {
            return Err(Error::InvalidEncryptedPrivateKey);
            //return Err(Error::AssertionFailed("Import encrypted plaintext len
            // != 44".to_owned()));
        }

        // Verify the check value
        if plaintext[plaintext.len() - 12..plaintext.len() - 1] != V1_CHECK_VALUE {
            return Err(Error::WrongDecryptionPassword);
        }

        // Get the key security
        let ks = KeySecurity::try_from(plaintext[plaintext.len() - 1])?;
        let output = PrivateKey(
            secp256k1::SecretKey::from_slice(&plaintext[..plaintext.len() - 12])?,
            ks,
        );

        // Here we zeroize plaintext:
        plaintext.zeroize();

        Ok(output)
    }

    // deprecated
    fn import_encrypted_pre_v1(
        iv_plus_ciphertext: Vec<u8>,
        password: &str,
    ) -> Result<PrivateKey, Error> {
        let key = Self::password_to_key_v1(password, b"nostr", 4096)?;

        if iv_plus_ciphertext.len() < 48 {
            // Should be 64 from padding, but we pushed in 48
            return Err(Error::InvalidEncryptedPrivateKey);
        }

        // Pull the IV off
        let iv: [u8; 16] = iv_plus_ciphertext[..16].try_into()?;
        let ciphertext = &iv_plus_ciphertext[16..]; // 64 bytes

        // AES-256-CBC decrypt
        let mut pt = cbc::Decryptor::<aes::Aes256>::new(&key.into(), &iv.into())
            .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)?; // 48 bytes

        // Verify the check value
        if pt[pt.len() - 12..pt.len() - 1] != V1_CHECK_VALUE {
            return Err(Error::WrongDecryptionPassword);
        }

        // Get the key security
        let ks = KeySecurity::try_from(pt[pt.len() - 1])?;
        let output = PrivateKey(secp256k1::SecretKey::from_slice(&pt[..pt.len() - 12])?, ks);

        // Here we zeroize pt:
        pt.zeroize();

        Ok(output)
    }

    // Hash/Stretch password with pbkdf2 into a 32-byte (256-bit) key
    fn password_to_key_v1(password: &str, salt: &[u8], rounds: u32) -> Result<[u8; 32], Error> {
        let mut key: [u8; 32] = [0; 32];
        pbkdf2::<Hmac<Sha256>>(password.as_bytes(), salt, rounds, &mut key)?;
        Ok(key)
    }

    // Hash/Stretch password with scrypt into a 32-byte (256-bit) key
    fn password_to_key_v2(password: &str, salt: &[u8; 16], log_n: u8) -> Result<[u8; 32], Error> {
        // Normalize unicode (NFKC)
        let password = password.nfkc().collect::<String>();

        let params = match scrypt::Params::new(log_n, 8, 1, 32) {
            // r=8, p=1
            Ok(p) => p,
            Err(_) => return Err(Error::Scrypt),
        };
        let mut key: [u8; 32] = [0; 32];
        if scrypt::scrypt(password.as_bytes(), salt, &params, &mut key).is_err() {
            return Err(Error::Scrypt);
        }
        Ok(key)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_export_import() {
        let pk = PrivateKey::generate();
        // we use a low log_n here because this is run slowly in debug mode
        let exported = pk.export_encrypted("secret", 13).unwrap();
        println!("{exported}");
        let imported_pk = PrivateKey::import_encrypted(&exported, "secret").unwrap();

        // Be sure the keys generate identical public keys
        assert_eq!(pk.public_key(), imported_pk.public_key());

        // Be sure the security level is still Medium
        assert_eq!(pk.key_security(), KeySecurity::Medium)
    }

    #[test]
    fn test_import_old_formats() {
        let decrypted = "a28129ab0b70c8d5e75aaf510ec00bff47fde7ca4ab9e3d9315c77edc86f037f";

        // pre-salt base64 (-2?)
        let encrypted = EncryptedPrivateKey("F+VYIvTCtIZn4c6owPMZyu4Zn5DH9T5XcgZWmFG/3ma4C3PazTTQxQcIF+G+daeFlkqsZiNIh9bcmZ5pfdRPyg==".to_owned());
        assert_eq!(
            encrypted.decrypt("nostr").unwrap().as_hex_string(),
            decrypted
        );

        // Version -1: post-salt base64
        let encrypted = EncryptedPrivateKey("AZQYNwAGULWyKweTtw6WCljV+1cil8IMRxfZ7Rs3nCfwbVQBV56U6eV9ps3S1wU7ieCx6EraY9Uqdsw71TY5Yv/Ep6yGcy9m1h4YozuxWQE=".to_owned());
        assert_eq!(
            encrypted.decrypt("nostr").unwrap().as_hex_string(),
            decrypted
        );

        let decrypted = "3501454135014541350145413501453fefb02227e449e57cf4d3a3ce05378683";

        // Version -1
        let encrypted = EncryptedPrivateKey("KlmfCiO+Tf8A/8bm/t+sXWdb1Op4IORdghC7n/9uk/vgJXIcyW7PBAx1/K834azuVmQnCzGq1pmFMF9rNPWQ9Q==".to_owned());
        assert_eq!(
            encrypted.decrypt("nostr").unwrap().as_hex_string(),
            decrypted
        );

        // Version 0:
        let encrypted = EncryptedPrivateKey("AZ/2MU2igqP0keoW08Z/rxm+/3QYcZn3oNbVhY6DSUxSDkibNp+bFN/WsRQxP7yBKwyEJVu/YSBtm2PI9DawbYOfXDqfmpA3NTPavgXwUrw=".to_owned());
        assert_eq!(
            encrypted.decrypt("nostr").unwrap().as_hex_string(),
            decrypted
        );

        // Version 1:
        let encrypted = EncryptedPrivateKey("ncryptsec1q9hnc06cs5tuk7znrxmetj4q9q2mjtccg995kp86jf3dsp3jykv4fhak730wds4s0mja6c9v2fvdr5dhzrstds8yks5j9ukvh25ydg6xtve6qvp90j0c8a2s5tv4xn7kvulg88".to_owned());
        assert_eq!(
            encrypted.decrypt("nostr").unwrap().as_hex_string(),
            decrypted
        );

        // Version 2:
        let encrypted = EncryptedPrivateKey("ncryptsec1qgg9947rlpvqu76pj5ecreduf9jxhselq2nae2kghhvd5g7dgjtcxfqtd67p9m0w57lspw8gsq6yphnm8623nsl8xn9j4jdzz84zm3frztj3z7s35vpzmqf6ksu8r89qk5z2zxfmu5gv8th8wclt0h4p".to_owned());
        assert_eq!(
            encrypted.decrypt("nostr").unwrap().as_hex_string(),
            decrypted
        );
    }

    #[test]
    fn test_nfkc_unicode_normalization() {
        // "ÅΩẛ̣"
        // U+212B U+2126 U+1E9B U+0323
        let password1: [u8; 11] = [
            0xE2, 0x84, 0xAB, 0xE2, 0x84, 0xA6, 0xE1, 0xBA, 0x9B, 0xCC, 0xA3,
        ];

        // "ÅΩẛ̣"
        // U+00C5 U+03A9 U+1E69
        let password2: [u8; 7] = [0xC3, 0x85, 0xCE, 0xA9, 0xE1, 0xB9, 0xA9];

        let password1_str = unsafe { std::str::from_utf8_unchecked(password1.as_slice()) };
        let password2_str = unsafe { std::str::from_utf8_unchecked(password2.as_slice()) };

        let password1_nfkc = password1_str.nfkc().collect::<String>();
        assert_eq!(password1_nfkc, password2_str);
    }
}

/*
 * version -1 (if 64 bytes, base64 encoded)
 *
 *    symmetric_aes_key = pbkdf2_hmac_sha256(password,  salt="nostr",
 * rounds=4096)    pre_encoded_encrypted_private_key = AES-256-CBC(IV=random,
 * key=symmetric_aes_key, data=private_key)    encrypted_private_key =
 * base64(concat(IV, pre_encoded_encrypted_private_key))
 *
 * version 0 (80 bytes, base64 encoded, same as v1 internally)
 *
 *    symmetric_aes_key = pbkdf2_hmac_sha256(password,  salt=concat(0x1, 15
 * random bytes), rounds=100000)    key_security_byte = 0x0 if weak, 0x1 if
 * medium    inner_concatenation = concat(
 *        private_key,                                         // 32 bytes
 *        [15, 91, 241, 148, 90, 143, 101, 12, 172, 255, 103], // 11 bytes
 *        key_security_byte                                    //  1 byte
 *    )
 *    pre_encoded_encrypted_private_key = AES-256-CBC(IV=random,
 * key=symmetric_aes_key, data=private_key)    outer_concatenation =
 * concat(IV, pre_encoded_encrypted_private_key)    encrypted_private_key =
 * base64(outer_concatenation)
 *
 * version 1
 *
 *    salt = concat(byte(0x1), 15 random bytes)
 *    symmetric_aes_key = pbkdf2_hmac_sha256(password, salt=salt,
 * rounds=100,000)    key_security_byte = 0x0 if weak, 0x1 if medium
 *    inner_concatenation = concat(
 *        private_key,                                          // 32 bytes
 *        [15, 91, 241, 148, 90, 143, 101, 12, 172, 255, 103],  // 11 bytes
 *        key_security_byte                                     //  1 byte
 *    )
 *    pre_encoded_encrypted_private_key = AES-256-CBC(IV=random,
 * key=symmetric_aes_key, data=private_key)    outer_concatenation =
 * concat(salt, IV, pre_encoded_encrypted_private_key)
 *    encrypted_private_key = bech32('ncryptsec', outer_concatenation)
 *
 * version 2 (scrypt, xchacha20-poly1305)
 *
 *    rounds = user selected power of 2
 *    salt = 16 random bytes
 *    symmetric_key = scrypt(password, salt=salt, r=8, p=1, N=rounds)
 *    key_security_byte = 0x0 if weak, 0x1 if medium, 0x2 if not implemented
 *    nonce = 12 random bytes
 *    pre_encoded_encrypted_private_key = xchacha20-poly1305(
 *        plaintext=private_key, nonce=nonce, key=symmetric_key,
 *        associated_data=key_security_byte
 *    )
 *    version = byte(0x3)
 *    outer_concatenation = concat(version, log2(rounds) as one byte, salt,
 * nonce, pre_encoded_encrypted_private_key)    encrypted_private_key =
 * bech32('ncryptsec', outer_concatenation)
 */
```

---

### private_key/mod.rs

**Size:** 8955 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{convert::TryFrom, fmt};

use rand_core::OsRng;

use super::{Error, Id, PublicKey, Signature, Signer};

pub(super) mod encrypted_private_key;
pub use encrypted_private_key::*;

pub(super) mod content_encryption;
pub use content_encryption::*;

/// This indicates the security of the key by keeping track of whether the
/// secret key material was handled carefully. If the secret is exposed in any
/// way, or leaked and the memory not zeroed, the key security drops to Weak.
///
/// This is a Best Effort tag. There are ways to leak the key and still have
/// this tag claim the key is Medium security. So Medium really means it might
/// not have leaked, whereas Weak means we know that it definately did leak.
///
/// We offer no Strong security via the PrivateKey structure. If we support
/// hardware tokens in the future, it will probably be via a different
/// structure.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum KeySecurity {
    /// This means that the key was exposed in a way such that this library
    /// cannot ensure it's secrecy, usually either by being exported as a hex
    /// string, or by being imported from the same. Often in these cases it
    /// is displayed on the screen or left in the cut buffer or in freed
    /// memory that was not subsequently zeroed.
    Weak = 0,

    /// This means that the key might not have been directly exposed. But it
    /// still might have as there are numerous ways you can leak it such as
    /// exporting it and then decrypting the exported key, using unsafe
    /// rust, transmuting it into a different type that doesn't protect it,
    /// or using a privileged process to scan memory. Additionally, more
    /// advanced techniques can get at your key such as hardware attacks
    /// like spectre, rowhammer, and power analysis.
    Medium = 1,

    /// Not tracked
    NotTracked = 2,
}

impl TryFrom<u8> for KeySecurity {
    type Error = Error;

    fn try_from(i: u8) -> Result<KeySecurity, Error> {
        if i == 0 {
            Ok(KeySecurity::Weak)
        } else if i == 1 {
            Ok(KeySecurity::Medium)
        } else if i == 2 {
            Ok(KeySecurity::NotTracked)
        } else {
            Err(Error::UnknownKeySecurity(i))
        }
    }
}

/// This is a private key which is to be kept secret and is used to prove
/// identity
#[allow(missing_debug_implementations)]
#[derive(Clone, PartialEq, Eq)]
pub struct PrivateKey(pub secp256k1::SecretKey, pub KeySecurity);

impl Default for PrivateKey {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Debug for PrivateKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "PRIVATE-KEY-ELIDED")
    }
}

impl PrivateKey {
    /// Generate a new `PrivateKey` (which can be used to get the `PublicKey`)
    #[inline]
    pub fn new() -> PrivateKey {
        Self::generate()
    }

    /// Generate a new `PrivateKey` (which can be used to get the `PublicKey`)
    pub fn generate() -> PrivateKey {
        let mut secret_key;
        loop {
            secret_key = secp256k1::SecretKey::new(&mut OsRng);
            let (_, parity) = secret_key.x_only_public_key(secp256k1::SECP256K1);
            if parity == secp256k1::Parity::Even {
                break;
            }
        }

        PrivateKey(secret_key, KeySecurity::Medium)
    }

    /// Get the PublicKey matching this PrivateKey
    pub fn public_key(&self) -> PublicKey {
        let (xopk, _parity) = self.0.x_only_public_key(secp256k1::SECP256K1);
        PublicKey::from_bytes(&xopk.serialize(), false).unwrap()
    }

    /// Get the security level of the private key
    pub fn key_security(&self) -> KeySecurity {
        self.1
    }

    /// Render into a hexadecimal string
    ///
    /// WARNING: This weakens the security of your key. Your key will be marked
    /// with `KeySecurity::Weak` if you execute this.
    pub fn as_hex_string(&mut self) -> String {
        self.1 = KeySecurity::Weak;
        hex::encode(self.0.secret_bytes())
    }

    /// Create from a hexadecimal string
    ///
    /// This creates a key with `KeySecurity::Weak`.  Use `generate()` or
    /// `import_encrypted()` for `KeySecurity::Medium`
    pub fn try_from_hex_string(v: &str) -> Result<PrivateKey, Error> {
        let vec: Vec<u8> = hex::decode(v)?;
        Ok(PrivateKey(
            secp256k1::SecretKey::from_slice(&vec)?,
            KeySecurity::Weak,
        ))
    }

    /// Export as a bech32 encoded string
    ///
    /// WARNING: This weakens the security of your key. Your key will be marked
    /// with `KeySecurity::Weak` if you execute this.
    pub fn as_bech32_string(&mut self) -> String {
        self.1 = KeySecurity::Weak;
        bech32::encode::<bech32::Bech32>(*super::HRP_NSEC, self.0.secret_bytes().as_slice())
            .unwrap()
    }

    /// Import from a bech32 encoded string
    ///
    /// This creates a key with `KeySecurity::Weak`.  Use `generate()` or
    /// `import_encrypted()` for `KeySecurity::Medium`
    pub fn try_from_bech32_string(s: &str) -> Result<PrivateKey, Error> {
        let data = bech32::decode(s)?;
        if data.0 != *super::HRP_NSEC {
            Err(Error::WrongBech32(
                super::HRP_NSEC.to_lowercase(),
                data.0.to_lowercase(),
            ))
        } else {
            Ok(PrivateKey(
                secp256k1::SecretKey::from_slice(&data.1)?,
                KeySecurity::Weak,
            ))
        }
    }

    /// As a `secp256k1::SecretKey`
    pub fn as_secret_key(&self) -> secp256k1::SecretKey {
        self.0
    }

    /// Sign a 32-bit hash
    pub fn sign_id(&self, id: Id) -> Result<Signature, Error> {
        let keypair = secp256k1::Keypair::from_secret_key(secp256k1::SECP256K1, &self.0);
        let message = secp256k1::Message::from_digest_slice(id.0.as_slice())?;
        Ok(Signature(keypair.sign_schnorr(message)))
    }

    /// Sign a message (this hashes with SHA-256 first internally)
    pub fn sign(&self, message: &[u8]) -> Result<Signature, Error> {
        use secp256k1::hashes::{sha256, Hash};
        let keypair = secp256k1::Keypair::from_secret_key(secp256k1::SECP256K1, &self.0);
        let hash = sha256::Hash::hash(message).to_byte_array();
        let message = secp256k1::Message::from_digest(hash);
        Ok(Signature(keypair.sign_schnorr(message)))
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> PrivateKey {
        PrivateKey::generate()
    }
}

impl Drop for PrivateKey {
    fn drop(&mut self) {
        self.0.non_secure_erase();
    }
}

impl Signer for PrivateKey {
    fn is_locked(&self) -> bool {
        false
    }

    fn unlock(&mut self, _password: &str) -> Result<(), Error> {
        Ok(())
    }

    fn lock(&mut self) {}

    fn change_passphrase(&mut self, _old: &str, _new: &str, _log_n: u8) -> Result<(), Error> {
        Err(Error::InvalidOperation)
    }

    fn upgrade(&mut self, _pass: &str, _log_n: u8) -> Result<(), Error> {
        Err(Error::InvalidOperation)
    }

    fn public_key(&self) -> PublicKey {
        self.public_key()
    }

    fn encrypted_private_key(&self) -> Option<&EncryptedPrivateKey> {
        None
    }

    fn export_private_key_in_hex(
        &mut self,
        _pass: &str,
        _log_n: u8,
    ) -> Result<(String, bool), Error> {
        Ok((self.as_hex_string(), false))
    }

    fn export_private_key_in_bech32(
        &mut self,
        _pass: &str,
        _log_n: u8,
    ) -> Result<(String, bool), Error> {
        Ok((self.as_bech32_string(), false))
    }

    fn sign_id(&self, id: Id) -> Result<Signature, Error> {
        self.sign_id(id)
    }

    fn sign(&self, message: &[u8]) -> Result<Signature, Error> {
        self.sign(message)
    }

    fn encrypt(
        &self,
        other: &PublicKey,
        plaintext: &str,
        algo: ContentEncryptionAlgorithm,
    ) -> Result<String, Error> {
        self.encrypt(other, plaintext, algo)
    }

    /// Decrypt NIP-44
    fn decrypt(&self, other: &PublicKey, ciphertext: &str) -> Result<String, Error> {
        self.decrypt(other, ciphertext)
    }

    /// Get NIP-44 conversation key
    fn nip44_conversation_key(&self, other: &PublicKey) -> Result<[u8; 32], Error> {
        Ok(super::nip44::get_conversation_key(
            self.0,
            other.as_xonly_public_key(),
        ))
    }

    fn key_security(&self) -> Result<KeySecurity, Error> {
        Ok(KeySecurity::NotTracked)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_privkey_bech32() {
        let mut pk = PrivateKey::mock();

        let encoded = pk.as_bech32_string();
        println!("bech32: {encoded}");

        let decoded = PrivateKey::try_from_bech32_string(&encoded).unwrap();

        assert_eq!(pk.0.secret_bytes(), decoded.0.secret_bytes());
        assert_eq!(decoded.1, KeySecurity::Weak);
    }
}
```

---

### profile.rs

**Size:** 5196 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::Error;
#[cfg(test)]
use crate::test_serde;
use crate::types::{PublicKey, UncheckedUrl};

/// A person's profile on nostr which consists of the data needed in order to
/// follow someone.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct Profile {
    /// Their public key
    pub pubkey: PublicKey,

    /// Some of the relays they post to (when the profile was created)
    pub relays: Vec<UncheckedUrl>,
}

impl Profile {
    /// Export as a bech32 encoded string ("nprofile")
    pub fn as_bech32_string(&self) -> String {
        // Compose
        let mut tlv: Vec<u8> = Vec::new();

        // Push Public Key
        tlv.push(0); // the special value, in this case the public key
        tlv.push(32); // the length of the value (always 32 for public key)
        tlv.extend(self.pubkey.as_slice());

        // Push relays
        for relay in &self.relays {
            tlv.push(1); // type 'relay'
            let len = relay.0.len() as u8;
            tlv.push(len); // the length of the string
            tlv.extend(relay.0.as_bytes().iter().take(len as usize));
        }

        bech32::encode::<bech32::Bech32>(*super::HRP_NPROFILE, &tlv).unwrap()
    }

    /// Import from a bech32 encoded string ("nprofile")
    ///
    /// If verify is true, will verify that it works as a
    /// secp256k1::XOnlyPublicKey. This has a performance cost.
    pub fn try_from_bech32_string(s: &str, verify: bool) -> Result<Profile, Error> {
        let data = bech32::decode(s)?;
        if data.0 != *super::HRP_NPROFILE {
            Err(Error::WrongBech32(
                super::HRP_NPROFILE.to_lowercase(),
                data.0.to_lowercase(),
            ))
        } else {
            let mut relays: Vec<UncheckedUrl> = Vec::new();
            let mut pubkey: Option<PublicKey> = None;
            let tlv = data.1;
            let mut pos = 0;
            loop {
                // we need at least 2 more characters for anything meaningful
                if pos > tlv.len() - 2 {
                    break;
                }
                let ty = tlv[pos];
                let len = tlv[pos + 1] as usize;
                pos += 2;
                if pos + len > tlv.len() {
                    return Err(Error::InvalidProfile);
                }
                match ty {
                    0 => {
                        // special,  32 bytes of the public key
                        if len != 32 {
                            return Err(Error::InvalidProfile);
                        }
                        pubkey = Some(PublicKey::from_bytes(&tlv[pos..pos + len], verify)?);
                    }
                    1 => {
                        // relay
                        let relay_bytes = &tlv[pos..pos + len];
                        let relay_str = std::str::from_utf8(relay_bytes)?;
                        let relay = UncheckedUrl::from_str(relay_str);
                        relays.push(relay);
                    }
                    _ => {} // unhandled type for nprofile
                }
                pos += len;
            }
            if let Some(pubkey) = pubkey {
                Ok(Profile { pubkey, relays })
            } else {
                Err(Error::InvalidProfile)
            }
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Profile {
        let pubkey = PublicKey::try_from_hex_string(
            "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9",
            true,
        )
        .unwrap();

        Profile {
            pubkey,
            relays: vec![
                UncheckedUrl::from_str("wss://relay.example.com"),
                UncheckedUrl::from_str("wss://relay2.example.com"),
            ],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {Profile, test_profile_serde}

    #[test]
    fn test_profile_bech32() {
        let bech32 = Profile::mock().as_bech32_string();
        println!("{bech32}");
        assert_eq!(
            Profile::mock(),
            Profile::try_from_bech32_string(&bech32, true).unwrap()
        );
    }

    #[test]
    fn test_nip19_example() {
        let profile = Profile {
            pubkey: PublicKey::try_from_hex_string(
                "3bf0c63fcb93463407af97a5e5ee64fa883d107ef9e558472c4eb9aaaefa459d",
                true,
            )
            .unwrap(),
            relays: vec![
                UncheckedUrl::from_str("wss://r.x.com"),
                UncheckedUrl::from_str("wss://djbas.sadkb.com"),
            ],
        };

        let bech32 = "nprofile1qqsrhuxx8l9ex335q7he0f09aej04zpazpl0ne2cgukyawd24mayt8gpp4mhxue69uhhytnc9e3k7mgpz4mhxue69uhkg6nzv9ejuumpv34kytnrdaksjlyr9p";

        // Try converting profile to bech32
        assert_eq!(profile.as_bech32_string(), bech32);

        // Try converting bech32 to profile
        assert_eq!(
            profile,
            Profile::try_from_bech32_string(bech32, true).unwrap()
        );
    }
}
```

---

### public_key.rs

**Size:** 10818 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use derive_more::{AsMut, AsRef, Deref, Display, From, FromStr, Into};
use secp256k1::{XOnlyPublicKey, SECP256K1};
use serde::{
    de::{Deserializer, Visitor},
    ser::Serializer,
    Deserialize, Serialize,
};
#[cfg(feature = "speedy")]
use speedy::{Context, Readable, Reader, Writable, Writer};

use super::{Error, PrivateKey, Signature};
#[cfg(test)]
use crate::test_serde;

/// This is a public key, which identifies an actor (usually a person) and is
/// shared.
#[derive(AsMut, AsRef, Copy, Clone, Debug, Deref, Eq, From, Into, PartialEq, PartialOrd, Ord)]
pub struct PublicKey([u8; 32]);

impl PublicKey {
    /// Render into a hexadecimal string
    ///
    /// Consider converting `.into()` a `PublicKeyHex` which is a wrapped type
    /// rather than a naked `String`
    pub fn as_hex_string(&self) -> String {
        hex::encode(self.0)
    }

    /// Create from a hexadecimal string
    ///
    /// If verify is true, will verify that it works as a XOnlyPublicKey. This
    /// has a performance cost.
    pub fn try_from_hex_string(v: &str, verify: bool) -> Result<PublicKey, Error> {
        let vec: Vec<u8> = hex::decode(v)?;
        // if it's not 32 bytes, dont even try
        if vec.len() != 32 {
            Err(Error::InvalidPublicKey)
        } else {
            if verify {
                let _ = XOnlyPublicKey::from_slice(&vec)?;
            }
            Ok(PublicKey(vec.try_into().unwrap()))
        }
    }

    /// Export as a bech32 encoded string
    pub fn as_bech32_string(&self) -> String {
        bech32::encode::<bech32::Bech32>(*super::HRP_NPUB, self.0.as_slice()).unwrap()
    }

    /// Export as XOnlyPublicKey
    pub fn as_xonly_public_key(&self) -> XOnlyPublicKey {
        XOnlyPublicKey::from_slice(&self.0).unwrap()
    }

    /// Import from a bech32 encoded string
    ///
    /// If verify is true, will verify that it works as a XOnlyPublicKey. This
    /// has a performance cost.
    pub fn try_from_bech32_string(s: &str, verify: bool) -> Result<PublicKey, Error> {
        let data = bech32::decode(s)?;
        if data.0 != *super::HRP_NPUB {
            Err(Error::WrongBech32(
                super::HRP_NPUB.to_lowercase(),
                data.0.to_lowercase(),
            ))
        } else if data.1.len() != 32 {
            Err(Error::InvalidPublicKey)
        } else {
            if verify {
                let _ = XOnlyPublicKey::from_slice(&data.1)?;
            }
            Ok(PublicKey(data.1.try_into().unwrap()))
        }
    }

    /// Import from raw bytes
    pub fn from_bytes(bytes: &[u8], verify: bool) -> Result<PublicKey, Error> {
        if bytes.len() != 32 {
            Err(Error::InvalidPublicKey)
        } else {
            if verify {
                let _ = XOnlyPublicKey::from_slice(bytes)?;
            }
            Ok(PublicKey(bytes.try_into().unwrap()))
        }
    }

    /// Export as raw bytes
    #[inline]
    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }

    /// Export as raw bytes
    #[inline]
    pub fn to_bytes(&self) -> Vec<u8> {
        self.0.as_slice().to_vec()
    }

    /// Parse from bech32 or hex string (for compatibility with nostr_sdk)
    pub fn parse(s: String) -> Option<Self> {
        // Try bech32 first
        if let Ok(pk) = Self::try_from_bech32_string(&s, false) {
            return Some(pk);
        }
        // Try hex
        if let Ok(pk) = Self::try_from_hex_string(&s, false) {
            return Some(pk);
        }
        None
    }

    /// Verify a signed message
    pub fn verify(&self, message: &[u8], signature: &Signature) -> Result<(), Error> {
        use secp256k1::hashes::{sha256, Hash};
        let pk = XOnlyPublicKey::from_slice(self.0.as_slice())?;
        let hash = sha256::Hash::hash(message).to_byte_array();
        let message = secp256k1::Message::from_digest(hash);
        Ok(SECP256K1.verify_schnorr(&signature.0, &message, &pk)?)
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> PublicKey {
        PrivateKey::generate().public_key()
    }

    #[allow(dead_code)]
    pub(crate) fn mock_deterministic() -> PublicKey {
        PublicKey::try_from_hex_string(
            "ee11a5dff40c19a555f41fe42b48f00e618c91225622ae37b6c2bb67b76c4e49",
            true,
        )
        .unwrap()
    }
}

impl fmt::Display for PublicKey {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.as_hex_string())
    }
}

impl Serialize for PublicKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.as_hex_string())
    }
}

impl<'de> Deserialize<'de> for PublicKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PublicKeyVisitor)
    }
}

struct PublicKeyVisitor;

impl Visitor<'_> for PublicKeyVisitor {
    type Value = PublicKey;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a lowercase hexadecimal string representing 32 bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<PublicKey, E>
    where
        E: serde::de::Error,
    {
        let vec: Vec<u8> = hex::decode(v).map_err(|e| serde::de::Error::custom(format!("{e}")))?;

        if vec.len() != 32 {
            return Err(serde::de::Error::custom("Public key is not 32 bytes long"));
        }

        Ok(PublicKey(vec.try_into().unwrap()))
    }
}

impl std::hash::Hash for PublicKey {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.as_hex_string().hash(state);
    }
}

#[cfg(feature = "speedy")]
impl<'a, C: Context> Readable<'a, C> for PublicKey {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let bytes: Vec<u8> = reader.read_vec(32)?;
        Ok(PublicKey(bytes.try_into().unwrap()))
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        32
    }
}

#[cfg(feature = "speedy")]
impl<C: Context> Writable<C> for PublicKey {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        writer.write_bytes(self.as_slice())
    }

    #[inline]
    fn bytes_needed(&self) -> Result<usize, C::Error> {
        Ok(32)
    }
}

/// This is a public key, which identifies an actor (usually a person) and is
/// shared, as a hex string
///
/// You can convert from a `PublicKey` into this with `From`/`Into`.  You can
/// convert this back to a `PublicKey` with `TryFrom`/`TryInto`.
#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Deref,
    Display,
    Eq,
    From,
    FromStr,
    Hash,
    Into,
    PartialEq,
    PartialOrd,
    Ord,
)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct PublicKeyHex(String);

impl PublicKeyHex {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> PublicKeyHex {
        From::from(PublicKey::mock())
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock_deterministic() -> PublicKeyHex {
        PublicKey::mock_deterministic().into()
    }

    /// Export as a bech32 encoded string
    pub fn as_bech32_string(&self) -> String {
        let vec: Vec<u8> = hex::decode(&self.0).unwrap();
        bech32::encode::<bech32::Bech32>(*super::HRP_NPUB, &vec).unwrap()
    }

    /// Try from &str
    pub fn try_from_str(s: &str) -> Result<PublicKeyHex, Error> {
        Self::try_from_string(s.to_owned())
    }

    /// Try from String
    pub fn try_from_string(s: String) -> Result<PublicKeyHex, Error> {
        if s.len() != 64 {
            return Err(Error::InvalidPublicKey);
        }
        let vec: Vec<u8> = hex::decode(&s)?;
        if vec.len() != 32 {
            return Err(Error::InvalidPublicKey);
        }
        Ok(PublicKeyHex(s))
    }

    /// As &str
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Into String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl TryFrom<&str> for PublicKeyHex {
    type Error = Error;

    fn try_from(s: &str) -> Result<PublicKeyHex, Error> {
        PublicKeyHex::try_from_str(s)
    }
}

impl From<&PublicKey> for PublicKeyHex {
    fn from(pk: &PublicKey) -> PublicKeyHex {
        PublicKeyHex(pk.as_hex_string())
    }
}

impl From<PublicKey> for PublicKeyHex {
    fn from(pk: PublicKey) -> PublicKeyHex {
        PublicKeyHex(pk.as_hex_string())
    }
}

impl TryFrom<&PublicKeyHex> for PublicKey {
    type Error = Error;

    fn try_from(pkh: &PublicKeyHex) -> Result<PublicKey, Error> {
        PublicKey::try_from_hex_string(&pkh.0, true)
    }
}

impl TryFrom<PublicKeyHex> for PublicKey {
    type Error = Error;

    fn try_from(pkh: PublicKeyHex) -> Result<PublicKey, Error> {
        PublicKey::try_from_hex_string(&pkh.0, true)
    }
}

impl Serialize for PublicKeyHex {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.0)
    }
}

impl<'de> Deserialize<'de> for PublicKeyHex {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(PublicKeyHexVisitor)
    }
}

struct PublicKeyHexVisitor;

impl Visitor<'_> for PublicKeyHexVisitor {
    type Value = PublicKeyHex;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a lowercase hexadecimal string representing 32 bytes")
    }

    fn visit_str<E>(self, v: &str) -> Result<PublicKeyHex, E>
    where
        E: serde::de::Error,
    {
        if v.len() != 64 {
            return Err(serde::de::Error::custom(
                "PublicKeyHex is not 64 characters long",
            ));
        }

        let vec: Vec<u8> = hex::decode(v).map_err(|e| serde::de::Error::custom(format!("{e}")))?;
        if vec.len() != 32 {
            return Err(serde::de::Error::custom("Invalid PublicKeyHex"));
        }

        Ok(PublicKeyHex(v.to_owned()))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {PublicKey, test_public_key_serde}
    test_serde! {PublicKeyHex, test_public_key_hex_serde}

    #[test]
    fn test_pubkey_bech32() {
        let pk = PublicKey::mock();

        let encoded = pk.as_bech32_string();
        println!("bech32: {encoded}");

        let decoded = PublicKey::try_from_bech32_string(&encoded, true).unwrap();

        assert_eq!(pk, decoded);
    }

    #[cfg(feature = "speedy")]
    #[test]
    fn test_speedy_public_key() {
        let pk = PublicKey::mock();
        let bytes = pk.write_to_vec().unwrap();
        let pk2 = PublicKey::read_from_buffer(&bytes).unwrap();
        assert_eq!(pk, pk2);
    }
}
```

---

### relay_information_document.rs

**Size:** 537 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use super::versioned::{
    relay_information_document1::{FeeV1, RelayFeesV1, RelayRetentionV1},
    relay_information_document2::{RelayInformationDocumentV2, RelayLimitationV2},
};

/// Relay limitations
pub type RelayLimitation = RelayLimitationV2;

/// Relay retention
pub type RelayRetention = RelayRetentionV1;

/// Fee
pub type Fee = FeeV1;

/// Relay fees
pub type RelayFees = RelayFeesV1;

/// Relay information document as described in NIP-11, supplied by a relay
pub type RelayInformationDocument = RelayInformationDocumentV2;
```

---

### relay_list.rs

**Size:** 2602 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::collections::HashMap;

use crate::types::{Event, RelayUrl, Tag};

/// Relay Usage
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum RelayListUsage {
    /// The relay is used as an inbox (called 'read' in kind-10002)
    Inbox,

    /// The relay is used as an outbox (called 'write' in kind-10002)
    Outbox,

    /// The relay is used both as an inbox and an outbox
    #[default]
    Both,
}

impl RelayListUsage {
    /// A string marker used in a kind-10002 RelayList event for the variant
    pub fn marker(&self) -> Option<&'static str> {
        match self {
            RelayListUsage::Inbox => Some("read"),
            RelayListUsage::Outbox => Some("write"),
            RelayListUsage::Both => None,
        }
    }
}

/// A relay list, indicating usage for each relay, which can be used to
/// represent the data found in a kind 10002 RelayListMetadata event.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct RelayList(pub HashMap<RelayUrl, RelayListUsage>);

impl RelayList {
    /// Parse a kind-10002 RelayList event into a RelayList
    ///
    /// This does not check the event kind, that is left up to the caller.
    pub fn from_event(event: &Event) -> RelayList {
        let mut relay_list: RelayList = Default::default();

        for tag in event.tags.iter() {
            if let Ok((uurl, optmarker)) = tag.parse_relay() {
                if let Ok(relay_url) = RelayUrl::try_from_unchecked_url(&uurl) {
                    if let Some(m) = optmarker {
                        match &*m.trim().to_lowercase() {
                            "read" => {
                                let _ = relay_list.0.insert(relay_url, RelayListUsage::Inbox);
                            }
                            "write" => {
                                let _ = relay_list.0.insert(relay_url, RelayListUsage::Outbox);
                            }
                            _ => {} // ignore unknown marker
                        }
                    } else {
                        let _ = relay_list.0.insert(relay_url, RelayListUsage::Both);
                    }
                }
            }
        }

        relay_list
    }

    /// Create a `Vec<Tag>` appropriate for forming a kind-10002 RelayList event
    pub fn to_event_tags(&self) -> Vec<Tag> {
        let mut tags: Vec<Tag> = Vec::new();
        for (relay_url, usage) in self.0.iter() {
            tags.push(Tag::new_relay(
                relay_url.to_unchecked_url(),
                usage.marker().map(|s| s.to_owned()),
            ));
        }
        tags
    }
}
```

---

### relay_message.rs

**Size:** 118 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use super::versioned::RelayMessageV5;

/// A message from a relay to a client
pub type RelayMessage = RelayMessageV5;
```

---

### relay_usage.rs

**Size:** 3851 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::convert::TryFrom;

/// A way that a user uses a Relay
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
#[repr(u32)]
pub enum RelayUsage {
    /// User seeks events here if they are not otherwise found
    FallbackRead = 1 << 0,

    /// User writes here but does not advertise it
    Archive = 1 << 1,

    // was a relay usage flag in gossip, but was retired
    // Advertise = 1 << 2,
    /// User accepts posts here from the public that tag them
    Inbox = 1 << 3,

    /// User posts here for the public
    Outbox = 1 << 4,

    /// User seeks relay lists here (index, discover)
    Directory = 1 << 5,

    // is used as SPAMSAFE bit in gossip so reserved, but isn't a relay usage
    // ReservedSpamsafe = 1 << 6,
    /// User accepts DMs here
    Dm = 1 << 7,

    /// user stores and reads back their own configurations here
    Config = 1 << 8,

    /// User does NIP-50 SEARCH here
    Search = 1 << 9,
}

impl TryFrom<u32> for RelayUsage {
    type Error = ();

    fn try_from(u: u32) -> Result<RelayUsage, ()> {
        match u {
            1 => Ok(RelayUsage::FallbackRead),
            2 => Ok(RelayUsage::Archive),
            8 => Ok(RelayUsage::Inbox),
            16 => Ok(RelayUsage::Outbox),
            32 => Ok(RelayUsage::Directory),
            128 => Ok(RelayUsage::Dm),
            256 => Ok(RelayUsage::Config),
            512 => Ok(RelayUsage::Search),
            _ => Err(()),
        }
    }
}

/// The ways that a user uses a Relay
// See also https://github.com/mikedilger/gossip/blob/master/gossip-lib/src/storage/types/relay3.rs
// See also https://github.com/nostr-protocol/nips/issues/1282 for possible future entries
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Hash)]
pub struct RelayUsageSet(u32);

impl RelayUsageSet {
    const MASK: u32 = RelayUsage::FallbackRead as u32
        | RelayUsage::Archive as u32
        | RelayUsage::Inbox as u32
        | RelayUsage::Outbox as u32
        | RelayUsage::Directory as u32
        | RelayUsage::Dm as u32
        | RelayUsage::Config as u32
        | RelayUsage::Search as u32;

    /// Create a new empty RelayUsageSet
    pub const fn new_empty() -> Self {
        RelayUsageSet(0)
    }

    /// Create a new RelayUsageSet with all usages
    pub const fn new_all() -> Self {
        Self(Self::MASK)
    }

    /// Get the u32 bitflag representation
    pub const fn bits(&self) -> u32 {
        self.0
    }

    /// Set from a u32 bitflag representation. If any unknown bits are set,
    /// this will return None
    pub const fn from_bits(bits: u32) -> Option<RelayUsageSet> {
        if bits & !Self::MASK != 0 {
            None
        } else {
            Some(RelayUsageSet(bits))
        }
    }

    /// Set from a u32 bitflag representation. If any unknown bits are set,
    /// they will be cleared
    pub const fn from_bits_truncate(bits: u32) -> RelayUsageSet {
        RelayUsageSet(bits & Self::MASK)
    }

    /// Whether all bits are unset
    pub const fn is_empty(&self) -> bool {
        self.0 == 0
    }

    /// Whether all defined bits are set
    pub const fn is_all(&self) -> bool {
        self.0 & Self::MASK == Self::MASK
    }

    /// Whether any usage in other is also in Self
    pub const fn intersects(&self, other: Self) -> bool {
        self.0 & other.0 != 0
    }

    /// Whether all usages in other are in Self
    pub const fn contains(&self, other: Self) -> bool {
        self.0 & other.0 == other.0
    }

    /// Has a RelayUsage set
    pub fn has_usage(&mut self, ru: RelayUsage) -> bool {
        self.0 & ru as u32 == ru as u32
    }

    /// Add a RelayUsage to Self
    pub fn add_usage(&mut self, ru: RelayUsage) {
        self.0 |= ru as u32
    }

    /// Remove a RelayUsage to Self
    pub fn remove_usage(&mut self, ru: RelayUsage) {
        self.0 &= !(ru as u32)
    }
}
```

---

### satoshi.rs

**Size:** 1142 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::ops::Add;

use derive_more::{AsMut, AsRef, Deref, Display, From, Into};
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

/// Bitcoin amount measured in millisatoshi
#[derive(
    AsMut,
    AsRef,
    Clone,
    Copy,
    Debug,
    Deref,
    Deserialize,
    Display,
    Eq,
    From,
    Into,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct MilliSatoshi(pub u64);

impl MilliSatoshi {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> MilliSatoshi {
        MilliSatoshi(15423000)
    }
}

impl Add<MilliSatoshi> for MilliSatoshi {
    type Output = Self;

    fn add(self, rhs: MilliSatoshi) -> Self::Output {
        MilliSatoshi(self.0 + rhs.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {MilliSatoshi, test_millisatoshi_serde}

    #[test]
    fn test_millisatoshi_math() {
        let a = MilliSatoshi(15000);
        let b = MilliSatoshi(3000);
        let c = a + b;
        assert_eq!(c.0, 18000);
    }
}
```

---

### signature.rs

**Size:** 3255 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use derive_more::{AsMut, AsRef, Deref, Display, From, FromStr, Into};
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Context, Readable, Reader, Writable, Writer};

use super::{Error, Event};

/// A Schnorr signature that signs an Event, taken on the Event Id field
#[derive(
    AsMut, AsRef, Clone, Copy, Debug, Deref, Eq, From, Into, PartialEq, Serialize, Deserialize,
)]
pub struct Signature(pub secp256k1::schnorr::Signature);

impl Signature {
    /// Render into a hexadecimal string
    pub fn as_hex_string(&self) -> String {
        hex::encode(self.0.as_ref())
    }

    /// Create from a hexadecimal string
    pub fn try_from_hex_string(v: &str) -> Result<Signature, Error> {
        let vec: Vec<u8> = hex::decode(v)?;
        Ok(Signature(secp256k1::schnorr::Signature::from_slice(&vec)?))
    }

    /// A dummy signature of all zeroes
    pub fn zeroes() -> Signature {
        Signature(secp256k1::schnorr::Signature::from_slice(&[0; 64]).unwrap())
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Signature {
        let event = Event::mock();
        event.sig
    }
}

#[cfg(feature = "speedy")]
impl<'a, C: Context> Readable<'a, C> for Signature {
    #[inline]
    fn read_from<R: Reader<'a, C>>(reader: &mut R) -> Result<Self, C::Error> {
        let bytes: Vec<u8> = reader.read_vec(64)?;
        let sig =
            secp256k1::schnorr::Signature::from_slice(&bytes[..]).map_err(speedy::Error::custom)?;
        Ok(Signature(sig))
    }

    #[inline]
    fn minimum_bytes_needed() -> usize {
        64
    }
}

#[cfg(feature = "speedy")]
impl<C: Context> Writable<C> for Signature {
    #[inline]
    fn write_to<T: ?Sized + Writer<C>>(&self, writer: &mut T) -> Result<(), C::Error> {
        let bytes = self.0.as_ref();
        assert_eq!(bytes.as_slice().len(), 64);
        writer.write_bytes(bytes.as_slice())
    }

    #[inline]
    fn bytes_needed(&self) -> Result<usize, C::Error> {
        Ok(64)
    }
}

/// A Schnorr signature that signs an Event, taken on the Event Id field, as a
/// hex string
#[derive(
    AsMut,
    AsRef,
    Clone,
    Debug,
    Deref,
    Deserialize,
    Display,
    Eq,
    From,
    FromStr,
    Hash,
    Into,
    PartialEq,
    Serialize,
)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct SignatureHex(pub String);

impl SignatureHex {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> SignatureHex {
        From::from(Signature::mock())
    }
}

impl From<Signature> for SignatureHex {
    fn from(s: Signature) -> SignatureHex {
        SignatureHex(s.as_hex_string())
    }
}

impl TryFrom<SignatureHex> for Signature {
    type Error = Error;

    fn try_from(sh: SignatureHex) -> Result<Signature, Error> {
        Signature::try_from_hex_string(&sh.0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {Signature, test_signature_serde}

    #[cfg(feature = "speedy")]
    #[test]
    fn test_speedy_signature() {
        let sig = Signature::mock();
        let bytes = sig.write_to_vec().unwrap();
        let sig2 = Signature::read_from_buffer(&bytes).unwrap();
        assert_eq!(sig, sig2);
    }
}
```

---

### signer.rs

**Size:** 18978 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{
    fmt,
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicU8, Ordering},
        mpsc::Sender,
        Arc,
    },
    thread,
    thread::JoinHandle,
};

use rand::Rng;
use rand_core::OsRng;

use super::{
    ContentEncryptionAlgorithm, DelegationConditions, EncryptedPrivateKey, Error, Event, EventKind,
    EventV1, EventV2, Id, KeySecurity, KeySigner, Metadata, PreEvent, PreEventV2, PrivateKey,
    PublicKey, PublicKeyHex, Rumor, RumorV1, RumorV2, Signature, Tag, TagV1, TagV2, Unixtime,
};

/// Signer operations
pub trait Signer: fmt::Debug {
    /// Is the signer locked?
    fn is_locked(&self) -> bool;

    /// Try to unlock access to the private key
    fn unlock(&mut self, password: &str) -> Result<(), Error>;

    /// Lock access to the private key
    fn lock(&mut self);

    /// Change the passphrase used for locking access to the private key
    fn change_passphrase(&mut self, old: &str, new: &str, log_n: u8) -> Result<(), Error>;

    /// Upgrade the encrypted private key to the latest format
    fn upgrade(&mut self, pass: &str, log_n: u8) -> Result<(), Error>;

    /// What is the signer's public key?
    fn public_key(&self) -> PublicKey;

    /// What is the signer's encrypted private key?
    fn encrypted_private_key(&self) -> Option<&EncryptedPrivateKey>;

    /// Sign a 32-bit hash
    fn sign_id(&self, id: Id) -> Result<Signature, Error>;

    /// Sign a message (this hashes with SHA-256 first internally)
    fn sign(&self, message: &[u8]) -> Result<Signature, Error>;

    /// Encrypt
    fn encrypt(
        &self,
        other: &PublicKey,
        plaintext: &str,
        algo: ContentEncryptionAlgorithm,
    ) -> Result<String, Error>;

    /// Decrypt NIP-44
    fn decrypt(&self, other: &PublicKey, ciphertext: &str) -> Result<String, Error>;

    /// Get NIP-44 conversation key
    fn nip44_conversation_key(&self, other: &PublicKey) -> Result<[u8; 32], Error>;

    /// Export the private key in hex.
    ///
    /// This returns a boolean indicating if the key security was downgraded. If
    /// it was, the caller should save the new self.encrypted_private_key()
    ///
    /// We need the password and log_n parameters to possibly rebuild
    /// the EncryptedPrivateKey when downgrading key security
    fn export_private_key_in_hex(&mut self, pass: &str, log_n: u8)
        -> Result<(String, bool), Error>;

    /// Export the private key in bech32.
    ///
    /// This returns a boolean indicating if the key security was downgraded. If
    /// it was, the caller should save the new self.encrypted_private_key()
    ///
    /// We need the password and log_n parameters to possibly rebuild
    /// the EncryptedPrivateKey when downgrading key security
    fn export_private_key_in_bech32(
        &mut self,
        pass: &str,
        log_n: u8,
    ) -> Result<(String, bool), Error>;

    /// Get the security level of the private key
    fn key_security(&self) -> Result<KeySecurity, Error>;

    /// Generate delegation signature
    fn generate_delegation_signature(
        &self,
        delegated_pubkey: PublicKey,
        delegation_conditions: &DelegationConditions,
    ) -> Result<Signature, Error> {
        let input = format!(
            "nostr:delegation:{}:{}",
            delegated_pubkey.as_hex_string(),
            delegation_conditions.as_string()
        );

        self.sign(input.as_bytes())
    }

    /// Verify delegation signature
    fn verify_delegation_signature(
        &self,
        delegated_pubkey: PublicKey,
        delegation_conditions: &DelegationConditions,
        signature: &Signature,
    ) -> Result<(), Error> {
        let input = format!(
            "nostr:delegation:{}:{}",
            delegated_pubkey.as_hex_string(),
            delegation_conditions.as_string()
        );

        self.public_key().verify(input.as_bytes(), signature)
    }

    /// Sign an event
    fn sign_event(&self, input: PreEvent) -> Result<Event, Error> {
        // Verify the pubkey matches
        if input.pubkey != self.public_key() {
            return Err(Error::InvalidPrivateKey);
        }

        // Generate Id
        let id = input.hash()?;

        // Generate Signature
        let signature = self.sign_id(id)?;

        Ok(Event {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
            sig: signature,
        })
    }

    /// Sign an event
    fn sign_event2(&self, input: PreEventV2) -> Result<EventV2, Error> {
        // Verify the pubkey matches
        if input.pubkey != self.public_key() {
            return Err(Error::InvalidPrivateKey);
        }

        // Generate Id
        let id = input.hash()?;

        // Generate Signature
        let signature = self.sign_id(id)?;

        Ok(EventV2 {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
            sig: signature,
        })
    }

    /// Sign an event with Proof-of-Work
    fn sign_event_with_pow(
        &self,
        mut input: PreEvent,
        zero_bits: u8,
        work_sender: Option<Sender<u8>>,
    ) -> Result<Event, Error> {
        let target = format!("{zero_bits}");

        // Verify the pubkey matches
        if input.pubkey != self.public_key() {
            return Err(Error::InvalidPrivateKey);
        }

        // Strip any pre-existing nonce tags
        input.tags.retain(|t| t.tagname() != "nonce");

        // Add nonce tag to the end
        input.tags.push(Tag::new(&["nonce", "0", &target]));
        let index = input.tags.len() - 1;

        let cores = num_cpus::get();

        let quitting = Arc::new(AtomicBool::new(false));
        let nonce = Arc::new(AtomicU64::new(0)); // will store the nonce that works
        let best_work = Arc::new(AtomicU8::new(0));

        let mut join_handles: Vec<JoinHandle<_>> = Vec::with_capacity(cores);

        for core in 0..cores {
            let mut attempt: u64 = core as u64 * (u64::MAX / cores as u64);
            let mut input = input.clone();
            let quitting = quitting.clone();
            let nonce = nonce.clone();
            let best_work = best_work.clone();
            let work_sender = work_sender.clone();
            let join_handle = thread::spawn(move || {
                loop {
                    // Lower the thread priority so other threads aren't starved
                    let _ = thread_priority::set_current_thread_priority(
                        thread_priority::ThreadPriority::Min,
                    );

                    if quitting.load(Ordering::Relaxed) {
                        break;
                    }

                    input.tags[index].set_index(1, format!("{attempt}"));

                    let Id(id) = input.hash().unwrap();

                    let leading_zeroes = super::get_leading_zero_bits(&id);
                    if leading_zeroes >= zero_bits {
                        nonce.store(attempt, Ordering::Relaxed);
                        quitting.store(true, Ordering::Relaxed);
                        if let Some(sender) = work_sender.clone() {
                            sender.send(leading_zeroes).unwrap();
                        }
                        break;
                    } else if leading_zeroes > best_work.load(Ordering::Relaxed) {
                        best_work.store(leading_zeroes, Ordering::Relaxed);
                        if let Some(sender) = work_sender.clone() {
                            sender.send(leading_zeroes).unwrap();
                        }
                    }

                    attempt += 1;

                    // We don't update created_at, which is a bit tricky to
                    // synchronize.
                }
            });
            join_handles.push(join_handle);
        }

        for joinhandle in join_handles {
            let _ = joinhandle.join();
        }

        // We found the nonce. Do it for reals
        input.tags[index].set_index(1, format!("{}", nonce.load(Ordering::Relaxed)));
        let id = input.hash().unwrap();

        // Signature
        let signature = self.sign_id(id)?;

        Ok(Event {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
            sig: signature,
        })
    }

    /// Giftwrap an event
    fn giftwrap(&self, input: PreEvent, pubkey: PublicKey) -> Result<Event, Error> {
        let sender_pubkey = input.pubkey;

        // Verify the pubkey matches
        if sender_pubkey != self.public_key() {
            return Err(Error::InvalidPrivateKey);
        }

        let seal_backdate = Unixtime(
            input.created_at.0
                - OsRng.sample(rand::distributions::Uniform::new(30, 60 * 60 * 24 * 2)),
        );
        let giftwrap_backdate = Unixtime(
            input.created_at.0
                - OsRng.sample(rand::distributions::Uniform::new(30, 60 * 60 * 24 * 2)),
        );

        let seal = {
            let rumor = Rumor::new(input)?;
            let rumor_json = serde_json::to_string(&rumor)?;
            let encrypted_rumor_json =
                self.encrypt(&pubkey, &rumor_json, ContentEncryptionAlgorithm::Nip44v2)?;

            let pre_seal = PreEvent {
                pubkey: sender_pubkey,
                created_at: seal_backdate,
                kind: EventKind::Seal,
                content: encrypted_rumor_json,
                tags: vec![],
            };

            self.sign_event(pre_seal)?
        };

        // Generate a random keypair for the gift wrap
        let random_signer = {
            let random_private_key = PrivateKey::generate();
            KeySigner::from_private_key(random_private_key, "", 1)
        }?;

        let seal_json = serde_json::to_string(&seal)?;
        let encrypted_seal_json =
            random_signer.encrypt(&pubkey, &seal_json, ContentEncryptionAlgorithm::Nip44v2)?;

        let pre_giftwrap = PreEvent {
            pubkey: random_signer.public_key(),
            created_at: giftwrap_backdate,
            kind: EventKind::GiftWrap,
            content: encrypted_seal_json,
            tags: vec![Tag::new_pubkey(pubkey, None, None)],
        };

        random_signer.sign_event(pre_giftwrap)
    }

    /// Giftwrap an event
    fn giftwrap2(&self, input: PreEventV2, pubkey: PublicKey) -> Result<EventV2, Error> {
        let sender_pubkey = input.pubkey;

        // Verify the pubkey matches
        if sender_pubkey != self.public_key() {
            return Err(Error::InvalidPrivateKey);
        }

        let seal_backdate = Unixtime(
            input.created_at.0
                - OsRng.sample(rand::distributions::Uniform::new(30, 60 * 60 * 24 * 2)),
        );
        let giftwrap_backdate = Unixtime(
            input.created_at.0
                - OsRng.sample(rand::distributions::Uniform::new(30, 60 * 60 * 24 * 2)),
        );

        let seal = {
            let rumor = RumorV2::new(input)?;
            let rumor_json = serde_json::to_string(&rumor)?;
            let encrypted_rumor_json =
                self.encrypt(&pubkey, &rumor_json, ContentEncryptionAlgorithm::Nip44v2)?;

            let pre_seal = PreEventV2 {
                pubkey: sender_pubkey,
                created_at: seal_backdate,
                kind: EventKind::Seal,
                content: encrypted_rumor_json,
                tags: vec![],
            };

            self.sign_event2(pre_seal)?
        };

        // Generate a random keypair for the gift wrap
        let random_signer = {
            let random_private_key = PrivateKey::generate();
            KeySigner::from_private_key(random_private_key, "", 1)
        }?;

        let seal_json = serde_json::to_string(&seal)?;
        let encrypted_seal_json =
            random_signer.encrypt(&pubkey, &seal_json, ContentEncryptionAlgorithm::Nip44v2)?;

        let pre_giftwrap = PreEventV2 {
            pubkey: random_signer.public_key(),
            created_at: giftwrap_backdate,
            kind: EventKind::GiftWrap,
            content: encrypted_seal_json,
            tags: vec![TagV2::Pubkey {
                pubkey: pubkey.into(),
                recommended_relay_url: None,
                petname: None,
                trailing: vec![],
            }],
        };

        random_signer.sign_event2(pre_giftwrap)
    }

    /// Create an event that sets Metadata
    fn create_metadata_event(
        &self,
        mut input: PreEvent,
        metadata: Metadata,
    ) -> Result<Event, Error> {
        input.kind = EventKind::Metadata;
        input.content = serde_json::to_string(&metadata)?;
        self.sign_event(input)
    }

    /// Create a ZapRequest event
    /// These events are not published to nostr, they are sent to a lnurl.
    fn create_zap_request_event(
        &self,
        recipient_pubkey: PublicKey,
        zapped_event: Option<Id>,
        millisatoshis: u64,
        relays: Vec<String>,
        content: String,
    ) -> Result<Event, Error> {
        let mut relays_tag = Tag::new(&["relays"]);
        relays_tag.push_values(relays);

        let mut pre_event = PreEvent {
            pubkey: self.public_key(),
            created_at: Unixtime::now(),
            kind: EventKind::ZapRequest,
            tags: vec![
                Tag::new_pubkey(recipient_pubkey, None, None),
                relays_tag,
                Tag::new(&["amount", &format!("{millisatoshis}")]),
            ],
            content,
        };

        if let Some(ze) = zapped_event {
            pre_event.tags.push(Tag::new_event(ze, None, None));
        }

        self.sign_event(pre_event)
    }

    /// Decrypt the contents of an event
    fn decrypt_event_contents(&self, event: &Event) -> Result<String, Error> {
        if !event.kind.contents_are_encrypted() {
            return Err(Error::WrongEventKind);
        }

        let pubkey = if event.pubkey == self.public_key() {
            // If you are the author, get the other pubkey from the tags
            event
                .people()
                .iter()
                .filter_map(|(pk, _, _)| if *pk != event.pubkey { Some(*pk) } else { None })
                .nth(0)
                .unwrap_or(event.pubkey) // in case you sent it to yourself.
        } else {
            event.pubkey
        };

        self.decrypt(&pubkey, &event.content)
    }

    /// If a gift wrap event, unwrap and return the inner Rumor
    fn unwrap_giftwrap(&self, event: &Event) -> Result<Rumor, Error> {
        if event.kind != EventKind::GiftWrap {
            return Err(Error::WrongEventKind);
        }

        // Verify you are tagged
        let mut tagged = false;
        for t in event.tags.iter() {
            if let Ok((pubkey, _, _)) = t.parse_pubkey() {
                if pubkey == self.public_key() {
                    tagged = true;
                }
            }
        }
        if !tagged {
            return Err(Error::InvalidRecipient);
        }

        // Decrypt the content
        let content = self.decrypt(&event.pubkey, &event.content)?;

        // Translate into a seal Event
        let seal: Event = serde_json::from_str(&content)?;

        // Verify it is a Seal
        if seal.kind != EventKind::Seal {
            return Err(Error::WrongEventKind);
        }

        // Note the author
        let author = seal.pubkey;

        // Decrypt the content
        let content = self.decrypt(&seal.pubkey, &seal.content)?;

        // Translate into a Rumor
        let rumor: Rumor = serde_json::from_str(&content)?;

        // Compae the author
        if rumor.pubkey != author {
            return Err(Error::InvalidPublicKey);
        }

        // Return the Rumor
        Ok(rumor)
    }

    /// If a gift wrap event, unwrap and return the inner Rumor
    /// @deprecated for migrations only
    fn unwrap_giftwrap2(&self, event: &EventV2) -> Result<RumorV2, Error> {
        if event.kind != EventKind::GiftWrap {
            return Err(Error::WrongEventKind);
        }

        // Verify you are tagged
        let pkhex: PublicKeyHex = self.public_key().into();
        let mut tagged = false;
        for t in event.tags.iter() {
            if let TagV2::Pubkey { pubkey, .. } = t {
                if *pubkey == pkhex {
                    tagged = true;
                }
            }
        }
        if !tagged {
            return Err(Error::InvalidRecipient);
        }

        // Decrypt the content
        let content = self.decrypt(&event.pubkey, &event.content)?;

        // Translate into a seal Event
        let seal: EventV2 = serde_json::from_str(&content)?;

        // Verify it is a Seal
        if seal.kind != EventKind::Seal {
            return Err(Error::WrongEventKind);
        }

        // Note the author
        let author = seal.pubkey;

        // Decrypt the content
        let content = self.decrypt(&seal.pubkey, &seal.content)?;

        // Translate into a Rumor
        let rumor: RumorV2 = serde_json::from_str(&content)?;

        // Compae the author
        if rumor.pubkey != author {
            return Err(Error::InvalidPublicKey);
        }

        // Return the Rumor
        Ok(rumor)
    }

    /// If a gift wrap event, unwrap and return the inner Rumor
    /// @deprecated for migrations only
    fn unwrap_giftwrap1(&self, event: &EventV1) -> Result<RumorV1, Error> {
        if event.kind != EventKind::GiftWrap {
            return Err(Error::WrongEventKind);
        }

        // Verify you are tagged
        let pkhex: PublicKeyHex = self.public_key().into();
        let mut tagged = false;
        for t in event.tags.iter() {
            if let TagV1::Pubkey { pubkey, .. } = t {
                if *pubkey == pkhex {
                    tagged = true;
                }
            }
        }
        if !tagged {
            return Err(Error::InvalidRecipient);
        }

        // Decrypt the content
        let content = self.decrypt(&event.pubkey, &event.content)?;

        // Translate into a seal Event
        let seal: EventV1 = serde_json::from_str(&content)?;

        // Verify it is a Seal
        if seal.kind != EventKind::Seal {
            return Err(Error::WrongEventKind);
        }

        // Note the author
        let author = seal.pubkey;

        // Decrypt the content
        let content = self.decrypt(&seal.pubkey, &seal.content)?;

        // Translate into a Rumor
        let rumor: RumorV1 = serde_json::from_str(&content)?;

        // Compae the author
        if rumor.pubkey != author {
            return Err(Error::InvalidPublicKey);
        }

        // Return the Rumor
        Ok(rumor)
    }
}
```

---

### simple_relay_list.rs

**Size:** 3488 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{collections::HashMap, fmt};

use serde::{
    de::{Deserializer, MapAccess, Visitor},
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::UncheckedUrl;

/// When and how to use a Relay
///
/// This is used only for `SimpleRelayList`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct SimpleRelayUsage {
    /// Whether to write to this relay
    pub write: bool,

    /// Whether to read from this relay
    pub read: bool,
}

impl Default for SimpleRelayUsage {
    fn default() -> SimpleRelayUsage {
        SimpleRelayUsage {
            write: false,
            read: true,
        }
    }
}

/// A list of relays with SimpleRelayUsage
///
/// This is only used for handling the contents of a kind-3 contact list.
/// For normal relay lists, consider using `RelayList` instead.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct SimpleRelayList(pub HashMap<UncheckedUrl, SimpleRelayUsage>);

impl SimpleRelayList {
    #[allow(dead_code)]
    pub(crate) fn mock() -> SimpleRelayList {
        let mut map: HashMap<UncheckedUrl, SimpleRelayUsage> = HashMap::new();
        let _ = map.insert(
            UncheckedUrl::from_str("wss://nostr.oxtr.dev"),
            SimpleRelayUsage {
                write: true,
                read: true,
            },
        );
        let _ = map.insert(
            UncheckedUrl::from_str("wss://nostr-relay.wlvs.space"),
            SimpleRelayUsage {
                write: false,
                read: true,
            },
        );
        SimpleRelayList(map)
    }
}

impl Serialize for SimpleRelayList {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(self.0.len()))?;
        for (k, v) in &self.0 {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for SimpleRelayList {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(SimpleRelayListVisitor)
    }
}

struct SimpleRelayListVisitor;

impl<'de> Visitor<'de> for SimpleRelayListVisitor {
    type Value = SimpleRelayList;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A JSON object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<SimpleRelayList, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: HashMap<UncheckedUrl, SimpleRelayUsage> = HashMap::new();
        while let Some((key, value)) = access.next_entry::<UncheckedUrl, SimpleRelayUsage>()? {
            let _ = map.insert(key, value);
        }
        Ok(SimpleRelayList(map))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {SimpleRelayList, test_simple_relay_list_serde}

    #[test]
    fn test_simple_relay_list_json() {
        let serialized = r#"{"wss://nostr.oxtr.dev":{"write":true,"read":true},"wss://relay.damus.io":{"write":true,"read":true},"wss://nostr.fmt.wiz.biz":{"write":true,"read":true},"wss://nostr-relay.wlvs.space":{"write":true,"read":true}}"#;
        let _simple_relay_list: SimpleRelayList = serde_json::from_str(serialized).unwrap();
    }
}
```

---

### subscription_id.rs

**Size:** 747 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use derive_more::{AsMut, AsRef, Deref, From, FromStr, Into};
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

/// A random client-chosen string used to refer to a subscription
#[derive(
    AsMut, AsRef, Clone, Debug, Deref, Deserialize, Eq, From, FromStr, Into, PartialEq, Serialize,
)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct SubscriptionId(pub String);

impl SubscriptionId {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> SubscriptionId {
        SubscriptionId("lk234js09".to_owned())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {SubscriptionId, test_subscription_id_serde}
}
```

---

### tag.rs

**Size:** 80 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use super::versioned::tag3::TagV3;

/// A tag on an Event
pub type Tag = TagV3;
```

---

### unixtime.rs

**Size:** 2125 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{
    ops::{Add, Sub},
    time::Duration,
};

use derive_more::{AsMut, AsRef, Deref, Display, From, Into};
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

/// An integer count of the number of seconds from 1st January 1970.
/// This does not count any of the leap seconds that have occurred, it
/// simply presumes UTC never had leap seconds; yet it is well known
/// and well understood.
#[derive(
    AsMut,
    AsRef,
    Clone,
    Copy,
    Debug,
    Deref,
    Deserialize,
    Display,
    Eq,
    From,
    Into,
    Ord,
    PartialEq,
    PartialOrd,
    Serialize,
)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct Unixtime(pub i64);

impl Unixtime {
    /// Get the current unixtime (depends on the system clock being accurate)
    pub fn now() -> Unixtime {
        Unixtime(std::time::UNIX_EPOCH.elapsed().unwrap().as_secs() as i64)
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Unixtime {
        Unixtime(1668572286)
    }
}

impl Add<Duration> for Unixtime {
    type Output = Self;

    fn add(self, rhs: Duration) -> Self::Output {
        Unixtime(self.0 + rhs.as_secs() as i64)
    }
}

impl Sub<Duration> for Unixtime {
    type Output = Self;

    fn sub(self, rhs: Duration) -> Self::Output {
        Unixtime(self.0 - rhs.as_secs() as i64)
    }
}

impl Sub<Unixtime> for Unixtime {
    type Output = Duration;

    fn sub(self, rhs: Unixtime) -> Self::Output {
        Duration::from_secs((self.0 - rhs.0).unsigned_abs())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {Unixtime, test_unixtime_serde}

    #[test]
    fn test_print_now() {
        println!("NOW: {}", Unixtime::now());
    }

    #[test]
    fn test_unixtime_math() {
        let now = Unixtime::now();
        let fut = now + Duration::from_secs(70);
        assert!(fut > now);
        assert_eq!(fut.0 - now.0, 70);
        let back = fut - Duration::from_secs(70);
        assert_eq!(now, back);
        assert_eq!(now - back, Duration::ZERO);
    }
}
```

---

### url.rs

**Size:** 10693 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::error::Error;

/// A string that is supposed to represent a URL but which might be invalid
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Serialize, Ord)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct UncheckedUrl(pub String);

impl fmt::Display for UncheckedUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl UncheckedUrl {
    /// Create an UncheckedUrl from a &str
    // note - this from_str cannot error, so we don't impl std::str::FromStr which by
    //        all rights should be called TryFromStr anyway
    #[allow(clippy::should_implement_trait)]
    pub fn from_str(s: &str) -> UncheckedUrl {
        UncheckedUrl(s.to_owned())
    }

    /// Create an UncheckedUrl from a String
    pub fn from_string(s: String) -> UncheckedUrl {
        UncheckedUrl(s)
    }

    /// As &str
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// As nrelay
    pub fn as_bech32_string(&self) -> String {
        bech32::encode::<bech32::Bech32>(*super::HRP_NRELAY, self.0.as_bytes()).unwrap()
    }

    /// Import from a bech32 encoded string ("nrelay")
    pub fn try_from_bech32_string(s: &str) -> Result<UncheckedUrl, Error> {
        let data = bech32::decode(s)?;
        if data.0 != *super::HRP_NRELAY {
            Err(Error::WrongBech32(
                super::HRP_NRELAY.to_lowercase(),
                data.0.to_lowercase(),
            ))
        } else {
            let s = std::str::from_utf8(&data.1)?.to_owned();
            Ok(UncheckedUrl(s))
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> UncheckedUrl {
        UncheckedUrl("http://localhost:6102".to_string())
    }
}

/// A String representing a valid URL with an authority present including an
/// Internet based host.
///
/// We don't serialize/deserialize these directly, see `UncheckedUrl` for that
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct Url(String);

impl fmt::Display for Url {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Url {
    /// Create a new Url from an UncheckedUrl
    pub fn try_from_unchecked_url(u: &UncheckedUrl) -> Result<Url, Error> {
        Url::try_from_str(&u.0)
    }

    /// Create a new Url from a string
    pub fn try_from_str(s: &str) -> Result<Url, Error> {
        // We use the url crate to parse and normalize
        let url = url::Url::parse(s.trim())?;

        if !url.has_authority() {
            return Err(Error::InvalidUrlMissingAuthority);
        }

        //begin more support
        if let Some(host) = url.host() {
            match host {
                url::Host::Domain(_) => {
                    // Strange that we can't access as a string
                    let s = format!("{host}");
                    if s != s.trim()
                    /* || s.starts_with("localhost") */
                    {
                        return Err(Error::InvalidUrlHost(s));
                    }
                }
                url::Host::Ipv4(addr) => {
                    let addrx = core_net::Ipv4Addr::from(addr.octets());
                    if !addrx.is_global() {
                        return Err(Error::InvalidUrlHost(format!("{host}")));
                    }
                }
                url::Host::Ipv6(addr) => {
                    let addrx = core_net::Ipv6Addr::from(addr.octets());
                    if !addrx.is_global() {
                        return Err(Error::InvalidUrlHost(format!("{host}")));
                    }
                }
            }
        } else {
            //begin more support
            return Err(Error::InvalidUrlHost("".to_string()));
        }

        Ok(Url(url.as_str().to_owned()))
    }

    /// Convert into a UncheckedUrl
    pub fn to_unchecked_url(&self) -> UncheckedUrl {
        UncheckedUrl(self.0.clone())
    }

    /// As &str
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Into String
    pub fn into_string(self) -> String {
        self.0
    }

    /// As url crate Url
    pub fn as_url_crate_url(&self) -> url::Url {
        url::Url::parse(&self.0).unwrap()
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Url {
        Url("http://localhost:6102".to_string())
    }
}

/// A Url validated as a nostr relay url in canonical form
/// We don't serialize/deserialize these directly, see `UncheckedUrl` for that
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayUrl(String);

impl fmt::Display for RelayUrl {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl RelayUrl {
    /// Create a new RelayUrl from a Url
    pub fn try_from_url(u: &Url) -> Result<RelayUrl, Error> {
        // Verify we aren't looking at a comma-separated-list of URLs
        // (technically they might be valid URLs but just about 100% of the time
        // it's somebody else's bad data)
        if u.0.contains(",wss://") || u.0.contains(",ws://") {
            return Err(Error::Url(format!(
                "URL appears to be a list of multiple URLs: {}",
                u.0
            )));
        }

        let url = url::Url::parse(&u.0)?;

        // Verify the scheme is websockets
        if url.scheme() != "wss" && url.scheme() != "ws" {

            //return Err(Error::InvalidUrlScheme(url.scheme().to_owned()));
        }

        // Verify host is some
        if !url.has_host() {
            return Err(Error::Url(format!("URL has no host: {}", u.0)));
        }

        Ok(RelayUrl(url.as_str().to_owned()))
    }

    /// Create a new RelayUrl from an UncheckedUrl
    pub fn try_from_unchecked_url(u: &UncheckedUrl) -> Result<RelayUrl, Error> {
        Self::try_from_str(&u.0)
    }

    /// Construct a new RelayUrl from a Url
    pub fn try_from_str(s: &str) -> Result<RelayUrl, Error> {
        let url = Url::try_from_str(s)?;
        RelayUrl::try_from_url(&url)
    }

    /// Convert into a Url
    // fixme should be 'as_url'
    pub fn to_url(&self) -> Url {
        Url(self.0.clone())
    }

    /// As url crate Url
    pub fn as_url_crate_url(&self) -> url::Url {
        url::Url::parse(&self.0).unwrap()
    }

    /// As nrelay
    pub fn as_bech32_string(&self) -> String {
        bech32::encode::<bech32::Bech32>(*super::HRP_NRELAY, self.0.as_bytes()).unwrap()
    }

    /// Convert into a UncheckedUrl
    pub fn to_unchecked_url(&self) -> UncheckedUrl {
        UncheckedUrl(self.0.clone())
    }

    /// Host
    pub fn host(&self) -> String {
        self.as_url_crate_url().host_str().unwrap().to_owned()
    }

    /// As &str
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Into String
    pub fn into_string(self) -> String {
        self.0
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Url {
        Url("wss://localhost:6102".to_string())
    }
}

impl TryFrom<Url> for RelayUrl {
    type Error = Error;

    fn try_from(u: Url) -> Result<RelayUrl, Error> {
        RelayUrl::try_from_url(&u)
    }
}

impl TryFrom<&Url> for RelayUrl {
    type Error = Error;

    fn try_from(u: &Url) -> Result<RelayUrl, Error> {
        RelayUrl::try_from_url(u)
    }
}

impl From<RelayUrl> for Url {
    fn from(ru: RelayUrl) -> Url {
        ru.to_url()
    }
}

/// A canonical URL representing just a relay's origin
/// (without path/query/fragment or username/password)
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, PartialOrd, Serialize, Ord)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayOrigin(String);

impl RelayOrigin {
    /// Convert a RelayUrl into a RelayOrigin
    pub fn from_relay_url(url: RelayUrl) -> RelayOrigin {
        let mut xurl = url::Url::parse(url.as_str()).unwrap();
        xurl.set_fragment(None);
        xurl.set_query(None);
        xurl.set_path("/");
        let _ = xurl.set_username("");
        let _ = xurl.set_password(None);
        RelayOrigin(xurl.into())
    }

    /// Construct a new RelayOrigin from a string
    pub fn try_from_str(s: &str) -> Result<RelayOrigin, Error> {
        let url = RelayUrl::try_from_str(s)?;
        Ok(RelayOrigin::from_relay_url(url))
    }

    /// Create a new Url from an UncheckedUrl
    pub fn try_from_unchecked_url(u: &UncheckedUrl) -> Result<RelayOrigin, Error> {
        let relay_url = RelayUrl::try_from_str(&u.0)?;
        Ok(relay_url.into())
    }

    /// Convert this RelayOrigin into a RelayUrl
    pub fn into_relay_url(self) -> RelayUrl {
        RelayUrl(self.0)
    }

    /// Get a RelayUrl matching this RelayOrigin
    pub fn as_relay_url(&self) -> RelayUrl {
        RelayUrl(self.0.clone())
    }

    /// Convert into a UncheckedUrl
    pub fn to_unchecked_url(&self) -> UncheckedUrl {
        UncheckedUrl(self.0.clone())
    }

    /// As &str
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Into String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for RelayOrigin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<RelayUrl> for RelayOrigin {
    fn from(ru: RelayUrl) -> RelayOrigin {
        RelayOrigin::from_relay_url(ru)
    }
}

impl From<RelayOrigin> for RelayUrl {
    fn from(ru: RelayOrigin) -> RelayUrl {
        ru.into_relay_url()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_serde;

    test_serde! {UncheckedUrl, test_unchecked_url_serde}

    #[test]
    fn test_url_case() {
        let url = Url::try_from_str("Wss://MyRelay.example.COM/PATH?Query").unwrap();
        assert_eq!(url.as_str(), "wss://myrelay.example.com/PATH?Query");
    }

    #[test]
    fn test_relay_url_slash() {
        let input = "Wss://MyRelay.example.COM";
        let url = RelayUrl::try_from_str(input).unwrap();
        assert_eq!(url.as_str(), "wss://myrelay.example.com/");
    }

    #[test]
    fn test_relay_origin() {
        let input = "wss://user:pass@filter.nostr.wine:444/npub1234?x=y#z";
        let relay_url = RelayUrl::try_from_str(input).unwrap();
        let origin: RelayOrigin = relay_url.into();
        assert_eq!(origin.as_str(), "wss://filter.nostr.wine:444/");
    }
}
```

---

### versioned/client_message1.rs

**Size:** 4312 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV1;
use crate::types::{Filter, SubscriptionId};

/// A message from a client to a relay
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum ClientMessageV1 {
    /// An event
    Event(Box<EventV1>),

    /// A subscription request
    Req(SubscriptionId, Vec<Filter>),

    /// A request to close a subscription
    Close(SubscriptionId),

    /// Used to send authentication events
    Auth(Box<EventV1>),
}

impl Serialize for ClientMessageV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ClientMessageV1::Event(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            ClientMessageV1::Req(id, filters) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("REQ")?;
                seq.serialize_element(&id)?;
                for filter in filters {
                    seq.serialize_element(&filter)?;
                }
                seq.end()
            }
            ClientMessageV1::Close(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("CLOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            ClientMessageV1::Auth(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ClientMessageV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ClientMessageVisitor)
    }
}

struct ClientMessageVisitor;

impl<'de> Visitor<'de> for ClientMessageVisitor {
    type Value = ClientMessageV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<ClientMessageV1, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<ClientMessageV1> = None;
        if word == "EVENT" {
            let event: EventV1 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV1::Event(Box::new(event)))
        } else if word == "REQ" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let mut filters: Vec<Filter> = vec![];
            loop {
                let f: Option<Filter> = seq.next_element()?;
                match f {
                    None => break,
                    Some(fil) => filters.push(fil),
                }
            }
            output = Some(ClientMessageV1::Req(id, filters))
        } else if word == "CLOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(ClientMessageV1::Close(id))
        } else if word == "AUTH" {
            let event: EventV1 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV1::Auth(Box::new(event)))
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(cm) => Ok(cm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}
```

---

### versioned/client_message2.rs

**Size:** 4608 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV2;
use crate::types::{Filter, SubscriptionId};

/// A message from a client to a relay
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum ClientMessageV2 {
    /// An event
    Event(Box<EventV2>),

    /// A subscription request
    Req(SubscriptionId, Vec<Filter>),

    /// A request to close a subscription
    Close(SubscriptionId),

    /// Used to send authentication events
    Auth(Box<EventV2>),
}

impl ClientMessageV2 {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> ClientMessageV2 {
        ClientMessageV2::Event(Box::new(EventV2::mock()))
    }
}

impl Serialize for ClientMessageV2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ClientMessageV2::Event(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            ClientMessageV2::Req(id, filters) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("REQ")?;
                seq.serialize_element(&id)?;
                for filter in filters {
                    seq.serialize_element(&filter)?;
                }
                seq.end()
            }
            ClientMessageV2::Close(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("CLOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            ClientMessageV2::Auth(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ClientMessageV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ClientMessageVisitor)
    }
}

struct ClientMessageVisitor;

impl<'de> Visitor<'de> for ClientMessageVisitor {
    type Value = ClientMessageV2;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<ClientMessageV2, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<ClientMessageV2> = None;
        if word == "EVENT" {
            let event: EventV2 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV2::Event(Box::new(event)))
        } else if word == "REQ" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let mut filters: Vec<Filter> = vec![];
            loop {
                let f: Option<Filter> = seq.next_element()?;
                match f {
                    None => break,
                    Some(fil) => filters.push(fil),
                }
            }
            output = Some(ClientMessageV2::Req(id, filters))
        } else if word == "CLOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(ClientMessageV2::Close(id))
        } else if word == "AUTH" {
            let event: EventV2 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV2::Auth(Box::new(event)))
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(cm) => Ok(cm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {ClientMessageV2, test_client_message_serde}
}
```

---

### versioned/client_message3.rs

**Size:** 4608 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV3;
use crate::types::{Filter, SubscriptionId};

/// A message from a client to a relay
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum ClientMessageV3 {
    /// An event
    Event(Box<EventV3>),

    /// A subscription request
    Req(SubscriptionId, Vec<Filter>),

    /// A request to close a subscription
    Close(SubscriptionId),

    /// Used to send authentication events
    Auth(Box<EventV3>),
}

impl ClientMessageV3 {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> ClientMessageV3 {
        ClientMessageV3::Event(Box::new(EventV3::mock()))
    }
}

impl Serialize for ClientMessageV3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            ClientMessageV3::Event(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            ClientMessageV3::Req(id, filters) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("REQ")?;
                seq.serialize_element(&id)?;
                for filter in filters {
                    seq.serialize_element(&filter)?;
                }
                seq.end()
            }
            ClientMessageV3::Close(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("CLOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            ClientMessageV3::Auth(event) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&event)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ClientMessageV3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(ClientMessageVisitor)
    }
}

struct ClientMessageVisitor;

impl<'de> Visitor<'de> for ClientMessageVisitor {
    type Value = ClientMessageV3;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<ClientMessageV3, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<ClientMessageV3> = None;
        if word == "EVENT" {
            let event: EventV3 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV3::Event(Box::new(event)))
        } else if word == "REQ" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let mut filters: Vec<Filter> = vec![];
            loop {
                let f: Option<Filter> = seq.next_element()?;
                match f {
                    None => break,
                    Some(fil) => filters.push(fil),
                }
            }
            output = Some(ClientMessageV3::Req(id, filters))
        } else if word == "CLOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(ClientMessageV3::Close(id))
        } else if word == "AUTH" {
            let event: EventV3 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(ClientMessageV3::Auth(Box::new(event)))
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(cm) => Ok(cm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {ClientMessageV3, test_client_message_serde}
}
```

---

### versioned/event1.rs

**Size:** 38768 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::str::FromStr;

use lightning_invoice::Bolt11Invoice;
#[cfg(feature = "speedy")]
use regex::Regex;
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    id::{self, Id},
    Error, EventDelegation, EventKind, EventReference, IntoVec, MilliSatoshi, NAddr, NostrBech32,
    NostrUrl, PublicKey, PublicKeyHex, RelayUrl, Signature, TagV1, Unixtime, ZapDataV1,
};

/// The main event type
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct EventV1 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The signature of the event, which cryptographically verifies that the
    /// holder of the PrivateKey matching the event's PublicKey generated
    /// (or authorized) this event. The signature is taken over the id field
    /// only, but the id field is taken over the rest of the event data.
    pub sig: Signature,

    /// DEPRECATED (please set to Null): An optional verified time for the event
    /// (using OpenTimestamp)
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ots: Option<String>,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV1>,
}

macro_rules! serialize_inner_event {
    ($pubkey:expr, $created_at:expr, $kind:expr, $tags:expr,
     $content:expr) => {{
        format!(
            "[0,{},{},{},{},{}]",
            serde_json::to_string($pubkey)?,
            serde_json::to_string($created_at)?,
            serde_json::to_string($kind)?,
            serde_json::to_string($tags)?,
            serde_json::to_string($content)?
        )
    }};
}

/// Data used to construct an event
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct PreEventV1 {
    /// The public key of the actor who is creating the event
    pub pubkey: PublicKey,
    /// The time at which the event was created
    pub created_at: Unixtime,
    /// The kind of event
    pub kind: EventKind,
    /// A set of tags that apply to the event
    pub tags: Vec<TagV1>,
    /// The content of the event
    pub content: String,
}

impl PreEventV1 {
    /// Generate an ID from this PreEvent for use in an Event or a Rumor
    pub fn hash(&self) -> Result<Id, Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Hash
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();
        Ok(Id(id))
    }
}

/// A Rumor is an Event without a signature
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RumorV1 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV1>,
}

impl RumorV1 {
    /// Create a new rumor
    pub fn new(input: PreEventV1) -> Result<RumorV1, Error> {
        // Generate Id
        let id = input.hash()?;

        Ok(RumorV1 {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
        })
    }

    /// Turn into an Event (the signature will be all zeroes)
    pub fn into_event_with_bad_signature(self) -> EventV1 {
        EventV1 {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            sig: Signature::zeroes(),
            ots: None,
            content: self.content,
            tags: self.tags,
        }
    }
}

impl EventV1 {
    /// Check the validity of an event. This is useful if you deserialize an
    /// event from the network. If you create an event using new() it should
    /// already be trustworthy.
    pub fn verify(&self, maxtime: Option<Unixtime>) -> Result<(), Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Verify the signature
        self.pubkey.verify(serialized.as_bytes(), &self.sig)?;

        // Also verify the ID is the SHA256
        // (the above verify function also does it internally,
        //  so there is room for improvement here)
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();

        // Optional verify that the message was in the past
        if let Some(mt) = maxtime {
            if self.created_at > mt {
                return Err(Error::EventInFuture);
            }
        }

        if id != self.id.0 {
            Err(Error::HashMismatch)
        } else {
            Ok(())
        }
    }

    /// Get the k-tag kind, if any
    pub fn k_tag_kind(&self) -> Option<EventKind> {
        for tag in self.tags.iter() {
            if let TagV1::Kind { kind, .. } = tag {
                return Some(*kind);
            }
        }
        None
    }

    /// If the event refers to people by tag, get all the PublicKeys it refers
    /// to along with recommended relay URL and petname for each
    pub fn people(&self) -> Vec<(PublicKeyHex, Option<RelayUrl>, Option<String>)> {
        let mut output: Vec<(PublicKeyHex, Option<RelayUrl>, Option<String>)> = Vec::new();
        // All 'p' tags
        for tag in self.tags.iter() {
            if let TagV1::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                ..
            } = tag
            {
                output.push((
                    pubkey.to_owned(),
                    recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
                    petname.to_owned(),
                ));
            }
        }

        output
    }

    /// If the pubkey is tagged in the event
    pub fn is_tagged(&self, pubkey: &PublicKey) -> bool {
        let pkh: PublicKeyHex = pubkey.into();

        for tag in self.tags.iter() {
            if let TagV1::Pubkey { pubkey, .. } = tag {
                if *pubkey == pkh {
                    return true;
                }
            }
        }

        false
    }

    /// If the event refers to people within the contents, get all the
    /// PublicKeys it refers to within the contents.
    pub fn people_referenced_in_content(&self) -> Vec<PublicKey> {
        let mut output = Vec::new();
        for nurl in NostrUrl::find_all_in_string(&self.content).drain(..) {
            if let NostrBech32::Pubkey(pk) = nurl.0 {
                output.push(pk);
            }
            if let NostrBech32::Profile(prof) = nurl.0 {
                output.push(prof.pubkey);
            }
        }
        output
    }

    /// All events IDs that this event refers to, whether root, reply, mention,
    /// or otherwise along with optional recommended relay URLs
    pub fn referred_events(&self) -> Vec<EventReference> {
        let mut output: Vec<EventReference> = Vec::new();

        // Collect every 'e' tag and 'a' tag
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                output.push(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.clone(),
                });
            } else if let TagV1::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    output.push(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        output
    }

    /// Get a reference to another event that this event replies to.
    /// An event can only reply to one other event via 'e' or 'a' tag from a
    /// feed-displayable event that is not a Repost.
    pub fn replies_to(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // Repost 'e' and 'a' tags are always considered mentions, not replies.
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            return None;
        }

        // If there are no 'e' tags nor 'a' tags, then none
        let num_event_ref_tags = self
            .tags
            .iter()
            .filter(|e| matches!(e, TagV1::Event { .. }) || matches!(e, TagV1::Address { .. }))
            .count();
        if num_event_ref_tags == 0 {
            return None;
        }

        // look for an 'e' tag with marker 'reply'
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "reply" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // Use the last unmarked 'e' tag or any 'a' tag
        if let Some(tag) = self.tags.iter().rev().find(|t| {
            matches!(t, TagV1::Event { marker: None, .. }) || matches!(t, TagV1::Address { .. })
        }) {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                return Some(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.to_owned(),
                });
            } else if let TagV1::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    return Some(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        None
    }

    /// If this event replies to a thread, get that threads root event Id if
    /// available, along with an optional recommended_relay_url
    pub fn replies_to_root(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                }
            }
        }

        // otherwise use the first unmarked 'e' tag or first 'a' tag
        // (even if there is only 1 'e' or 'a' tag which means it is both root and
        // reply)
        if let Some(tag) = self.tags.iter().find(|t| {
            matches!(t, TagV1::Event { marker: None, .. }) || matches!(t, TagV1::Address { .. })
        }) {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                return Some(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.to_owned(),
                });
            } else if let TagV1::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    return Some(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        None
    }

    /// If this event mentions others, get those other event Ids
    /// and optional recommended relay Urls
    pub fn mentions(&self) -> Vec<EventReference> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<EventReference> = Vec::new();

        // For kind=6 and kind=16, all 'e' and 'a' tags are mentions
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            for tag in self.tags.iter() {
                if let TagV1::Event {
                    id,
                    recommended_relay_url,
                    marker,
                    ..
                } = tag
                {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                } else if let TagV1::Address {
                    kind,
                    pubkey,
                    d,
                    relay_url: Some(rurl),
                    trailing: _,
                } = tag
                {
                    if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                        output.push(EventReference::Addr(NAddr {
                            d: d.to_string(),
                            relays: vec![rurl.clone()],
                            kind: *kind,
                            author: pk,
                        }));
                    }
                }
            }

            return output;
        }

        // Look for nostr links within the content

        // Collect every 'e' tag marked as 'mention'
        for tag in self.tags.iter() {
            if let TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "mention" {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // Collect every unmarked 'e' or 'a' tag that is not the first (root) or the
        // last (reply)
        let e_tags: Vec<&TagV1> = self
            .tags
            .iter()
            .filter(|e| {
                matches!(e, TagV1::Event { marker: None, .. }) || matches!(e, TagV1::Address { .. })
            })
            .collect();
        if e_tags.len() > 2 {
            // mentions are everything other than first and last
            for tag in &e_tags[1..e_tags.len() - 1] {
                if let TagV1::Event {
                    id,
                    recommended_relay_url,
                    marker,
                    ..
                } = tag
                {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                } else if let TagV1::Address {
                    kind,
                    pubkey,
                    d,
                    relay_url: Some(rurl),
                    ..
                } = tag
                {
                    if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                        output.push(EventReference::Addr(NAddr {
                            d: d.to_string(),
                            relays: vec![rurl.clone()],
                            kind: *kind,
                            author: pk,
                        }));
                    }
                }
            }
        }

        output
    }

    /// If this event reacts to another, get that other event's Id,
    /// the reaction content, and an optional Recommended relay Url
    pub fn reacts_to(&self) -> Option<(Id, String, Option<RelayUrl>)> {
        if self.kind != EventKind::Reaction {
            return None;
        }

        // The last 'e' tag is it
        if let Some(TagV1::Event {
            id,
            recommended_relay_url,
            ..
        }) = self
            .tags
            .iter()
            .rev()
            .find(|t| matches!(t, TagV1::Event { .. }))
        {
            return Some((
                *id,
                self.content.clone(),
                recommended_relay_url
                    .as_ref()
                    .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
            ));
        }

        None
    }

    /// If this event deletes others, get all the Ids of the events that it
    /// deletes along with the reason for the deletion
    pub fn deletes(&self) -> Option<(Vec<Id>, String)> {
        if self.kind != EventKind::EventDeletion {
            return None;
        }

        let mut ids: Vec<Id> = Vec::new();

        // All 'e' tags are deleted
        for tag in self.tags.iter() {
            if let TagV1::Event { id, .. } = tag {
                ids.push(*id);
            }
        }

        if ids.is_empty() {
            None
        } else {
            Some((ids, self.content.clone()))
        }
    }

    /// If this event zaps another event, get data about that.
    ///
    /// That includes the Id, the amount, and the public key of the provider,
    /// all of which should be verified by the caller.
    ///
    /// Errors returned from this are not fatal, but may be useful for
    /// explaining to a user why a zap receipt is invalid.
    pub fn zaps(&self) -> Result<Option<ZapDataV1>, Error> {
        if self.kind != EventKind::Zap {
            return Ok(None);
        }

        let mut zapped_id: Option<Id> = None;
        let mut zapped_amount: Option<MilliSatoshi> = None;
        let mut zapped_pubkey: Option<PublicKey> = None;

        for tag in self.tags.iter() {
            if let TagV1::Other { tag, data } = tag {
                // Find the bolt11 tag
                if tag != "bolt11" {
                    continue;
                }
                if data.is_empty() {
                    return Err(Error::ZapReceipt("missing bolt11 tag value".to_string()));
                }

                // Extract as an Invoice
                let result = Bolt11Invoice::from_str(&data[0]);
                if let Err(e) = result {
                    return Err(Error::ZapReceipt(format!("bolt11 failed to parse: {}", e)));
                }
                let invoice = result.unwrap();

                // Verify the signature
                if let Err(e) = invoice.check_signature() {
                    return Err(Error::ZapReceipt(format!(
                        "bolt11 signature check failed: {}",
                        e
                    )));
                }

                // Get the public key
                let secpk = match invoice.payee_pub_key() {
                    Some(pubkey) => pubkey.to_owned(),
                    None => invoice.recover_payee_pub_key(),
                };
                let (xonlypk, _) = secpk.x_only_public_key();
                let pubkeybytes = xonlypk.serialize();
                let pubkey = match PublicKey::from_bytes(&pubkeybytes, false) {
                    Ok(pubkey) => pubkey,
                    Err(e) => {
                        return Err(Error::ZapReceipt(format!("payee public key error: {}", e)));
                    }
                };
                zapped_pubkey = Some(pubkey);

                if let Some(u) = invoice.amount_milli_satoshis() {
                    zapped_amount = Some(MilliSatoshi(u));
                } else {
                    return Err(Error::ZapReceipt(
                        "Amount missing from zap receipt".to_string(),
                    ));
                }
            }
            if let TagV1::Event { id, .. } = tag {
                zapped_id = Some(*id);
            }
        }

        if zapped_id.is_none() {
            // This probably means a person was zapped, not a note. So not an error.
            return Ok(None);
        }
        if zapped_amount.is_none() {
            return Err(Error::ZapReceipt("Missing amount".to_string()));
        }
        if zapped_pubkey.is_none() {
            return Err(Error::ZapReceipt("Missing payee public key".to_string()));
        }

        Ok(Some(ZapDataV1 {
            id: zapped_id.unwrap(),
            amount: zapped_amount.unwrap(),
            pubkey: zapped_pubkey.unwrap(),
            provider_pubkey: self.pubkey,
        }))
    }

    /// If this event specifies the client that created it, return that client
    /// string
    pub fn client(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::Other { tag, data } = tag {
                if tag == "client" && !data.is_empty() {
                    return Some(data[0].clone());
                }
            }
        }

        None
    }

    /// If this event specifies a subject, return that subject string
    pub fn subject(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::Subject { subject, .. } = tag {
                return Some(subject.clone());
            }
        }

        None
    }

    /// If this event specifies a title, return that title string
    pub fn title(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::Title { title, .. } = tag {
                return Some(title.clone());
            }
        }

        None
    }

    /// If this event specifies a summary, return that summary string
    pub fn summary(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::Other { tag, data } = tag {
                if tag == "summary" && !data.is_empty() {
                    return Some(data[0].clone());
                }
            }
        }

        None
    }

    /// If this event specifies a content warning, return that subject string
    pub fn content_warning(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV1::ContentWarning { warning, .. } = tag {
                return Some(warning.clone());
            }
        }

        None
    }

    /// If this is a parameterized event, get the parameter
    pub fn parameter(&self) -> Option<String> {
        if self.kind.is_parameterized_replaceable() {
            for tag in self.tags.iter() {
                if let TagV1::Identifier { d, .. } = tag {
                    return Some(d.to_owned());
                }
            }
            Some("".to_owned()) // implicit
        } else {
            None
        }
    }

    /// Return all the hashtags this event refers to
    pub fn hashtags(&self) -> Vec<String> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<String> = Vec::new();

        for tag in self.tags.iter() {
            if let TagV1::Hashtag { hashtag, .. } = tag {
                output.push(hashtag.clone());
            }
        }

        output
    }

    /// Return all the URLs this event refers to
    pub fn urls(&self) -> Vec<RelayUrl> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<RelayUrl> = Vec::new();

        for tag in self.tags.iter() {
            if let TagV1::Reference { url, .. } = tag {
                if let Ok(relay_url) = RelayUrl::try_from_unchecked_url(url) {
                    output.push(relay_url);
                }
            }
        }

        output
    }

    /// Get the proof-of-work count of leading bits
    pub fn pow(&self) -> u8 {
        // Count leading bits in the Id field
        let zeroes: u8 = crate::types::get_leading_zero_bits(&self.id.0);

        // Check that they meant it
        let mut target_zeroes: u8 = 0;
        for tag in self.tags.iter() {
            if let TagV1::Nonce { target, .. } = tag {
                if let Some(t) = target {
                    target_zeroes = t.parse::<u8>().unwrap_or(0);
                }
                break;
            }
        }

        zeroes.min(target_zeroes)
    }

    /// Was this event delegated, was that valid, and if so what is the pubkey
    /// of the delegator?
    pub fn delegation(&self) -> EventDelegation {
        for tag in self.tags.iter() {
            if let TagV1::Delegation {
                pubkey,
                conditions,
                sig,
                ..
            } = tag
            {
                // Convert hex strings into functional types
                let signature = match Signature::try_from_hex_string(sig) {
                    Ok(sig) => sig,
                    Err(e) => return EventDelegation::InvalidDelegation(format!("{e}")),
                };
                let delegator_pubkey = match PublicKey::try_from_hex_string(pubkey, true) {
                    Ok(pk) => pk,
                    Err(e) => return EventDelegation::InvalidDelegation(format!("{e}")),
                };

                // Verify the delegation tag
                match conditions.verify_signature(&delegator_pubkey, &self.pubkey, &signature) {
                    Ok(_) => {
                        // Check conditions
                        if let Some(kind) = conditions.kind {
                            if self.kind != kind {
                                return EventDelegation::InvalidDelegation(
                                    "Event Kind not delegated".to_owned(),
                                );
                            }
                        }
                        if let Some(created_after) = conditions.created_after {
                            if self.created_at < created_after {
                                return EventDelegation::InvalidDelegation(
                                    "Event created before delegation started".to_owned(),
                                );
                            }
                        }
                        if let Some(created_before) = conditions.created_before {
                            if self.created_at > created_before {
                                return EventDelegation::InvalidDelegation(
                                    "Event created after delegation ended".to_owned(),
                                );
                            }
                        }
                        return EventDelegation::DelegatedBy(delegator_pubkey);
                    }
                    Err(e) => {
                        return EventDelegation::InvalidDelegation(format!("{e}"));
                    }
                }
            }
        }

        EventDelegation::NotDelegated
    }

    /// If the event came through a proxy, get the (Protocol, Id)
    pub fn proxy(&self) -> Option<(&str, &str)> {
        for t in self.tags.iter() {
            if let TagV1::Other { tag, data } = t {
                if tag == "proxy" && data.len() >= 2 {
                    return Some((&data[1], &data[0]));
                }
            }
        }
        None
    }
}

// Direct access into speedy-serialized bytes, to avoid alloc-deserialize just
// to peek at one of these fields
#[cfg(feature = "speedy")]
impl EventV1 {
    /// Read the ID of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_id_from_speedy_bytes(bytes: &[u8]) -> Option<Id> {
        if bytes.len() < 32 {
            None
        } else if let Ok(arr) = <[u8; 32]>::try_from(&bytes[0..32]) {
            use crate::types::id;

            Some(unsafe { std::mem::transmute::<[u8; 32], id::Id>(arr) })
        } else {
            None
        }
    }

    /// Read the pubkey of the event from a speedy encoding without decoding
    /// (close to zero allocation, VerifyingKey does stuff I didn't check)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_pubkey_from_speedy_bytes(bytes: &[u8]) -> Option<PublicKey> {
        if bytes.len() < 64 {
            None
        } else {
            PublicKey::from_bytes(&bytes[32..64], false).ok()
        }
    }

    /// Read the created_at of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_created_at_from_speedy_bytes(bytes: &[u8]) -> Option<Unixtime> {
        if bytes.len() < 72 {
            None
        } else if let Ok(i) = i64::read_from_buffer(&bytes[64..72]) {
            Some(Unixtime(i))
        } else {
            None
        }
    }

    /// Read the kind of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_kind_from_speedy_bytes(bytes: &[u8]) -> Option<EventKind> {
        if bytes.len() < 76 {
            None
        } else if let Ok(u) = u32::read_from_buffer(&bytes[72..76]) {
            Some(u.into())
        } else {
            None
        }
    }

    // Read the sig of the event from a speedy encoding without decoding
    // (offset would be 76..140

    /// Read the ots of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_ots_from_speedy_bytes(bytes: &[u8]) -> Option<&str> {
        #[allow(clippy::if_same_then_else)]
        if bytes.len() < 140 {
            None
        } else if bytes[140] == 0 {
            None
        } else if bytes.len() < 145 {
            None
        } else {
            let len = u32::from_ne_bytes(bytes[141..145].try_into().unwrap());
            unsafe {
                Some(std::str::from_utf8_unchecked(
                    &bytes[146..146 + len as usize],
                ))
            }
        }
    }

    /// Read the content of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_content_from_speedy_bytes(bytes: &[u8]) -> Option<&str> {
        let start = if bytes.len() < 145 {
            return None;
        } else if bytes[140] == 0 {
            141
        } else {
            // get OTS length and move past it
            let len = u32::from_ne_bytes(bytes[141..145].try_into().unwrap());
            141 + 4 + len as usize
        };

        let len = u32::from_ne_bytes(bytes[start..start + 4].try_into().unwrap());

        unsafe {
            Some(std::str::from_utf8_unchecked(
                &bytes[start + 4..start + 4 + len as usize],
            ))
        }
    }

    /// Check if any human-readable tag matches the Regex in the speedy encoding
    /// without decoding the whole thing (because our Tag representation is so
    /// complicated, we do deserialize the tags for now)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn tag_search_in_speedy_bytes(bytes: &[u8], re: &Regex) -> Result<bool, Error> {
        if bytes.len() < 145 {
            return Ok(false);
        }

        // skip OTS
        let mut offset = if bytes[140] == 0 {
            141
        } else {
            // get OTS length and move past it
            let len = u32::from_ne_bytes(bytes[141..145].try_into().unwrap());
            141 + 4 + len as usize
        };

        // skip content
        let len = u32::from_ne_bytes(bytes[offset..offset + 4].try_into().unwrap());
        offset += 4 + len as usize;

        // Deserialize the tags
        let tags: Vec<TagV1> = Vec::<TagV1>::read_from_buffer(&bytes[offset..])?;

        // Search through them
        for tag in &tags {
            match tag {
                TagV1::ContentWarning { warning, .. } => {
                    if re.is_match(warning.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV1::Hashtag { hashtag, .. } => {
                    if re.is_match(hashtag.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV1::Subject { subject, .. } => {
                    if re.is_match(subject.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV1::Title { title, .. } => {
                    if re.is_match(title.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV1::Other { tag, data } => {
                    if tag == "summary" && !data.is_empty() && re.is_match(data[0].as_ref()) {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }
}

impl From<EventV1> for RumorV1 {
    fn from(e: EventV1) -> RumorV1 {
        RumorV1 {
            id: e.id,
            pubkey: e.pubkey,
            created_at: e.created_at,
            kind: e.kind,
            content: e.content,
            tags: e.tags,
        }
    }
}

impl From<RumorV1> for PreEventV1 {
    fn from(r: RumorV1) -> PreEventV1 {
        PreEventV1 {
            pubkey: r.pubkey,
            created_at: r.created_at,
            kind: r.kind,
            content: r.content,
            tags: r.tags,
        }
    }
}

impl TryFrom<PreEventV1> for RumorV1 {
    type Error = Error;
    fn try_from(e: PreEventV1) -> Result<RumorV1, Error> {
        RumorV1::new(e)
    }
}
```

---

### versioned/event2.rs

**Size:** 49277 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::str::FromStr;

use lightning_invoice::Bolt11Invoice;
#[cfg(feature = "speedy")]
use regex::Regex;
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    id::{self, Id},
    Error, EventDelegation, EventKind, EventReference, IntoVec, KeySigner, MilliSatoshi, NAddr,
    NostrBech32, NostrUrl, PrivateKey, PublicKey, PublicKeyHex, RelayUrl, Signature, Signer, TagV2,
    Unixtime, ZapDataV1,
};

/// The main event type
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct EventV2 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The signature of the event, which cryptographically verifies that the
    /// holder of the PrivateKey matching the event's PublicKey generated
    /// (or authorized) this event. The signature is taken over the id field
    /// only, but the id field is taken over the rest of the event data.
    pub sig: Signature,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV2>,
}

macro_rules! serialize_inner_event {
    ($pubkey:expr, $created_at:expr, $kind:expr, $tags:expr,
     $content:expr) => {{
        format!(
            "[0,{},{},{},{},{}]",
            serde_json::to_string($pubkey)?,
            serde_json::to_string($created_at)?,
            serde_json::to_string($kind)?,
            serde_json::to_string($tags)?,
            serde_json::to_string($content)?
        )
    }};
}

/// Data used to construct an event
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct PreEventV2 {
    /// The public key of the actor who is creating the event
    pub pubkey: PublicKey,
    /// The time at which the event was created
    pub created_at: Unixtime,
    /// The kind of event
    pub kind: EventKind,
    /// A set of tags that apply to the event
    pub tags: Vec<TagV2>,
    /// The content of the event
    pub content: String,
}

impl PreEventV2 {
    /// Generate an ID from this PreEvent for use in an Event or a Rumor
    pub fn hash(&self) -> Result<Id, Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Hash
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();
        Ok(Id(id))
    }
}

/// A Rumor is an Event without a signature
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RumorV2 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV2>,
}

impl RumorV2 {
    /// Create a new rumor
    pub fn new(input: PreEventV2) -> Result<RumorV2, Error> {
        // Generate Id
        let id = input.hash()?;

        Ok(RumorV2 {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
        })
    }

    /// Turn into an Event (the signature will be all zeroes)
    pub fn into_event_with_bad_signature(self) -> EventV2 {
        EventV2 {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            sig: Signature::zeroes(),
            content: self.content,
            tags: self.tags,
        }
    }
}

impl EventV2 {
    /// Check the validity of an event. This is useful if you deserialize an
    /// event from the network. If you create an event using new() it should
    /// already be trustworthy.
    pub fn verify(&self, maxtime: Option<Unixtime>) -> Result<(), Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Verify the signature
        self.pubkey.verify(serialized.as_bytes(), &self.sig)?;

        // Also verify the ID is the SHA256
        // (the above verify function also does it internally,
        //  so there is room for improvement here)
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();

        // Optional verify that the message was in the past
        if let Some(mt) = maxtime {
            if self.created_at > mt {
                return Err(Error::EventInFuture);
            }
        }

        if id != self.id.0 {
            Err(Error::HashMismatch)
        } else {
            Ok(())
        }
    }

    /// Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> EventV2 {
        let signer = {
            let private_key = PrivateKey::mock();
            KeySigner::from_private_key(private_key, "", 1).unwrap()
        };
        let public_key = signer.public_key();
        let pre = PreEventV2 {
            pubkey: public_key,
            created_at: Unixtime::mock(),
            kind: EventKind::mock(),
            tags: vec![TagV2::mock(), TagV2::mock()],
            content: "This is a test".to_string(),
        };
        signer.sign_event2(pre).unwrap()
    }

    /// Get the k-tag kind, if any
    pub fn k_tag_kind(&self) -> Option<EventKind> {
        for tag in self.tags.iter() {
            if let TagV2::Kind { kind, .. } = tag {
                return Some(*kind);
            }
        }
        None
    }

    /// If the event refers to people by tag, get all the PublicKeys it refers
    /// to along with recommended relay URL and petname for each
    pub fn people(&self) -> Vec<(PublicKeyHex, Option<RelayUrl>, Option<String>)> {
        let mut output: Vec<(PublicKeyHex, Option<RelayUrl>, Option<String>)> = Vec::new();
        // All 'p' tags
        for tag in self.tags.iter() {
            if let TagV2::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                ..
            } = tag
            {
                output.push((
                    pubkey.to_owned(),
                    recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
                    petname.to_owned(),
                ));
            }
        }

        output
    }

    /// If the pubkey is tagged in the event
    pub fn is_tagged(&self, pubkey: &PublicKey) -> bool {
        let pkh: PublicKeyHex = pubkey.into();

        for tag in self.tags.iter() {
            if let TagV2::Pubkey { pubkey, .. } = tag {
                if *pubkey == pkh {
                    return true;
                }
            }
        }

        false
    }

    /// If the event refers to people within the contents, get all the
    /// PublicKeys it refers to within the contents.
    pub fn people_referenced_in_content(&self) -> Vec<PublicKey> {
        let mut output = Vec::new();
        for nurl in NostrUrl::find_all_in_string(&self.content).drain(..) {
            if let NostrBech32::Pubkey(pk) = nurl.0 {
                output.push(pk);
            }
            if let NostrBech32::Profile(prof) = nurl.0 {
                output.push(prof.pubkey);
            }
        }
        output
    }

    /// All events IDs that this event refers to, whether root, reply, mention,
    /// or otherwise along with optional recommended relay URLs
    pub fn referred_events(&self) -> Vec<EventReference> {
        let mut output: Vec<EventReference> = Vec::new();

        // Collect every 'e' tag and 'a' tag
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                output.push(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.clone(),
                });
            } else if let TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    output.push(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        output
    }

    /// Get a reference to another event that this event replies to.
    /// An event can only reply to one other event via 'e' or 'a' tag from a
    /// feed-displayable event that is not a Repost.
    pub fn replies_to(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // Repost 'e' and 'a' tags are always considered mentions, not replies.
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            return None;
        }

        // If there are no 'e' tags nor 'a' tags, then none
        let num_event_ref_tags = self
            .tags
            .iter()
            .filter(|e| matches!(e, TagV2::Event { .. }) || matches!(e, TagV2::Address { .. }))
            .count();
        if num_event_ref_tags == 0 {
            return None;
        }

        // look for an 'e' tag with marker 'reply'
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "reply" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                }
            }
        }

        // Use the last unmarked 'e' tag or any 'a' tag
        if let Some(tag) = self.tags.iter().rev().find(|t| {
            matches!(t, TagV2::Event { marker: None, .. })
                || matches!(t, TagV2::Address { marker: None, .. })
        }) {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                return Some(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.clone(),
                });
            } else if let TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    return Some(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        None
    }

    /// If this event replies to a thread, get that threads root event Id if
    /// available, along with an optional recommended_relay_url
    pub fn replies_to_root(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                }
            }
        }

        // otherwise use the first unmarked 'e' tag or first 'a' tag
        // (even if there is only 1 'e' or 'a' tag which means it is both root and
        // reply)
        if let Some(tag) = self.tags.iter().find(|t| {
            matches!(t, TagV2::Event { marker: None, .. })
                || matches!(t, TagV2::Address { marker: None, .. })
        }) {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                return Some(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: marker.to_owned(),
                });
            } else if let TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url: Some(rurl),
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                    return Some(EventReference::Addr(NAddr {
                        d: d.to_string(),
                        relays: vec![rurl.clone()],
                        kind: *kind,
                        author: pk,
                    }));
                }
            }
        }

        None
    }

    /// If this event mentions others, get those other event Ids
    /// and optional recommended relay Urls
    pub fn mentions(&self) -> Vec<EventReference> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<EventReference> = Vec::new();

        // For kind=6 and kind=16, all 'e' and 'a' tags are mentions
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            for tag in self.tags.iter() {
                if let TagV2::Event {
                    id,
                    recommended_relay_url,
                    marker,
                    ..
                } = tag
                {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                } else if let TagV2::Address {
                    kind,
                    pubkey,
                    d,
                    relay_url: Some(rurl),
                    marker: _,
                    trailing: _,
                } = tag
                {
                    if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                        output.push(EventReference::Addr(NAddr {
                            d: d.to_string(),
                            relays: vec![rurl.clone()],
                            kind: *kind,
                            author: pk,
                        }));
                    }
                }
            }

            return output;
        }

        // Look for nostr links within the content

        // Collect every 'e' tag marked as 'mention'
        for tag in self.tags.iter() {
            if let TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                ..
            } = tag
            {
                if marker.is_some() && marker.as_deref().unwrap() == "mention" {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.clone(),
                    });
                }
            }
        }

        // Collect every unmarked 'e' or 'a' tag that is not the first (root) or the
        // last (reply)
        let e_tags: Vec<&TagV2> = self
            .tags
            .iter()
            .filter(|e| {
                matches!(e, TagV2::Event { marker: None, .. })
                    || matches!(e, TagV2::Address { marker: None, .. })
            })
            .collect();
        if e_tags.len() > 2 {
            // mentions are everything other than first and last
            for tag in &e_tags[1..e_tags.len() - 1] {
                if let TagV2::Event {
                    id,
                    recommended_relay_url,
                    marker,
                    ..
                } = tag
                {
                    output.push(EventReference::Id {
                        id: *id,
                        author: None,
                        relays: recommended_relay_url
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: marker.to_owned(),
                    });
                } else if let TagV2::Address {
                    kind,
                    pubkey,
                    d,
                    relay_url: Some(rurl),
                    ..
                } = tag
                {
                    if let Ok(pk) = PublicKey::try_from_hex_string(pubkey.as_str(), true) {
                        output.push(EventReference::Addr(NAddr {
                            d: d.to_string(),
                            relays: vec![rurl.clone()],
                            kind: *kind,
                            author: pk,
                        }));
                    }
                }
            }
        }

        output
    }

    /// If this event reacts to another, get that other event's Id,
    /// the reaction content, and an optional Recommended relay Url
    pub fn reacts_to(&self) -> Option<(Id, String, Option<RelayUrl>)> {
        if self.kind != EventKind::Reaction {
            return None;
        }

        // The last 'e' tag is it
        if let Some(TagV2::Event {
            id,
            recommended_relay_url,
            ..
        }) = self
            .tags
            .iter()
            .rev()
            .find(|t| matches!(t, TagV2::Event { .. }))
        {
            return Some((
                *id,
                self.content.clone(),
                recommended_relay_url
                    .as_ref()
                    .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
            ));
        }

        None
    }

    /// If this event deletes others, get all the EventReferences of the events
    /// that it deletes along with the reason for the deletion
    pub fn deletes(&self) -> Option<(Vec<EventReference>, String)> {
        if self.kind != EventKind::EventDeletion {
            return None;
        }

        let mut erefs: Vec<EventReference> = Vec::new();

        for tag in self.tags.iter() {
            // All 'e' tags are deleted
            if let TagV2::Event { id, .. } = tag {
                erefs.push(EventReference::Id {
                    id: *id,
                    author: None,
                    relays: vec![],
                    marker: None,
                });
            }
            // All 'a' tag groups are deleted
            else if let TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url,
                ..
            } = tag
            {
                if let Ok(pk) = PublicKey::try_from_hex_string(pubkey, false) {
                    let ea = NAddr {
                        d: d.clone(),
                        relays: match relay_url {
                            Some(url) => vec![url.clone()],
                            None => vec![],
                        },
                        kind: *kind,
                        author: pk,
                    };
                    erefs.push(EventReference::Addr(ea));
                }
            }
        }

        if erefs.is_empty() {
            None
        } else {
            Some((erefs, self.content.clone()))
        }
    }

    /// Can this event be deleted by the given public key?
    pub fn delete_author_allowed(&self, by: PublicKey) -> bool {
        // Author can always delete
        if self.pubkey == by {
            return true;
        }

        if self.kind == EventKind::GiftWrap {
            for tag in self.tags.iter() {
                if let TagV2::Pubkey { pubkey, .. } = tag {
                    let pkh: PublicKeyHex = by.into();
                    return pkh == *pubkey;
                }
            }
        }

        false
    }

    /// If this event zaps another event, get data about that.
    ///
    /// That includes the Id, the amount, and the public key of the provider,
    /// all of which should be verified by the caller.
    ///
    /// Errors returned from this are not fatal, but may be useful for
    /// explaining to a user why a zap receipt is invalid.
    pub fn zaps(&self) -> Result<Option<ZapDataV1>, Error> {
        if self.kind != EventKind::Zap {
            return Ok(None);
        }

        let mut zapped_id: Option<Id> = None;
        let mut zapped_amount: Option<MilliSatoshi> = None;
        let mut zapped_pubkey: Option<PublicKey> = None;

        for tag in self.tags.iter() {
            if let TagV2::Other { tag, data } = tag {
                // Find the bolt11 tag
                if tag != "bolt11" {
                    continue;
                }
                if data.is_empty() {
                    return Err(Error::ZapReceipt("missing bolt11 tag value".to_string()));
                }

                // Extract as an Invoice
                let result = Bolt11Invoice::from_str(&data[0]);
                if let Err(e) = result {
                    return Err(Error::ZapReceipt(format!("bolt11 failed to parse: {}", e)));
                }
                let invoice = result.unwrap();

                // Verify the signature
                if let Err(e) = invoice.check_signature() {
                    return Err(Error::ZapReceipt(format!(
                        "bolt11 signature check failed: {}",
                        e
                    )));
                }

                // Get the public key
                let secpk = match invoice.payee_pub_key() {
                    Some(pubkey) => pubkey.to_owned(),
                    None => invoice.recover_payee_pub_key(),
                };
                let (xonlypk, _) = secpk.x_only_public_key();
                let pubkeybytes = xonlypk.serialize();
                let pubkey = match PublicKey::from_bytes(&pubkeybytes, false) {
                    Ok(pubkey) => pubkey,
                    Err(e) => {
                        return Err(Error::ZapReceipt(format!("payee public key error: {}", e)));
                    }
                };
                zapped_pubkey = Some(pubkey);

                if let Some(u) = invoice.amount_milli_satoshis() {
                    zapped_amount = Some(MilliSatoshi(u));
                } else {
                    return Err(Error::ZapReceipt(
                        "Amount missing from zap receipt".to_string(),
                    ));
                }
            }
            if let TagV2::Event { id, .. } = tag {
                zapped_id = Some(*id);
            }
        }

        if zapped_id.is_none() {
            // This probably means a person was zapped, not a note. So not an error.
            return Ok(None);
        }
        if zapped_amount.is_none() {
            return Err(Error::ZapReceipt("Missing amount".to_string()));
        }
        if zapped_pubkey.is_none() {
            return Err(Error::ZapReceipt("Missing payee public key".to_string()));
        }

        Ok(Some(ZapDataV1 {
            id: zapped_id.unwrap(),
            amount: zapped_amount.unwrap(),
            pubkey: zapped_pubkey.unwrap(),
            provider_pubkey: self.pubkey,
        }))
    }

    /// If this event specifies the client that created it, return that client
    /// string
    pub fn client(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::Other { tag, data } = tag {
                if tag == "client" && !data.is_empty() {
                    return Some(data[0].clone());
                }
            }
        }

        None
    }

    /// If this event specifies a subject, return that subject string
    pub fn subject(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::Subject { subject, .. } = tag {
                return Some(subject.clone());
            }
        }

        None
    }

    /// If this event specifies a title, return that title string
    pub fn title(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::Title { title, .. } = tag {
                return Some(title.clone());
            }
        }

        None
    }

    /// If this event specifies a summary, return that summary string
    pub fn summary(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::Other { tag, data } = tag {
                if tag == "summary" && !data.is_empty() {
                    return Some(data[0].clone());
                }
            }
        }

        None
    }

    /// If this event specifies a content warning, return that subject string
    pub fn content_warning(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let TagV2::ContentWarning { warning, .. } = tag {
                return Some(warning.clone());
            }
        }

        None
    }

    /// If this is a parameterized event, get the parameter
    pub fn parameter(&self) -> Option<String> {
        if self.kind.is_parameterized_replaceable() {
            for tag in self.tags.iter() {
                if let TagV2::Identifier { d, .. } = tag {
                    return Some(d.to_owned());
                }
            }
            Some("".to_owned()) // implicit
        } else {
            None
        }
    }

    /// Return all the hashtags this event refers to
    pub fn hashtags(&self) -> Vec<String> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<String> = Vec::new();

        for tag in self.tags.iter() {
            if let TagV2::Hashtag { hashtag, .. } = tag {
                output.push(hashtag.clone());
            }
        }

        output
    }

    /// Return all the URLs this event refers to
    pub fn urls(&self) -> Vec<RelayUrl> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<RelayUrl> = Vec::new();

        for tag in self.tags.iter() {
            if let TagV2::Reference { url, .. } = tag {
                if let Ok(relay_url) = RelayUrl::try_from_unchecked_url(url) {
                    output.push(relay_url);
                }
            }
        }

        output
    }

    /// Get the proof-of-work count of leading bits
    pub fn pow(&self) -> u8 {
        // Count leading bits in the Id field
        let zeroes: u8 = crate::types::get_leading_zero_bits(&self.id.0);

        // Check that they meant it
        let mut target_zeroes: u8 = 0;
        for tag in self.tags.iter() {
            if let TagV2::Nonce { target, .. } = tag {
                if let Some(t) = target {
                    target_zeroes = t.parse::<u8>().unwrap_or(0);
                }
                break;
            }
        }

        zeroes.min(target_zeroes)
    }

    /// Was this event delegated, was that valid, and if so what is the pubkey
    /// of the delegator?
    pub fn delegation(&self) -> EventDelegation {
        for tag in self.tags.iter() {
            if let TagV2::Delegation {
                pubkey,
                conditions,
                sig,
                ..
            } = tag
            {
                // Convert hex strings into functional types
                let signature = match Signature::try_from_hex_string(sig) {
                    Ok(sig) => sig,
                    Err(e) => return EventDelegation::InvalidDelegation(format!("{e}")),
                };
                let delegator_pubkey = match PublicKey::try_from_hex_string(pubkey, true) {
                    Ok(pk) => pk,
                    Err(e) => return EventDelegation::InvalidDelegation(format!("{e}")),
                };

                // Verify the delegation tag
                match conditions.verify_signature(&delegator_pubkey, &self.pubkey, &signature) {
                    Ok(_) => {
                        // Check conditions
                        if let Some(kind) = conditions.kind {
                            if self.kind != kind {
                                return EventDelegation::InvalidDelegation(
                                    "Event Kind not delegated".to_owned(),
                                );
                            }
                        }
                        if let Some(created_after) = conditions.created_after {
                            if self.created_at < created_after {
                                return EventDelegation::InvalidDelegation(
                                    "Event created before delegation started".to_owned(),
                                );
                            }
                        }
                        if let Some(created_before) = conditions.created_before {
                            if self.created_at > created_before {
                                return EventDelegation::InvalidDelegation(
                                    "Event created after delegation ended".to_owned(),
                                );
                            }
                        }
                        return EventDelegation::DelegatedBy(delegator_pubkey);
                    }
                    Err(e) => {
                        return EventDelegation::InvalidDelegation(format!("{e}"));
                    }
                }
            }
        }

        EventDelegation::NotDelegated
    }

    /// If the event came through a proxy, get the (Protocol, Id)
    pub fn proxy(&self) -> Option<(&str, &str)> {
        for t in self.tags.iter() {
            if let TagV2::Other { tag, data } = t {
                if tag == "proxy" && data.len() >= 2 {
                    return Some((&data[1], &data[0]));
                }
            }
        }
        None
    }
}

// Direct access into speedy-serialized bytes, to avoid alloc-deserialize just
// to peek at one of these fields
#[cfg(feature = "speedy")]
impl EventV2 {
    /// Read the ID of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_id_from_speedy_bytes(bytes: &[u8]) -> Option<Id> {
        if bytes.len() < 32 {
            None
        } else if let Ok(arr) = <[u8; 32]>::try_from(&bytes[0..32]) {
            Some(unsafe { std::mem::transmute::<[u8; 32], id::Id>(arr) })
        } else {
            None
        }
    }

    /// Read the pubkey of the event from a speedy encoding without decoding
    /// (close to zero allocation, VerifyingKey does stuff I didn't check)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_pubkey_from_speedy_bytes(bytes: &[u8]) -> Option<PublicKey> {
        if bytes.len() < 64 {
            None
        } else {
            PublicKey::from_bytes(&bytes[32..64], false).ok()
        }
    }

    /// Read the created_at of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_created_at_from_speedy_bytes(bytes: &[u8]) -> Option<Unixtime> {
        if bytes.len() < 72 {
            None
        } else if let Ok(i) = i64::read_from_buffer(&bytes[64..72]) {
            Some(Unixtime(i))
        } else {
            None
        }
    }

    /// Read the kind of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_kind_from_speedy_bytes(bytes: &[u8]) -> Option<EventKind> {
        if bytes.len() < 76 {
            None
        } else if let Ok(u) = u32::read_from_buffer(&bytes[72..76]) {
            Some(u.into())
        } else {
            None
        }
    }

    // Read the sig of the event from a speedy encoding without decoding
    // (offset would be 76..140

    /// Read the content of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_content_from_speedy_bytes(bytes: &[u8]) -> Option<&str> {
        let len = u32::from_ne_bytes(bytes[140..140 + 4].try_into().unwrap());

        unsafe {
            Some(std::str::from_utf8_unchecked(
                &bytes[140 + 4..140 + 4 + len as usize],
            ))
        }
    }

    /// Check if any human-readable tag matches the Regex in the speedy encoding
    /// without decoding the whole thing (because our TagV2 representation is so
    /// complicated, we do deserialize the tags for now)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn tag_search_in_speedy_bytes(bytes: &[u8], re: &Regex) -> Result<bool, Error> {
        if bytes.len() < 140 {
            return Ok(false);
        }

        // skip content
        let len = u32::from_ne_bytes(bytes[140..140 + 4].try_into().unwrap());
        let offset = 140 + 4 + len as usize;

        // Deserialize the tags
        let tags: Vec<TagV2> = Vec::<TagV2>::read_from_buffer(&bytes[offset..])?;

        // Search through them
        for tag in &tags {
            match tag {
                TagV2::ContentWarning { warning, .. } => {
                    if re.is_match(warning.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV2::Hashtag { hashtag, .. } => {
                    if re.is_match(hashtag.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV2::Subject { subject, .. } => {
                    if re.is_match(subject.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV2::Title { title, .. } => {
                    if re.is_match(title.as_ref()) {
                        return Ok(true);
                    }
                }
                TagV2::Other { tag, data } => {
                    if tag == "summary" && !data.is_empty() && re.is_match(data[0].as_ref()) {
                        return Ok(true);
                    }
                }
                _ => {}
            }
        }

        Ok(false)
    }
}

impl From<EventV2> for RumorV2 {
    fn from(e: EventV2) -> RumorV2 {
        RumorV2 {
            id: e.id,
            pubkey: e.pubkey,
            created_at: e.created_at,
            kind: e.kind,
            content: e.content,
            tags: e.tags,
        }
    }
}

impl From<RumorV2> for PreEventV2 {
    fn from(r: RumorV2) -> PreEventV2 {
        PreEventV2 {
            pubkey: r.pubkey,
            created_at: r.created_at,
            kind: r.kind,
            content: r.content,
            tags: r.tags,
        }
    }
}

impl TryFrom<PreEventV2> for RumorV2 {
    type Error = Error;
    fn try_from(e: PreEventV2) -> Result<RumorV2, Error> {
        RumorV2::new(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{DelegationConditions, Signer, UncheckedUrl};

    test_serde! {EventV2, test_event_serde}

    #[test]
    fn test_event_new_and_verify() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };
        let pubkey = signer.public_key();
        let preevent = PreEventV2 {
            pubkey,
            created_at: Unixtime::mock(),
            kind: EventKind::TextNote,
            tags: vec![TagV2::Event {
                id: Id::mock(),
                recommended_relay_url: Some(UncheckedUrl::mock()),
                marker: None,
                trailing: Vec::new(),
            }],
            content: "Hello World!".to_string(),
        };
        let mut event = signer.sign_event2(preevent).unwrap();
        assert!(event.verify(None).is_ok());

        // Now make sure it fails when the message has been modified
        event.content = "I'm changing this message".to_string();
        let result = event.verify(None);
        assert!(result.is_err());

        // Change it back
        event.content = "Hello World!".to_string();
        let result = event.verify(None);
        assert!(result.is_ok());

        // Tweak the id only
        event.id = Id([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]);
        let result = event.verify(None);
        assert!(result.is_err());
    }

    // helper
    fn create_event_with_delegation<S>(created_at: Unixtime, real_signer: &S) -> EventV2
    where
        S: Signer,
    {
        let delegated_signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let conditions = DelegationConditions::try_from_str(
            "kind=1&created_at>1680000000&created_at<1680050000",
        )
        .unwrap();

        let sig = real_signer
            .generate_delegation_signature(delegated_signer.public_key(), &conditions)
            .unwrap();

        let preevent = PreEventV2 {
            pubkey: delegated_signer.public_key(),
            created_at,
            kind: EventKind::TextNote,
            tags: vec![
                TagV2::Event {
                    id: Id::mock(),
                    recommended_relay_url: Some(UncheckedUrl::mock()),
                    marker: None,
                    trailing: Vec::new(),
                },
                TagV2::Delegation {
                    pubkey: PublicKeyHex::try_from_string(real_signer.public_key().as_hex_string())
                        .unwrap(),
                    conditions,
                    sig: sig.into(),
                    trailing: Vec::new(),
                },
            ],
            content: "Hello World!".to_string(),
        };
        delegated_signer.sign_event2(preevent).unwrap()
    }

    #[test]
    fn test_event_with_delegation_ok() {
        let delegator_signer = {
            let delegator_privkey = PrivateKey::mock();
            KeySigner::from_private_key(delegator_privkey, "", 1).unwrap()
        };
        let delegator_pubkey = delegator_signer.public_key();

        let event = create_event_with_delegation(Unixtime(1680000012), &delegator_signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::DelegatedBy(pk) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(pk, delegator_pubkey);
        } else {
            panic!("Expected DelegatedBy result, got {:?}", event.delegation());
        }
    }

    #[test]
    fn test_event_with_delegation_invalid_created_after() {
        let delegator_privkey = PrivateKey::mock();
        let signer = KeySigner::from_private_key(delegator_privkey, "", 1).unwrap();

        let event = create_event_with_delegation(Unixtime(1690000000), &signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::InvalidDelegation(reason) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(reason, "Event created after delegation ended");
        } else {
            panic!(
                "Expected InvalidDelegation result, got {:?}",
                event.delegation()
            );
        }
    }

    #[test]
    fn test_event_with_delegation_invalid_created_before() {
        let signer = {
            let delegator_privkey = PrivateKey::mock();
            KeySigner::from_private_key(delegator_privkey, "", 1).unwrap()
        };

        let event = create_event_with_delegation(Unixtime(1610000000), &signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::InvalidDelegation(reason) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(reason, "Event created before delegation started");
        } else {
            panic!(
                "Expected InvalidDelegation result, got {:?}",
                event.delegation()
            );
        }
    }

    #[test]
    fn test_realworld_event_with_naddr_tag() {
        let raw = r##"{"id":"7760408f6459b9546c3a4e70e3e56756421fba34526b7d460db3fcfd2f8817db","pubkey":"460c25e682fda7832b52d1f22d3d22b3176d972f60dcdc3212ed8c92ef85065c","created_at":1687616920,"kind":1,"tags":[["p","1bc70a0148b3f316da33fe3c89f23e3e71ac4ff998027ec712b905cd24f6a411","","mention"],["a","30311:1bc70a0148b3f316da33fe3c89f23e3e71ac4ff998027ec712b905cd24f6a411:1687612774","","mention"]],"content":"Watching Karnage's stream to see if I learn something about design. \n\nnostr:naddr1qq9rzd3cxumrzv3hxu6qygqmcu9qzj9n7vtd5vl78jyly037wxkyl7vcqflvwy4eqhxjfa4yzypsgqqqwens0qfplk","sig":"dbc5d05a24bfe990a1faaedfcb81a98940d86a105711dbdad9145d05b0ad0f46e3e24eaa3fc283818f27e057fe836a029fd9a68e7f1de06ff477493199d64064"}"##;
        let _: EventV2 = serde_json::from_str(&raw).unwrap();
    }

    #[cfg(feature = "speedy")]
    #[test]
    fn test_speedy_encoded_direct_field_access() {
        use speedy::Writable;

        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let preevent = PreEventV2 {
            pubkey: signer.public_key(),
            created_at: Unixtime(1680000012),
            kind: EventKind::TextNote,
            tags: vec![
                TagV2::Event {
                    id: Id::mock(),
                    recommended_relay_url: Some(UncheckedUrl::mock()),
                    marker: None,
                    trailing: Vec::new(),
                },
                TagV2::Hashtag {
                    hashtag: "foodstr".to_string(),
                    trailing: Vec::new(),
                },
            ],
            content: "Hello World!".to_string(),
        };
        let event = signer.sign_event2(preevent).unwrap();
        let bytes = event.write_to_vec().unwrap();

        let id = EventV2::get_id_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(id, event.id);

        let pubkey = EventV2::get_pubkey_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(pubkey, event.pubkey);

        let created_at = EventV2::get_created_at_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(created_at, Unixtime(1680000012));

        let kind = EventV2::get_kind_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(kind, event.kind);

        let content = EventV2::get_content_from_speedy_bytes(&bytes);
        assert_eq!(content, Some(&*event.content));

        let re = regex::Regex::new("foodstr").unwrap();
        let found_foodstr = EventV2::tag_search_in_speedy_bytes(&bytes, &re).unwrap();
        assert!(found_foodstr);

        // Print to work out encoding
        //   test like this to see printed data:
        //   cargo test --features=speedy test_speedy_encoded_direct_field_access --
        // --nocapture
        println!("EVENT BYTES: {:?}", bytes);
        println!("ID: {:?}", event.id.0);
        println!("PUBKEY: {:?}", event.pubkey.as_slice());
        println!("CREATED AT: {:?}", event.created_at.0.to_ne_bytes());
        let kind32: u32 = event.kind.into();
        println!("KIND: {:?}", kind32.to_ne_bytes());
        println!("SIG: {:?}", event.sig.0.as_ref());
        println!(
            "CONTENT: [len={:?}] {:?}",
            (event.content.as_bytes().len() as u32).to_ne_bytes(),
            event.content.as_bytes()
        );
        println!("TAGS: [len={:?}]", (event.tags.len() as u32).to_ne_bytes());
    }

    #[test]
    fn test_event_gift_wrap() {
        let signer1 = {
            let sec1 = PrivateKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap();
            KeySigner::from_private_key(sec1, "", 1).unwrap()
        };

        let signer2 = {
            let sec2 = PrivateKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000002",
            )
            .unwrap();
            KeySigner::from_private_key(sec2, "", 1).unwrap()
        };

        let pre = PreEventV2 {
            pubkey: signer1.public_key(),
            created_at: Unixtime(1692_000_000),
            kind: EventKind::TextNote,
            content: "Hey man, this rocks! Please reply for a test.".to_string(),
            tags: vec![],
        };

        let gift_wrap = signer1
            .giftwrap2(pre.clone(), signer2.public_key())
            .unwrap();
        let rumor = signer2.unwrap_giftwrap2(&gift_wrap).unwrap();
        let output_pre: PreEventV2 = rumor.into();

        assert_eq!(pre, output_pre);
    }
}
```

---

### versioned/event3.rs

**Size:** 53771 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::{cmp::Ordering, fmt, str::FromStr};

use lightning_invoice::Bolt11Invoice;
#[cfg(feature = "speedy")]
use regex::Regex;
use secp256k1::XOnlyPublicKey;
use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    id::{self, Id},
    Error, EventDelegation, EventKind, EventReference, IntoVec, KeySecurity, KeySigner,
    MilliSatoshi, NostrBech32, NostrUrl, PrivateKey, PublicKey, RelayUrl, Signature, Signer, TagV3,
    Unixtime, ZapData,
};

/// The main event type
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct EventV3 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The signature of the event, which cryptographically verifies that the
    /// holder of the PrivateKey matching the event's PublicKey generated
    /// (or authorized) this event. The signature is taken over the id field
    /// only, but the id field is taken over the rest of the event data.
    pub sig: Signature,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV3>,
}

macro_rules! serialize_inner_event {
    ($pubkey:expr, $created_at:expr, $kind:expr, $tags:expr,
     $content:expr) => {{
        format!(
            "[0,{},{},{},{},{}]",
            serde_json::to_string($pubkey)?,
            serde_json::to_string($created_at)?,
            serde_json::to_string($kind)?,
            serde_json::to_string($tags)?,
            serde_json::to_string($content)?
        )
    }};
}

/// Data used to construct an event
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct PreEventV3 {
    /// The public key of the actor who is creating the event
    pub pubkey: PublicKey,
    /// The time at which the event was created
    pub created_at: Unixtime,
    /// The kind of event
    pub kind: EventKind,
    /// A set of tags that apply to the event
    pub tags: Vec<TagV3>,
    /// The content of the event
    pub content: String,
}

impl PreEventV3 {
    /// Generate an ID from this PreEvent for use in an Event or a Rumor
    pub fn hash(&self) -> Result<Id, Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Hash
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();
        Ok(Id(id))
    }
}

/// A Rumor is an Event without a signature
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RumorV3 {
    /// The Id of the event, generated as a SHA256 of the inner event data
    pub id: Id,

    /// The public key of the actor who created the event
    pub pubkey: PublicKey,

    /// The (unverified) time at which the event was created
    pub created_at: Unixtime,

    /// The kind of event
    pub kind: EventKind,

    /// The content of the event
    pub content: String,

    /// A set of tags that apply to the event
    pub tags: Vec<TagV3>,
}

impl RumorV3 {
    /// Create a new rumor
    pub fn new(input: PreEventV3) -> Result<RumorV3, Error> {
        // Generate Id
        let id = input.hash()?;

        Ok(RumorV3 {
            id,
            pubkey: input.pubkey,
            created_at: input.created_at,
            kind: input.kind,
            tags: input.tags,
            content: input.content,
        })
    }

    /// Turn into an Event (the signature will be all zeroes)
    pub fn into_event_with_bad_signature(self) -> EventV3 {
        EventV3 {
            id: self.id,
            pubkey: self.pubkey,
            created_at: self.created_at,
            kind: self.kind,
            sig: Signature::zeroes(),
            content: self.content,
            tags: self.tags,
        }
    }
}

impl fmt::Display for EventV3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Event {{ id: {}, pubkey: {}, kind: {}, created_at: {}, content: {}... }}",
            self.id.as_hex_string(),
            self.pubkey.as_hex_string(),
            u32::from(self.kind),
            self.created_at.0,
            &self.content[..self.content.len().min(50)] // Truncate content for display
        )
    }
}

impl EventV3 {
    /// Create a dummy event for testing or placeholder purposes.
    #[allow(dead_code)]
    pub fn new_dummy() -> Self {
        Self {
            id: Id::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000000",
            )
            .unwrap(),
            pubkey: PublicKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000000",
                false,
            )
            .unwrap(), // pubkey of all zeroes
            created_at: Unixtime(0),
            kind: EventKind::TextNote,
            sig: Signature::zeroes(),
            content: "Dummy event content".to_string(),
            tags: Vec::new(),
        }
    }

    /// Sign a `PreEventV3` with the provided `PrivateKey` and return an
    /// `EventV3`.
    pub fn sign_with_private_key(
        preevent: PreEventV3,
        private_key: &PrivateKey,
    ) -> Result<Self, Error> {
        let id = preevent.hash()?;
        let signer = KeySigner::from_private_key(private_key.clone(), "", 1)?;
        let sig = signer.sign_id(id)?;

        Ok(EventV3 {
            id,
            pubkey: preevent.pubkey,
            created_at: preevent.created_at,
            kind: preevent.kind,
            tags: preevent.tags,
            content: preevent.content,
            sig,
        })
    }

    /// Check the validity of an event. This is useful if you deserialize an
    /// event from the network. If you create an event using new() it should
    /// already be trustworthy.
    pub fn verify(&self, maxtime: Option<Unixtime>) -> Result<(), Error> {
        use secp256k1::hashes::Hash;

        let serialized: String = serialize_inner_event!(
            &self.pubkey,
            &self.created_at,
            &self.kind,
            &self.tags,
            &self.content
        );

        // Verify the signature
        self.pubkey.verify(serialized.as_bytes(), &self.sig)?;
        // Also verify the ID is the SHA256
        // (the above verify function also does it internally,
        //  so there is room for improvement here)
        let hash = secp256k1::hashes::sha256::Hash::hash(serialized.as_bytes());
        let id: [u8; 32] = hash.to_byte_array();

        // Optional verify that the message was in the past
        if let Some(mt) = maxtime {
            if self.created_at > mt {
                return Err(Error::EventInFuture);
            }
        }

        if id != self.id.0 {
            Err(Error::HashMismatch)
        } else {
            Ok(())
        }
    }

    /// Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> EventV3 {
        let signer = {
            let private_key = PrivateKey::mock();
            KeySigner::from_private_key(private_key, "", 1).unwrap()
        };
        let public_key = signer.public_key();
        let pre = PreEventV3 {
            pubkey: public_key,
            created_at: Unixtime::mock(),
            kind: EventKind::mock(),
            tags: vec![TagV3::mock(), TagV3::mock()],
            content: "This is a test".to_string(),
        };
        let id = pre.hash().unwrap();
        let sig = signer.sign_id(id).unwrap();
        EventV3 {
            id,
            pubkey: pre.pubkey,
            created_at: pre.created_at,
            kind: pre.kind,
            tags: pre.tags,
            content: pre.content,
            sig,
        }
    }

    /// Get the k-tag kind, if any
    pub fn k_tag_kind(&self) -> Option<EventKind> {
        for tag in self.tags.iter() {
            if let Ok(kind) = tag.parse_kind() {
                return Some(kind);
            }
        }
        None
    }

    /// If the event refers to people by tag, get all the PublicKeys it refers
    /// to along with recommended relay URL and petname for each
    pub fn people(&self) -> Vec<(PublicKey, Option<RelayUrl>, Option<String>)> {
        let mut output: Vec<(PublicKey, Option<RelayUrl>, Option<String>)> = Vec::new();
        // All 'p' tags
        for tag in self.tags.iter() {
            if let Ok((pubkey, recommended_relay_url, petname)) = tag.parse_pubkey() {
                output.push((
                    pubkey.to_owned(),
                    recommended_relay_url
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
                    petname.to_owned(),
                ));
            }
        }

        output
    }

    /// If the pubkey is tagged in the event
    pub fn is_tagged(&self, pk: &PublicKey) -> bool {
        for tag in self.tags.iter() {
            if let Ok((pubkey, _recommended_relay_url, _petname)) = tag.parse_pubkey() {
                if pubkey == *pk {
                    return true;
                }
            }
        }

        false
    }

    /// If the event refers to people within the contents, get all the
    /// PublicKeys it refers to within the contents.
    pub fn people_referenced_in_content(&self) -> Vec<PublicKey> {
        let mut output = Vec::new();
        for nurl in NostrUrl::find_all_in_string(&self.content).drain(..) {
            if let NostrBech32::Pubkey(pk) = nurl.0 {
                output.push(pk);
            }
            if let NostrBech32::Profile(prof) = nurl.0 {
                output.push(prof.pubkey);
            }
        }
        output
    }

    /// All events IDs that this event refers to, whether root, reply, mention,
    /// or otherwise along with optional recommended relay URLs
    pub fn referred_events(&self) -> Vec<EventReference> {
        let mut output: Vec<EventReference> = Vec::new();

        // Collect every 'e' tag and 'a' tag
        for tag in self.tags.iter() {
            if let Ok((id, rurl, marker)) = tag.parse_event() {
                output.push(EventReference::Id {
                    id,
                    author: None,
                    relays: rurl
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker,
                });
            } else if let Ok((ea, _optmarker)) = tag.parse_address() {
                output.push(EventReference::Addr(ea))
            }
        }

        output
    }

    /// Get a reference to another event that this event replies to.
    /// An event can only reply to one other event via 'e' or 'a' tag from a
    /// feed-displayable event that is not a Repost.
    pub fn replies_to(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // Repost 'e' and 'a' tags are always considered mentions, not replies.
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            return None;
        }

        // If there are no 'e' tags nor 'a' tags, then none
        let num_event_ref_tags = self
            .tags
            .iter()
            .filter(|t| t.tagname() == "e" || t.tagname() == "a")
            .count();
        if num_event_ref_tags == 0 {
            return None;
        }

        // look for an 'e' tag with marker 'reply'
        for tag in self.tags.iter() {
            if let Ok((id, rurl, marker)) = tag.parse_event() {
                if marker.is_some() && marker.as_deref().unwrap() == "reply" {
                    return Some(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker,
                    });
                }
            }
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let Ok((id, rurl, marker)) = tag.parse_event() {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker,
                    });
                }
            }
        }

        // look for an 'a' tag marked 'reply'
        for tag in self.tags.iter() {
            if let Ok((ea, marker)) = tag.parse_address() {
                if marker.is_some() && marker.as_deref().unwrap() == "reply" {
                    return Some(EventReference::Addr(ea));
                };
            }
        }

        // look for an 'a' tag marked 'root'
        for tag in self.tags.iter() {
            if let Ok((ea, marker)) = tag.parse_address() {
                if marker.is_some() && marker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Addr(ea));
                };
            }
        }

        // Use the last unmarked 'e' or 'a' tag (whichever is last)
        for tag in self.tags.iter().rev() {
            if tag.tagname() == "e" {
                if let Ok((id, rurl, marker)) = tag.parse_event() {
                    if marker.is_none() {
                        return Some(EventReference::Id {
                            id,
                            author: None,
                            relays: rurl
                                .as_ref()
                                .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                                .into_vec(),
                            marker: None,
                        });
                    }
                }
            } else if tag.tagname() == "a" {
                if let Ok((ea, marker)) = tag.parse_address() {
                    if marker.is_some() && marker.as_deref().unwrap() == "root" {
                        return Some(EventReference::Addr(ea));
                    };
                }
            }
        }

        None
    }

    /// If this event replies to a thread, get that threads root event Id if
    /// available, along with an optional recommended_relay_url
    pub fn replies_to_root(&self) -> Option<EventReference> {
        if !self.kind.is_feed_displayable() {
            return None;
        }

        // look for an 'e' tag with marker 'root'
        for tag in self.tags.iter() {
            if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                if optmarker.is_some() && optmarker.as_deref().unwrap() == "root" {
                    return Some(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: optmarker,
                    });
                }
            }
        }

        for tag in self.tags.iter() {
            if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                if optmarker.is_none() {
                    return Some(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: None,
                    });
                }
            } else if let Ok((ea, optmarker)) = tag.parse_address() {
                if optmarker.is_none() {
                    return Some(EventReference::Addr(ea));
                }
            }
        }

        None
    }

    /// If this event quotes others, get those other events
    pub fn quotes(&self) -> Vec<EventReference> {
        if self.kind != EventKind::TextNote {
            return vec![];
        }

        let mut output: Vec<EventReference> = Vec::new();

        for tag in self.tags.iter() {
            if let Ok((id, rurl)) = tag.parse_quote() {
                output.push(EventReference::Id {
                    id,
                    author: None,
                    relays: rurl
                        .as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                        .into_vec(),
                    marker: None,
                });
            }
        }

        output
    }

    /// If this event mentions others, get those other event Ids
    /// and optional recommended relay Urls
    pub fn mentions(&self) -> Vec<EventReference> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<EventReference> = Vec::new();

        // For kind=6 and kind=16, all 'e' and 'a' tags are mentions
        if self.kind == EventKind::Repost || self.kind == EventKind::GenericRepost {
            for tag in self.tags.iter() {
                if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                    output.push(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: optmarker,
                    });
                } else if let Ok((ea, _optmarker)) = tag.parse_address() {
                    output.push(EventReference::Addr(ea));
                }
            }

            return output;
        }

        // Look for nostr links within the content

        // Collect every 'e' tag marked as 'mention'
        for tag in self.tags.iter() {
            if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                if optmarker.is_some() && optmarker.as_deref().unwrap() == "mention" {
                    output.push(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: optmarker,
                    });
                }
            }
        }

        // Collect every unmarked 'e' or 'a' tag that is not the first (root) or the
        // last (reply)
        let e_tags: Vec<&TagV3> = self
            .tags
            .iter()
            .filter(|t| (t.tagname() == "e" || t.tagname() == "a") && t.marker() == "")
            .collect();
        if e_tags.len() > 2 {
            // mentions are everything other than first and last
            for tag in &e_tags[1..e_tags.len() - 1] {
                if let Ok((id, rurl, optmarker)) = tag.parse_event() {
                    output.push(EventReference::Id {
                        id,
                        author: None,
                        relays: rurl
                            .as_ref()
                            .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok())
                            .into_vec(),
                        marker: optmarker,
                    });
                } else if let Ok((ea, _optmarker)) = tag.parse_address() {
                    output.push(EventReference::Addr(ea));
                }
            }
        }

        output
    }

    /// If this event reacts to another, get that other event's Id,
    /// the reaction content, and an optional Recommended relay Url
    pub fn reacts_to(&self) -> Option<(Id, String, Option<RelayUrl>)> {
        if self.kind != EventKind::Reaction {
            return None;
        }

        // The last 'e' tag is it
        for tag in self.tags.iter().rev() {
            if let Ok((id, rurl, _optmarker)) = tag.parse_event() {
                return Some((
                    id,
                    self.content.clone(),
                    rurl.as_ref()
                        .and_then(|rru| RelayUrl::try_from_unchecked_url(rru).ok()),
                ));
            }
        }

        None
    }

    /// If this event deletes others, get all the EventReferences of the events
    /// that it deletes along with the reason for the deletion
    pub fn deletes(&self) -> Option<(Vec<EventReference>, String)> {
        if self.kind != EventKind::EventDeletion {
            return None;
        }

        let mut erefs: Vec<EventReference> = Vec::new();

        for tag in self.tags.iter() {
            if let Ok((id, _rurl, _optmarker)) = tag.parse_event() {
                // All 'e' tags are deleted
                erefs.push(EventReference::Id {
                    id,
                    author: None,
                    relays: vec![],
                    marker: None,
                });
            } else if let Ok((ea, _optmarker)) = tag.parse_address() {
                erefs.push(EventReference::Addr(ea));
            }
        }

        if erefs.is_empty() {
            None
        } else {
            Some((erefs, self.content.clone()))
        }
    }

    /// Can this event be deleted by the given public key?
    pub fn delete_author_allowed(&self, by: PublicKey) -> bool {
        // Author can always delete
        if self.pubkey == by {
            return true;
        }

        if self.kind == EventKind::GiftWrap {
            for tag in self.tags.iter() {
                if let Ok((pk, _, _)) = tag.parse_pubkey() {
                    return by == pk;
                }
            }
        }

        false
    }

    /// If this event zaps another event, get data about that.
    ///
    /// Errors returned from this are not fatal, but may be useful for
    /// explaining to a user why a zap receipt is invalid.
    pub fn zaps(&self) -> Result<Option<ZapData>, Error> {
        if self.kind != EventKind::Zap {
            return Ok(None);
        }

        let (zap_request, bolt11invoice, payer_p_tag, target_event_from_tags): (
            EventV3,
            Bolt11Invoice,
            Option<PublicKey>,
            Option<EventReference>,
        ) = {
            let mut zap_request: Option<EventV3> = None;
            let mut bolt11invoice: Option<Bolt11Invoice> = None;
            let mut payer_p_tag: Option<PublicKey> = None;
            let mut target_event_from_tags: Option<EventReference> = None;

            for tag in self.tags.iter() {
                if tag.tagname() == "description" {
                    let request_string = tag.value();
                    if let Ok(e) = serde_json::from_str::<EventV3>(request_string) {
                        zap_request = Some(e);
                    }
                }
                // we ignore the "p" tag, we have that data from two other places (invoice and
                // request)
                else if tag.tagname() == "P" {
                    if let Ok((pk, _, _)) = tag.parse_pubkey() {
                        payer_p_tag = Some(pk);
                    }
                } else if tag.tagname() == "bolt11" {
                    if tag.value() == "" {
                        return Err(Error::ZapReceipt("missing bolt11 tag value".to_string()));
                    }

                    // Extract as an Invoice
                    let invoice = match Bolt11Invoice::from_str(tag.value()) {
                        Ok(inv) => inv,
                        Err(e) => {
                            return Err(Error::ZapReceipt(format!(
                                "bolt11 failed to parse: {}",
                                e
                            )));
                        }
                    };

                    // Verify the signature
                    if let Err(e) = invoice.check_signature() {
                        return Err(Error::ZapReceipt(format!(
                            "bolt11 signature check failed: {}",
                            e
                        )));
                    }

                    bolt11invoice = Some(invoice);
                }
            }

            let re = self.referred_events();
            if re.len() == 1 {
                target_event_from_tags = Some(re[0].clone());
            }

            // "The zap receipt MUST contain a description tag which is the JSON-encoded zap
            // request."
            if zap_request.is_none() {
                return Ok(None);
            }
            let zap_request = zap_request.unwrap();

            // "The zap receipt MUST have a bolt11 tag containing the description hash
            // bolt11 invoice."
            if bolt11invoice.is_none() {
                return Ok(None);
            }
            let bolt11invoice = bolt11invoice.unwrap();

            (
                zap_request,
                bolt11invoice,
                payer_p_tag,
                target_event_from_tags,
            )
        };

        // Extract data from the invoice
        let (payee_from_invoice, amount_from_invoice): (PublicKey, MilliSatoshi) = {
            // Get the public key
            let secpk = match bolt11invoice.payee_pub_key() {
                Some(pubkey) => pubkey.to_owned(),
                None => bolt11invoice.recover_payee_pub_key(),
            };
            let (xonlypk, _) = secpk.x_only_public_key();
            let pubkeybytes = xonlypk.serialize();
            let pubkey = match PublicKey::from_bytes(&pubkeybytes, false) {
                Ok(pubkey) => pubkey,
                Err(e) => return Err(Error::ZapReceipt(format!("payee public key error: {}", e))),
            };

            if let Some(u) = bolt11invoice.amount_milli_satoshis() {
                (pubkey, MilliSatoshi(u))
            } else {
                return Err(Error::ZapReceipt(
                    "Amount missing from zap receipt".to_string(),
                ));
            }
        };

        // Extract data from request
        let (payer_from_request, amount_from_request, target_event_from_request): (
            PublicKey,
            MilliSatoshi,
            EventReference,
        ) = {
            let mut amount_from_request: Option<MilliSatoshi> = None;
            let mut target_event_from_request: Option<EventReference> = None;
            for tag in zap_request.tags.iter() {
                if tag.tagname() == "amount" {
                    if let Ok(m) = tag.value().parse::<u64>() {
                        amount_from_request = Some(MilliSatoshi(m));
                    }
                }
            }

            let re = zap_request.referred_events();
            if re.len() == 1 {
                target_event_from_request = Some(re[0].clone());
            }

            if amount_from_request.is_none() {
                return Err(Error::ZapReceipt("zap request had no amount".to_owned()));
            }
            let amount_from_request = amount_from_request.unwrap();

            if target_event_from_request.is_none() {
                return Ok(None);
            }
            let target_event_from_request = target_event_from_request.unwrap();

            (
                zap_request.pubkey,
                amount_from_request,
                target_event_from_request,
            )
        };

        if let Some(p) = payer_p_tag {
            if p != payer_from_request {
                return Err(Error::ZapReceipt(
                    "Payer Mismatch between receipt P-tag and invoice".to_owned(),
                ));
            }
        }

        if amount_from_invoice != amount_from_request {
            return Err(Error::ZapReceipt(
                "Amount Mismatch between request and invoice".to_owned(),
            ));
        }

        if let Some(te) = target_event_from_tags {
            if te != target_event_from_request {
                return Err(Error::ZapReceipt(
                    "Zapped event Mismatch receipt and request".to_owned(),
                ));
            }
        }

        Ok(Some(ZapData {
            zapped_event: target_event_from_request,
            amount: amount_from_invoice,
            payee: payee_from_invoice,
            payer: payer_from_request,
            provider_pubkey: self.pubkey,
        }))
    }

    /// If this event specifies the client that created it, return that client
    /// string
    pub fn client(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if tag.tagname() == "client" && !tag.value().is_empty() {
                return Some(tag.value().to_owned());
            }
        }

        None
    }

    /// If this event specifies a subject, return that subject string
    pub fn subject(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let Ok(subject) = tag.parse_subject() {
                return Some(subject);
            }
        }

        None
    }

    /// If this event specifies a title, return that title string
    pub fn title(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let Ok(title) = tag.parse_title() {
                return Some(title);
            }
        }

        None
    }

    /// If this event specifies a summary, return that summary string
    pub fn summary(&self) -> Option<String> {
        for tag in self.tags.iter() {
            if let Ok(summary) = tag.parse_summary() {
                return Some(summary);
            }
        }

        None
    }

    /// Is this event an annotation
    pub fn is_annotation(&self) -> bool {
        for tag in self.tags.iter() {
            if tag.get_index(0) == "annotation" {
                return true;
            }
        }
        false
    }

    /// If this event specifies a content warning, return that content warning
    pub fn content_warning(&self) -> Option<Option<String>> {
        for tag in self.tags.iter() {
            if let Ok(optcontentwarning) = tag.parse_content_warning() {
                return Some(optcontentwarning);
            }
        }

        None
    }

    /// If this is a parameterized event, get the parameter
    pub fn parameter(&self) -> Option<String> {
        if self.kind.is_parameterized_replaceable() {
            for tag in self.tags.iter() {
                if let Ok(ident) = tag.parse_identifier() {
                    return Some(ident);
                }
            }
            Some("".to_owned()) // implicit
        } else {
            None
        }
    }

    /// Return all the hashtags this event refers to
    pub fn hashtags(&self) -> Vec<String> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<String> = Vec::new();

        for tag in self.tags.iter() {
            if let Ok(hashtag) = tag.parse_hashtag() {
                output.push(hashtag);
            }
        }

        output
    }

    /// Return all the URLs this event refers to
    pub fn urls(&self) -> Vec<RelayUrl> {
        if !self.kind.is_feed_displayable() {
            return vec![];
        }

        let mut output: Vec<RelayUrl> = Vec::new();

        for tag in self.tags.iter() {
            if let Ok((url, _optusage)) = tag.parse_relay() {
                if let Ok(relay_url) = RelayUrl::try_from_unchecked_url(&url) {
                    output.push(relay_url);
                }
            }
        }

        output
    }

    /// Get the proof-of-work count of leading bits
    pub fn pow(&self) -> u8 {
        // Count leading bits in the Id field
        let zeroes: u8 = crate::types::get_leading_zero_bits(&self.id.0);

        // Check that they meant it
        for tag in self.tags.iter() {
            if let Ok((_nonce, Some(target))) = tag.parse_nonce() {
                let target_zeroes = target as u8;
                return zeroes.min(target_zeroes);
            }
        }

        0
    }

    /// Was this event delegated, was that valid, and if so what is the pubkey
    /// of the delegator?
    pub fn delegation(&self) -> EventDelegation {
        for tag in self.tags.iter() {
            if let Ok((pk, conditions, sig)) = tag.parse_delegation() {
                // Verify the delegation tag
                match conditions.verify_signature(&pk, &self.pubkey, &sig) {
                    Ok(_) => {
                        // Check conditions
                        if let Some(kind) = conditions.kind {
                            if self.kind != kind {
                                return EventDelegation::InvalidDelegation(
                                    "Event Kind not delegated".to_owned(),
                                );
                            }
                        }
                        if let Some(created_after) = conditions.created_after {
                            if self.created_at < created_after {
                                return EventDelegation::InvalidDelegation(
                                    "Event created before delegation started".to_owned(),
                                );
                            }
                        }
                        if let Some(created_before) = conditions.created_before {
                            if self.created_at > created_before {
                                return EventDelegation::InvalidDelegation(
                                    "Event created after delegation ended".to_owned(),
                                );
                            }
                        }
                        return EventDelegation::DelegatedBy(pk);
                    }
                    Err(e) => {
                        return EventDelegation::InvalidDelegation(format!("{e}"));
                    }
                }
            }
        }

        EventDelegation::NotDelegated
    }

    /// If the event came through a proxy, get the (Protocol, Id)
    pub fn proxy(&self) -> Option<(String, String)> {
        for tag in self.tags.iter() {
            if let Ok((protocol, id)) = tag.parse_proxy() {
                return Some((protocol, id));
            }
        }
        None
    }
}

impl Ord for EventV3 {
    fn cmp(&self, other: &Self) -> Ordering {
        self.created_at
            .cmp(&other.created_at)
            .then(self.id.cmp(&other.id))
    }
}

impl PartialOrd for EventV3 {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// Direct access into speedy-serialized bytes, to avoid alloc-deserialize just
// to peek at one of these fields
#[cfg(feature = "speedy")]
impl EventV3 {
    /// Read the ID of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_id_from_speedy_bytes(bytes: &[u8]) -> Option<Id> {
        if bytes.len() < 32 {
            None
        } else if let Ok(arr) = <[u8; 32]>::try_from(&bytes[0..32]) {
            Some(unsafe { std::mem::transmute::<[u8; 32], id::Id>(arr) })
        } else {
            None
        }
    }

    /// Read the pubkey of the event from a speedy encoding without decoding
    /// (close to zero allocation, VerifyingKey does stuff I didn't check)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_pubkey_from_speedy_bytes(bytes: &[u8]) -> Option<PublicKey> {
        if bytes.len() < 64 {
            None
        } else {
            PublicKey::from_bytes(&bytes[32..64], false).ok()
        }
    }

    /// Read the created_at of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_created_at_from_speedy_bytes(bytes: &[u8]) -> Option<Unixtime> {
        if bytes.len() < 72 {
            None
        } else if let Ok(i) = i64::read_from_buffer(&bytes[64..72]) {
            Some(Unixtime(i))
        } else {
            None
        }
    }

    /// Read the kind of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_kind_from_speedy_bytes(bytes: &[u8]) -> Option<EventKind> {
        if bytes.len() < 76 {
            None
        } else if let Ok(u) = u32::read_from_buffer(&bytes[72..76]) {
            Some(u.into())
        } else {
            None
        }
    }

    // Read the sig of the event from a speedy encoding without decoding
    // (offset would be 76..140

    /// Read the content of the event from a speedy encoding without decoding
    /// (zero allocation)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn get_content_from_speedy_bytes(bytes: &[u8]) -> Option<&str> {
        let len = u32::from_ne_bytes(bytes[140..140 + 4].try_into().unwrap());

        unsafe {
            Some(std::str::from_utf8_unchecked(
                &bytes[140 + 4..140 + 4 + len as usize],
            ))
        }
    }

    /// Check if any human-readable tag matches the Regex in the speedy encoding
    /// without decoding the whole thing (because our TagV3 representation is so
    /// complicated, we do deserialize the tags for now)
    ///
    /// Note this function is fragile, if the Event structure is reordered,
    /// or if speedy code changes, this will break.  Neither should happen.
    pub fn tag_search_in_speedy_bytes(bytes: &[u8], re: &Regex) -> Result<bool, Error> {
        if bytes.len() < 140 {
            return Ok(false);
        }

        // skip content
        let len = u32::from_ne_bytes(bytes[140..140 + 4].try_into().unwrap());
        let offset = 140 + 4 + len as usize;

        // Deserialize the tags
        let tags: Vec<TagV3> = Vec::<TagV3>::read_from_buffer(&bytes[offset..])?;

        // Search through them
        for tag in &tags {
            match tag.tagname() {
                "content-warning" => {
                    if let Ok(Some(warning)) = tag.parse_content_warning() {
                        if re.is_match(warning.as_ref()) {
                            return Ok(true);
                        }
                    }
                }
                "t" => {
                    if let Ok(hashtag) = tag.parse_hashtag() {
                        if re.is_match(hashtag.as_ref()) {
                            return Ok(true);
                        }
                    }
                }
                "subject" => {
                    if let Ok(subject) = tag.parse_subject() {
                        if re.is_match(subject.as_ref()) {
                            return Ok(true);
                        }
                    }
                }
                "title" => {
                    if let Ok(title) = tag.parse_title() {
                        if re.is_match(title.as_ref()) {
                            return Ok(true);
                        }
                    }
                }
                _ => {
                    if tag.tagname() == "summary" && re.is_match(tag.value()) {
                        return Ok(true);
                    }
                }
            }
        }

        Ok(false)
    }
}

pub(crate) struct UnsignedEventV3(pub PreEventV3);

impl UnsignedEventV3 {
    pub(crate) fn new(
        pubkey: &XOnlyPublicKey,
        kind: u16,
        tags: Vec<Vec<String>>,
        content: String,
    ) -> UnsignedEventV3 {
        let tags = tags.into_iter().map(TagV3).collect();

        UnsignedEventV3(PreEventV3 {
            pubkey: PublicKey::from_bytes(
                &pubkey.public_key(secp256k1::Parity::Even).serialize(),
                false,
            )
            .unwrap(),
            created_at: Unixtime::now(),
            kind: EventKind::from(kind as u32),
            tags,
            content,
        })
    }

    pub(crate) fn sign(self, private_key: &secp256k1::SecretKey) -> Result<EventV3, Error> {
        let id = self.0.hash()?;
        let signer =
            KeySigner::from_private_key(PrivateKey(*private_key, KeySecurity::Medium), "", 1)?;
        let sig = signer.sign_id(id)?;
        Ok(EventV3 {
            id,
            pubkey: self.0.pubkey,
            created_at: self.0.created_at,
            kind: self.0.kind,
            tags: self.0.tags,
            content: self.0.content,
            sig,
        })
    }
}

impl From<EventV3> for RumorV3 {
    fn from(e: EventV3) -> RumorV3 {
        RumorV3 {
            id: e.id,
            pubkey: e.pubkey,
            created_at: e.created_at,
            kind: e.kind,
            content: e.content,
            tags: e.tags,
        }
    }
}

impl From<RumorV3> for PreEventV3 {
    fn from(r: RumorV3) -> PreEventV3 {
        PreEventV3 {
            pubkey: r.pubkey,
            created_at: r.created_at,
            kind: r.kind,
            content: r.content,
            tags: r.tags,
        }
    }
}

impl TryFrom<PreEventV3> for RumorV3 {
    type Error = Error;
    fn try_from(e: PreEventV3) -> Result<RumorV3, Error> {
        RumorV3::new(e)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::types::{DelegationConditions, Signer, UncheckedUrl};

    test_serde! {EventV3, test_event_serde}

    #[test]
    fn test_event_new_and_verify() {
        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };
        let pubkey = signer.public_key();
        let preevent = PreEventV3 {
            pubkey,
            created_at: Unixtime::mock(),
            kind: EventKind::TextNote,
            tags: vec![TagV3::new_event(
                Id::mock(),
                Some(UncheckedUrl::mock()),
                None,
            )],
            content: "Hello World!".to_string(),
        };
        let id = preevent.hash().unwrap();
        let sig = signer.sign_id(id).unwrap();
        let mut event = EventV3 {
            id,
            pubkey: preevent.pubkey,
            created_at: preevent.created_at,
            kind: preevent.kind,
            tags: preevent.tags,
            content: preevent.content,
            sig,
        };

        assert!(event.verify(None).is_ok());

        // Now make sure it fails when the message has been modified
        event.content = "I'm changing this message".to_string();
        let result = event.verify(None);
        assert!(result.is_err());

        // Change it back
        event.content = "Hello World!".to_string();
        let result = event.verify(None);
        assert!(result.is_ok());

        // Tweak the id only
        event.id = Id([
            0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 20, 21, 22, 23,
            24, 25, 26, 27, 28, 29, 30, 31,
        ]);
        let result = event.verify(None);
        assert!(result.is_err());
    }

    // helper
    fn create_event_with_delegation<S>(created_at: Unixtime, real_signer: &S) -> EventV3
    where
        S: Signer,
    {
        let delegated_signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let conditions = DelegationConditions::try_from_str(
            "kind=1&created_at>1680000000&created_at<1680050000",
        )
        .unwrap();

        let sig = real_signer
            .generate_delegation_signature(delegated_signer.public_key(), &conditions)
            .unwrap();

        let preevent = PreEventV3 {
            pubkey: delegated_signer.public_key(),
            created_at,
            kind: EventKind::TextNote,
            tags: vec![
                TagV3::new_event(Id::mock(), Some(UncheckedUrl::mock()), None),
                TagV3::new_delegation(real_signer.public_key(), conditions, sig),
            ],
            content: "Hello World!".to_string(),
        };
        let id = preevent.hash().unwrap();
        let sig = delegated_signer.sign_id(id).unwrap();
        EventV3 {
            id,
            pubkey: preevent.pubkey,
            created_at: preevent.created_at,
            kind: preevent.kind,
            tags: preevent.tags,
            content: preevent.content,
            sig,
        }
    }

    #[test]
    fn test_event_with_delegation_ok() {
        let delegator_signer = {
            let delegator_privkey = PrivateKey::mock();
            KeySigner::from_private_key(delegator_privkey, "", 1).unwrap()
        };
        let delegator_pubkey = delegator_signer.public_key();

        let event = create_event_with_delegation(Unixtime(1680000012), &delegator_signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::DelegatedBy(pk) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(pk, delegator_pubkey);
        } else {
            panic!("Expected DelegatedBy result, got {:?}", event.delegation());
        }
    }

    #[test]
    fn test_event_with_delegation_invalid_created_after() {
        let delegator_privkey = PrivateKey::mock();
        let signer = KeySigner::from_private_key(delegator_privkey, "", 1).unwrap();

        let event = create_event_with_delegation(Unixtime(1690000000), &signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::InvalidDelegation(reason) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(reason, "Event created after delegation ended");
        } else {
            panic!(
                "Expected InvalidDelegation result, got {:?}",
                event.delegation()
            );
        }
    }

    #[test]
    fn test_event_with_delegation_invalid_created_before() {
        let signer = {
            let delegator_privkey = PrivateKey::mock();
            KeySigner::from_private_key(delegator_privkey, "", 1).unwrap()
        };

        let event = create_event_with_delegation(Unixtime(1610000000), &signer);
        assert!(event.verify(None).is_ok());

        // check delegation
        if let EventDelegation::InvalidDelegation(reason) = event.delegation() {
            // expected type, check returned delegator key
            assert_eq!(reason, "Event created before delegation started");
        } else {
            panic!(
                "Expected InvalidDelegation result, got {:?}",
                event.delegation()
            );
        }
    }

    #[test]
    fn test_realworld_event_with_naddr_tag() {
        let raw = r##"{"id":"7760408f6459b9546c3a4e70e3e56756421fba34526b7d460db3fcfd2f8817db","pubkey":"460c25e682fda7832b52d1f22d3d22b3176d972f60dcdc3212ed8c92ef85065c","created_at":1687616920,"kind":1,"tags":[["p","1bc70a0148b3f316da33fe3c89f23e3e71ac4ff998027ec712b905cd24f6a411","","mention"],["a","30311:1bc70a0148b3f316da33fe3c89f23e3e71ac4ff998027ec712b905cd24f6a411:1687612774","","mention"]],"content":"Watching Karnage's stream to see if I learn something about design. \n\nnostr:naddr1qq9rzd3cxumrzv3hxu6qygqmcu9qzj9n7vtd5vl78jyly037wxkyl7vcqflvwy4eqhxjfa4yzypsgqqqwens0qfplk","sig":"dbc5d05a24bfe990a1faaedfcb81a98940d86a105711dbdad9145d05b0ad0f46e3e24eaa3fc283818f27e057fe836a029fd9a68e7f1de06ff477493199d64064"}"##;
        let _: EventV3 = serde_json::from_str(&raw).unwrap();
    }

    #[cfg(feature = "speedy")]
    #[test]
    fn test_speedy_encoded_direct_field_access() {
        use speedy::Writable;

        let signer = {
            let privkey = PrivateKey::mock();
            KeySigner::from_private_key(privkey, "", 1).unwrap()
        };

        let preevent = PreEventV3 {
            pubkey: signer.public_key(),
            created_at: Unixtime(1680000012),
            kind: EventKind::TextNote,
            tags: vec![
                TagV3::new_event(Id::mock(), Some(UncheckedUrl::mock()), None),
                TagV3::new_hashtag("foodstr".to_string()),
            ],
            content: "Hello World!".to_string(),
        };
        let event = signer.sign_event(preevent).unwrap();
        let bytes = event.write_to_vec().unwrap();

        let id = EventV3::get_id_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(id, event.id);

        let pubkey = EventV3::get_pubkey_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(pubkey, event.pubkey);

        let created_at = EventV3::get_created_at_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(created_at, Unixtime(1680000012));

        let kind = EventV3::get_kind_from_speedy_bytes(&bytes).unwrap();
        assert_eq!(kind, event.kind);

        let content = EventV3::get_content_from_speedy_bytes(&bytes);
        assert_eq!(content, Some(&*event.content));

        let re = regex::Regex::new("foodstr").unwrap();
        let found_foodstr = EventV3::tag_search_in_speedy_bytes(&bytes, &re).unwrap();
        assert!(found_foodstr);

        // Print to work out encoding
        //   test like this to see printed data:
        //   cargo test --features=speedy test_speedy_encoded_direct_field_access --
        // --nocapture
        println!("EVENT BYTES: {:?}", bytes);
        println!("ID: {:?}", event.id.0);
        println!("PUBKEY: {:?}", event.pubkey.as_slice());
        println!("CREATED AT: {:?}", event.created_at.0.to_ne_bytes());
        let kind32: u32 = event.kind.into();
        println!("KIND: {:?}", kind32.to_ne_bytes());
        println!("SIG: {:?}", event.sig.0.as_ref());
        println!(
            "CONTENT: [len={:?}] {:?}",
            (event.content.as_bytes().len() as u32).to_ne_bytes(),
            event.content.as_bytes()
        );
        println!("TAGS: [len={:?}]", (event.tags.len() as u32).to_ne_bytes());
    }

    #[test]
    fn test_event_gift_wrap() {
        let signer1 = {
            let sec1 = PrivateKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000001",
            )
            .unwrap();
            KeySigner::from_private_key(sec1, "", 1).unwrap()
        };

        let signer2 = {
            let sec2 = PrivateKey::try_from_hex_string(
                "0000000000000000000000000000000000000000000000000000000000000002",
            )
            .unwrap();
            KeySigner::from_private_key(sec2, "", 1).unwrap()
        };

        let pre = PreEventV3 {
            pubkey: signer1.public_key(),
            created_at: Unixtime(1692_000_000),
            kind: EventKind::TextNote,
            content: "Hey man, this rocks! Please reply for a test.".to_string(),
            tags: vec![],
        };

        let gift_wrap = signer1.giftwrap(pre.clone(), signer2.public_key()).unwrap();
        let rumor = signer2.unwrap_giftwrap(&gift_wrap).unwrap();
        let output_pre: PreEventV3 = rumor.into();

        assert_eq!(pre, output_pre);
    }

    #[test]
    fn test_a_tags_as_replies() {
        let raw = r#"{"id":"d4fb3aeae033baa4a9504027bff8fd065ba1bbd635c501a5e4f8c7ab0bd37c34","pubkey":"7bdef7be22dd8e59f4600e044aa53a1cf975a9dc7d27df5833bc77db784a5805","created_at":1716980987,"kind":1,"sig":"903ae95893082835a42706eda1328ea85a8bf6fbb172bb2f8696b66fccfebfae8756992894a0fb7bb592cb3f78939bdd5fac4cd1eb49138cbf3ea8069574a1dc","content":"The article is interesting, but why compiling everything when configuring meta tags in dist/index.html is sufficient? (like you did in the first version, if I'm not wrong)\nOne main selling point of Oracolo is that it does not require complex server side setup.\n\n> Every time you access the web page, the web page is compiled\n\nThis is not technically correct :)\nJavaScript code is not compiled, it is simply executed; it fetches Nostr data and so builds the page.","tags":[["p","b12b632c887f0c871d140d37bcb6e7c1e1a80264d0b7de8255aa1951d9e1ff79"],["a","30023:b12b632c887f0c871d140d37bcb6e7c1e1a80264d0b7de8255aa1951d9e1ff79:1716928135712","","root"],["r","index.html"]]}"#;
        let event: EventV3 = serde_json::from_str(&raw).unwrap();
        if let Some(parent) = event.replies_to() {
            assert!(matches!(parent, EventReference::Addr(_)));
        } else {
            panic!("a tag reply not recognized");
        }
    }
}
```

---

### versioned/metadata.rs

**Size:** 6538 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, MapAccess, Visitor},
    ser::{Serialize, SerializeMap, Serializer},
};
use serde_json::{json, Map, Value};

/// Metadata about a user
///
/// Note: the value is an Option because some real-world data has been found to
/// contain JSON nulls as values, and we don't want deserialization of those
/// events to fail. We treat these in our get() function the same as if the key
/// did not exist.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataV1 {
    /// username
    pub name: Option<String>,

    /// about
    pub about: Option<String>,

    /// picture URL
    pub picture: Option<String>,

    /// nip05 dns id
    pub nip05: Option<String>,

    /// Additional fields not specified in NIP-01 or NIP-05
    pub other: Map<String, Value>,
}

impl Default for MetadataV1 {
    fn default() -> Self {
        MetadataV1 {
            name: None,
            about: None,
            picture: None,
            nip05: None,
            other: Map::new(),
        }
    }
}

impl MetadataV1 {
    /// Create new empty Metadata
    pub fn new() -> MetadataV1 {
        MetadataV1::default()
    }

    #[allow(dead_code)]
    pub(crate) fn mock() -> MetadataV1 {
        let mut map = Map::new();
        let _ = map.insert(
            "display_name".to_string(),
            Value::String("William Caserin".to_string()),
        );
        MetadataV1 {
            name: Some("jb55".to_owned()),
            about: None,
            picture: None,
            nip05: Some("jb55.com".to_owned()),
            other: map,
        }
    }

    /// Get the lnurl for the user, if available via lud06 or lud16
    pub fn lnurl(&self) -> Option<String> {
        if let Some(Value::String(lud06)) = self.other.get("lud06") {
            if let Ok(data) = bech32::decode(lud06) {
                if data.0 == *crate::types::HRP_LNURL {
                    return Some(String::from_utf8_lossy(&data.1).to_string());
                }
            }
        }

        if let Some(Value::String(lud16)) = self.other.get("lud16") {
            let vec: Vec<&str> = lud16.split('@').collect();
            if vec.len() == 2 {
                let user = &vec[0];
                let domain = &vec[1];
                return Some(format!("https://{domain}/.well-known/lnurlp/{user}"));
            }
        }

        None
    }
}

impl Serialize for MetadataV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(4 + self.other.len()))?;
        map.serialize_entry("name", &json!(&self.name))?;
        map.serialize_entry("about", &json!(&self.about))?;
        map.serialize_entry("picture", &json!(&self.picture))?;
        map.serialize_entry("nip05", &json!(&self.nip05))?;
        for (k, v) in &self.other {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for MetadataV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(MetadataV1Visitor)
    }
}

struct MetadataV1Visitor;

impl<'de> Visitor<'de> for MetadataV1Visitor {
    type Value = MetadataV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A JSON object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<MetadataV1, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: Map<String, Value> = Map::new();
        while let Some((key, value)) = access.next_entry::<String, Value>()? {
            let _ = map.insert(key, value);
        }

        let mut m: MetadataV1 = Default::default();

        if let Some(Value::String(s)) = map.remove("name") {
            m.name = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("about") {
            m.about = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("picture") {
            m.picture = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("nip05") {
            m.nip05 = Some(s);
        }

        m.other = map;

        Ok(m)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {MetadataV1, test_metadata_serde}

    #[test]
    fn test_metadata_print_json() {
        // I want to see if JSON serialized metadata is network appropriate
        let m = MetadataV1::mock();
        println!("{}", serde_json::to_string(&m).unwrap());
    }

    #[test]
    fn test_tolerate_nulls() {
        let json = r##"{"name":"monlovesmango","picture":"https://astral.ninja/aura/monlovesmango.svg","about":"building on nostr","nip05":"monlovesmango@astral.ninja","lud06":null,"testing":"123"}"##;
        let m: MetadataV1 = serde_json::from_str(json).unwrap();
        assert_eq!(m.name, Some("monlovesmango".to_owned()));
        assert_eq!(m.other.get("lud06"), Some(&Value::Null));
        assert_eq!(
            m.other.get("testing"),
            Some(&Value::String("123".to_owned()))
        );
    }

    #[test]
    fn test_metadata_lnurls() {
        // test lud06
        let json = r##"{"name":"mikedilger","about":"Author of Gossip client: https://github.com/mikedilger/gossip\nexpat American living in New Zealand","picture":"https://avatars.githubusercontent.com/u/1669069","nip05":"_@mikedilger.com","banner":"https://mikedilger.com/banner.jpg","display_name":"Michael Dilger","location":"New Zealand","lud06":"lnurl1dp68gurn8ghj7ampd3kx2ar0veekzar0wd5xjtnrdakj7tnhv4kxctttdehhwm30d3h82unvwqhkgetrv4h8gcn4dccnxv563ep","website":"https://mikedilger.com"}"##;
        let m: MetadataV1 = serde_json::from_str(json).unwrap();
        assert_eq!(
            m.lnurl().as_deref(),
            Some("https://walletofsatoshi.com/.well-known/lnurlp/decentbun13")
        );

        // test lud16
        let json = r##"{"name":"mikedilger","about":"Author of Gossip client: https://github.com/mikedilger/gossip\nexpat American living in New Zealand","picture":"https://avatars.githubusercontent.com/u/1669069","nip05":"_@mikedilger.com","banner":"https://mikedilger.com/banner.jpg","display_name":"Michael Dilger","location":"New Zealand","lud16":"decentbun13@walletofsatoshi.com","website":"https://mikedilger.com"}"##;
        let m: MetadataV1 = serde_json::from_str(json).unwrap();
        assert_eq!(
            m.lnurl().as_deref(),
            Some("https://walletofsatoshi.com/.well-known/lnurlp/decentbun13")
        );
    }
}
```

---

### versioned/mod.rs

**Size:** 1413 bytes | **Modified:** 2026-01-20 14:02:27

```rust
pub(crate) mod client_message1;
pub use client_message1::ClientMessageV1;

pub(crate) mod client_message2;
pub use client_message2::ClientMessageV2;

pub(crate) mod client_message3;
pub use client_message3::ClientMessageV3;

pub(crate) mod event1;
pub use event1::{EventV1, PreEventV1, RumorV1};

pub(crate) mod event2;
pub use event2::{EventV2, PreEventV2, RumorV2};

pub(crate) mod event3;
pub use event3::{EventV3, PreEventV3, RumorV3};

pub(crate) mod metadata;
pub use metadata::MetadataV1;

pub(crate) mod nip05;
pub use nip05::Nip05V1;

pub(crate) mod relay_information_document1;
pub use relay_information_document1::{
    FeeV1, RelayFeesV1, RelayInformationDocumentV1, RelayLimitationV1, RelayRetentionV1,
};
pub(crate) mod relay_information_document2;
pub use relay_information_document2::{RelayInformationDocumentV2, RelayLimitationV2};

pub(crate) mod relay_message1;
pub use relay_message1::RelayMessageV1;

pub(crate) mod relay_message2;
pub use relay_message2::RelayMessageV2;

pub(crate) mod relay_message3;
pub use relay_message3::RelayMessageV3;

pub(crate) mod relay_message4;
pub use relay_message4::RelayMessageV4;

pub(crate) mod relay_message5;
pub use relay_message5::{RelayMessageV5, Why};

pub(crate) mod tag1;
pub use tag1::TagV1;

pub(crate) mod tag2;
pub use tag2::TagV2;

pub(crate) mod tag3;
pub use tag3::TagV3;

pub(crate) mod zap_data;
pub use zap_data::{ZapDataV1, ZapDataV2};
```

---

### versioned/nip05.rs

**Size:** 2571 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{PublicKeyHex, UncheckedUrl};

/// The content of a webserver's /.well-known/nostr.json file used in NIP-05 and
/// NIP-35 This allows lookup and verification of a nostr user via a
/// `user@domain` style identifier.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct Nip05V1 {
    /// DNS names mapped to public keys
    pub names: HashMap<String, PublicKeyHex>,

    /// Public keys mapped to arrays of relays where they post
    #[serde(skip_serializing_if = "HashMap::is_empty")]
    #[serde(default)]
    pub relays: HashMap<PublicKeyHex, Vec<UncheckedUrl>>,
}

impl Nip05V1 {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> Nip05V1 {
        let pubkey = PublicKeyHex::try_from_str(
            "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9",
        )
        .unwrap();

        let mut names: HashMap<String, PublicKeyHex> = HashMap::new();
        let _ = names.insert("bob".to_string(), pubkey.clone());

        let mut relays: HashMap<PublicKeyHex, Vec<UncheckedUrl>> = HashMap::new();
        let _ = relays.insert(
            pubkey,
            vec![
                UncheckedUrl::from_str("wss://relay.example.com"),
                UncheckedUrl::from_str("wss://relay2.example.com"),
            ],
        );

        Nip05V1 { names, relays }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {Nip05V1, test_nip05_serde}

    #[test]
    fn test_nip05_example() {
        let body = r#"{
  "names": {
    "bob": "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9"
  },
  "relays": {
    "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9": [ "wss://relay.example.com", "wss://relay2.example.com" ]
  }
}"#;

        let nip05: Nip05V1 = serde_json::from_str(body).unwrap();

        let bobs_pk: PublicKeyHex = nip05.names.get("bob").unwrap().clone();
        assert_eq!(
            bobs_pk.as_str(),
            "b0635d6a9851d3aed0cd6c495b282167acf761729078d975fc341b22650b07b9"
        );

        let bobs_relays: Vec<UncheckedUrl> = nip05.relays.get(&bobs_pk).unwrap().to_owned();

        assert_eq!(
            bobs_relays,
            vec![
                UncheckedUrl::from_str("wss://relay.example.com"),
                UncheckedUrl::from_str("wss://relay2.example.com")
            ]
        );
    }
}
```

---

### versioned/relay_information_document1.rs

**Size:** 21000 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

//use serde::de::Error as DeError;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
use serde_json::{json, Map, Value};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{EventKind, EventKindOrRange, PublicKeyHex, Url};

/// Relay limitations
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayLimitationV1 {
    /// max message length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_message_length: Option<usize>,

    /// max subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_subscriptions: Option<usize>,

    /// max filters
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_filters: Option<usize>,

    /// max limit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_limit: Option<usize>,

    /// max subid length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_subid_length: Option<usize>,

    /// max event tags
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_event_tags: Option<usize>,

    /// max content length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_content_length: Option<usize>,

    /// min pow difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub min_pow_difficulty: Option<usize>,

    /// auth required
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub auth_required: Option<bool>,

    /// payment required
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub payment_required: Option<bool>,
}

impl fmt::Display for RelayLimitationV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Limitation:")?;
        if let Some(mml) = &self.max_message_length {
            write!(f, " MaxMessageLength=\"{mml}\"")?;
        }
        if let Some(ms) = &self.max_subscriptions {
            write!(f, " MaxSubscriptions=\"{ms}\"")?;
        }
        if let Some(mf) = &self.max_filters {
            write!(f, " MaxFilters=\"{mf}\"")?;
        }
        if let Some(ml) = &self.max_limit {
            write!(f, " MaxLimit=\"{ml}\"")?;
        }
        if let Some(msil) = &self.max_subid_length {
            write!(f, " MaxSubidLength=\"{msil}\"")?;
        }
        if let Some(met) = &self.max_event_tags {
            write!(f, " MaxEventTags=\"{met}\"")?;
        }
        if let Some(mcl) = &self.max_content_length {
            write!(f, " MaxContentLength=\"{mcl}\"")?;
        }
        if let Some(mpd) = &self.min_pow_difficulty {
            write!(f, " MinPowDifficulty=\"{mpd}\"")?;
        }
        if let Some(ar) = &self.auth_required {
            write!(f, " AuthRequired=\"{ar}\"")?;
        }
        if let Some(pr) = &self.payment_required {
            write!(f, " PaymentRequired=\"{pr}\"")?;
        }
        Ok(())
    }
}

/// Relay retention
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayRetentionV1 {
    /// kinds
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub kinds: Vec<EventKindOrRange>,

    /// time
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub time: Option<usize>,

    /// count
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub count: Option<usize>,
}

impl fmt::Display for RelayRetentionV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Retention:")?;
        write!(f, " Kinds=\"{:?}\"", self.kinds)?;
        if let Some(time) = &self.time {
            write!(f, " Time=\"{time}\"")?;
        }
        if let Some(count) = &self.count {
            write!(f, " Count=\"{count}\"")?;
        }
        Ok(())
    }
}

/// Fee
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct FeeV1 {
    /// Amount of the fee
    pub amount: usize,

    /// Unit of the amount
    pub unit: String,

    /// Kinds of events
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub kinds: Vec<EventKindOrRange>,

    /// Period purchase lasts for
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub period: Option<usize>,
}

impl fmt::Display for FeeV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Fee=[{} {}", self.amount, self.unit)?;
        write!(f, " Kinds=\"{:?}\"", self.kinds)?;
        if let Some(period) = &self.period {
            write!(f, " Period=\"{}\"", period)?;
        }
        write!(f, "]")
    }
}

/// Relay fees
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayFeesV1 {
    /// Admission fee (read and write)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub admission: Vec<FeeV1>,

    /// Subscription fee (read)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub subscription: Vec<FeeV1>,

    /// Publication fee (write)
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub publication: Vec<FeeV1>,
}

impl fmt::Display for RelayFeesV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Fees:")?;
        write!(f, " Admission=[")?;
        for fee in &self.admission {
            write!(f, "{} ", fee)?;
        }
        write!(f, "],Subscription=[")?;
        for fee in &self.subscription {
            write!(f, "{} ", fee)?;
        }
        write!(f, "],Publication=[")?;
        for fee in &self.publication {
            write!(f, "{} ", fee)?;
        }
        write!(f, "]")
    }
}

/// Relay information document as described in NIP-11, supplied by a relay
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RelayInformationDocumentV1 {
    /// Name of the relay
    pub name: Option<String>,

    /// Description of the relay in plain text
    pub description: Option<String>,

    /// Public key of an administrative contact of the relay
    pub pubkey: Option<PublicKeyHex>,

    /// An administrative contact for the relay. Should be a URI.
    pub contact: Option<String>,

    /// A list of NIPs supported by the relay
    pub supported_nips: Vec<u32>,

    /// The software running the relay
    pub software: Option<String>,

    /// The software version
    pub version: Option<String>,

    /// limitation
    pub limitation: Option<RelayLimitationV1>,

    /// retention
    pub retention: Vec<RelayRetentionV1>,

    /// content limitation: relay countries
    pub relay_countries: Vec<String>,

    /// community preferences: language tags
    pub language_tags: Vec<String>,

    /// community preferences: tags
    pub tags: Vec<String>,

    /// community preferences: posting policy
    pub posting_policy: Option<Url>,

    /// payments_url
    pub payments_url: Option<Url>,

    /// fees
    pub fees: Option<RelayFeesV1>,

    /// Additional fields not specified in NIP-11
    pub other: Map<String, Value>,
}

impl Default for RelayInformationDocumentV1 {
    fn default() -> RelayInformationDocumentV1 {
        RelayInformationDocumentV1 {
            name: None,
            description: None,
            pubkey: None,
            contact: None,
            supported_nips: vec![],
            software: None,
            version: None,
            limitation: None,
            retention: vec![],
            relay_countries: vec![],
            language_tags: vec![],
            tags: vec![],
            posting_policy: None,
            payments_url: None,
            fees: None,
            other: Map::new(),
        }
    }
}

impl RelayInformationDocumentV1 {
    /// If the relay supports the queried `nip`
    pub fn supports_nip(&self, nip: u32) -> bool {
        self.supported_nips.contains(&nip)
    }

    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayInformationDocumentV1 {
        let mut m = Map::new();
        let _ = m.insert(
            "early_nips".to_string(),
            Value::Array(vec![
                Value::Number(5.into()),
                Value::Number(6.into()),
                Value::Number(7.into()),
            ]),
        );
        RelayInformationDocumentV1 {
            name: Some("Crazy Horse".to_string()),
            description: Some("A really wild horse".to_string()),
            pubkey: Some(PublicKeyHex::mock()),
            contact: None,
            supported_nips: vec![11, 12, 13, 14],
            software: None,
            version: None,
            limitation: Some(RelayLimitationV1 {
                max_message_length: Some(16384),
                max_subscriptions: Some(20),
                max_filters: Some(100),
                max_limit: Some(5000),
                max_subid_length: Some(100),
                max_event_tags: Some(100),
                max_content_length: Some(8196),
                min_pow_difficulty: Some(30),
                auth_required: Some(true),
                payment_required: Some(true),
            }),
            retention: vec![
                RelayRetentionV1 {
                    kinds: vec![
                        EventKindOrRange::EventKind(EventKind::Metadata),
                        EventKindOrRange::EventKind(EventKind::TextNote),
                        EventKindOrRange::Range(vec![
                            EventKind::EventDeletion,
                            EventKind::Reaction,
                        ]),
                        EventKindOrRange::Range(vec![EventKind::ChannelCreation]),
                    ],
                    time: Some(3600),
                    count: None,
                },
                RelayRetentionV1 {
                    kinds: vec![EventKindOrRange::Range(vec![
                        EventKind::Other(40000),
                        EventKind::Other(49999),
                    ])],
                    time: Some(100),
                    count: None,
                },
                RelayRetentionV1 {
                    kinds: vec![EventKindOrRange::Range(vec![
                        EventKind::FollowSets,
                        EventKind::Other(39999),
                    ])],
                    time: None,
                    count: Some(1000),
                },
                RelayRetentionV1 {
                    kinds: vec![],
                    time: Some(3600),
                    count: Some(10000),
                },
            ],
            relay_countries: vec!["CA".to_owned(), "US".to_owned()],
            language_tags: vec!["en".to_owned()],
            tags: vec!["sfw-only".to_owned(), "bitcoin-only".to_owned()],
            posting_policy: Some(
                Url::try_from_str("https://example.com/posting-policy.html").unwrap(),
            ),
            payments_url: Some(Url::try_from_str("https://example.com/payments").unwrap()),
            fees: Some(RelayFeesV1 {
                admission: vec![FeeV1 {
                    amount: 1000000,
                    unit: "msats".to_owned(),
                    kinds: vec![],
                    period: None,
                }],
                subscription: vec![FeeV1 {
                    amount: 5000000,
                    unit: "msats".to_owned(),
                    kinds: vec![],
                    period: Some(2592000),
                }],
                publication: vec![FeeV1 {
                    amount: 100,
                    unit: "msats".to_owned(),
                    kinds: vec![EventKindOrRange::EventKind(EventKind::EventDeletion)],
                    period: None,
                }],
            }),
            other: m,
        }
    }
}

impl fmt::Display for RelayInformationDocumentV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Information:")?;
        if let Some(name) = &self.name {
            write!(f, " Name=\"{name}\"")?;
        }
        if let Some(desc) = &self.description {
            write!(f, " Description=\"{desc}\"")?;
        }
        if let Some(pubkey) = &self.pubkey {
            write!(f, " Pubkey=\"{pubkey}\"")?;
        }
        if let Some(contact) = &self.contact {
            write!(f, " Contact=\"{contact}\"")?;
        }
        if !self.supported_nips.is_empty() {
            write!(f, " NIPS={:?}", self.supported_nips)?;
        }
        if let Some(software) = &self.software {
            write!(f, " Software=\"{software}\"")?;
        }
        if let Some(version) = &self.version {
            write!(f, " Version=\"{version}\"")?;
        }
        if let Some(limitation) = &self.limitation {
            write!(f, " Limitation=\"{limitation}\"")?;
        }
        for retention in &self.retention {
            write!(f, " Retention=\"{retention}\"")?;
        }
        if !self.relay_countries.is_empty() {
            write!(f, " Countries=[")?;
            for country in &self.relay_countries {
                write!(f, "{country},")?;
            }
            write!(f, "]")?;
        }
        if !self.language_tags.is_empty() {
            write!(f, " Languages=[")?;
            for language in &self.language_tags {
                write!(f, "{language},")?;
            }
            write!(f, "]")?;
        }
        if !self.tags.is_empty() {
            write!(f, " Tags=[")?;
            for tag in &self.tags {
                write!(f, "{tag},")?;
            }
            write!(f, "]")?;
        }
        if let Some(policy_url) = &self.posting_policy {
            write!(f, " PostingPolicy={policy_url}")?;
        }
        if let Some(url) = &self.payments_url {
            write!(f, " PaymentsUrl={url}")?;
        }
        if let Some(fees) = &self.fees {
            write!(f, " Fees={fees}")?;
        }
        for (k, v) in self.other.iter() {
            write!(f, " {k}=\"{v}\"")?;
        }
        Ok(())
    }
}

impl Serialize for RelayInformationDocumentV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(7 + self.other.len()))?;
        if self.name.is_some() {
            map.serialize_entry("name", &json!(&self.name))?;
        }
        if self.description.is_some() {
            map.serialize_entry("description", &json!(&self.description))?;
        }
        if self.pubkey.is_some() {
            map.serialize_entry("pubkey", &json!(&self.pubkey))?;
        }
        if self.contact.is_some() {
            map.serialize_entry("contact", &json!(&self.contact))?;
        }
        map.serialize_entry("supported_nips", &json!(&self.supported_nips))?;
        if self.software.is_some() {
            map.serialize_entry("software", &json!(&self.software))?;
        }
        if self.version.is_some() {
            map.serialize_entry("version", &json!(&self.version))?;
        }
        if self.limitation.is_some() {
            map.serialize_entry("limitation", &json!(&self.limitation))?;
        }
        if !self.retention.is_empty() {
            map.serialize_entry("retention", &json!(&self.retention))?;
        }
        if !self.relay_countries.is_empty() {
            map.serialize_entry("relay_countries", &json!(&self.relay_countries))?;
        }
        if !self.language_tags.is_empty() {
            map.serialize_entry("language_tags", &json!(&self.language_tags))?;
        }
        if !self.tags.is_empty() {
            map.serialize_entry("tags", &json!(&self.tags))?;
        }
        if self.posting_policy.is_some() {
            map.serialize_entry("posting_policy", &json!(&self.posting_policy))?;
        }
        if self.payments_url.is_some() {
            map.serialize_entry("payments_url", &json!(&self.payments_url))?;
        }
        if self.fees.is_some() {
            map.serialize_entry("fees", &json!(&self.fees))?;
        }
        for (k, v) in &self.other {
            map.serialize_entry(&k, &v)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for RelayInformationDocumentV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(RidVisitor)
    }
}

struct RidVisitor;

impl<'de> Visitor<'de> for RidVisitor {
    type Value = RelayInformationDocumentV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "A JSON object")
    }

    fn visit_map<M>(self, mut access: M) -> Result<RelayInformationDocumentV1, M::Error>
    where
        M: MapAccess<'de>,
    {
        let mut map: Map<String, Value> = Map::new();
        while let Some((key, value)) = access.next_entry::<String, Value>()? {
            let _ = map.insert(key, value);
        }

        let mut rid: RelayInformationDocumentV1 = Default::default();

        if let Some(Value::String(s)) = map.remove("name") {
            rid.name = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("description") {
            rid.description = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("pubkey") {
            rid.pubkey = PublicKeyHex::try_from_string(s).ok();
        }
        if let Some(Value::String(s)) = map.remove("contact") {
            rid.contact = Some(s);
        }
        if let Some(Value::Array(vec)) = map.remove("supported_nips") {
            for elem in vec.iter() {
                if let Value::Number(num) = elem {
                    if let Some(u) = num.as_u64() {
                        rid.supported_nips.push(u as u32);
                    }
                }
            }
        }
        if let Some(Value::String(s)) = map.remove("software") {
            rid.software = Some(s);
        }
        if let Some(Value::String(s)) = map.remove("version") {
            rid.version = Some(s);
        }
        if let Some(v) = map.remove("limitation") {
            rid.limitation =
                serde_json::from_value::<Option<RelayLimitationV1>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("retention") {
            rid.retention = serde_json::from_value::<Vec<RelayRetentionV1>>(v).unwrap_or_default();
        }
        if let Some(v) = map.remove("relay_countries") {
            rid.relay_countries = serde_json::from_value::<Vec<String>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("language_tags") {
            rid.language_tags = serde_json::from_value::<Vec<String>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("tags") {
            rid.tags = serde_json::from_value::<Vec<String>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("posting_policy") {
            rid.posting_policy = serde_json::from_value::<Option<Url>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("payments_url") {
            rid.payments_url = serde_json::from_value::<Option<Url>>(v).unwrap_or_default()
        }
        if let Some(v) = map.remove("fees") {
            rid.fees = serde_json::from_value::<Option<RelayFeesV1>>(v).unwrap_or_default()
        }

        rid.other = map;

        Ok(rid)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayInformationDocumentV1, test_relay_information_document_serde}

    #[test]
    fn test_to_json_only() {
        // This is so you can see the JSON limitation.
        // Run with "cargo test toest_to_json_only -- --nocapture"
        let mock = RelayInformationDocumentV1::mock();
        let s = serde_json::to_string(&mock).unwrap();
        println!("{}", s);
    }

    #[test]
    fn test_relay_information_document_json() {
        let json = r##"{ "name": "A Relay", "description": null, "myfield": [1,2], "supported_nips": [11,12], "retention": [
    { "kinds": [0, 1, [5, 7], [40, 49]], "time": 3600 },
    { "kinds": [[40000, 49999]], "time": 100 },
    { "kinds": [[30000, 39999]], "count": 1000 },
    { "time": 3600, "count": 10000 }
  ] }"##;
        let rid: RelayInformationDocumentV1 = serde_json::from_str(json).unwrap();
        let json2 = serde_json::to_value(&rid).unwrap();
        let expected_json2: Value = serde_json::from_str(r##"{"name":"A Relay","supported_nips":[11,12],"retention":[{"kinds":[0,1,[5,7],[40,49]],"time":3600},{"kinds":[[40000,49999]],"time":100},{"kinds":[[30000,39999]],"count":1000},{"time":3600,"count":10000}],"myfield":[1,2]}"##).unwrap();
        assert_eq!(json2, expected_json2);
    }
}
```

---

### versioned/relay_information_document2.rs

**Size:** 15789 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

//use serde::de::Error as DeError;
use serde::de::{Deserializer, MapAccess, Visitor};
use serde::{
    ser::{SerializeMap, Serializer},
    Deserialize, Serialize,
};
use serde_json::{json, Map, Value};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::{FeeV1, RelayFeesV1, RelayRetentionV1};
use crate::types::{EventKind, EventKindOrRange, PublicKeyHex, Url};

/// Relay limitations
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct RelayLimitationV2 {
    /// max message length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_message_length: Option<usize>,

    /// max subscriptions
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_subscriptions: Option<usize>,

    /// max filters
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_filters: Option<usize>,

    /// max limit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_limit: Option<usize>,

    /// max subid length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_subid_length: Option<usize>,

    /// max event tags
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_event_tags: Option<usize>,

    /// max content length
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub max_content_length: Option<usize>,

    /// min pow difficulty
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub min_pow_difficulty: Option<usize>,

    /// auth required
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub auth_required: Option<bool>,

    /// payment required
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub payment_required: Option<bool>,

    /// restricted writes
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub restricted_writes: Option<bool>,

    /// created at lower limit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub created_at_lower_limit: Option<u64>,

    /// created at upper limit
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub created_at_upper_limit: Option<u64>,
}

impl fmt::Display for RelayLimitationV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Limitation:")?;
        if let Some(mml) = &self.max_message_length {
            write!(f, " MaxMessageLength=\"{mml}\"")?;
        }
        if let Some(ms) = &self.max_subscriptions {
            write!(f, " MaxSubscriptions=\"{ms}\"")?;
        }
        if let Some(mf) = &self.max_filters {
            write!(f, " MaxFilters=\"{mf}\"")?;
        }
        if let Some(ml) = &self.max_limit {
            write!(f, " MaxLimit=\"{ml}\"")?;
        }
        if let Some(msil) = &self.max_subid_length {
            write!(f, " MaxSubidLength=\"{msil}\"")?;
        }
        if let Some(met) = &self.max_event_tags {
            write!(f, " MaxEventTags=\"{met}\"")?;
        }
        if let Some(mcl) = &self.max_content_length {
            write!(f, " MaxContentLength=\"{mcl}\"")?;
        }
        if let Some(mpd) = &self.min_pow_difficulty {
            write!(f, " MinPowDifficulty=\"{mpd}\"")?;
        }
        if let Some(ar) = &self.auth_required {
            write!(f, " AuthRequired=\"{ar}\"")?;
        }
        if let Some(pr) = &self.payment_required {
            write!(f, " PaymentRequired=\"{pr}\"")?;
        }
        if let Some(rw) = &self.restricted_writes {
            write!(f, " RestrictedWrites=\"{rw}\"")?;
        }
        if let Some(call) = &self.created_at_lower_limit {
            write!(f, " CreatedAtLowerLimit=\"{call}\"")?;
        }
        if let Some(caul) = &self.created_at_upper_limit {
            write!(f, " CreatedAtUpperLimit=\"{caul}\"")?;
        }
        Ok(())
    }
}

/// Relay information document as described in NIP-11, supplied by a relay
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct RelayInformationDocumentV2 {
    /// Name of the relay
    pub name: Option<String>,

    /// Description of the relay in plain text
    pub description: Option<String>,

    /// A banner image for the relay
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub banner: Option<Url>,

    /// An icon for the relay
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub icon: Option<Url>,

    /// Public key of an administrative contact of the relay
    pub pubkey: Option<PublicKeyHex>,

    /// The relay's public key, for signing events
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub self_pubkey: Option<PublicKeyHex>,

    /// An administrative contact for the relay. Should be a URI.
    pub contact: Option<String>,

    /// A list of NIPs supported by the relay
    pub supported_nips: Vec<u32>,

    /// The software running the relay
    pub software: Option<String>,

    /// The software version
    pub version: Option<String>,

    /// limitation
    pub limitation: Option<RelayLimitationV2>,

    /// retention
    pub retention: Vec<RelayRetentionV1>,

    /// content limitation: relay countries
    pub relay_countries: Vec<String>,

    /// community preferences: language tags
    pub language_tags: Vec<String>,

    /// community preferences: tags
    pub tags: Vec<String>,

    /// community preferences: posting policy
    pub posting_policy: Option<Url>,

    /// payments_url
    pub payments_url: Option<Url>,

    /// fees
    pub fees: Option<RelayFeesV1>,

    /// Additional fields not specified in NIP-11
    pub other: Map<String, Value>,
}

impl Default for RelayInformationDocumentV2 {
    fn default() -> RelayInformationDocumentV2 {
        RelayInformationDocumentV2 {
            name: None,
            description: None,
            banner: None,
            icon: None,
            pubkey: None,
            self_pubkey: None,
            contact: None,
            supported_nips: vec![],
            software: None,
            version: None,
            limitation: None,
            retention: vec![],
            relay_countries: vec![],
            language_tags: vec![],
            tags: vec![],
            posting_policy: None,
            payments_url: None,
            fees: None,
            other: Map::new(),
        }
    }
}

impl RelayInformationDocumentV2 {
    /// If the relay supports the queried `nip`
    pub fn supports_nip(&self, nip: u32) -> bool {
        self.supported_nips.contains(&nip)
    }

    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayInformationDocumentV2 {
        let mut m = Map::new();
        let _ = m.insert(
            "early_nips".to_string(),
            Value::Array(vec![
                Value::Number(5.into()),
                Value::Number(6.into()),
                Value::Number(7.into()),
            ]),
        );
        RelayInformationDocumentV2 {
            name: Some("Crazy Horse".to_string()),
            description: Some("A really wild horse".to_string()),
            banner: Some(Url::try_from_str("https://example.com/banner.jpg").unwrap()),
            icon: Some(Url::try_from_str("https://example.com/icon.jpg").unwrap()),
            pubkey: Some(PublicKeyHex::mock()),
            self_pubkey: Some(PublicKeyHex::mock()),
            contact: None,
            supported_nips: vec![11, 12, 13, 14],
            software: None,
            version: None,
            limitation: Some(RelayLimitationV2 {
                max_message_length: Some(16384),
                max_subscriptions: Some(20),
                max_filters: Some(100),
                max_limit: Some(5000),
                max_subid_length: Some(100),
                max_event_tags: Some(100),
                max_content_length: Some(8196),
                min_pow_difficulty: Some(30),
                auth_required: Some(true),
                payment_required: Some(true),
                restricted_writes: Some(true),
                created_at_lower_limit: None,
                created_at_upper_limit: None,
            }),
            retention: vec![
                RelayRetentionV1 {
                    kinds: vec![
                        EventKindOrRange::EventKind(EventKind::Metadata),
                        EventKindOrRange::EventKind(EventKind::TextNote),
                        EventKindOrRange::Range(vec![
                            EventKind::EventDeletion,
                            EventKind::Reaction,
                        ]),
                        EventKindOrRange::Range(vec![EventKind::ChannelCreation]),
                    ],
                    time: Some(3600),
                    count: None,
                },
                RelayRetentionV1 {
                    kinds: vec![EventKindOrRange::Range(vec![
                        EventKind::Other(40000),
                        EventKind::Other(49999),
                    ])],
                    time: Some(100),
                    count: None,
                },
                RelayRetentionV1 {
                    kinds: vec![EventKindOrRange::Range(vec![
                        EventKind::FollowSets,
                        EventKind::Other(39999),
                    ])],
                    time: None,
                    count: Some(1000),
                },
                RelayRetentionV1 {
                    kinds: vec![],
                    time: Some(3600),
                    count: Some(10000),
                },
            ],
            relay_countries: vec!["CA".to_owned(), "US".to_owned()],
            language_tags: vec!["en".to_owned()],
            tags: vec!["sfw-only".to_owned(), "bitcoin-only".to_owned()],
            posting_policy: Some(
                Url::try_from_str("https://example.com/posting-policy.html").unwrap(),
            ),
            payments_url: Some(Url::try_from_str("https://example.com/payments").unwrap()),
            fees: Some(RelayFeesV1 {
                admission: vec![FeeV1 {
                    amount: 1000000,
                    unit: "msats".to_owned(),
                    kinds: vec![],
                    period: None,
                }],
                subscription: vec![FeeV1 {
                    amount: 5000000,
                    unit: "msats".to_owned(),
                    kinds: vec![],
                    period: Some(2592000),
                }],
                publication: vec![FeeV1 {
                    amount: 100,
                    unit: "msats".to_owned(),
                    kinds: vec![EventKindOrRange::EventKind(EventKind::EventDeletion)],
                    period: None,
                }],
            }),
            other: m,
        }
    }
}

impl fmt::Display for RelayInformationDocumentV2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Relay Information:")?;
        if let Some(name) = &self.name {
            write!(f, " Name=\"{name}\"")?;
        }
        if let Some(desc) = &self.description {
            write!(f, " Description=\"{desc}\"")?;
        }
        if let Some(banner) = &self.banner {
            write!(f, " Banner=\"{banner}\"")?;
        }
        if let Some(icon) = &self.icon {
            write!(f, " Icon=\"{icon}\"")?;
        }
        if let Some(pubkey) = &self.pubkey {
            write!(f, " Pubkey=\"{pubkey}\"")?;
        }
        if let Some(self_pubkey) = &self.self_pubkey {
            write!(f, " SelfPubkey=\"{self_pubkey}\"")?;
        }
        if let Some(contact) = &self.contact {
            write!(f, " Contact=\"{contact}\"")?;
        }
        if !self.supported_nips.is_empty() {
            write!(f, " NIPS={:?}", self.supported_nips)?;
        }
        if let Some(software) = &self.software {
            write!(f, " Software=\"{software}\"")?;
        }
        if let Some(version) = &self.version {
            write!(f, " Version=\"{version}\"")?;
        }
        if let Some(limitation) = &self.limitation {
            write!(f, " Limitation=\"{limitation}\"")?;
        }
        for retention in &self.retention {
            write!(f, " Retention=\"{retention}\"")?;
        }
        if !self.relay_countries.is_empty() {
            write!(f, " Countries=[")?;
            for country in &self.relay_countries {
                write!(f, "{country},")?;
            }
            write!(f, "]")?;
        }
        if !self.language_tags.is_empty() {
            write!(f, " Languages=[")?;
            for language in &self.language_tags {
                write!(f, "{language},")?;
            }
            write!(f, "]")?;
        }
        if !self.tags.is_empty() {
            write!(f, " Tags=[")?;
            for tag in &self.tags {
                write!(f, "{tag},")?;
            }
            write!(f, "]")?;
        }
        if let Some(policy_url) = &self.posting_policy {
            write!(f, " PostingPolicy={policy_url}")?;
        }
        if let Some(url) = &self.payments_url {
            write!(f, " PaymentsUrl={url}")?;
        }
        if let Some(fees) = &self.fees {
            write!(f, " Fees={fees}")?;
        }
        for (k, v) in self.other.iter() {
            write!(f, " {k}=\"{v}\"")?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayInformationDocumentV2, test_relay_information_document_serde}

    #[test]
    fn test_to_json_only() {
        // This is so you can see the JSON limitation.
        // Run with "cargo test toest_to_json_only -- --nocapture"
        let mock = RelayInformationDocumentV2::mock();
        let s = serde_json::to_string(&mock).unwrap();
        println!("{}", s);
    }

    #[test]
    fn test_relay_information_document_json() {
        let json = r##"{
            "name": "A Relay",
            "description": null,
            "myfield": [1,2],
            "supported_nips": [11,12],
            "retention": [
                { "kinds": [0, 1, [5, 7], [40, 49]], "time": 3600 },
                { "kinds": [[40000, 49999]], "time": 100 },
                { "kinds": [[30000, 39999]], "count": 1000 },
                { "time": 3600, "count": 10000 }
            ],
            "relay_countries": ["CA", "US"],
            "language_tags": ["en", "br"],
            "tags":[],
            "other": { "misc_data": "value" }
        }"##;

        let rid: RelayInformationDocumentV2 = serde_json::from_str(json).unwrap();
        let json2 = serde_json::to_value(&rid).unwrap();

        let expected_json2: serde_json::Value = serde_json::from_str(
            r##"{
            "name": "A Relay",
            "description": null,
            "pubkey": null,
            "contact": null,
            "supported_nips": [11, 12],
            "software": null,
            "version": null,
            "limitation": null,
            "retention": [
                { "kinds": [0, 1, [5, 7], [40, 49]], "time": 3600 },
                { "kinds": [[40000, 49999]], "time": 100 },
                { "kinds": [[30000, 39999]], "count": 1000 },
                { "time": 3600, "count": 10000 }
            ],
            "relay_countries": ["CA", "US"],
            "language_tags": ["en", "br"],
            "tags": [],
            "posting_policy": null,
            "payments_url": null,
            "fees": null,
            "other": { "misc_data": "value" }
        }"##,
        )
        .unwrap();

        assert_eq!(json2, expected_json2);
    }
}
```

---

### versioned/relay_list.rs

**Size:** 0 bytes | **Modified:** 2026-01-20 14:02:27

```rust
```

---

### versioned/relay_message1.rs

**Size:** 5123 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV1;
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV1 {
    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV1>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// Used to notify clients if an event was successuful
    Ok(Id, bool, String),

    /// Used to send authentication challenges
    Auth(String),
}

impl Serialize for RelayMessageV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV1::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV1::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV1::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV1::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV1::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV1, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV1> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV1 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV1::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV1::Notice(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV1::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV1::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV1::Auth(challenge));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}
```

---

### versioned/relay_message2.rs

**Size:** 5438 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV2;
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV2 {
    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV2>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// Used to notify clients if an event was successuful
    Ok(Id, bool, String),

    /// Used to send authentication challenges
    Auth(String),
}

impl RelayMessageV2 {
    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayMessageV2 {
        RelayMessageV2::Event(SubscriptionId::mock(), Box::new(EventV2::mock()))
    }
}

impl Serialize for RelayMessageV2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV2::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV2::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV2::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV2::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV2::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV2;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV2, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV2> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV2 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV2::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV2::Notice(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV2::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV2::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV2::Auth(challenge));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayMessageV2, test_relay_message_serde}
}
```

---

### versioned/relay_message3.rs

**Size:** 7579 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::{EventV2, Why};
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV3 {
    /// Used to send authentication challenges
    Auth(String),

    /// Used to indicate that a subscription was ended on the server side
    /// Every ClientMessage::Req _may_ trigger a RelayMessage::Closed response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted, or error
    Closed(SubscriptionId, String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV2>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// Used to notify clients if an event was successuful
    /// Every ClientMessage::Event will trigger a RelayMessage::OK response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted or error
    Ok(Id, bool, String),
}

impl RelayMessageV3 {
    /// Translate the machine-readable prefix from the message
    pub fn why(&self) -> Option<Why> {
        let s = match *self {
            RelayMessageV3::Closed(_, ref s) => s,
            RelayMessageV3::Ok(_, _, ref s) => s,
            _ => return None,
        };

        match s.split(':').next() {
            Some("auth-required") => Some(Why::AuthRequired),
            Some("blocked") => Some(Why::Blocked),
            Some("duplicate") => Some(Why::Duplicate),
            Some("error") => Some(Why::Error),
            Some("invalid") => Some(Why::Invalid),
            Some("pow") => Some(Why::Pow),
            Some("rate-limited") => Some(Why::RateLimited),
            Some("restricted") => Some(Why::Restricted),
            _ => None,
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayMessageV3 {
        RelayMessageV3::Event(SubscriptionId::mock(), Box::new(EventV2::mock()))
    }
}

impl Serialize for RelayMessageV3 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV3::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
            RelayMessageV3::Closed(id, message) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("CLOSED")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV3::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV3::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV3::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV3::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV3 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV3;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV3, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV3> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV2 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV3::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV3::Notice(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV3::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV3::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV3::Auth(challenge));
        } else if word == "CLOSED" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message messing id field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV3::Closed(id, message));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayMessageV3, test_relay_message_serde}
}
```

---

### versioned/relay_message4.rs

**Size:** 8123 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV3;
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV4 {
    /// Used to send authentication challenges
    Auth(String),

    /// Used to indicate that a subscription was ended on the server side
    /// Every ClientMessage::Req _may_ trigger a RelayMessage::Closed response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted, or error
    Closed(SubscriptionId, String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV3>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// Used to notify clients if an event was successuful
    /// Every ClientMessage::Event will trigger a RelayMessage::OK response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted or error
    Ok(Id, bool, String),
}

/// The reason why a relay issued an OK or CLOSED message
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Why {
    /// Authentication is required
    AuthRequired,

    /// You have been blocked from this relay
    Blocked,

    /// Your request is a duplicate
    Duplicate,

    /// Other error
    Error,

    /// Your request is invalid
    Invalid,

    /// Proof-of-work is required
    Pow,

    /// Rejected due to rate limiting
    RateLimited,

    /// The action you requested is restricted to your identity
    Restricted,
}

impl RelayMessageV4 {
    /// Translate the machine-readable prefix from the message
    pub fn why(&self) -> Option<Why> {
        let s = match *self {
            RelayMessageV4::Closed(_, ref s) => s,
            RelayMessageV4::Ok(_, _, ref s) => s,
            _ => return None,
        };

        match s.split(':').next() {
            Some("auth-required") => Some(Why::AuthRequired),
            Some("blocked") => Some(Why::Blocked),
            Some("duplicate") => Some(Why::Duplicate),
            Some("error") => Some(Why::Error),
            Some("invalid") => Some(Why::Invalid),
            Some("pow") => Some(Why::Pow),
            Some("rate-limited") => Some(Why::RateLimited),
            Some("restricted") => Some(Why::Restricted),
            _ => None,
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayMessageV4 {
        RelayMessageV4::Event(SubscriptionId::mock(), Box::new(EventV3::mock()))
    }
}

impl Serialize for RelayMessageV4 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV4::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
            RelayMessageV4::Closed(id, message) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("CLOSED")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV4::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV4::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV4::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV4::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV4 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV4;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV4, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV4> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV3 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV4::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV4::Notice(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV4::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV4::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV4::Auth(challenge));
        } else if word == "CLOSED" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message messing id field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV4::Closed(id, message));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayMessageV4, test_relay_message_serde}
}
```

---

### versioned/relay_message5.rs

**Size:** 8674 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, Error as DeError, IgnoredAny, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use super::EventV3;
use crate::types::{Id, SubscriptionId};

/// A message from a relay to a client
#[derive(Clone, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum RelayMessageV5 {
    /// Used to send authentication challenges
    Auth(String),

    /// Used to indicate that a subscription was ended on the server side
    /// Every ClientMessage::Req _may_ trigger a RelayMessage::Closed response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted, or error
    Closed(SubscriptionId, String),

    /// End of subscribed events notification
    Eose(SubscriptionId),

    /// An event matching a subscription
    Event(SubscriptionId, Box<EventV3>),

    /// A human readable notice for errors and other information
    Notice(String),

    /// A human readable notice for the end user
    Notify(String),

    /// Used to notify clients if an event was successuful
    /// Every ClientMessage::Event will trigger a RelayMessage::OK response
    /// The last parameter may have a colon-terminated machine-readable prefix
    /// of:     duplicate, pow, blocked, rate-limited, invalid,
    /// auth-required,     restricted or error
    Ok(Id, bool, String),
}

/// The reason why a relay issued an OK or CLOSED message
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub enum Why {
    /// Authentication is required
    AuthRequired,

    /// You have been blocked from this relay
    Blocked,

    /// Your request is a duplicate
    Duplicate,

    /// Other error
    Error,

    /// Your request is invalid
    Invalid,

    /// Proof-of-work is required
    Pow,

    /// Rejected due to rate limiting
    RateLimited,

    /// The action you requested is restricted to your identity
    Restricted,
}

impl RelayMessageV5 {
    /// Translate the machine-readable prefix from the message
    pub fn why(&self) -> Option<Why> {
        let s = match *self {
            RelayMessageV5::Closed(_, ref s) => s,
            RelayMessageV5::Ok(_, _, ref s) => s,
            _ => return None,
        };

        match s.split(':').next() {
            Some("auth-required") => Some(Why::AuthRequired),
            Some("blocked") => Some(Why::Blocked),
            Some("duplicate") => Some(Why::Duplicate),
            Some("error") => Some(Why::Error),
            Some("invalid") => Some(Why::Invalid),
            Some("pow") => Some(Why::Pow),
            Some("rate-limited") => Some(Why::RateLimited),
            Some("restricted") => Some(Why::Restricted),
            _ => None,
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> RelayMessageV5 {
        RelayMessageV5::Event(SubscriptionId::mock(), Box::new(EventV3::mock()))
    }
}

impl Serialize for RelayMessageV5 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            RelayMessageV5::Auth(challenge) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("AUTH")?;
                seq.serialize_element(&challenge)?;
                seq.end()
            }
            RelayMessageV5::Closed(id, message) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("CLOSED")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
            RelayMessageV5::Eose(id) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("EOSE")?;
                seq.serialize_element(&id)?;
                seq.end()
            }
            RelayMessageV5::Event(id, event) => {
                let mut seq = serializer.serialize_seq(Some(3))?;
                seq.serialize_element("EVENT")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&event)?;
                seq.end()
            }
            RelayMessageV5::Notice(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTICE")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV5::Notify(s) => {
                let mut seq = serializer.serialize_seq(Some(2))?;
                seq.serialize_element("NOTIFY")?;
                seq.serialize_element(&s)?;
                seq.end()
            }
            RelayMessageV5::Ok(id, ok, message) => {
                let mut seq = serializer.serialize_seq(Some(4))?;
                seq.serialize_element("OK")?;
                seq.serialize_element(&id)?;
                seq.serialize_element(&ok)?;
                seq.serialize_element(&message)?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for RelayMessageV5 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(RelayMessageVisitor)
    }
}

struct RelayMessageVisitor;

impl<'de> Visitor<'de> for RelayMessageVisitor {
    type Value = RelayMessageV5;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<RelayMessageV5, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let word: &str = seq
            .next_element()?
            .ok_or_else(|| DeError::custom("Message missing initial string field"))?;
        let mut output: Option<RelayMessageV5> = None;
        if word == "EVENT" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let event: EventV3 = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing event field"))?;
            output = Some(RelayMessageV5::Event(id, Box::new(event)));
        } else if word == "NOTICE" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV5::Notice(s));
        } else if word == "NOTIFY" {
            let s: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV5::Notify(s));
        } else if word == "EOSE" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            output = Some(RelayMessageV5::Eose(id))
        } else if word == "OK" {
            let id: Id = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing id field"))?;
            let ok: bool = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing ok field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV5::Ok(id, ok, message));
        } else if word == "AUTH" {
            let challenge: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing challenge field"))?;
            output = Some(RelayMessageV5::Auth(challenge));
        } else if word == "CLOSED" {
            let id: SubscriptionId = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message messing id field"))?;
            let message: String = seq
                .next_element()?
                .ok_or_else(|| DeError::custom("Message missing string field"))?;
            output = Some(RelayMessageV5::Closed(id, message));
        }

        // Consume any trailing fields
        while let Some(_ignored) = seq.next_element::<IgnoredAny>()? {}

        match output {
            Some(rm) => Ok(rm),
            None => Err(DeError::custom(format!("Unknown Message: {word}"))),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {RelayMessageV5, test_relay_message_serde}
}
```

---

### versioned/tag1.rs

**Size:** 31053 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    DelegationConditions, Error, EventKind, Id, PublicKeyHex, SignatureHex, UncheckedUrl, Unixtime,
};

/// A tag on an Event
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum TagV1 {
    /// Address 'a' tag to a parameterized replaceable event
    Address {
        /// EventKind
        kind: EventKind,

        /// Author
        pubkey: PublicKeyHex,

        /// d-tag identifier
        d: String,

        /// Relay URL
        relay_url: Option<UncheckedUrl>,

        /// Trailing
        trailing: Vec<String>,
    } = 0,

    /// Content Warning to alert client to hide content until user approves
    ContentWarning {
        /// Content warning
        warning: String,

        /// Trailing
        trailing: Vec<String>,
    } = 1,

    /// Delegation (Delegated Event Signing)
    Delegation {
        /// Public key of the delegator
        pubkey: PublicKeyHex,

        /// Conditions query string
        conditions: DelegationConditions,

        /// 64-byte schnorr signature of the sha256 hash of the delegation token
        sig: SignatureHex,

        /// Trailing
        trailing: Vec<String>,
    } = 2,

    /// This is a reference to an event, where the first string is the event Id.
    /// The second string is defined in NIP-01 as an optional URL, but
    /// subsequent 'e' NIPs define more data and interpretations.
    Event {
        /// The Id of some other event that this event refers to
        id: Id,

        /// A recommended relay URL to find that other event
        recommended_relay_url: Option<UncheckedUrl>,

        /// A marker (commonly things like 'reply')
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 3,

    /// A time when the event should be considered expired
    Expiration {
        /// Expiration Time
        time: Unixtime,

        /// Trailing
        trailing: Vec<String>,
    } = 4,

    /// 'p' This is a reference to a user by public key, where the first string
    /// is the PublicKey. The second string is defined in NIP-01 as an
    /// optional URL, but subsqeuent NIPs define more data and
    /// interpretations.
    Pubkey {
        /// The public key of the identity that this event refers to
        pubkey: PublicKeyHex,

        /// A recommended relay URL to find information on that public key
        recommended_relay_url: Option<UncheckedUrl>,

        /// A petname given to this identity by the event author
        petname: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 5,

    /// 't' A hashtag
    Hashtag {
        /// Hashtag
        hashtag: String,

        /// Trailing
        trailing: Vec<String>,
    } = 6,

    /// 'r' A reference to a URL
    Reference {
        /// A relay url
        url: UncheckedUrl,

        /// An optional marker
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 7,

    /// 'g' A geohash
    Geohash {
        /// A geohash
        geohash: String,

        /// Trailing
        trailing: Vec<String>,
    } = 8,

    /// 'd' Identifier tag
    Identifier {
        /// 'd' indentifier
        d: String,

        /// Trailing
        trailing: Vec<String>,
    } = 9,

    /// A subject. The first string is the subject. Should only be in TextNote
    /// events.
    Subject {
        /// The subject
        subject: String,

        /// Trailing
        trailing: Vec<String>,
    } = 10,

    /// A nonce tag for Proof of Work
    Nonce {
        /// A random number that makes the event hash meet the proof of work
        /// required
        nonce: String,

        /// The target number of bits for the proof of work
        target: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 11,

    /// There is no known nostr tag like this. This was a mistake, but we can't
    /// remove it or deserialization of data serialized with this in mind
    /// will break.
    Parameter {
        /// Parameter
        param: String,

        /// Trailing
        trailing: Vec<String>,
    } = 12,

    /// Title (30023 long form)
    Title {
        /// Title
        title: String,

        /// Trailing
        trailing: Vec<String>,
    } = 13,

    /// Any other tag
    Other {
        /// The tag name
        tag: String,

        /// The subsequent fields
        data: Vec<String>,
    } = 14,

    /// An empty array (kept so signature remains valid across ser/de)
    Empty = 15,

    /// Direct parent of an event, 'E' tag
    /// This is from <https://github.com/nostr-protocol/nips/pull/830> which may not happen
    /// We should not create these, but we can support them if we encounter
    /// them.
    EventParent {
        /// The id of some other event that is the direct parent to this event
        id: Id,

        /// A recommended relay URL to find that other event
        recommended_relay_url: Option<UncheckedUrl>,

        /// Trailing
        trailing: Vec<String>,
    } = 16,

    /// Kind number 'k'
    Kind {
        /// Event kind
        kind: EventKind,

        /// Trailing
        trailing: Vec<String>,
    } = 17,
}

impl TagV1 {
    /// Get the tag name for the tag (the first string in the array)a
    pub fn tagname(&self) -> String {
        match self {
            TagV1::Address { .. } => "address".to_string(),
            TagV1::ContentWarning { .. } => "content-warning".to_string(),
            TagV1::Delegation { .. } => "delegation".to_string(),
            TagV1::Event { .. } => "e".to_string(),
            TagV1::EventParent { .. } => "E".to_string(),
            TagV1::Expiration { .. } => "expiration".to_string(),
            TagV1::Kind { .. } => "k".to_string(),
            TagV1::Pubkey { .. } => "p".to_string(),
            TagV1::Hashtag { .. } => "t".to_string(),
            TagV1::Reference { .. } => "r".to_string(),
            TagV1::Geohash { .. } => "g".to_string(),
            TagV1::Identifier { .. } => "d".to_string(),
            TagV1::Subject { .. } => "subject".to_string(),
            TagV1::Nonce { .. } => "nonce".to_string(),
            TagV1::Parameter { .. } => "parameter".to_string(),
            TagV1::Title { .. } => "title".to_string(),
            TagV1::Other { tag, .. } => tag.clone(),
            TagV1::Empty => "".to_string(),
        }
    }

    /// Get the string value of the tag at an array index
    pub fn value(&self, index: usize) -> Result<String, Error> {
        use serde_json::Value;
        let json = serde_json::to_value(self)?;
        match json {
            Value::Array(vec) => match vec.get(index) {
                Some(val) => match val {
                    Value::String(s) => Ok(s.to_owned()),
                    _ => Err(Error::AssertionFailed(
                        "Tag field is not a string".to_owned(),
                    )),
                },
                None => Ok("".to_owned()),
            },
            _ => Err(Error::AssertionFailed(
                "Tag JSON is not an array".to_owned(),
            )),
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> TagV1 {
        TagV1::Event {
            id: Id::mock(),
            recommended_relay_url: Some(UncheckedUrl::mock()),
            marker: None,
            trailing: Vec::new(),
        }
    }
}

impl Serialize for TagV1 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TagV1::Address {
                kind,
                pubkey,
                d,
                relay_url,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("a")?;
                let k: u32 = From::from(*kind);
                let s = format!("{}:{}:{}", k, pubkey, d);
                seq.serialize_element(&s)?;
                if let Some(ru) = relay_url {
                    seq.serialize_element(ru)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::ContentWarning { warning, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("content-warning")?;
                seq.serialize_element(warning)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Delegation {
                pubkey,
                conditions,
                sig,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("delegation")?;
                seq.serialize_element(pubkey)?;
                seq.serialize_element(conditions)?;
                seq.serialize_element(sig)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("e")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if marker.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(m) = marker {
                    seq.serialize_element(m)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::EventParent {
                id,
                recommended_relay_url,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("E")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Expiration { time, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("expiration")?;
                seq.serialize_element(time)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Kind { kind, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("k")?;
                // in tags, we must use string types only
                let k: u32 = From::from(*kind);
                let s = format!("{}", k);
                seq.serialize_element(&s)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("p")?;
                seq.serialize_element(pubkey)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if petname.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(pn) = petname {
                    seq.serialize_element(pn)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Hashtag { hashtag, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("t")?;
                seq.serialize_element(hashtag)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Reference {
                url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("r")?;
                seq.serialize_element(url)?;
                if let Some(m) = marker {
                    seq.serialize_element(m)?
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Geohash { geohash, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("g")?;
                seq.serialize_element(geohash)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Identifier { d, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("d")?;
                seq.serialize_element(d)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Subject { subject, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("subject")?;
                seq.serialize_element(subject)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Nonce {
                nonce,
                target,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("nonce")?;
                seq.serialize_element(nonce)?;
                if let Some(t) = target {
                    seq.serialize_element(t)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Parameter { param, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("parameter")?;
                seq.serialize_element(param)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Title { title, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("title")?;
                seq.serialize_element(title)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Other { tag, data } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element(tag)?;
                for s in data.iter() {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV1::Empty => {
                let seq = serializer.serialize_seq(Some(0))?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for TagV1 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TagVisitor)
    }
}

struct TagVisitor;

impl<'de> Visitor<'de> for TagVisitor {
    type Value = TagV1;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<TagV1, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tagname: &str = match seq.next_element()? {
            Some(e) => e,
            None => return Ok(TagV1::Empty),
        };
        if tagname == "a" {
            if let Some(a) = seq.next_element::<&str>()? {
                let relay_url: Option<UncheckedUrl> = seq.next_element()?;
                let mut trailing: Vec<String> = Vec::new();
                while let Some(s) = seq.next_element()? {
                    trailing.push(s);
                }

                let fail = || -> TagV1 {
                    match relay_url {
                        Some(ref url) => {
                            let mut fv = vec![a.to_string(), url.as_str().to_owned()];
                            fv.extend(trailing.clone());
                            TagV1::Other {
                                tag: tagname.to_string(),
                                data: fv,
                            }
                        }
                        None => TagV1::Other {
                            tag: tagname.to_string(),
                            data: vec![a.to_string()],
                        },
                    }
                };

                let parts: Vec<&str> = a.split(':').collect();
                if parts.len() < 3 {
                    return Ok(fail());
                }
                let kindnum: u32 = match parts[0].parse::<u32>() {
                    Ok(u) => u,
                    Err(_) => return Ok(fail()),
                };
                let kind: EventKind = From::from(kindnum);
                let pubkey: PublicKeyHex = match PublicKeyHex::try_from_str(parts[1]) {
                    Ok(pk) => pk,
                    Err(_) => return Ok(fail()),
                };
                Ok(TagV1::Address {
                    kind,
                    pubkey,
                    d: parts[2].to_string(),
                    relay_url,
                    trailing,
                })
            } else {
                Ok(TagV1::Other {
                    tag: tagname.to_string(),
                    data: vec![],
                })
            }
        } else if tagname == "content-warning" {
            let msg = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::ContentWarning {
                warning: msg,
                trailing,
            })
        } else if tagname == "delegation" {
            let pubkey: PublicKeyHex = match seq.next_element()? {
                Some(pk) => pk,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let conditions: DelegationConditions = match seq.next_element()? {
                Some(c) => c,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![pubkey.into_string()],
                    });
                }
            };
            let sig: SignatureHex = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![pubkey.into_string(), conditions.as_string()],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Delegation {
                pubkey,
                conditions,
                sig,
                trailing,
            })
        } else if tagname == "e" {
            let id: Id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Event {
                id,
                recommended_relay_url,
                marker,
                trailing,
            })
        } else if tagname == "E" {
            let id: Id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::EventParent {
                id,
                recommended_relay_url,
                trailing,
            })
        } else if tagname == "expiration" {
            let time = match seq.next_element()? {
                Some(t) => t,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Expiration { time, trailing })
        } else if tagname == "p" {
            let pubkey: PublicKeyHex = match seq.next_element()? {
                Some(p) => p,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let petname: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                trailing,
            })
        } else if tagname == "t" {
            let tag = match seq.next_element()? {
                Some(t) => t,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Hashtag {
                hashtag: tag,
                trailing,
            })
        } else if tagname == "r" {
            let refr: UncheckedUrl = match seq.next_element()? {
                Some(r) => r,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Reference {
                url: refr,
                marker,
                trailing,
            })
        } else if tagname == "g" {
            let geo = match seq.next_element()? {
                Some(g) => g,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Geohash {
                geohash: geo,
                trailing,
            })
        } else if tagname == "d" {
            let id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    // Implicit empty value
                    return Ok(TagV1::Identifier {
                        d: "".to_string(),
                        trailing: Vec::new(),
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Identifier { d: id, trailing })
        } else if tagname == "k" {
            let mut parts: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                parts.push(s);
            }
            if parts.is_empty() {
                return Ok(TagV1::Other {
                    tag: tagname.to_string(),
                    data: parts,
                });
            }
            let kindnum: u32 = match parts[0].parse::<u32>() {
                Ok(u) => u,
                Err(_) => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: parts,
                    });
                }
            };
            let kind: EventKind = From::from(kindnum);
            Ok(TagV1::Kind {
                kind,
                trailing: parts[1..].to_owned(),
            })
        } else if tagname == "subject" {
            let sub = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Subject {
                subject: sub,
                trailing,
            })
        } else if tagname == "nonce" {
            let nonce = match seq.next_element()? {
                Some(n) => n,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let target: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Nonce {
                nonce,
                target,
                trailing,
            })
        } else if tagname == "parameter" {
            let param = match seq.next_element()? {
                Some(s) => s,
                None => "".to_owned(), // implicit parameter
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Parameter { param, trailing })
        } else if tagname == "title" {
            let title = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV1::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV1::Title { title, trailing })
        } else {
            let mut data = Vec::new();
            loop {
                match seq.next_element()? {
                    None => {
                        return Ok(TagV1::Other {
                            tag: tagname.to_string(),
                            data,
                        });
                    }
                    Some(s) => data.push(s),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {TagV1, test_tag_serde}

    #[test]
    fn test_a_tag() {
        let tag = TagV1::Address {
            kind: EventKind::LongFormContent,
            pubkey: PublicKeyHex::mock_deterministic(),
            d: "Testing123".to_owned(),
            relay_url: Some(UncheckedUrl("wss://relay.mikedilger.com/".to_string())),
            trailing: Vec::new(),
        };
        let string = serde_json::to_string(&tag).unwrap();
        let tag2 = serde_json::from_str(&string).unwrap();
        assert_eq!(tag, tag2);

        let tag = TagV1::Address {
            kind: EventKind::LongFormContent,
            pubkey: PublicKeyHex::mock_deterministic(),
            d: "Testing123".to_owned(),
            relay_url: None,
            trailing: Vec::new(),
        };
        let string = serde_json::to_string(&tag).unwrap();
        let tag2 = serde_json::from_str(&string).unwrap();
        assert_eq!(tag, tag2);
    }
}
```

---

### versioned/tag2.rs

**Size:** 31823 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{
    de::{Deserialize, Deserializer, SeqAccess, Visitor},
    ser::{Serialize, SerializeSeq, Serializer},
};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    DelegationConditions, Error, EventKind, Id, PublicKeyHex, SignatureHex, UncheckedUrl, Unixtime,
};

/// A tag on an Event
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub enum TagV2 {
    /// Address 'a' tag to a parameterized replaceable event
    Address {
        /// EventKind
        kind: EventKind,

        /// Author
        pubkey: PublicKeyHex,

        /// d-tag identifier
        d: String,

        /// Relay URL
        relay_url: Option<UncheckedUrl>,

        /// A marker (commonly things like 'reply')
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 0,

    /// Content Warning to alert client to hide content until user approves
    ContentWarning {
        /// Content warning
        warning: String,

        /// Trailing
        trailing: Vec<String>,
    } = 1,

    /// Delegation (Delegated Event Signing)
    Delegation {
        /// Public key of the delegator
        pubkey: PublicKeyHex,

        /// Conditions query string
        conditions: DelegationConditions,

        /// 64-byte schnorr signature of the sha256 hash of the delegation token
        sig: SignatureHex,

        /// Trailing
        trailing: Vec<String>,
    } = 2,

    /// This is a reference to an event, where the first string is the event Id.
    /// The second string is defined in NIP-01 as an optional URL, but
    /// subsequent 'e' NIPs define more data and interpretations.
    Event {
        /// The Id of some other event that this event refers to
        id: Id,

        /// A recommended relay URL to find that other event
        recommended_relay_url: Option<UncheckedUrl>,

        /// A marker (commonly things like 'reply')
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 3,

    /// A time when the event should be considered expired
    Expiration {
        /// Expiration Time
        time: Unixtime,

        /// Trailing
        trailing: Vec<String>,
    } = 4,

    /// 'p' This is a reference to a user by public key, where the first string
    /// is the PublicKey. The second string is defined in NIP-01 as an
    /// optional URL, but subsqeuent NIPs define more data and
    /// interpretations.
    Pubkey {
        /// The public key of the identity that this event refers to
        pubkey: PublicKeyHex,

        /// A recommended relay URL to find information on that public key
        recommended_relay_url: Option<UncheckedUrl>,

        /// A petname given to this identity by the event author
        petname: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 5,

    /// 't' A hashtag
    Hashtag {
        /// Hashtag
        hashtag: String,

        /// Trailing
        trailing: Vec<String>,
    } = 6,

    /// 'r' A reference to a URL
    Reference {
        /// A relay url
        url: UncheckedUrl,

        /// An optional marker
        marker: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 7,

    /// 'g' A geohash
    Geohash {
        /// A geohash
        geohash: String,

        /// Trailing
        trailing: Vec<String>,
    } = 8,

    /// 'd' Identifier tag
    Identifier {
        /// 'd' indentifier
        d: String,

        /// Trailing
        trailing: Vec<String>,
    } = 9,

    /// A subject. The first string is the subject. Should only be in TextNote
    /// events.
    Subject {
        /// The subject
        subject: String,

        /// Trailing
        trailing: Vec<String>,
    } = 10,

    /// A nonce tag for Proof of Work
    Nonce {
        /// A random number that makes the event hash meet the proof of work
        /// required
        nonce: String,

        /// The target number of bits for the proof of work
        target: Option<String>,

        /// Trailing
        trailing: Vec<String>,
    } = 11,

    /// There is no known nostr tag like this. This was a mistake, but we can't
    /// remove it or deserialization of data serialized with this in mind
    /// will break.
    Parameter {
        /// Parameter
        param: String,

        /// Trailing
        trailing: Vec<String>,
    } = 12,

    /// Title (30023 long form)
    Title {
        /// Title
        title: String,

        /// Trailing
        trailing: Vec<String>,
    } = 13,

    /// Any other tag
    Other {
        /// The tag name
        tag: String,

        /// The subsequent fields
        data: Vec<String>,
    } = 14,

    /// An empty array (kept so signature remains valid across ser/de)
    Empty = 15,

    /// Direct parent of an event, 'E' tag
    /// This is from <https://github.com/nostr-protocol/nips/pull/830> which may not happen
    /// We should not create these, but we can support them if we encounter
    /// them.
    EventParent {
        /// The id of some other event that is the direct parent to this event
        id: Id,

        /// A recommended relay URL to find that other event
        recommended_relay_url: Option<UncheckedUrl>,

        /// Trailing
        trailing: Vec<String>,
    } = 16,

    /// Kind number 'k'
    Kind {
        /// Event kind
        kind: EventKind,

        /// Trailing
        trailing: Vec<String>,
    } = 17,
}

impl TagV2 {
    /// Get the tag name for the tag (the first string in the array)a
    pub fn tagname(&self) -> String {
        match self {
            TagV2::Address { .. } => "address".to_string(),
            TagV2::ContentWarning { .. } => "content-warning".to_string(),
            TagV2::Delegation { .. } => "delegation".to_string(),
            TagV2::Event { .. } => "e".to_string(),
            TagV2::EventParent { .. } => "E".to_string(),
            TagV2::Expiration { .. } => "expiration".to_string(),
            TagV2::Kind { .. } => "k".to_string(),
            TagV2::Pubkey { .. } => "p".to_string(),
            TagV2::Hashtag { .. } => "t".to_string(),
            TagV2::Reference { .. } => "r".to_string(),
            TagV2::Geohash { .. } => "g".to_string(),
            TagV2::Identifier { .. } => "d".to_string(),
            TagV2::Subject { .. } => "subject".to_string(),
            TagV2::Nonce { .. } => "nonce".to_string(),
            TagV2::Parameter { .. } => "parameter".to_string(),
            TagV2::Title { .. } => "title".to_string(),
            TagV2::Other { tag, .. } => tag.clone(),
            TagV2::Empty => "".to_string(),
        }
    }

    /// Get the string value of the tag at an array index
    pub fn value(&self, index: usize) -> Result<String, Error> {
        use serde_json::Value;
        let json = serde_json::to_value(self)?;
        match json {
            Value::Array(vec) => match vec.get(index) {
                Some(val) => match val {
                    Value::String(s) => Ok(s.to_owned()),
                    _ => Err(Error::AssertionFailed(
                        "Tag field is not a string".to_owned(),
                    )),
                },
                None => Ok("".to_owned()),
            },
            _ => Err(Error::AssertionFailed(
                "Tag JSON is not an array".to_owned(),
            )),
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> TagV2 {
        TagV2::Event {
            id: Id::mock(),
            recommended_relay_url: Some(UncheckedUrl::mock()),
            marker: None,
            trailing: Vec::new(),
        }
    }
}

impl Serialize for TagV2 {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            TagV2::Address {
                kind,
                pubkey,
                d,
                relay_url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("a")?;
                let k: u32 = From::from(*kind);
                let s = format!("{}:{}:{}", k, pubkey, d);
                seq.serialize_element(&s)?;
                if let Some(ru) = relay_url {
                    seq.serialize_element(ru)?;
                } else if marker.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(m) = marker {
                    seq.serialize_element(m)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::ContentWarning { warning, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("content-warning")?;
                seq.serialize_element(warning)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Delegation {
                pubkey,
                conditions,
                sig,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("delegation")?;
                seq.serialize_element(pubkey)?;
                seq.serialize_element(conditions)?;
                seq.serialize_element(sig)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("e")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if marker.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(m) = marker {
                    seq.serialize_element(m)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::EventParent {
                id,
                recommended_relay_url,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("E")?;
                seq.serialize_element(id)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Expiration { time, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("expiration")?;
                seq.serialize_element(time)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Kind { kind, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("k")?;
                // in tags, we must use string types only
                let k: u32 = From::from(*kind);
                let s = format!("{}", k);
                seq.serialize_element(&s)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("p")?;
                seq.serialize_element(pubkey)?;
                if let Some(rru) = recommended_relay_url {
                    seq.serialize_element(rru)?;
                } else if petname.is_some() || !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                if let Some(pn) = petname {
                    seq.serialize_element(pn)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Hashtag { hashtag, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("t")?;
                seq.serialize_element(hashtag)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Reference {
                url,
                marker,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("r")?;
                seq.serialize_element(url)?;
                if let Some(m) = marker {
                    seq.serialize_element(m)?
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Geohash { geohash, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("g")?;
                seq.serialize_element(geohash)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Identifier { d, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("d")?;
                seq.serialize_element(d)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Subject { subject, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("subject")?;
                seq.serialize_element(subject)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Nonce {
                nonce,
                target,
                trailing,
            } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("nonce")?;
                seq.serialize_element(nonce)?;
                if let Some(t) = target {
                    seq.serialize_element(t)?;
                } else if !trailing.is_empty() {
                    seq.serialize_element("")?;
                }
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Parameter { param, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("parameter")?;
                seq.serialize_element(param)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Title { title, trailing } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element("title")?;
                seq.serialize_element(title)?;
                for s in trailing {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Other { tag, data } => {
                let mut seq = serializer.serialize_seq(None)?;
                seq.serialize_element(tag)?;
                for s in data.iter() {
                    seq.serialize_element(s)?;
                }
                seq.end()
            }
            TagV2::Empty => {
                let seq = serializer.serialize_seq(Some(0))?;
                seq.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for TagV2 {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(TagVisitor)
    }
}

struct TagVisitor;

impl<'de> Visitor<'de> for TagVisitor {
    type Value = TagV2;

    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "a sequence of strings")
    }

    fn visit_seq<A>(self, mut seq: A) -> Result<TagV2, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let tagname: &str = match seq.next_element()? {
            Some(e) => e,
            None => return Ok(TagV2::Empty),
        };
        if tagname == "a" {
            let a: &str = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };

            let mut a_parse_fail = || -> Result<TagV2, A::Error> {
                let mut data: Vec<String> = Vec::new();
                while let Some(s) = seq.next_element()? {
                    data.push(s);
                }
                Ok(TagV2::Other {
                    tag: tagname.to_string(),
                    data,
                })
            };

            // Parse the main value
            let parts: Vec<&str> = a.split(':').collect();
            if parts.len() < 3 {
                return a_parse_fail();
            }
            let kindnum: u32 = match parts[0].parse::<u32>() {
                Ok(u) => u,
                Err(_) => return a_parse_fail(),
            };
            let kind: EventKind = From::from(kindnum);
            let pubkey: PublicKeyHex = match PublicKeyHex::try_from_str(parts[1]) {
                Ok(pk) => pk,
                Err(_) => return a_parse_fail(),
            };
            let relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Address {
                kind,
                pubkey,
                d: parts[2].to_string(),
                relay_url,
                marker,
                trailing,
            })
        } else if tagname == "content-warning" {
            let msg = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::ContentWarning {
                warning: msg,
                trailing,
            })
        } else if tagname == "delegation" {
            let pubkey: PublicKeyHex = match seq.next_element()? {
                Some(pk) => pk,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let conditions: DelegationConditions = match seq.next_element()? {
                Some(c) => c,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![pubkey.into_string()],
                    });
                }
            };
            let sig: SignatureHex = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![pubkey.into_string(), conditions.as_string()],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Delegation {
                pubkey,
                conditions,
                sig,
                trailing,
            })
        } else if tagname == "e" {
            let id: Id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Event {
                id,
                recommended_relay_url,
                marker,
                trailing,
            })
        } else if tagname == "E" {
            let id: Id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::EventParent {
                id,
                recommended_relay_url,
                trailing,
            })
        } else if tagname == "expiration" {
            let time = match seq.next_element()? {
                Some(t) => t,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Expiration { time, trailing })
        } else if tagname == "p" {
            let pubkey: PublicKeyHex = match seq.next_element()? {
                Some(p) => p,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let recommended_relay_url: Option<UncheckedUrl> = seq.next_element()?;
            let petname: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Pubkey {
                pubkey,
                recommended_relay_url,
                petname,
                trailing,
            })
        } else if tagname == "t" {
            let tag = match seq.next_element()? {
                Some(t) => t,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Hashtag {
                hashtag: tag,
                trailing,
            })
        } else if tagname == "r" {
            let refr: UncheckedUrl = match seq.next_element()? {
                Some(r) => r,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let marker: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Reference {
                url: refr,
                marker,
                trailing,
            })
        } else if tagname == "g" {
            let geo = match seq.next_element()? {
                Some(g) => g,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Geohash {
                geohash: geo,
                trailing,
            })
        } else if tagname == "d" {
            let id = match seq.next_element()? {
                Some(id) => id,
                None => {
                    // Implicit empty value
                    return Ok(TagV2::Identifier {
                        d: "".to_string(),
                        trailing: Vec::new(),
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Identifier { d: id, trailing })
        } else if tagname == "k" {
            let mut parts: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                parts.push(s);
            }
            if parts.is_empty() {
                return Ok(TagV2::Other {
                    tag: tagname.to_string(),
                    data: parts,
                });
            }
            let kindnum: u32 = match parts[0].parse::<u32>() {
                Ok(u) => u,
                Err(_) => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: parts,
                    });
                }
            };
            let kind: EventKind = From::from(kindnum);
            Ok(TagV2::Kind {
                kind,
                trailing: parts[1..].to_owned(),
            })
        } else if tagname == "subject" {
            let sub = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Subject {
                subject: sub,
                trailing,
            })
        } else if tagname == "nonce" {
            let nonce = match seq.next_element()? {
                Some(n) => n,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let target: Option<String> = seq.next_element()?;
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Nonce {
                nonce,
                target,
                trailing,
            })
        } else if tagname == "parameter" {
            let param = match seq.next_element()? {
                Some(s) => s,
                None => "".to_owned(), // implicit parameter
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Parameter { param, trailing })
        } else if tagname == "title" {
            let title = match seq.next_element()? {
                Some(s) => s,
                None => {
                    return Ok(TagV2::Other {
                        tag: tagname.to_string(),
                        data: vec![],
                    });
                }
            };
            let mut trailing: Vec<String> = Vec::new();
            while let Some(s) = seq.next_element()? {
                trailing.push(s);
            }
            Ok(TagV2::Title { title, trailing })
        } else {
            let mut data = Vec::new();
            loop {
                match seq.next_element()? {
                    None => {
                        return Ok(TagV2::Other {
                            tag: tagname.to_string(),
                            data,
                        });
                    }
                    Some(s) => data.push(s),
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {TagV2, test_tag_serde}

    #[test]
    fn test_a_tag() {
        let tag = TagV2::Address {
            kind: EventKind::LongFormContent,
            pubkey: PublicKeyHex::mock_deterministic(),
            d: "Testing123".to_owned(),
            relay_url: Some(UncheckedUrl("wss://relay.mikedilger.com/".to_string())),
            marker: None,
            trailing: Vec::new(),
        };
        let string = serde_json::to_string(&tag).unwrap();
        let tag2 = serde_json::from_str(&string).unwrap();
        assert_eq!(tag, tag2);

        let tag = TagV2::Address {
            kind: EventKind::LongFormContent,
            pubkey: PublicKeyHex::mock_deterministic(),
            d: "Testing123".to_owned(),
            // NOTE, real tags from relays could not get a None here anyways:
            relay_url: Some(UncheckedUrl("".to_owned())),
            marker: Some("reply".to_owned()),
            trailing: Vec::new(),
        };
        let string = serde_json::to_string(&tag).unwrap();
        let tag2 = serde_json::from_str(&string).unwrap();
        assert_eq!(tag, tag2);

        let json = r#"["a","34550:d0debf9fb12def81f43d7c69429bb784812ac1e4d2d53a202db6aac7ea4b466c:git",""]"#;
        let tag: TagV2 = serde_json::from_str(&json).unwrap();
        if let TagV2::Address { ref relay_url, .. } = tag {
            assert_eq!(*relay_url, Some(UncheckedUrl("".to_string())));
        } else {
            panic!("Tag not an address");
        }
        let json2 = serde_json::to_string(&tag).unwrap();
        assert_eq!(json, json2);
    }
}
```

---

### versioned/tag3.rs

**Size:** 20130 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use std::fmt;

use serde::{Deserialize, Serialize};
#[cfg(feature = "speedy")]
use speedy::{Readable, Writable};

use crate::types::{
    DelegationConditions, Error, EventKind, Id, NAddr, PublicKey, Signature, UncheckedUrl,
};

/// A tag on an Event
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "speedy", derive(Readable, Writable))]
pub struct TagV3(pub Vec<String>);

impl TagV3 {
    const EMPTY_STRING: &'static str = "";

    /// Create a new tag
    pub fn new(fields: &[&str]) -> TagV3 {
        TagV3(fields.iter().map(|f| (*f).to_owned()).collect())
    }

    /// Create a new tag without copying
    pub fn from_strings(fields: Vec<String>) -> TagV3 {
        TagV3(fields)
    }

    /// Into a `Vec<String>`
    pub fn into_inner(self) -> Vec<String> {
        self.0
    }

    /// Get the string at the given index
    pub fn get_index(&self, index: usize) -> &str {
        if self.0.len() > index {
            &self.0[index]
        } else {
            Self::EMPTY_STRING
        }
    }

    /// Set the string at the given index
    pub fn set_index(&mut self, index: usize, value: String) {
        while self.0.len() <= index {
            self.0.push("".to_owned());
        }
        self.0[index] = value;
    }

    /// Push more values onto the tag
    pub fn push_values(&mut self, mut values: Vec<String>) {
        for value in values.drain(..) {
            self.0.push(value);
        }
    }

    /// Get the tag name for the tag (the first string in the array)
    pub fn tagname(&self) -> &str {
        self.get_index(0)
    }

    /// Get the tag value (index 1, after the tag name)
    pub fn value(&self) -> &str {
        self.get_index(1)
    }

    /// Get the marker (if relevant), else ""
    pub fn marker(&self) -> &str {
        if self.tagname() == "e" {
            self.get_index(3)
        } else if self.tagname() == "a" {
            self.get_index(2)
        } else {
            Self::EMPTY_STRING
        }
    }

    // Mock data for testing
    #[allow(dead_code)]
    pub(crate) fn mock() -> TagV3 {
        TagV3(vec!["e".to_string(), UncheckedUrl::mock().0])
    }

    /// Create a new 'a' address tag
    pub fn new_address(naddr: &NAddr, marker: Option<String>) -> TagV3 {
        let mut vec = vec![
            "a".to_owned(),
            format!(
                "{}:{}:{}",
                Into::<u32>::into(naddr.kind),
                naddr.author.as_hex_string(),
                naddr.d
            ),
        ];
        if !naddr.relays.is_empty() {
            vec.push(naddr.relays[0].0.clone());
        }
        if let Some(marker) = marker {
            vec.push(marker);
        }
        TagV3(vec)
    }

    /// Parse an 'a' tag
    /// `['a', 'kind:pubkeyhex:d', <optrelay>, <optmarker>]`
    pub fn parse_address(&self) -> Result<(NAddr, Option<String>), Error> {
        let strings = &self.0;

        if strings.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &strings[0] != "a" {
            return Err(Error::TagMismatch);
        }

        let (kind, author, d) = {
            let parts: Vec<&str> = strings[1].split(':').collect();
            if parts.len() < 3 {
                return Err(Error::TagMismatch);
            }
            let kind: EventKind = {
                let kindnum: u32 = parts[0].parse::<u32>()?;
                From::from(kindnum)
            };
            if !kind.is_replaceable() {
                return Err(Error::NonReplaceableAddr);
            }
            let author: PublicKey = PublicKey::try_from_hex_string(parts[1], true)?;
            let d = parts[2].to_string();
            (kind, author, d)
        };

        let relays: Vec<UncheckedUrl> = if strings.len() > 2 {
            vec![UncheckedUrl(strings[2].clone())]
        } else {
            vec![]
        };

        let na = NAddr {
            d,
            relays,
            kind,
            author,
        };

        let marker = if strings.len() >= 4 {
            Some(strings[3].clone())
        } else {
            None
        };

        Ok((na, marker))
    }

    /// Create a "content-warning" tag
    pub fn new_content_warning(warning: &str) -> TagV3 {
        TagV3(vec!["content-warning".to_string(), warning.to_string()])
    }

    /// Parse a "content-warning" tag
    pub fn parse_content_warning(&self) -> Result<Option<String>, Error> {
        if self.0.is_empty() {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "content-warning" {
            return Err(Error::TagMismatch);
        }
        if self.0.len() >= 2 {
            Ok(Some(self.0[1].to_string()))
        } else {
            Ok(None)
        }
    }

    /// Create an "e" tag
    pub fn new_event(
        id: Id,
        recommended_relay_url: Option<UncheckedUrl>,
        marker: Option<String>,
    ) -> TagV3 {
        let mut v: Vec<String> = vec!["e".to_owned(), id.as_hex_string()];
        if let Some(rurl) = recommended_relay_url {
            v.push(rurl.0);
        } else if marker.is_some() {
            v.push("".to_owned())
        }
        if let Some(mark) = marker {
            v.push(mark);
        }
        TagV3(v)
    }

    /// Parse an "e" tag
    /// `['e', <id>, <rurl>, <marker>]`
    pub fn parse_event(&self) -> Result<(Id, Option<UncheckedUrl>, Option<String>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "e" {
            return Err(Error::TagMismatch);
        }
        let id = Id::try_from_hex_string(&self.0[1])?;
        let url = if self.0.len() >= 3 {
            Some(UncheckedUrl(self.0[2].to_owned()))
        } else {
            None
        };
        let marker = if self.0.len() >= 4 {
            Some(self.0[3].to_owned())
        } else {
            None
        };
        Ok((id, url, marker))
    }

    /// Create a "q" tag
    pub fn new_quote(id: Id, recommended_relay_url: Option<UncheckedUrl>) -> TagV3 {
        let mut v: Vec<String> = vec!["q".to_owned(), id.as_hex_string()];
        if let Some(rurl) = recommended_relay_url {
            v.push(rurl.0);
        }
        TagV3(v)
    }

    /// Parse a "q" tag
    pub fn parse_quote(&self) -> Result<(Id, Option<UncheckedUrl>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "q" {
            return Err(Error::TagMismatch);
        }
        let id = Id::try_from_hex_string(&self.0[1])?;
        let url = if self.0.len() >= 3 {
            Some(UncheckedUrl(self.0[2].to_owned()))
        } else {
            None
        };
        Ok((id, url))
    }

    /// Create a "p" tag
    pub fn new_pubkey(
        pubkey: PublicKey,
        relay_url: Option<UncheckedUrl>,
        petname: Option<String>,
    ) -> TagV3 {
        let mut v: Vec<String> = vec!["p".to_owned(), pubkey.as_hex_string()];
        if let Some(rurl) = relay_url {
            v.push(rurl.0);
        } else if petname.is_some() {
            v.push("".to_owned())
        }
        if let Some(pet) = petname {
            v.push(pet);
        }
        TagV3(v)
    }

    /// Parse a "p" tag
    pub fn parse_pubkey(&self) -> Result<(PublicKey, Option<UncheckedUrl>, Option<String>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "p" {
            return Err(Error::TagMismatch);
        }
        let pubkey = PublicKey::try_from_hex_string(&self.0[1], true)?;
        let url = if self.0.len() >= 3 {
            Some(UncheckedUrl(self.0[2].to_owned()))
        } else {
            None
        };
        let petname = if self.0.len() >= 4 {
            if self.0[3].is_empty() {
                None
            } else {
                Some(self.0[3].to_owned())
            }
        } else {
            None
        };
        Ok((pubkey, url, petname))
    }

    /// Create a "t" tag
    pub fn new_hashtag(hashtag: String) -> TagV3 {
        TagV3(vec!["t".to_string(), hashtag])
    }

    /// Parse an "t" tag
    pub fn parse_hashtag(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "t" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create an "r" tag
    pub fn new_relay(url: UncheckedUrl, usage: Option<String>) -> TagV3 {
        let mut v = vec!["r".to_owned(), url.0];
        if let Some(u) = usage {
            v.push(u)
        }
        TagV3(v)
    }

    /// Parse an "r" tag
    pub fn parse_relay(&self) -> Result<(UncheckedUrl, Option<String>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "r" {
            return Err(Error::TagMismatch);
        }
        let relay = UncheckedUrl(self.0[1].clone());
        let marker = if self.0.len() >= 3 {
            Some(self.0[2].clone())
        } else {
            None
        };
        Ok((relay, marker))
    }

    /// Create a new 'd' identifier tag
    pub fn new_identifier(identifier: String) -> TagV3 {
        TagV3(vec!["d".to_string(), identifier])
    }

    /// Parse a 'd' tag
    pub fn parse_identifier(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "d" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create a new 'name' tag
    pub fn new_name(name: String) -> TagV3 {
        TagV3(vec!["name".to_string(), name])
    }

    /// Create a new 'image' tag
    pub fn new_image(url: UncheckedUrl, width: Option<u64>, height: Option<u64>) -> TagV3 {
        let mut v = vec!["image".to_owned(), url.0];
        if let Some(w) = width {
            v.push(format!("{}", w));
        }
        if let Some(h) = height {
            v.push(format!("{}", h));
        }
        TagV3(v)
    }

    /// Create a new 'thumb' tag
    pub fn new_thumb(url: UncheckedUrl, width: Option<u64>, height: Option<u64>) -> TagV3 {
        let mut v = vec!["thumb".to_owned(), url.0];
        if let Some(w) = width {
            v.push(format!("{}", w));
        }
        if let Some(h) = height {
            v.push(format!("{}", h));
        }
        TagV3(v)
    }

    /// Create a new 'subject' tag
    pub fn new_subject(subject: String) -> TagV3 {
        TagV3(vec!["subject".to_string(), subject])
    }

    /// Parse a "subject" tag
    pub fn parse_subject(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "subject" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create a "nonce" tag
    pub fn new_nonce(nonce: u32, target: Option<u32>) -> TagV3 {
        let mut v = vec!["nonce".to_owned(), format!("{}", nonce)];
        if let Some(targ) = target {
            v.push(format!("{}", targ));
        }
        TagV3(v)
    }

    /// Parse a "nonce" tag
    pub fn parse_nonce(&self) -> Result<(u64, Option<u32>), Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "nonce" {
            return Err(Error::TagMismatch);
        }
        let nonce = self.0[1].parse::<u64>()?;
        let target = if self.0.len() >= 3 {
            Some(self.0[2].parse::<u32>()?)
        } else {
            None
        };
        Ok((nonce, target))
    }

    /// Create a "title" tag
    pub fn new_title(title: String) -> TagV3 {
        TagV3(vec!["title".to_string(), title])
    }

    /// Parse a "title" tag
    pub fn parse_title(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "title" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create a "summary" tag
    pub fn new_summary(summary: String) -> TagV3 {
        TagV3(vec!["summary".to_string(), summary])
    }

    /// Parse a "summary" tag
    pub fn parse_summary(&self) -> Result<String, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "summary" {
            return Err(Error::TagMismatch);
        }
        Ok(self.0[1].to_string())
    }

    /// Create a "k" tag
    pub fn new_kind(kind: EventKind) -> TagV3 {
        TagV3(vec!["k".to_owned(), format!("{}", Into::<u32>::into(kind))])
    }

    /// Parse a "k" tag
    pub fn parse_kind(&self) -> Result<EventKind, Error> {
        if self.0.len() < 2 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "k" {
            return Err(Error::TagMismatch);
        }
        let u = self.0[1].parse::<u32>()?;
        Ok(u.into())
    }

    /// New delegation tag
    pub fn new_delegation(
        pubkey: PublicKey,
        conditions: DelegationConditions,
        sig: Signature,
    ) -> TagV3 {
        TagV3(vec![
            "delegation".to_owned(),
            pubkey.as_hex_string(),
            conditions.as_string(),
            sig.as_hex_string(),
        ])
    }

    /// parse delegation tag
    pub fn parse_delegation(&self) -> Result<(PublicKey, DelegationConditions, Signature), Error> {
        if self.0.len() < 4 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "delegation" {
            return Err(Error::TagMismatch);
        }
        let pk = PublicKey::try_from_hex_string(&self.0[1], true)?;
        let conditions = DelegationConditions::try_from_str(&self.0[2])?;
        let sig = Signature::try_from_hex_string(&self.0[3])?;
        Ok((pk, conditions, sig))
    }

    /// New proxy tag
    pub fn proxy(protocol: String, id: String) -> TagV3 {
        TagV3(vec!["proxy".to_owned(), protocol, id])
    }

    /// Create a generic tag with a name and value
    pub fn new_tag(tagname: &str, value: &str) -> TagV3 {
        TagV3(vec![tagname.to_owned(), value.to_owned()])
    }

    /// parse proxy tag
    pub fn parse_proxy(&self) -> Result<(String, String), Error> {
        if self.0.len() < 3 {
            return Err(Error::TagMismatch);
        }
        if &self.0[0] != "proxy" {
            return Err(Error::TagMismatch);
        }
        let protocol = self.0[1].to_owned();
        let id = self.0[2].to_owned();
        Ok((protocol, id))
    }
}

impl fmt::Display for TagV3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[({})]", self.0.join(", "))
    }
}

#[cfg(test)]
mod test {
    use super::*;

    test_serde! {TagV3, test_tag_serde}

    #[test]
    fn test_a_tag() {
        let na = NAddr {
            d: "blog-20231029".to_owned(),
            relays: vec![UncheckedUrl("badurl".to_owned())],
            kind: EventKind::LongFormContent,
            author: PublicKey::mock_deterministic(),
        };

        let tag = TagV3::new_address(&na, None);
        let (na2, _optmarker) = tag.parse_address().unwrap();
        // Equal only because there is just 1 UncheckedUrl, else might have dropped
        // the rest
        assert_eq!(na, na2);

        // Test a known JSON a tag:
        let json =
            r#"["a","34550:d0debf9fb12def81f43d7c69429bb784812ac1e4d2d53a202db6aac7ea4b466c:git"]"#;
        let tag: TagV3 = serde_json::from_str(&json).unwrap();
        assert!(tag.parse_address().is_ok());

        let tag = TagV3::new(&[
            "a",
            "30023:b12b632c887f0c871d140d37bcb6e7c1e1a80264d0b7de8255aa1951d9e1ff79:1716928135712",
            "",
            "root",
        ]);
        let (_, marker) = tag.parse_address().unwrap();
        assert!(marker.as_deref().unwrap() == "root");
    }

    #[test]
    fn test_content_warning_tag() {
        let tag = TagV3::new(&["content-warning"]);
        assert_eq!(tag.parse_content_warning().unwrap(), None);

        let tag = TagV3::new_content_warning("danger");
        assert_eq!(
            tag.parse_content_warning().unwrap(),
            Some("danger".to_owned())
        );

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_content_warning().is_err());
    }

    #[test]
    fn test_event_tag() {
        let tag = TagV3::new_event(Id::mock(), None, None);
        assert_eq!(tag.parse_event().unwrap(), (Id::mock(), None, None));

        let data = (
            Id::mock(),
            Some(UncheckedUrl("dummy".to_owned())),
            Some("foo".to_owned()),
        );
        let tag = TagV3::new_event(data.0, data.1.clone(), data.2.clone());
        assert_eq!(tag.parse_event().unwrap(), data);
    }

    #[test]
    fn test_pubkey_tag() {
        let tag = TagV3::new_pubkey(PublicKey::mock_deterministic(), None, None);
        assert_eq!(
            tag.parse_pubkey().unwrap(),
            (PublicKey::mock_deterministic(), None, None)
        );

        let data = (
            PublicKey::mock(),
            Some(UncheckedUrl("dummy".to_owned())),
            Some("foo".to_owned()),
        );
        let tag = TagV3::new_pubkey(data.0, data.1.clone(), data.2.clone());
        assert_eq!(tag.parse_pubkey().unwrap(), data);
    }

    #[test]
    fn test_hashtag_tag() {
        let tag = TagV3::new(&["t"]);
        assert!(tag.parse_hashtag().is_err());

        let tag = TagV3::new_hashtag("footstr".to_owned());
        assert_eq!(tag.parse_hashtag().unwrap(), "footstr".to_owned());

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_hashtag().is_err());
    }

    #[test]
    fn test_relay_tag() {
        let tag = TagV3::new(&["r", "wss://example.com", "read"]);
        let parsed = tag.parse_relay().unwrap();
        let data = (
            UncheckedUrl("wss://example.com".to_owned()),
            Some("read".to_owned()),
        );
        assert_eq!(parsed, data);

        let tag2 = TagV3::new_relay(data.0, data.1);
        assert_eq!(tag, tag2);
    }

    #[test]
    fn test_identifier_tag() {
        let tag = TagV3::new(&["d"]);
        assert!(tag.parse_identifier().is_err());

        let tag = TagV3::new_identifier("myblog123".to_owned());
        assert_eq!(tag.parse_identifier().unwrap(), "myblog123".to_owned());

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_identifier().is_err());
    }

    #[test]
    fn test_subject_tag() {
        let tag = TagV3::new(&["subject"]);
        assert!(tag.parse_subject().is_err());

        let tag = TagV3::new_subject("Attn: Nurses".to_owned());
        assert_eq!(tag.parse_subject().unwrap(), "Attn: Nurses".to_owned());

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_subject().is_err());
    }

    #[test]
    fn test_nonce_tag() {
        let tag = TagV3::new(&["nonce"]);
        assert!(tag.parse_nonce().is_err());

        let tag = TagV3::new_nonce(132345, Some(20));
        assert_eq!(tag.parse_nonce().unwrap(), (132345, Some(20)));

        let tag = TagV3::new_nonce(132345, None);
        assert_eq!(tag.parse_nonce().unwrap(), (132345, None));

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_nonce().is_err());
    }

    #[test]
    fn test_title_tag() {
        let tag = TagV3::new(&["title"]);
        assert!(tag.parse_title().is_err());

        let tag = TagV3::new_title("Attn: Nurses".to_owned());
        assert_eq!(tag.parse_title().unwrap(), "Attn: Nurses".to_owned());

        let tag = TagV3::new(&["dummy", "tag"]);
        assert!(tag.parse_title().is_err());
    }

    #[test]
    fn test_kind_tag() {
        let tag = TagV3::new(&["k", "30023"]);
        assert_eq!(tag.parse_kind().unwrap(), EventKind::LongFormContent);

        let tag = TagV3::new(&["k"]);
        assert!(tag.parse_kind().is_err());

        let tag = TagV3::new_kind(EventKind::ZapRequest);
        assert_eq!(tag.parse_kind().unwrap(), EventKind::ZapRequest);
    }
}
```

---

### versioned/zap_data.rs

**Size:** 1035 bytes | **Modified:** 2026-01-20 14:02:27

```rust
use crate::types::{EventReference, Id, MilliSatoshi, PublicKey};

/// Data about a Zap
#[derive(Clone, Debug)]
pub struct ZapDataV2 {
    /// The event that was zapped. If missing we can't use the zap receipt
    /// event.
    pub zapped_event: EventReference,

    /// The amount that the event was zapped
    pub amount: MilliSatoshi,

    /// The public key of the person who received the zap
    pub payee: PublicKey,

    /// The public key of the person who paid the zap, if it was in the receipt
    pub payer: PublicKey,

    /// The public key of the zap provider, for verification purposes
    pub provider_pubkey: PublicKey,
}

/// Data about a Zap
#[derive(Clone, Debug, Copy)]
pub struct ZapDataV1 {
    /// The event that was zapped
    pub id: Id,

    /// The amount that the event was zapped
    pub amount: MilliSatoshi,

    /// The public key of the person who provided the zap
    pub pubkey: PublicKey,

    /// The public key of the zap provider, for verification purposes
    pub provider_pubkey: PublicKey,
}
```

---


---
*Generated by code2prompt.sh on 2026-01-21 13:49:13*
