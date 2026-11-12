---
name: gnostr-sniper
description: Perform relay-sniping actions.
---

# sniper

Use this skill for relay-sniping related actions.

## Verified notes

- `cargo run --bin gnostr -- sniper --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- sniper --help
gnostr sniper --help
```

## Rules

- Keep targets and relay selection explicit.
