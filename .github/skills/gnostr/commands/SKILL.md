---
name: gnostr-commands
description: Work with the full gnostr top-level command list.
---

# gnostr commands

Use this skill as the index for the top-level `gnostr` subcommands.

## Verified notes

- `cargo run --bin gnostr -- -h` shows the live top-level command list.
- `gnostr -V` should be checked when behavior differs across environments.
- For details on any entry below, run `gnostr <subcommand> --help`.

## Command index

- `award-badge` — publish award badge event
- `bech32-to-any` — convert bech32 string to other formats
- `broadcast-events` — broadcast events from file
- `chat` — chat subcommands
- `convert-key` — convert key between bech32 and hex
- `create-badge` — create a new badge
- `create-public-channel` — create a new public channel
- `crawler` — crawler subcommands
- `custom-event` — create custom event
- `delete-event` — delete an event
- `delete-profile` — delete a profile
- `dm` — send a NIP-44 direct message
- `fetch-by-id` — fetch an event by ID
- `generate-keypair` — generate a new keypair
- `git` — git subcommands
- `ngit` — ngit passthrough subcommands
- `hide-public-channel-message` — hide a message in a public chat room
- `legit` — legit subcommands
- `list-events` — get all events
- `mute-public-key` — mute a public key
- `nip34` — NIP-34 subcommands
- `note` — send text note
- `privkey-to-bech32` — convert a private key to bech32
- `profile-badges` — set profile badges
- `publish-contact-list-csv` — publish contacts from a CSV file
- `query` — query subcommand
- `react` — react to an event
- `relay` — relay subcommands
- `server` — run the Blossom server
- `send-channel-message` — send a message to a public channel
- `set-channel-metadata` — update channel metadata
- `set-metadata` — set metadata / replace kind 0 event
- `set-user-status` — create a user status event
- `sniper` — perform actions related to sniping relays
- `tui` — gnostr subcommands
- `vanity` — vanity public key mining
- `xor` — XOR utility subcommand
- `help` — print this message or subcommand help

## Common commands

```bash
cargo run --bin gnostr -- -h
gnostr <subcommand> --help
```

## Rules

- Keep the command descriptions aligned with the live CLI output.
- Do not rely on stale subcommand names when a help run shows a different surface.
