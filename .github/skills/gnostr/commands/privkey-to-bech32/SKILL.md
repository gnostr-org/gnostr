---
name: gnostr-privkey-to-bech32
description: Convert a private key to bech32.
---

# privkey-to-bech32

Use this skill for converting private keys to bech32.

## Verified notes

- `cargo run --bin gnostr -- privkey-to-bech32 --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- privkey-to-bech32 --help
gnostr privkey-to-bech32 --help
```

## Rules

- Do not expose secrets in logs.
