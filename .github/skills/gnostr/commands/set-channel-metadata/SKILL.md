---
name: gnostr-set-channel-metadata
description: Update channel metadata.
---

# set-channel-metadata

Use this skill for updating public channel metadata.

## Verified notes

- `cargo run --bin gnostr -- set-channel-metadata --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- set-channel-metadata --help
gnostr set-channel-metadata --help
```

## Rules

- Keep metadata explicit.
