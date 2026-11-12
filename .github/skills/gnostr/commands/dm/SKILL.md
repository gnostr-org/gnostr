---
name: gnostr-dm
description: Send a NIP-44 direct message.
---

# dm

Use this skill for sending a NIP-44 direct message or listing/decrypting a recipient inbox.

## Verified notes

- `cargo run --bin gnostr -- dm --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr dm` without `--message` switches to inbox mode.

## Common commands

```bash
cargo run --bin gnostr -- dm --help
gnostr dm --help
gnostr dm --recipient <npub> --json
```

## Examples

```bash
gnostr dm --recipient <npub> --message "hello"
gnostr dm --recipient <npub> --limit 100
gnostr dm --recipient <npub> --limit 100 --json
gnostr --nsec <nsec> dm --recipient <npub> --relay ws://127.0.0.1:8080 --relay wss://relay.damus.io
```

## Rules

- Keep recipient and message explicit.
- Use `--json` when you want decrypted inbox events as JSON instead of the readable summary.
