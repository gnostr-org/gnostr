# Nostr Types Module (`gnostr/src/lib/types`)

This module provides a comprehensive, well-organized set of types and utilities for handling the Nostr protocol. It offers a robust and extensible foundation for building Nostr-compatible applications, with clear separation of concerns and comprehensive error handling.

## Overview

The `types` module is designed to encapsulate all data structures and related logic essential for interacting with the Nostr network. It abstracts away the complexities of serialization, deserialization, and event validation, allowing developers to focus on application-level logic.

## Architecture

The module is organized into several logical categories:

### Core Protocol Types
- **Event & EventBuilder**: Core Nostr event structures and builders
- **PreEvent**: Events before signing
- **Id, PublicKey, PrivateKey**: Cryptographic identifiers
- **Signature**: Event signatures
- **Tag**: Event metadata and references
- **Unixtime**: Timestamp handling

### Communication Types
- **ClientMessage**: Messages from client to relay
- **RelayMessage**: Messages from relay to client
- **Filter**: Event subscription filters
- **SubscriptionId**: Subscription identifiers

### NIP Implementations
The module includes comprehensive support for various Nostr Improvement Proposals (NIPs), organized logically:

#### Core NIPs (Basic Protocol)
- `nip0`: Basic protocol definitions
- `nip2`: Contact List and Petnames
- `nip3`: OpenTimestamps Attestations
- `nip4`: Encrypted Direct Messages

#### Identity & Verification
- `nip5`: DNS-based identifier verification
- `nip6`: Mnemonic seed phrase key derivation
- `nip9`: Event Deletion

#### Content & Structure
- `nip10`: Text Notes and Threads
- `nip13`: Proof of Work
- `nip18`: Reposts
- `nip25`: Reactions
- `nip28`: Public Chat Channels

#### Advanced Features
- `nip44`: Enhanced encrypted content
- `nip53`: Live Activities
- `nip59`: Gift Wrap
- `nip94`: File metadata

#### Encoding & Sharing
- `nip19`: bech32-encoded entities

### Versioned Types
For backwards compatibility, the module maintains versioned representations of core types:
- `EventV1`, `EventV2`, `EventV3`
- `ClientMessageV1`, `ClientMessageV2`, `ClientMessageV3`
- And more...

### Error Handling
The module provides comprehensive error handling with:
- Specific error types for different operations
- Contextual error messages
- Proper error propagation chains

## Usage Examples

### Creating a Simple Text Note

```rust
use gnostr_types::{
    EventBuilder, EventKind, KeySigner, PrivateKey, Tag, Unixtime
};

// Create a signer
let private_key = PrivateKey::generate()?;
let signer = KeySigner::from_private_key(private_key, "", 1)?;

// Create and sign an event
let event = EventBuilder::new(signer.clone())
    .content("Hello, Nostr!")
    .kind(EventKind::TextNote)
    .build()?;

println!("Created event: {}", event.id.as_hex_string());
```

### Working with Git Integration

```rust
use gnostr_legit::{
    GitRepository, GitEventManager, derive_keys_from_commit_hash
};

// Discover and work with a git repository
let repo = GitRepository::discover(".")?;
let commit = repo.get_head_commit()?;
let commit_info = commit.extract_info()?;

// Derive keys from commit hash
let signer = derive_keys_from_commit_hash(&commit_info.id)?;

// Create events from commit data
let mut manager = GitEventManager::new(signer)?;
let event = manager.publish_commit_event(&commit_info).await?;
```

### Advanced Event with Custom Tags

```rust
use std::collections::HashMap;
use gnostr_legit::EventBuilder;

let mut custom_tags = HashMap::new();
custom_tags.insert("category".to_string(), vec!["technology".to_string()]);
custom_tags.insert("language".to_string(), vec!["rust".to_string()]);

let event = EventBuilder::new(signer)
    .content("Discussion about Rust programming")
    .add_tag("t", vec!["programming".to_string()])
    .add_tag("subject", vec!["Rust".to_string()])
    .build()?;
```

## Best Practices

### Error Handling
Always handle errors properly and provide context:

```rust
match repo.get_head_commit() {
    Ok(commit) => {
        // Process commit
    }
    Err(e) => {
        eprintln!("Failed to get HEAD commit: {}", e);
        return Err(e.into());
    }
}
```

### Content Validation
Validate and sanitize content before creating events:

```rust
use gnostr_legit::{sanitize_content, validate_content_length};

let content = sanitize_content(user_input);
validate_content_length(&content)?;

// Now it's safe to create the event
```

### Async Operations
Use proper async patterns for network operations:

```rust
let mut manager = GitEventManager::new(signer).await?;
manager.publisher_mut().connect_to_relays().await?;
let events = manager.publish_commit_event(&commit_info).await?;
```

## Testing

The module includes comprehensive test coverage. Run tests with:

```bash
cargo test --package gnostr --lib types
cargo test --package gnostr --lib legit
```

## Contributing

When contributing to this module:

1. **Follow the existing architecture** - Keep related functionality together
2. **Add comprehensive tests** - Include both unit and integration tests
3. **Document all public APIs** - Use rustdoc with examples
4. **Handle errors properly** - Use the established error types
5. **Maintain backwards compatibility** - Use versioned types when needed

## Migration Guide

### From Legacy Code

If you're migrating from older versions:

1. **Replace direct event creation** with `EventBuilder`
2. **Use the new error types** instead of generic errors
3. **Leverage the modular structure** - Import specific modules you need
4. **Update async patterns** - Use the new structured concurrency

### Example Migration

**Before:**
```rust
// Old way - lots of manual code
let mut tags = Vec::new();
tags.push(Tag::new(&["category", "tech"]));
let pre_event = PreEvent { /* manual setup */ };
let event = sign_event(pre_event, &signer)?;
```

**After:**
```rust
// New way - clean and safe
let event = EventBuilder::new(signer)
    .content("My content")
    .add_tag("category", vec!["tech"])
    .build()?;
```

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
