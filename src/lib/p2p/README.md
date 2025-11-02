# P2P Module

This module contains functionalities related to peer-to-peer (P2P) networking, enabling decentralized communication and data sharing within the `gnostr` ecosystem. It provides core networking capabilities, distributed data storage, and a real-time chat application, leveraging the `libp2p` framework.

## Purpose

The P2P module serves as the foundation for decentralized applications within `gnostr`. It facilitates peer discovery, secure connections, and efficient message exchange. Key functionalities include a distributed key-value store, a P2P chat application with Git and Nostr integration, and tools for publishing Git repository information to the decentralized network.

## Key Features

*   **Robust P2P Networking:** Built on `libp2p`, offering:
    *   **Gossipsub:** For efficient publish/subscribe messaging.
    *   **Kademlia DHT:** For distributed hash table functionalities, including content routing and peer discovery.
    *   **mDNS:** For local peer discovery.
    *   **Noise & Yamux:** For secure and multiplexed connections.
    *   **TCP & QUIC:** For transport layer protocols.
*   **P2P Chat Application:** A TUI-based chat application (`chat` submodule) that integrates Git commit information and the Nostr protocol for unique messaging and identity.
*   **Distributed Key-Value Store (KVS):** Provides functionalities for storing and retrieving data in a decentralized manner using Kademlia DHT.
*   **Git Integration & Publishing:** Tools to publish Git repository data (commits, diffs, tags) to the P2P network, enabling decentralized version control information sharing.
*   **Command-Line Interface (CLI) Tools:** Various subcommands and arguments for interacting with P2P functionalities, managing network configurations, and performing Kademlia operations.
*   **Flexible Network Configuration:** Supports different P2P networks like IPFS, Kusama, Polkadot, and Ursa with configurable bootnodes.

## Core Components

*   **`args.rs`**: Defines command-line argument structures (`Args`) for configuring P2P tools and Git-related operations.
*   **`behaviour.rs`**: Implements the `NetworkBehaviour` trait, combining various `libp2p` behaviors (Gossipsub, Kademlia, mDNS, Identify, Rendezvous, Ping) into a single, cohesive network behavior for the `gnostr` P2P node.
*   **`chat/`**: A submodule dedicated to the P2P chat application.
    *   **`chat/mod.rs`**: The main entry point for the chat application, handling CLI arguments, Nostr integration, Git commit serialization/deserialization, key generation from commit hashes, and orchestrating the TUI and P2P event loop.
    *   **`chat/msg.rs`**: Defines the `Msg` struct and `MsgKind` enum for chat messages, including formatting and styling for the TUI.
    *   **`chat/p2p.rs`**: Manages the `libp2p` swarm specifically for the chat application, handling message publishing via Gossipsub and peer discovery via mDNS.
    *   **`chat/ui.rs`**: Implements the Text User Interface (TUI) for the chat application using `ratatui`, managing input, message display, and user interactions.
    *   **`chat/tests/`**: Contains unit and integration tests for the chat module.
*   **`command_handler.rs`**: Processes user input commands for interacting with the `libp2p` swarm, primarily for Kademlia DHT operations (GET, PUT, PROVIDE) and Gossipsub topic subscriptions.
*   **`event_handler.rs`**: Handles and logs various `libp2p::SwarmEvent`s, including peer discovery, Kademlia query results, and incoming Gossipsub messages.
*   **`git_integration.rs`**: Provides utility functions for interacting with local Git repositories, such as retrieving commit diffs and commit IDs from tags.
*   **`git_publisher.rs`**: Publishes Git repository information (tags, commit messages, commit diffs) to the Kademlia DHT, making it discoverable by other peers. It also subscribes to Gossipsub topics based on Git tags.
*   **`kvs.rs`**: Implements a distributed Key-Value Store using `libp2p`'s Kademlia DHT and a request-response protocol, allowing peers to store and retrieve arbitrary data.
*   **`mod.rs`**: The top-level module for `p2p`, re-exporting submodules and containing the main P2P event loop (`evt_loop`) for the `gnostr` application, which integrates with `gossipsub`, `mdns`, and fetches external data like block height and hash.
*   **`network_config.rs`**: Defines network-specific configurations, including `Network` enum for different P2P networks (e.g., IPFS, Kusama) and their respective bootnode addresses and protocols.
*   **`opt.rs`**: Defines command-line options (`Opt`) for a `libp2p` file sharing example, including subcommands for providing and getting files, and KVS interactions.
*   **`swarm_builder.rs`**: Provides a function (`build_swarm`) to construct and configure a `libp2p::Swarm` instance with all the necessary behaviors and transport settings.
*   **`utils.rs`**: Contains general utility functions, including `init_subscriber` for logging setup and `generate_ed25519` for generating `libp2p` identity keypairs.

## Usage

The functionalities within the P2P module are typically accessed through the main `gnostr` application's CLI. Depending on the specific feature, users can interact with the chat application, query the KVS, or publish Git data using appropriate commands and arguments. Refer to the `gnostr` main documentation for detailed CLI usage.

## Dependencies

*   `libp2p`: Core P2P networking stack.
*   `tokio`: Asynchronous runtime.
*   `tracing`, `tracing-subscriber`: For logging and diagnostics.
*   `serde`, `serde_json`: For serialization and deserialization of data structures.
*   `clap`: For robust command-line argument parsing.
*   `git2`: For Git repository interactions (used in `chat`, `git_integration`, `git_publisher`).
*   `nostr-sdk`: For Nostr protocol integration (used in `chat`).
*   `ratatui`: For building Text User Interfaces (used in `chat`).
*   `ureq`: For making HTTP requests (used in `chat/p2p.rs` and `p2p/mod.rs`).
*   `hostname`: For retrieving the local hostname.
*   `backtrace`: For panic handling.
*   `crossbeam-channel`: For multi-threaded communication.
*   `scopeguard`: For resource management.
*   `scopetime`: For performance profiling.
*   `chrono`: For date and time operations (used in `p2p/mod.rs`).
*   `anyhow`: For simplified error handling.
*   `once_cell`: For lazy static initialization.
*   `tui-input`: For TUI input handling (used in `chat/ui.rs`).
*   `gnostr_crawler`: For `BOOTSTRAP_RELAYS` (used in `chat/mod.rs`).
*   `gnostr_asyncgit`: For `CommitId` (used in `chat/msg.rs`).
*   `gnostr_blockhash`, `gnostr_blockheight`: For fetching block data (used in `p2p/mod.rs`).