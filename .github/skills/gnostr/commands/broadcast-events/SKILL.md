---
name: gnostr-broadcast-events
description: Broadcast events from a file.
---

# broadcast-events

Use this skill for broadcasting event arrays from a file.

## Verified notes

- `cargo run --bin gnostr -- broadcast-events --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- broadcast-events --help
gnostr broadcast-events --help
```

## Rules

- Keep file input explicit and validated.
