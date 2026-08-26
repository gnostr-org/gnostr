---
name: gnostr-asyncgit
description: Work with asyncgit, NIP-34 note publishing, and asyncgit test flows.
---

# asyncgit

Use this skill for repo-local asyncgit work: git note helpers, NIP-34 event generation, relay publishing, and asyncgit test runs.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- asyncgit powers the repo's git+nostr publish and query paths.
- The NIP-34 matrix covers plain/mined commits, plain/mined notes, and plain/PoW events.
- Each matrix case is also replayed as a NIP-44 DM to the shared default recipient key.
- `DEFAULT_GNOSTR_PRIVATE_KEY` is the shared deterministic key used by asyncgit tests and fixtures.
- `asyncgit/src/lib/types/nip34.rs` contains the git note event builders and PoW helper.
- `asyncgit/src/lib/sync/notes.rs` contains the end-to-end git note matrix test.

## Common commands

```bash
cargo test -p gnostr-asyncgit --lib -- --nocapture
cargo test -p gnostr-asyncgit git_note_event_matrix_covers_commit_and_pow_variants -- --nocapture
./scripts/gnostr-asyncgit-tests.sh --nocapture
```

## What to check

- Use `generate_git_note_event` for plain note events.
- Use `generate_git_note_event_with_pow` for mined note events.
- Keep the commit-link tags intact when changing note handling.
- Prefer best-effort relay publishing so dead relays do not hang tests.
- Keep test output verbose when validating relay publish/query behavior.

## Good examples

```bash
cargo test -p gnostr-asyncgit git_note_event_matrix_covers_commit_and_pow_variants -- --nocapture
cargo test -p gnostr-asyncgit build_nip44_direct_message_event_uses_real_keys_and_recipient_tag -- --nocapture
```

## Rules

- Do not add silent fallbacks for missing relay responses.
- Do not change note identity after PoW mining.
- Do not strip the DM replay step from the matrix tests.
