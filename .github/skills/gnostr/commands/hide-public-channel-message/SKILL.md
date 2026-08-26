---
name: gnostr-hide-public-channel-message
description: Hide a message in a public chat room.
---

# hide-public-channel-message

Use this skill for hiding a public channel message.

## Verified notes

- `cargo run --bin gnostr -- hide-public-channel-message --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- hide-public-channel-message --help
gnostr hide-public-channel-message --help
```

## Rules

- Keep the target event and reason explicit.
