---
name: gnostr-relay
description: Work with the git+nostr relay crate.
---

# relay

Use this skill for the relay crate and protocol-relay behavior.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-relay` is the repo's git+nostr protocol relay crate.
- Keep relay behavior compatible with extension crates and client helpers.

## Common commands

```bash
cargo test -p gnostr-relay -- --nocapture
cargo check -p gnostr-relay
```

## Rules

- Preserve protocol compatibility.
- Avoid changing relay defaults without reviewing dependent crates.
