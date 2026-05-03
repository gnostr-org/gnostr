---
name: gnostr-p2p
description: Work with the P2P networking crate.
---

# p2p

Use this skill for the repo's P2P networking crate and peer-related helpers.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-p2p` provides the repo's P2P support crate.

## Common commands

```bash
cargo test -p gnostr-p2p -- --nocapture
cargo check -p gnostr-p2p
```

## Rules

- Keep peer messaging behavior explicit.
- Do not introduce hidden network side effects.
