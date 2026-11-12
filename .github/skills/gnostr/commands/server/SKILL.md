---
name: gnostr-server
description: Run the Blossom server.
---

# server

Use this skill for running the Blossom server.

## Verified notes

- `cargo run --bin gnostr -- server --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- server --help
gnostr server --help
```

## Rules

- Keep listen addresses and storage paths explicit.
