---
name: gnostr-publish-contact-list-csv
description: Publish contacts from a CSV file.
---

# publish-contact-list-csv

Use this skill for publishing contact lists from CSV input.

## Verified notes

- `cargo run --bin gnostr -- publish-contact-list-csv --help` shows the live flags.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- publish-contact-list-csv --help
gnostr publish-contact-list-csv --help
```

## Rules

- Keep the CSV path explicit.
