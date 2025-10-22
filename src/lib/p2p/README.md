# P2P Module

This module contains functionalities related to peer-to-peer (P2P) networking, enabling decentralized communication and data sharing. It includes implementations for chat applications, key-value stores, and other P2P-related services.

## Purpose

The P2P module provides the core networking capabilities for decentralized applications. It leverages libraries like `libp2p` to establish connections, discover peers, and manage communication protocols. This module is foundational for features like real-time chat, distributed data storage, and other decentralized services.

## Key Features

*   **Peer-to-Peer Networking:** Implements robust P2P communication using `libp2p`, including:
    *   **Gossipsub:** For efficient message broadcasting and pub/sub messaging.
    *   **mDNS:** For local peer discovery.
    *   **Noise:** For secure peer connections.
    *   **Yamux:** For multiplexing streams over a single connection.
    *   **TCP & QUIC:** For transport layer protocols.
*   **Chat Application:** The `chat` submodule provides a full-featured TUI-based P2P chat application that integrates with Git commits and Nostr.
*   **Key-Value Store (KVS):** The `kvs` submodule likely provides a distributed key-value store functionality using Kademlia DHT.
*   **Command-Line Interface (CLI) Tools:** Various subcommands are available for interacting with P2P functionalities, such as managing chat, querying data, and handling network configurations.

## Core Components

*   **`chat`**: Implements a P2P chat application with a TUI, integrating Git commit information and Nostr. It includes modules for message handling (`msg`), P2P networking (`p2p`), UI rendering (`ui`), and tests.
*   **`handle_input.rs`**: Handles user input for P2P commands, likely related to Kademlia operations.
*   **`kvs.rs`**: Provides the implementation for a key-value store, likely using `libp2p`'s Kademlia DHT for distributed storage and retrieval.
*   **`opt.rs`**: Defines command-line argument structures for P2P-related tools, such as the `libp2p` file sharing example.
*   **`mod.rs`**: The main module that aggregates the submodules and provides the top-level P2P functionality.

## Usage

Specific usage instructions would depend on the particular P2P feature being utilized (e.g., running the chat application, interacting with the KVS). Generally, these functionalities are accessed through CLI commands provided by the main application.

## Dependencies

*   `libp2p`: Core P2P networking stack.
*   `tokio`: Asynchronous runtime.
*   `tracing`, `tracing-subscriber`: For logging.
*   `serde`, `serde_json`: For serialization and deserialization.
*   `clap`: For command-line argument parsing.
*   `git2`: For Git integration (used in `chat`).
*   `nostr-sdk`: For Nostr integration (used in `chat`).
*   `ratatui`: For TUI rendering (used in `chat`).
*   `ureq`: For HTTP requests (used in `chat`).
*   `hostname`: For getting the hostname (used in `chat`).
*   `backtrace`: For panic handling (used in `chat`).
*   `crossbeam-channel`: For multi-threaded communication (used in `chat`).
*   `scopeguard`: For resource management (used in `chat`).
*   `scopetime`: For performance profiling (used in `chat`).
