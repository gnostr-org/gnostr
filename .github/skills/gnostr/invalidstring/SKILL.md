---
name: gnostr-invalidstring
description: Work with the invalid string test crate.
---

# invalidstring

Use this skill for the invalid string test crate.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-invalidstring` exists to exercise invalid string data paths.

## Common commands

```bash
cargo test -p gnostr-invalidstring -- --nocapture
cargo check -p gnostr-invalidstring
```

## Rules

- Keep invalid-data fixtures explicit.
- Do not normalize away the edge cases this crate is meant to cover.
