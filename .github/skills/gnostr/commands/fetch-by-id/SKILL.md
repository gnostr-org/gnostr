---
name: gnostr-fetch-by-id
description: Fetch an event by ID.
---

# fetch-by-id

Use this skill for fetching an event by its hex ID.

## Verified notes

- `cargo run --bin gnostr -- fetch-by-id --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- fetch-by-id --help
gnostr fetch-by-id --help
```

## Rules

- Keep event ID and relay explicit.
