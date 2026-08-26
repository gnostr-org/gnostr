---
name: gnostr-chat-command
description: Use the gnostr chat subcommand.
---

# chat

Use this skill for the `gnostr chat` subcommand.

## Verified notes

- `cargo run --bin gnostr -- chat --help` shows the live flags.
- `gnostr chat` accepts oneshot status updates with a workdir argument.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- chat --help
gnostr chat --help
```

## Rules

- Keep chat messages short and status-first.
