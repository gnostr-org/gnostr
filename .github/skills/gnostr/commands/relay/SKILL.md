---
name: gnostr-relay-command
description: Run the gnostr relay server.
---

# relay

Use this skill for the `gnostr relay` subcommand.

## Verified notes

- `cargo run --bin gnostr -- relay --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- relay --help
gnostr relay --help
```

## Rules

- Keep relay URLs and modes explicit.
