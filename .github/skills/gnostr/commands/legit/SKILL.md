---
name: gnostr-legit-command
description: Work with legit subcommands.
---

# legit

Use this skill for the `gnostr legit` subcommand and its git+nostr workflow modes.

## Verified notes

- `cargo run --bin gnostr -- legit --help` shows the live subcommands.
- Current nested commands include `fetch`, `init`, `send`, `list`, `push`, `pull`, `login`, and `mine`.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- legit --help
gnostr legit --help
```

## Rules

- Keep PoW, send, and pull flows aligned.
