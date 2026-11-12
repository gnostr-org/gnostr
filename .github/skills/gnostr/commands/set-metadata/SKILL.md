---
name: gnostr-set-metadata
description: Set kind 0 profile metadata.
---

# set-metadata

Use this skill for setting kind 0 profile metadata.

## Verified notes

- `cargo run --bin gnostr -- set-metadata --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- set-metadata --help
gnostr set-metadata --help
```

## Rules

- Treat this as a replacement, not a merge.
