---
name: gnostr-bech32-to-any
description: Convert bech32 strings to other formats.
---

# bech32-to-any

Use this skill for converting bech32 strings to other formats.

## Verified notes

- `cargo run --bin gnostr -- bech32-to-any --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- bech32-to-any --help
gnostr bech32-to-any --help
```

## Rules

- Preserve input/output format expectations.
