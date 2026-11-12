---
name: gnostr-crawler-command
description: Use the gnostr crawler subcommand.
---

# crawler

Use this skill for the `gnostr crawler` subcommand and its nested modes.

## Verified notes

- `cargo run --bin gnostr -- crawler --help` shows the live subcommands.
- Crawler subcommands include `sniper`, `watch`, `nip34`, `crawl`, and `serve`.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- crawler --help
gnostr crawler --help
```

## Rules

- Keep relay discovery and query behavior time-bounded.
