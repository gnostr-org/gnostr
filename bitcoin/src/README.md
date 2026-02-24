# Gnostr Bitcoin P2P Client - Source Code Documentation

This directory contains the source code for the Gnostr Bitcoin P2P Client.

## Core Components

*   **`lib.rs`**: Implements the core Bitcoin P2P protocol logic, including message serialization/deserialization, network connection management, and handshake procedures. It defines constants for network parameters, handles VarInt encoding/decoding, and includes helper functions for building various Bitcoin P2P messages (e.g., `version`, `verack`, `ping`, `getaddr`). It also contains unit tests for protocol-level functionalities.

*   **`main.rs`**: The main entry point of the application. It initializes the logger, sets up signal handling for graceful shutdown, manages shared application state (like connected peers and log messages) using `Arc` and `Mutex`, and orchestrates the TUI and network threads. It handles loading and saving peer data and initiates the connection process.

*   **`ui.rs`**: Responsible for the terminal user interface (TUI) using the `ratatui` crate. It defines the application's layout, renders UI elements such as the block height, instructions, log messages, and peer lists. It also handles user input events and manages the TUI's state, including scrolling and focus.

## Key Features Implemented in Source Code

*   **P2P Connection:** Establishes TCP connections to Bitcoin nodes.
*   **Handshake:** Implements the Bitcoin P2P version handshake.
*   **Message Handling:** Parses and constructs various Bitcoin P2P messages.
*   **TUI:** Provides a real-time, interactive terminal interface.
*   **Logging:** Uses `simplelog` for logging network activity.
*   **Peer Management:** Tracks connected peers and their traffic statistics.
*   **Configuration:** Loads and saves peer data for persistence.
