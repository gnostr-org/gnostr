# Subcommands

This module contains various subcommands for interacting with the Nostr protocol and Git repositories.

## Subcommands

*   **`award_badge`**: Awards a badge to specified public keys by creating and publishing a Nostr event.
*   **`broadcast_events`**: Broadcasts multiple Nostr events from a JSON file to specified relays.
*   **`convert_key`**: Converts Nostr keys between bech32 and hex formats.
*   **`create_badge`**: Creates and publishes a Nostr badge definition event.
*   **`create_public_channel`**: Creates and publishes a Nostr channel metadata event.
*   **`custom_event`**: Creates and publishes a custom Nostr event with a specified kind, content, and tags.
*   **`delete_event`**: Deletes a Nostr event by publishing a delete event.
*   **`delete_profile`**: Deletes Nostr events or the entire profile by publishing a delete event or updating metadata.
*   **`generate_keypair`**: Generates new Nostr key pairs (public and private keys) in hex or bech32 format.
*   **`hide_public_channel_message`**: Hides a public channel message by publishing a hide message event.
*   **`legit`**: Handles Git-related operations, likely for managing Nostr proposals.
*   **`list_events`**: Fetches and lists Nostr events based on various filters (IDs, authors, kinds, tags, time ranges).
*   **`list`**: Lists Nostr proposals and allows interaction with them (checkout, apply patches, etc.).
*   **`login`**: Handles user login, potentially using NSEC keys, passwords, or Bunker integration.
*   **`mute_publickey`**: Mutes a public key by publishing a mute event.
*   **`ngit`**: A wrapper command that routes to other `ngit` subcommands like `login`, `init`, `send`, `list`, `pull`, `push`, `fetch`, and `query`.
*   **`note`**: Publishes a Nostr text note (NIP-01) with optional subject, tags, and expiration.
*   **`profile_badges`**: Sets the user's profile badges by publishing a Nostr event with badge definition and award information.
*   **`publish_contactlist_csv`**: Publishes a Nostr contact list from a CSV file.
*   **`pull`**: Pulls Nostr proposal changes from relays and applies them to the local Git branch.
*   **`push`**: Pushes Git commits as Nostr proposals or revisions.
*   **`query`**: Sends Nostr queries to relays based on specified filters.
*   **`react`**: Publishes a Nostr reaction event to a specific event.
*   **`send`**: Handles sending Git commits as Nostr proposals or revisions.
*   **`send_channel_message`**: Sends a message to a Nostr channel.
*   **`set_channel_metadata`**: Sets metadata for a Nostr channel.
*   **`set_metadata`**: Sets the user's Nostr profile metadata (name, about, picture, NIP-05, etc.).
*   **`tui`**: Runs the Nostr TUI (Text User Interface) application.
*   **`user_status`**: Sets the user's status with optional tags and expiration.
*   **`vanity`**: Generates Nostr public keys with specific prefixes.
