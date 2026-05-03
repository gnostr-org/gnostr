---
name: gnostr-git
description: Work with git subcommands.
---

# git

Use this skill for the `gnostr git` subcommand and its nested git helpers.

## Verified notes

- `cargo run --bin gnostr -- git --help` shows the live subcommands.
- The current nested commands include `tag`, `checkout`, `info`, and `tui`.
- `gnostr -V` should be checked when behavior differs across environments.

## Common commands

```bash
cargo run --bin gnostr -- git --help
gnostr git --help
```

## Rules

- Keep git-facing actions aligned with repo workflow helpers.
