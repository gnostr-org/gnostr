---
name: gnostr-qr
description: Work with the QR utility crate.
---

# qr

Use this skill for the QR utility crate in the git+nostr toolchain.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr_qr` is part of the repo's workflow utility set.

## Common commands

```bash
cargo test -p gnostr_qr -- --nocapture
cargo check -p gnostr_qr
```

## Rules

- Keep QR output stable for downstream consumers.
- Avoid changing encoding defaults without updating callers.
