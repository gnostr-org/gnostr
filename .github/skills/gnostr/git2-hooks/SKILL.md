---
name: gnostr-git2-hooks
description: Work with git2-based hook support for gnostr.
---

# git2-hooks

Use this skill for the git hook support crate built on git2-rs.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-git2-hooks` provides hook support used by the repo's git workflow.
- Keep hook behavior compatible with the vendored gitui integration.

## Common commands

```bash
cargo test -p gnostr-git2-hooks -- --nocapture
cargo check -p gnostr-git2-hooks
```

## Rules

- Preserve hook contract compatibility.
- Avoid changing upstream vendored expectations unless required.
