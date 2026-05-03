---
name: gnostr-set-user-status
description: Create a user status event.
---

# set-user-status

Use this skill for creating user status events.

## Verified notes

- `cargo run --bin gnostr -- set-user-status --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- set-user-status --help
gnostr set-user-status --help
```

## Rules

- Keep status content explicit.
