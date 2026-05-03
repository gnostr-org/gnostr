---
name: gnostr-dm
description: Send a NIP-44 direct message.
---

# dm

Use this skill for sending a NIP-44 direct message.

## Verified notes

- `cargo run --bin gnostr -- dm --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- dm --help
gnostr dm --help
```

## Rules

- Keep recipient and message explicit.
