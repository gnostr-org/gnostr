---
name: gnostr-nip34
description: Work with NIP-34 helpers.
---

# nip34

Use this skill for the `gnostr nip34` subcommand and git-collaboration helpers.

## Verified notes

- `cargo run --bin gnostr -- nip34 --help` shows the live subcommands.
- `gnostr -V` should be checked when behavior differs across environments.
- NIP-34 work in this repo covers git note events, repo state, and related publish/query flows.

## Common commands

```bash
cargo run --bin gnostr -- nip34 --help
gnostr nip34 --help
```

## Rules

- Keep note-linking and proof-of-work behavior aligned with the shared helpers.
