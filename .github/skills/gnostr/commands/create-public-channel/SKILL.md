---
name: gnostr-create-public-channel
description: Create a new public channel.
---

# create-public-channel

Use this skill for creating a public channel.

## Verified notes

- `cargo run --bin gnostr -- create-public-channel --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- create-public-channel --help
gnostr create-public-channel --help
```

## Rules

- Keep channel metadata explicit.
