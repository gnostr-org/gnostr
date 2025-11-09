# gnostr-legit

This module integrates Git repository operations with the Nostr protocol, enabling the creation and publication of Nostr events derived from Git commits.

## Features

*   **Git Commit Mining:** Generates Nostr events based on Git commit data, potentially using commit hashes for key derivation.
*   **Nostr Event Creation:** Creates Nostr text note events containing serialized Git commit information.
*   **Custom Tagging:** Attaches custom tags to Nostr events, including information derived from Git commits and repository metadata.
*   **Key Derivation:** Generates Nostr keys from Git commit hashes, allowing for unique event identities tied to specific commits.
*   **Event Publishing:** Publishes generated Nostr events to a predefined list of bootstrap relays.
*   **Git State Handling:** Checks for clean Git repository state before proceeding and captures Git diffs for event content.

## Usage

The primary entry point for this module's functionality is the `run_legit_command` function. This function orchestrates the process of:

1.  Discovering and opening a Git repository.
2.  Checking the repository's current state.
3.  Mining a commit (or using provided message/diff) to generate a unique identifier.
4.  Deriving Nostr keys from the commit's hash.
5.  Creating and signing a Nostr event containing serialized commit details and custom tags.
6.  Publishing the event to Nostr relays.

Example of how `run_legit_command` might be called (within the context of the `gnostr` application):

```rust
// Assuming `opts` is a gitminer::Options struct populated with repository path and message
use gnostr_legit::command::run_legit_command;
use gitminer::Options; // Assuming Options is accessible

let mut opts = Options {
    repo: "/path/to/your/repo".to_string(),
    message: vec!["Initial commit".to_string()],
    // ... other options
};

match run_legit_command(opts).await {
    Ok(_) => println!("Legit command executed successfully."),
    Err(e) => eprintln!("Error executing legit command: {}", e),
}
```

## Internal Components

*   **`Gitminer`:** Handles the core logic of mining Git commits and generating unique identifiers.
*   **`create_event` / `create_event_with_custom_tags`:** Functions for constructing and signing Nostr events with specific content and tags.
*   **`serialize_commit` / `deserialize_commit`:** Utilities for converting Git `Commit` objects to and from JSON representations.
*   **`generate_nostr_keys_from_commit_hash`:** Creates Nostr `Keys` from a given commit hash.
*   **`gnostr_legit_event`:** An asynchronous function that orchestrates the Git-to-Nostr event pipeline, including spawning background tasks for event creation and publishing.

## Dependencies

This module relies on the following key libraries:

*   `git2`: For interacting with Git repositories.
*   `nostr-sdk`: For Nostr client functionality (event creation, signing, publishing).
*   `tokio`: For asynchronous runtime and I/O operations.
*   `serde` & `serde_json`: For JSON serialization and deserialization.
*   `clap` (implied): For command-line argument parsing if used in a CLI context.
*   `chrono` & `time`: For handling timestamps.
*   `anyhow`: For error handling.
