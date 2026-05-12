---
name: gnostr-blossom-tui
description: Work with the Blossom terminal UI crate.
---

# tui

Use this skill for the Blossom terminal UI crate.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-blossom-tui` provides the repo's terminal UI for Blossom storage.

## Common commands

```bash
cargo test -p gnostr-blossom-tui -- --nocapture
cargo check -p gnostr-blossom-tui
```

## Rules

- Keep UI behavior predictable.
- Preserve terminal interaction flow when changing widgets or layout.
