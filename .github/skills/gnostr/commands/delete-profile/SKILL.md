---
name: gnostr-delete-profile
description: Delete profile events.
---

# delete-profile

Use this skill for deleting profile-related events.

## Verified notes

- `cargo run --bin gnostr -- delete-profile --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- delete-profile --help
gnostr delete-profile --help
```

## Rules

- Keep the selected kinds and reason explicit.
