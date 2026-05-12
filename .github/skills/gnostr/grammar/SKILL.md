---
name: gnostr-grammar
description: Work with the tree-sitter grammar crate.
---

# grammar

Use this skill for the tree-sitter grammar crate and its dynamic-linking support.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-grammar` packages the repo's tree-sitter grammars.
- Keep grammar and parser output stable across platforms.

## Common commands

```bash
cargo test -p gnostr-grammar -- --nocapture
cargo check -p gnostr-grammar
```

## Rules

- Preserve parser compatibility.
- Avoid breaking dynamic linking behavior.
