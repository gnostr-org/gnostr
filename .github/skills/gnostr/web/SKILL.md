---
name: gnostr-web
description: Work with the web server crate.
---

# web

Use this skill for the repo's web server crate and HTTP-facing behavior.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-web` provides the web server side of the toolchain.

## Common commands

```bash
cargo test -p gnostr-web -- --nocapture
cargo check -p gnostr-web
```

## Rules

- Keep HTTP behavior aligned with the relay and client layers.
- Avoid breaking routes without updating callers.
