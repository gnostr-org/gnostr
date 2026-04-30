---
name: gnostr-chat-ci-updates
description: Send short P2P status updates for CI jobs and local gnostr chat runs.
---

# gnostr chat CI updates

Use `gnostr chat` to broadcast short, readable status messages from CI jobs and local runs.

## Verified gnostr notes

- `gnostr -V` prints the installed version and is useful when CI and local behavior differ.
- `gnostr chat` supports short oneshot messages with `--topic`, `--name`, and `--oneshot`.
- `gnostr legit` is for git/nostr workflow tasks, not chat updates.
- When using repeated `-m` flags with `gnostr legit`, wrap each line to 80 columns.
- `gnostr server` runs the Blossom server.
- `gnostr ngit` is the passthrough for ngit subcommands.
- Useful top-level flags include `--workdir`, `--gitdir`, `--tab`, `--relays`,
  `--difficulty-target`, `--screenshots`, and `--bugreport`.
- `gnostr --command` can run an explicit command string when needed.

## Command

```bash
cargo run --bin gnostr -- chat --topic gnostr --name copilot --oneshot "message"
```

## When to send updates

- job start
- job success
- job failure
- retry or rerun
- matrix leg completion
- artifact ready
- local validation milestones

## Message style

- short
- clear
- status first
- optionally witty, never noisy

## Rules

- never send secrets, tokens, raw logs, or private paths
- keep `--topic gnostr` unless a different topic is explicitly needed
- use a distinct `--name` when identity matters
- if the chat run starts Blossom automatically, do not double-start it unless required
- for unattended runs, prefer headless mode

## Local versus CI

- In GitHub Actions, send a oneshot update from the relevant job step.
- When running locally, keep the same message shape so the output stays comparable.
- For workflow steps that should only run in CI, gate them with `if: ${{ !env.ACT }}`.

## Good examples

- `CI started: gnostr-test-matrix is warming up`
- `stable passed; nightly is still doing its thing`
- `build green: artifact ready to ship`
- `retrying after cache wobble`
- `p2p update: peer discovery is alive and well`

## Example

```bash
cargo run --bin gnostr -- chat --topic gnostr --name copilot --oneshot "CI green: gnostr-test-matrix finished successfully"
```
