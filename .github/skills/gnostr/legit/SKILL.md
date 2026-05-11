---
name: gnostr-legit
description: Work with legit, the git-proof-of-work crate.
---

# legit

Use this skill for the git+nostr workflow crate that adds proof of work to commits.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-legit` handles the repo's commit PoW workflow.
- Keep publish and pull flows aligned with the asyncgit and ngit behavior.

## Common commands

```bash
cargo test -p gnostr-legit -- --nocapture
cargo check -p gnostr-legit
```

## Rules

- Preserve commit PoW semantics.
- Keep git event and relay behavior consistent with shared helpers.
