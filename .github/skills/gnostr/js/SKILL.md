---
name: gnostr-js
description: Work with the embedded JavaScript assets crate.
---

# js

Use this skill for the embedded JavaScript assets used by the repo's web and UI surfaces.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-js` packages embedded JavaScript assets.

## Common commands

```bash
cargo test -p gnostr-js -- --nocapture
cargo check -p gnostr-js
```

## Rules

- Keep asset contents in sync with the UI that consumes them.
- Avoid changing asset names without updating consumers.
