# types

This directory contains the core data structures and type definitions used throughout the `gnostr` project, primarily focused on the Nostr protocol. It defines various entities and messages essential for Nostr communication and data handling.

Key categories of types include:

- **Events**: Definitions for Nostr events, including different versions to support protocol evolution.
- **Messages**: Structures for client-to-relay and relay-to-client messages.
- **Identifiers**: Types for various identifiers like event IDs, public keys, and subscription IDs.
- **Tags**: Structures for event tags.
- **Relay Information**: Types related to relay metadata and usage.
- **Private Key Management**: Definitions for private key handling, including encryption.
- **NIPs**: Implementations of various Nostr Improvement Proposals (NIPs) as type definitions (e.g., NIP05).

The `versioned` subdirectory indicates support for different versions of these core Nostr types, allowing for compatibility with various stages of the Nostr protocol.
