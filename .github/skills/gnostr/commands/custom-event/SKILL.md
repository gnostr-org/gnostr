---
name: gnostr-custom-event
description: Create a custom event.
---

# custom-event

Use this skill for creating custom events.

## Verified notes

- `cargo run --bin gnostr -- custom-event --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- custom-event --help
gnostr custom-event --help
```

## Rules

- Keep event kind, content, and tags explicit.
