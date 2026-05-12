---
name: gnostr-list-events
description: List all events.
---

# list-events

Use this skill for listing events.

## Verified notes

- `cargo run --bin gnostr -- list-events --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- list-events --help
gnostr list-events --help
```

## Rules

- Keep filters explicit.
