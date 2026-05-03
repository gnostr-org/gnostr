---
name: gnostr-send-channel-message
description: Send a message to a public channel.
---

# send-channel-message

Use this skill for sending a public channel message.

## Verified notes

- `cargo run --bin gnostr -- send-channel-message --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- send-channel-message --help
gnostr send-channel-message --help
```

## Rules

- Keep channel and message explicit.
