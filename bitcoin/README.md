# Gnostr Bitcoin P2P Client

This project is a command-line interface (CLI) application written in Rust that connects to the Bitcoin peer-to-peer network. It provides a real-time, terminal-based user interface (TUI) to monitor network activity, connected peers, and log messages.

## Features

*   **Bitcoin P2P Network Connection:** Connects to Bitcoin nodes using the standard P2P protocol.
*   **Peer Discovery:** Utilizes DNS seeds to discover initial peers and exchanges peer addresses with connected nodes.
*   **Handshake Protocol:** Implements the Bitcoin P2P handshake, including sending and receiving `version` and `verack` messages.
*   **Real-time Monitoring:** Displays connected peers, their inbound/outbound traffic, and connection status.
*   **Log Viewer:** Shows a live stream of network events, connection status, and errors in a scrollable log.
*   **TUI Interface:** Provides an interactive terminal user interface for monitoring and basic control.
*   **Graceful Shutdown:** Handles `Ctrl+C` signals for a clean exit, saving peer data.
*   **Persistence:** Stores known peer information and traffic statistics to a local file (`peers.json`).
*   **Message Handling:** Processes common P2P messages like `version`, `verack`, `ping`, `pong`, `mempool`, `inv`, `tx`, `block`, `headers`, `getheaders`, `getdata`, and `addr`.

## Getting Started

### Prerequisites

*   **Rust Toolchain:** Ensure you have Rust and Cargo installed. You can install them from [rustup.rs](https://rustup.rs/).

### Building the Project

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/gnostr-org/gnostr-bitcoin.git
    cd gnostr-bitcoin
    ```

2.  **Build the application:**
    ```bash
    cargo build --release
    ```

### Running the Application

After building, you can run the application using Cargo or directly execute the binary:

*   **Using Cargo:**
    ```bash
    cargo run --release
    ```

*   **Directly executing the binary:**
    The binary will be located in `target/release/`.
    ```bash
    ./target/release/gnostr-bitcoin
    ```

## Usage

The application launches into a TUI that displays the following information:

*   **Block Height & Hash:** Shows the current block height and hash of the connected peer.
*   **Instructions:** Provides basic keybindings for navigation and control.
*   **Log:** Displays real-time messages, connection events, and errors.
    *   **Navigation:** Use `Up`/`Down` arrow keys to scroll through the log. Press `Enter` to auto-scroll to the bottom.
    *   **Visibility:** Press `l` to toggle the visibility of the log panel.
*   **Peers:** Lists connected peers, their total inbound/outbound traffic, and connection time.
    *   **Visibility:** Press `p` to toggle the visibility of the peer list panel.
    *   **Sorting:** Peers are sorted by inbound traffic (descending) and then by connection time (ascending).

### Keybindings

*   `q`: Quit the application.
*   `Tab`: Cycle focus between UI elements (Block Height -> Instructions -> Log -> Peers -> Block Height).
*   `Up`/`Down`: Scroll the log panel.
*   `Enter`: Auto-scroll the log to the bottom.
*   `l`: Toggle the visibility of the log panel.
*   `p`: Toggle the visibility of the peer list panel.
*   `Esc`: Enable auto-scrolling for the log if manual scrolling was used.

## Project Structure

*   `src/lib.rs`: Contains core Bitcoin P2P protocol logic, message building, connection handling, and utility functions.
*   `src/main.rs`: The application's entry point, responsible for setting up threads, managing shared state, initializing the TUI, and running the main application loop.
*   `src/ui.rs`: Implements the terminal user interface using the `ratatui` crate, handling rendering and user input.
*   `Cargo.toml`: Defines project dependencies and metadata.
*   `README.md`: This file.

## Dependencies

This project relies on several external crates, including:

*   `anyhow`: For flexible error handling.
*   `log` & `simplelog`: For logging.
*   `sha2`: For SHA-256 hashing (used for checksums).
*   `dirs`: To find user-specific directories for configuration and logs.
*   `ctrlc`: To handle Ctrl+C signals.
*   `serde` & `serde_json`: For serializing/deserializing peer data.
*   `crossterm` & `ratatui`: For building the terminal user interface.
*   `time`: For timestamp formatting.

## Contributing

Contributions are welcome! Please refer to the project's issue tracker or submit a pull request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
