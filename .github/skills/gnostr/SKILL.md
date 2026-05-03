---
name: gnostr
description: Use the gnostr CLI for git+nostr workflows in this repository.
---

# gnostr

Use `gnostr` for repo-local git+nostr workflows, relay actions, chat updates, and NIP-34 helpers.

## Verified notes

- `gnostr -V` prints the installed version and should be checked when behavior differs across environments.
- `gnostr legit` is the git/nostr workflow entrypoint.
- `gnostr chat` is for short status updates and oneshot messages.
- `gnostr dm` sends NIP-44 direct messages.
- `gnostr note` sends a text note.
- `gnostr ngit` is the passthrough for ngit subcommands.
- `gnostr nip34` exposes the NIP-34 helpers used by this repo.
- Useful top-level flags include `--workdir`, `--gitdir`, `--nsec`, `--relays`, `--difficulty-target`, `--tab`, `--command`, and `--bugreport`.

## Common commands

```bash
gnostr -V
gnostr legit --help
gnostr chat --help
gnostr dm --help
gnostr nip34 --help
```

## Repo workflow

- Use `--workdir` and `--gitdir` when pointing at a specific checkout.
- Prefer `--nsec` only when a private key is required for the task.
- Use `--relays` to keep relay selection explicit in tests and manual runs.
- Keep updates short when using `gnostr chat`; status first, details second.
- Use `--command` when you need to drive a subcommand from a scripted invocation.

## Good examples

```bash
gnostr legit . list
gnostr --workdir . chat --topic gnostr --name copilot --oneshot "build green"
gnostr --workdir . dm --help
gnostr --workdir . nip34 --help
```

## Rules

- Do not send secrets, raw logs, or private repository paths in chat or DMs.
- Prefer the smallest command that answers the question.
- When debugging, capture the exact `gnostr -V` output alongside the command used.
