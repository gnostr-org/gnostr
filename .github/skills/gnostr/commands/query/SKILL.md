---
name: gnostr-query
description: Query relays and indexed events.
---

# query

Use this skill for querying relays and indexed events.

## Verified notes

- `cargo run --bin gnostr -- query --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- query --help
gnostr query --help
```

## Rules

- Keep query targets and relay URLs explicit.
