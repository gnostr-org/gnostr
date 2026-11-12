---
name: gnostr-crawler
description: Work with the crawler crate and its live relay/query flows.
---

# crawler

Use this skill for crawler changes: relay query helpers, best-effort live tests, and NIP-34 matrix coverage.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-crawler` is the repo's crawler utility crate.
- Crawler tests publish to and query from real relays in a best-effort way.
- Query attempts should be time-bounded and logged.

## Common commands

```bash
cargo test -p gnostr-crawler -- --nocapture
cargo check -p gnostr-crawler
```

## Rules

- Keep relay attempts explicit and time-bounded.
- Do not silence relay query failures.
