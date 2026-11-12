---
name: gnostr-mute-public-key
description: Mute a public key.
---

# mute-public-key

Use this skill for muting a public key.

## Verified notes

- `cargo run --bin gnostr -- mute-public-key --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- mute-public-key --help
gnostr mute-public-key --help
```

## Rules

- Keep the muted key explicit.
