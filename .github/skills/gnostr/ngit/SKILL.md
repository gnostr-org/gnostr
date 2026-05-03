---
name: gnostr-ngit
description: Work with the ngit Nostr plugin crate.
---

# ngit

Use this skill for the ngit crate, which provides the Nostr plugin layer for git.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-ngit` is the repo's git plugin crate for Nostr workflows.
- Keep note/event generation aligned with asyncgit and legit.

## Common commands

```bash
cargo test -p gnostr-ngit -- --nocapture
cargo check -p gnostr-ngit
```

## Rules

- Preserve git event shape and tagging.
- Keep relay and publish behavior consistent with the shared helpers.
