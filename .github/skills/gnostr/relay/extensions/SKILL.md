---
name: gnostr-extensions
description: Work with relay extension crates.
---

# relay extensions

Use this skill for relay extension types and helpers used by the relay crate.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-extensions` provides relay extension support.
- Keep extension behavior aligned with the main relay crate.

## Common commands

```bash
cargo test -p gnostr-extensions -- --nocapture
cargo check -p gnostr-extensions
```

## Rules

- Preserve extension compatibility.
- Keep protocol changes coordinated with relay consumers.
