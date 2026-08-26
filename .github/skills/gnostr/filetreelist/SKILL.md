---
name: gnostr-filetreelist
description: Work with the filetree navigation and filtering crate.
---

# filetreelist

Use this skill for the filetree abstraction crate: sorted paths, folding, scrolling, and key navigation.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-filetreelist` provides filetree navigation primitives used by the crawler UI.
- Keep path ordering and keyboard navigation stable when changing tree behavior.

## Common commands

```bash
cargo test -p gnostr-filetreelist -- --nocapture
cargo check -p gnostr-filetreelist
```

## Rules

- Preserve selection, folding, and scrolling semantics.
- Keep path ordering deterministic.
