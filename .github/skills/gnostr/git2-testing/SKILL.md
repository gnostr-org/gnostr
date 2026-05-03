---
name: gnostr-git2-testing
description: Work with git2 testing helpers used across the repo.
---

# git2-testing

Use this skill for the git2 testing helper crate and its repository fixtures.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-git2-testing` provides convenience helpers for git2-based tests.
- Keep test helpers deterministic and fixture-friendly.

## Common commands

```bash
cargo test -p gnostr-git2-testing -- --nocapture
cargo check -p gnostr-git2-testing
```

## Rules

- Prefer stable helpers over ad hoc test setup.
- Keep fixture behavior explicit.
