---
name: gnostr-vanity
description: Mine vanity public keys.
---

# vanity

Use this skill for vanity public key mining.

## Verified notes

- `cargo run --bin gnostr -- vanity --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- vanity --help
gnostr vanity --help
```

## Rules

- Keep mining targets explicit.
