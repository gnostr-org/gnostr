# Copilot instructions for gnostr

## Build, test, and lint

- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo test --workspace`
- `cargo test -p gnostr-asyncgit --lib <exact_test_name> -- --nocapture`
- `./scripts/gnostr-tests.sh --test <exact_test_name> -- --nocapture`
- `./scripts/gnostr-asyncgit-tests.sh nip34 --nocapture`
- `./scripts/gnostr-asyncgit-tests.sh full --nocapture`
- `./scripts/gnostr-ngit-tests.sh --notes --nocapture`

Makefile shortcuts:

- `make cargo-build`
- `make cargo-test`
- `make cargo-test-workspace`
- `make cargo-clippy-workspace`

## High-level architecture

- The root `gnostr` binary is a `clap`-driven CLI that sets up logging, repo context, and dispatches into `src/lib/sub_commands`.
- The workspace is split into focused crates: `asyncgit`, `crawler`, `filetreelist`, `git2-hooks`, `git2-testing`, `grammar`, `invalidstring`, `legit`, `js`, `p2p`, `ngit`, `qr`, `relay`, `relay/extensions`, `scopetime`, `web`, and `chat`.
- `p2p` is library-first. Its browser-side implementation lives in `p2p/src/js/`, while `p2p/src/bridge.rs` and `p2p/src/relay_bridge.rs` provide the nostr compatibility bridge. `p2p::p2p::...` is kept as a compatibility namespace for older paths.
- `asyncgit` owns git/commit mining and the NIP-34 event helpers used by `legit` and the asyncgit tests.
- `relay` is the nostr relay library/server side, and `web` ties the browser UI to relay process control.

## Key conventions

- Check `gnostr -V` and `gnostr <subcommand> --help` when behavior differs across environments; the live CLI surface is the source of truth.
- Prefer explicit repo and relay context with `--gitdir`, `--workdir`, and `--relays` when writing or debugging commands.
- The test wrappers intentionally silence noisy crates with `RUST_LOG=...ureq=off,serial_test=off,mio=off,tungstenite=off,tokio_tungstenite=off`; keep that pattern when adding new shell runners.
- Use exact test names with `./scripts/gnostr-tests.sh --test ...`; the wrapper is built around exact-name selection.
- Cross-platform changes should respect the CI matrix and target-specific Cargo sections, especially the Linux/macOS/Windows and musl build paths.
