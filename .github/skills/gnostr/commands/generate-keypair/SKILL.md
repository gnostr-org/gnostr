---
name: gnostr-generate-keypair
description: Generate a new keypair.
---

# generate-keypair

Use this skill for generating new keypairs.

## Verified notes

- `cargo run --bin gnostr -- generate-keypair --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- generate-keypair --help
gnostr generate-keypair --help
```

## Rules

- Do not expose generated secrets in logs.
