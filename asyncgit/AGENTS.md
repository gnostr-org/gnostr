# AGENTS.md — `gnostr-asyncgit`

Guide for AI agents working on the `gnostr-asyncgit` crate, the async Git + Nostr compatibility layer used by the root `gnostr` CLI and TUI binaries.

## Overview

`gnostr-asyncgit` wraps `git2` and re-exports a `gitui`-derived async Git API, adds Nostr-specific git-note helpers (NIP-34 patches), deterministic test fixtures, embedded assets, block height/hash helpers, and an optional `ratatui` diff/log viewer.

## Source map

| Path | Purpose |
|------|---------|
| `src/lib/mod.rs` | Crate root: re-exports, `AsyncGitNotification`, `ureq_sync`/`ureq_async`, crypto-provider install |
| `src/lib/sync/` | Synchronous Git operations (status, diff, commits, notes, hooks, remotes, merge, rebase, etc.) |
| `src/lib/asyncjob/` | Async job framework for background Git work |
| `src/lib/blame.rs`, `src/lib/diff.rs`, `src/lib/status.rs`, ... | Async wrappers around sync operations |
| `src/lib/notes.rs` | Nostr git-note async helpers |
| `src/lib/types/` | Re-export of shared `gnostr_types::nostr::*` plus NIP-34 patch types |
| `src/lib/profiles.rs` | Deterministic Nostr profile fixtures for tests |
| `src/lib/blockheight.rs` | Fetch current Bitcoin block height from public explorers |
| `src/lib/blockhash.rs` | Fetch current Bitcoin block hash from public explorers |
| `src/lib/weeble.rs`, `src/lib/wobble.rs` | Deterministic/randomized helper utilities |
| `src/lib/images/`, `src/lib/js/`, `src/lib/css/` | Embedded asset bundles |
| `src/lib/tui/` | Optional `ratatui` diff/log TUI (behind `tui` feature) |
| `src/lib/tui/shared/syntax_parser/` | Tree-sitter syntax highlighting via `gnostr-grammar` |

## Key concepts

### Async job model

Most Git operations are synchronous and blocking. `gnostr-asyncgit` exposes them through async wrappers that spawn the work on a thread pool and emit `AsyncGitNotification` events. Use `AsyncStatus`, `AsyncDiff`, `AsyncLog`, etc. the same way `gitui` does.

### NIP-34 patches

`src/lib/types/nip34.rs` and `src/lib/sync/notes.rs` implement git-over-Nostr patches:

- `generate_git_note_event` creates a git-note event from a patch.
- `event_is_patch_set_root`, `event_is_revision_root`, etc. classify events.
- `RepoRef`, `RepoState`, `Nip34Event` model repository announcements and state.

### Deterministic fixtures

`DEFAULT_GNOSTR_PRIVATE_KEY` and `profiles.rs` provide a shared test identity. Do not use these for real keys.

### Block height / hash helpers

`blockheight.rs` and `blockhash.rs` query public Bitcoin explorers and set `BLOCKHEIGHT` / `BLOCKHASH` environment variables. These are fallible external dependencies; callers should handle `"0"` fallbacks.

## Feature flags

Key Cargo features (`asyncgit/Cargo.toml`):

- `tui` — enables the `ratatui` diff/log viewer and `syntax_parser` tree-sitter highlighting.
- `trace-libgit` — registers `git2` tracing logging.
- `default` typically includes `tui`.

## Common tasks for agents

### Adding a new async Git job

1. Add the sync implementation in `src/lib/sync/`.
2. Add the async wrapper in `src/lib/` (e.g., `src/lib/my_job.rs`).
3. Add a variant to `AsyncGitNotification` in `src/lib/mod.rs`.
4. Re-export the public types from `src/lib/mod.rs`.

### Changing how HTTP helpers work

- Edit `ureq_sync` and `ureq_async` in `src/lib/mod.rs`.
- Keep timeouts and error handling consistent.

### Working with the TUI / syntax highlighting

- The TUI feature depends on `gnostr-grammar`, which builds tree-sitter grammars at compile time.
- Grammars that fail on a platform are listed in `grammar/build.rs` blacklists.
- To disable syntax highlighting entirely, build without the `tui` feature.

## Pitfalls

- `install_rustls_crypto_provider()` must be called early (the `gnostr` binary calls it in `main`) or rustls-using dependencies will panic at runtime.
- `git2` is re-exported and used directly; version mismatches with other crates in the workspace can cause type errors.
- The TUI tree-sitter build is heavy and platform-sensitive; macOS in particular may need extra grammars blacklisted in `grammar/build.rs`.
- Block height/hash helpers rely on live network services and can return `"0"` when offline.
- Many operations `unsafe`ly set environment variables (`USER`, `BLOCKHEIGHT`, `BLOCKHASH`) for compatibility; avoid in multi-threaded contexts where possible.
