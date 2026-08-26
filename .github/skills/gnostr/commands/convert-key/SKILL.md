---
name: gnostr-convert-key
description: Convert keys between bech32 and hex.
---

# convert-key

Use this skill for converting keys between bech32 and hex.

## Verified notes

- `cargo run --bin gnostr -- convert-key --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- convert-key --help
gnostr convert-key --help
```

## Rules

- Keep key-format handling explicit.
