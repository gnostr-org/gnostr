---
name: gnostr-scopetime
description: Work with the scope timing crate.
---

# scopetime

Use this skill for the runtime scope timing crate.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-scopetime` logs runtime for arbitrary scopes.

## Common commands

```bash
cargo test -p gnostr-scopetime -- --nocapture
cargo check -p gnostr-scopetime
```

## Rules

- Keep timing output lightweight.
- Do not change timing semantics without updating call sites.
