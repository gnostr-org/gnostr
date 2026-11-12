---
name: gnostr-tui-command
description: Work with the gnostr TUI subcommand.
---

# tui

Use this skill for the `gnostr tui` subcommand.

## Verified notes

- `cargo run --bin gnostr -- tui --help` shows the live subcommands.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- tui --help
gnostr tui --help
```

## Rules

- Keep interactive behavior predictable.
