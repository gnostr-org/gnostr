---
name: gnostr-chat
description: Work with the chat transport crate.
---

# chat

Use this skill for the chat transport crate and chat-related helpers.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-chat` is the repo's chat transport crate.
- Keep chat event behavior aligned with `gnostr chat` CLI usage.

## Common commands

```bash
cargo test -p gnostr-chat -- --nocapture
cargo check -p gnostr-chat
```

## Rules

- Keep chat payloads and transport behavior explicit.
- Do not introduce silent message drops.
