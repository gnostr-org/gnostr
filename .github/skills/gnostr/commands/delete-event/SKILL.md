---
name: gnostr-delete-event
description: Delete an event.
---

# delete-event

Use this skill for deleting an event by ID.

## Verified notes

- `cargo run --bin gnostr -- delete-event --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- delete-event --help
gnostr delete-event --help
```

## Rules

- Keep event IDs explicit.
