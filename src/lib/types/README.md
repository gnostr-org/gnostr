# Nostr Types Module (`gnostr/src/lib/types`)

This module provides a comprehensive set of types and utilities for handling the Nostr protocol. It aims to offer a robust and extensible foundation for building Nostr-compatible applications, covering event structures, client-relay communication, cryptographic elements, and NIP-specific implementations.

## Overview

The `types` module is designed to encapsulate all data structures and related logic essential for interacting with the Nostr network. It abstracts away the complexities of serialization, deserialization, and event validation, allowing developers to focus on application-level logic.

## Key Components

### Core Event Structures

*   **`Event`**: Represents a Nostr event, including its ID, public key, creation timestamp, kind, tags, content, and signature.
*   **`PreEvent`**: A precursor to `Event`, used for signing, containing all event data except the signature itself.
*   **`Rumor`**: Represents an event received from a relay before full validation.
*   **`EventKind`**: An enum defining the various types of Nostr events (e.g., TextNote, RecommendRelay, ChannelCreation).
*   **`Tag`**: Represents a generic Nostr tag, used to attach metadata or references to events.
*   **`Id`**: Represents the unique identifier of an event.
*   **`PublicKey` / `PrivateKey`**: Cryptographic keys used for Nostr identities and event signing.
*   **`Signature`**: The cryptographic signature of an event.
*   **`Unixtime`**: A timestamp type for Nostr events.

### Client-Relay Communication

*   **`ClientMessage`**: Messages sent from a client to a Nostr relay (e.g., publishing events, subscribing to filters).
*   **`RelayMessage`**: Messages received from a Nostr relay (e.g., events, EOSE messages, notices).
*   **`Filter`**: Used by clients to request specific events from relays based on various criteria.
*   **`SubscriptionId`**: A unique identifier for a client's subscription to a relay.

### NIP-Specific Implementations

The module includes specific implementations and types related to various Nostr Improvement Proposals (NIPs):

*   **NIP-00 (`nip0`)**: Basic protocol definitions.
*   **NIP-02 (`nip2`)**: Contact List and Petnames.
*   **NIP-04 (`nip4`)**: Encrypted Direct Message.
*   **NIP-05 (`nip05`)**: Mapping Nostr keys to DNS-based internet identifiers.
*   **NIP-06 (`nip6`)**: Basic key derivation from mnemonic seed phrase.
*   **NIP-09 (`nip9`)**: Event Deletion.
*   **NIP-15 (`nip15`)**: End of Stored Events Notice.
*   **NIP-26 (`nip26`)**: Delegation.
*   **NIP-28 (`nip28`)**: Public Chat Channels (includes `ChannelCreationEvent`, `ChannelMetadataEvent`, `create_channel`, `set_channel_metadata`, `create_channel_message`, `hide_message`, `mute_user` and their parsing counterparts).
*   **NIP-34 (`nip34`)**: Git notes integration.
*   **NIP-44 (`nip44`)**: Encrypted content for secure direct messages.

### Utility Types

*   **`Url` / `UncheckedUrl`**: Types for handling URLs.
*   **`NostrUrl` / `NostrBech32`**: Utilities for Nostr-specific URI schemes and Bech32 encoding.
*   **`Metadata`**: Event content for Kind 0 (profile metadata).
*   **`Profile`**: Represents a user profile with metadata.
*   **`RelayInformationDocument`**: Structure for relay information (NIP-11).
*   **`KeySigner`**: Concrete implementation of the `Signer` trait.
*   **`IntoVec` trait**: A generic trait for converting `Option<T>` into `Vec<T>`.
*   **`add_pubkey_to_tags`, `add_event_to_tags`, `add_addr_to_tags`, `add_subject_to_tags_if_missing`**: Helper functions for managing event tags.
*   **`get_leading_zero_bits`**: Utility function.

### Versioned Types

The module also provides versioned representations of core Nostr types, allowing for compatibility with different protocol iterations. These include `ClientMessageV1`, `EventV1`, `TagV1`, etc., up to their latest versions (`V3`, `V4`, `V5` where applicable).

## Usage

Developers can import and utilize these types to:

*   Construct and sign Nostr events.
*   Parse incoming messages from Nostr relays.
*   Implement NIP-specific functionalities like chat channels or encrypted DMs.
*   Manage user profiles and relay information.

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