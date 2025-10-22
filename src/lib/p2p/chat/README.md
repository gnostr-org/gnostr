# P2P Chat Module

This module implements a peer-to-peer chat application that integrates with Git commits and the Nostr protocol, providing a TUI interface for communication.

## Purpose

The `p2p/chat` module enables users to engage in real-time peer-to-peer conversations. It leverages `libp2p` for networking, `nostr-sdk` for Nostr interactions, and `git2` for incorporating Git commit information into chat messages. The application features a Text User Interface (TUI) built with `ratatui` for a rich user experience.

## Key Features

*   **Peer-to-Peer Networking:** Utilizes `libp2p` with Gossipsub for message dissemination and mDNS for peer discovery.
*   **Nostr Integration:** Generates Nostr events, signs them with keys derived from Git commit hashes, and publishes them to Nostr relays.
*   **Git Commit Integration:** Serializes and deserializes Git commit information, embedding it within chat messages and Nostr events.
*   **TUI Interface:** Provides a user-friendly command-line interface with input handling, message display, scrolling, and status indicators using `ratatui`.
*   **Command-Line Interface:** Supports starting the chat application with various configurations, including NSEC keys, topics, and logging levels.
*   **Message Formatting:** Defines message structures (`Msg`) and kinds (`MsgKind`) for different types of communication, including system messages and Git commit details, with TUI-specific styling.

## Core Components

*   **`mod.rs`**: The main module orchestrating the chat application. It handles CLI argument parsing (`ChatCli`, `ChatSubCommands`), Nostr client setup, Git integration, P2P networking initialization (`evt_loop`), TUI setup, and logging.
*   **`msg.rs`**: Defines the `Msg` struct and `MsgKind` enum for chat messages. It includes logic for formatting messages for display in the TUI, including styling for different message types and Git commit information.
*   **`p2p.rs`**: Implements the peer-to-peer networking layer using `libp2p`. It sets up the `MyBehaviour` (combining Gossipsub and mDNS), manages network connections, message publishing, and subscription to topics.
*   **`ui.rs`**: Contains the TUI rendering logic using `ratatui`. It handles the layout, input field, message display area, scrolling, and user interactions within the terminal.
*   **`tests/`**: Includes unit and integration tests to verify the functionality of the chat module, message handling, P2P communication, and Git integration.

## Usage

The chat functionality can typically be launched via a command-line interface, likely using `gnostr chat` with various arguments for configuration.

## Dependencies

*   `libp2p`: For P2P networking.
*   `nostr-sdk`: For Nostr protocol interactions.
*   `git2`: For Git repository operations.
*   `ratatui`: For building the TUI.
*   `tokio`: For asynchronous runtime.
*   `tracing`, `tracing-subscriber`: For logging.
*   `serde`, `serde_json`: For serialization and deserialization.
*   `clap`: For command-line argument parsing.
*   `once_cell`: For lazy static initialization.
*   `ureq`: For making HTTP requests (used in `async_prompt`).
*   `hostname`: For getting the hostname.
*   `backtrace`: For panic handling.
*   `crossbeam-channel`: For multi-threaded communication.
*   `scopeguard`: For ensuring resources are cleaned up.
*   `scopetime`: For performance profiling.
