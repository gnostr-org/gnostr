---
name: gnostr-relay-command
description: Work with relay subcommands.
---

# relay

Use this skill for the `gnostr relay` subcommand and its relay-management modes.

## Verified notes

- `cargo run --bin gnostr -- relay --help` shows the live subcommands.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- relay --help
gnostr relay --help
```

## Rules

- Keep relay URLs and modes explicit.
