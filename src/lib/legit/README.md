# Legit Module

This module provides functionality for mining Git commits and publishing Nostr events based on the mined data.

## Purpose

The `legit` module allows users to generate Git commits with a Proof-of-Work (PoW) component. The PoW involves finding a commit hash that starts with a specific target prefix. This process is orchestrated by the `Gitminer`, which utilizes multiple worker threads to speed up the computation. The resulting mined commit, along with other Git repository information and external data (like `weeble`, `wobble`, and `blockheight`), is then used to construct and publish a Nostr event.

## Features

*   **Git Commit Mining:** Performs Proof-of-Work to find a Git commit hash matching a specified target prefix.
*   **Multi-threaded Mining:** Utilizes multiple threads to accelerate the mining process.
*   **Nostr Event Generation:** Creates Nostr events that include information from the mined Git commit, repository details, and external data sources.
*   **Git Integration:** Reads Git configuration for author information and Nostr relays, and directly manipulates Git commits.
*   **Command-Line Interface (CLI):** Provides a CLI tool for initiating the mining and event publishing process.

## How it Works

1.  **Initialization:** The `Gitminer` is initialized with options including the number of threads, target prefix, commit message, repository path, and timestamp. It loads Git configuration for author details and Nostr relays.
2.  **Data Fetching:** The CLI tool fetches external data such as `weeble`, `wobble`, and `blockheight` by executing shell commands.
3.  **Proof-of-Work:** The `Gitminer` spawns multiple `Worker` threads. Each worker iteratively generates commit data (including a nonce) and calculates its SHA1 hash. When a hash matching the target prefix is found, the worker sends the result back.
4.  **Commit Creation:** The `Gitminer` takes the successful hash and commit data to create a new Git commit using commands like `git hash-object` and `git reset --hard`.
5.  **Nostr Event Publishing:** The mined commit hash, along with the fetched external data and Git repository information, is used to construct a Nostr event via the `gnostr` command-line tool. This event is then published.

## Usage

To use the `legit` module, you can execute the CLI tool:

```bash
# Example command (actual arguments may vary based on your needs)
gnostr-legit -p <target_prefix> -m <commit_message> -t <num_threads> <repository_path>
```

*   `-p, --prefix`: The desired prefix for the mined commit hash.
*   `-m, --message`: The commit message to be used.
*   `-t, --threads`: The number of worker threads to use for mining.
*   `<repository_path>`: The path to the Git repository.

## Dependencies

*   `git2`: For interacting with Git repositories.
*   `nostr-sdk`: For Nostr event creation and publishing.
*   `sha2`, `crypto` (for SHA1): For hashing algorithms.
*   `argparse`: For command-line argument parsing.
*   `chrono`, `time`: For handling timestamps.
*   `log`, `tracing`: For logging.
*   `pad`: For string padding.
*   `gnostr-crawler`: For fetching bootstrap relays.
*   External commands: `gnostr-weeble`, `gnostr-wobble`, `gnostr-blockheight`, `gnostr-git`.

## Configuration

*   **Author Information:** `user.name` and `user.email` are read from the Git configuration.
*   **Nostr Relays:** The `gnostr.relays` configuration in Git is used to specify relays. If not found, default bootstrap relays are used.
