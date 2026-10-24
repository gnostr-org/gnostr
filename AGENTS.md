# AGENTS.md

Guide for AI agents and automated tools working in the `gnostr` repository.

## Project overview

`gnostr` is a git + nostr workflow utility: a Rust monorepo that bundles a CLI,
libraries, a Nostr relay, git remote helpers, a TUI, chat/P2P experiments, and
web bindings. The default binary is `gnostr` (`src/main.rs`) and the shared
library lives in `src/lib/`.

The workspace uses a custom versioning scheme recorded in `Cargo.toml`:
`blockheight.weeble.wobble` (e.g. `970410.1846.41651`). There is no
`rust-toolchain.toml`; CI tests against both `stable` and `nightly`.

This repo vendors several upstream crates under `vendor/` and patches them via
`[patch.crates-io]` in the root `Cargo.toml`. Do not upgrade those dependencies
without checking the vendored patches.

## Technology stack

- **Language:** Rust (mostly edition 2021; `gnostr-bins`, `gnostr-types`, and
  `ngit` use edition 2024).
- **Async runtime:** `tokio`.
- **Web frameworks:** `axum`, `actix-web`, `warp`.
- **Nostr protocol:** `nostr-sdk` / `nostr` 0.44.x family, `nostr-relay` 0.4.8,
  `nostr-relay-pool`, `nostr-lmdb`, `nostr-database`.
- **Git:** `git2`, `gix`, `auth-git2`; plus a forked `asyncgit` in
  `asyncgit/` and `vendor/gitui/asyncgit/`.
- **Cryptography:** `secp256k1`, `k256`, `bitcoin_hashes`, `sha2`, `chacha20poly1305`.
- **P2P:** `libp2p` 0.54 with most features enabled; optional Tor via
  `arti-client` in `gnostr-p2p`.
- **Storage:** `rocksdb` (link against a system lib when available), `sled`,
  LMDB via `nostr-lmdb`.
- **TUI:** `ratatui`, `crossterm`, `tui-input`.
- **Build/release:** `cargo`, `cargo-dist`, `cross`, Docker, Nix (`flake.nix`).

## Workspace layout

| Crate | Path | Purpose |
|-------|------|---------|
| `gnostr` | `.` | Root CLI binary and library (`src/main.rs`, `src/lib/`) |
| `gnostr-bins` | `bins/` | Feature-gated extra binaries (chat, relay, crawler, git helpers, TUI, etc.) |
| `gnostr-asyncgit` | `asyncgit/` | Async Git operations; forked from gitui's asyncgit |
| `gnostr-ngit` | `ngit/` | Nostr plugin for git (`ngit`, `git-remote-nostr`) |
| `gnostr-relay` | `relay/` | Nostr relay library/server wrapper around `nostr-relay` |
| `gnostr-extensions` | `relay/extensions/` | Relay extension points |
| `gnostr-types` | `types/` | Shared Nostr / git / key types |
| `gnostr-chat` | `chat/` | Chat transport and TUI primitives |
| `gnostr-p2p` | `p2p/` | libp2p and optional Tor networking |
| `gnostr-crawler` | `crawler/` | Relay/content crawler |
| `gnostr-web` | `web/` | Web server and asset embedding |
| `gnostr-js` | `js/` | Web bindings and small embedded web server |
| `gnostr-legit` | `legit/` | Proof-of-work git commit mining |
| `gnostr-git-helpers` | `git-helpers/` | Standalone git remote helpers (blossom, nostr, ipfs, pkarr, tor) |
| `gnostr-grammar` | `grammar/` | tree-sitter grammar compilation |
| `gnostr-filetreelist` | `filetreelist/` | File-tree abstraction for TUIs |
| `gnostr-git2-hooks` | `git2-hooks/` | git2-based hook runner |
| `gnostr-git2-testing` | `git2-testing/` | Helpers for git2 unit tests |
| `gnostr-scopetime` | `scopetime/` | Scope timing utility |
| `gnostr-invalidstring` | `invalidstring/` | Invalid-string test fixture crate |
| `nips` | `nips/` | NIP reference data / helpers |

Vendored forks live in `vendor/` (e.g. `vendor/gitui`, `vendor/nostr-relay`,
`vendor/time`, `vendor/ed25519-dalek`, `vendor/curve25519-dalek`, etc.).

## Network architecture

The workspace spans several independent network planes. Each plane has its own
crate and feature gates.

### 1. P2P / libp2p plane (`gnostr-p2p`, `gnostr-chat`)

- TCP + Noise + Yamux, QUIC, WebSocket, optional Tor (`arti-client`).
- Gossipsub, Kademlia DHT, mDNS, circuit relay, rendezvous, AutoNAT.
- Chat uses an embedded libp2p relay service for hole punching.
- See `p2p/AGENTS.md` and `chat/AGENTS.md`.

#### Chat session flow

`gnostr chat` builds a libp2p swarm in `chat/src/p2p.rs` (`evt_loop`):

1. `ChatSession::connect(topic)` (`chat/src/session.rs`) starts a local relay
   service, opens an mpsc command channel, and spawns the event loop.
2. The swarm subscribes to the Gossipsub topic, listens on ephemeral
   `0.0.0.0` TCP/QUIC ports, and discovers peers via mDNS on the local LAN.
3. When mDNS discovers a peer, the swarm adds it as an explicit Gossipsub peer,
   registers it with AutoNAT, and dials it (TCP and QUIC paths).
4. `ConnectionEstablished` triggers a flush of any messages that were queued
   because of "insufficient peers".
5. Outgoing chat messages are serialized to JSON and published with
   `gossipsub::Behaviour::publish`.  The `MessagePublished` notification is
   emitted when the publish succeeds locally.

#### Modes

- **TUI mode** (default): opens the ratatui interface in `chat/src/tui.rs`.
- **Headless mode** (`--headless`): sets a process title, prints notifications,
  and blocks on `tokio::signal::ctrl_c()` until interrupted.
- **One-shot mode** (`--oneshot "<message>"`): waits up to 30 s for a peer,
  publishes the message, waits up to 30 s for `MessagePublished`, sleeps 500 ms
  for propagation, and exits.  It is meant for scripts and CI.

#### Smoke-test script

`scripts/gnostr-chat-one-shot.sh` automates the one-shot flow:

```bash
# Default: start a headless receiver, send two messages, assert delivery.
./scripts/gnostr-chat-one-shot.sh

# Custom topic and messages.
./scripts/gnostr-chat-one-shot.sh --topic my-topic "hello" "world"

# Long-running headless receiver (Ctrl+C to stop).
./scripts/gnostr-chat-one-shot.sh --listen --topic my-topic

# Pass through chat flags such as --debug or --nsec.
./scripts/gnostr-chat-one-shot.sh --topic my-topic --debug --nsec <hex> "msg"
```

The script waits for the receiver to log `Local node is listening`, sends each
message from a fresh sender process, and verifies the receiver received every
message.

### 2. Nostr relay / WebSocket plane (`gnostr-relay`, `gnostr-crawler`, root CLI, `ngit`)

- Actix-web/WebSocket relay (vendored `nostr-relay` 0.4.8).
- `nostr_sdk::Client` relay pool with custom Happy Eyeballs transport in `ngit`.
- Crawler discovers relays, fetches NIP-11 metadata, and serves an Axum API.
- Root `gnostr` subcommands (`note`, `dm`, `query`, `sniper`, etc.) publish/query
  relays via `create_client()`.
- See `relay/AGENTS.md`, `crawler/AGENTS.md`, and `ngit/AGENTS.md`.

### 3. Web / HTTP plane (`gnostr-web`, `gnostr-js`)

- `gnostr-web` / `gnostr-gnit` / `gnostr-org` use axum.
- `gnostr-js` and embedded apps use warp.
- The `gnostr-js` binary can also launch a relay and a crawler API server.
- See `web/AGENTS.md`.

### 4. Git remote-helper plane (`gnostr-git-helpers`)

- `git-remote-blossom`, `git-remote-nostr`, `git-remote-ipfs`,
  `git-remote-pkarr`, `git-remote-tor`.
- Blossom, Nostr relay + git smart HTTP, Kubo IPFS, PKARR, Tor SOCKS5.
- See `git-helpers/AGENTS.md`.

### 5. Workspace binary plane (`gnostr-bins`)

- Feature-gated binaries that expose the above crates as standalone commands.
- Features: `blossom`, `web`, `relay`, `chat`, `p2p`, `crawler`, `git`, `legit`.
- See `bins/AGENTS.md`.

## Source organization

- `src/main.rs` — CLI entry point, `tokio::main`, logging, argument dispatch.
- `src/lib/mod.rs` — public library surface.
- `src/lib/cli.rs` — `clap` structs (`GnostrCli`, `GnostrCommands`) and app
  cache/config helpers.
- `src/lib/sub_commands/` — one module per top-level `gnostr` subcommand
  (`note`, `dm`, `chat`, `relay`, `query`, `server`, `legit`, `sniper`, etc.).
- `src/lib/core/` — shared TUI / app core reused by the `tui` subcommand.
- `src/lib/tabs/`, `src/lib/components/`, `src/lib/popups/`, `src/lib/ui/` —
  TUI screens and widgets.
- `src/lib/test_utils/tests/` — root-crate integration / CLI tests.
- `bins/src/bin/` — additional binaries, each gated by a feature in
  `bins/Cargo.toml`.
- `ngit/src/bin/` — `ngit` and `git-remote-nostr`.
- `git-helpers/src/bins/` — standalone remote helpers.

## Cargo features

Root `Cargo.toml` default features:
`ghemoji`, `regex-fancy`, `trace-libgit`, `blossom`.

`gnostr-bins` default feature: `blossom`.

Important optional features used across the workspace:

- `blossom` — Blossom server and related git remote helpers.
- `web` — web server / JS binding binaries.
- `chat` — chat binaries and TUI demos.
- `p2p` — libp2p binaries and Tor support.
- `crawler` — crawler binaries.
- `git` — git/TUI binaries (`gnostr-tui`, `git-tui`, `gnostr-nip34`).
- `legit` — proof-of-work git helpers.
- `relay` — relay binaries.
- `vendor-openssl` / `vendored-openssl` — static OpenSSL builds for portable
  binaries and cross-compilation.
- `long_tests` / `expensive_tests` — longer-running test suites.

## Build commands

```bash
# Fast development build of the default gnostr binary
cargo build

# Release build
cargo build --release

# Build specific feature-gated binaries in the bins crate
cargo build -p gnostr-bins --features chat,p2p,crawler,blossom

# Build all workspace members with all features (slow)
cargo build --workspace --all-features

# Makefile / just wrappers (both delegate to the Makefile)
make cargo-build
make cargo-build-release
just cargo-build
```

Most Makefile/scripts call `scripts/with-system-rocksdb.sh` first. That script
sets `ROCKSDB_COMPILE=0` and points Cargo at a system `rocksdb` when
`pkg-config rocksdb` is available, avoiding a from-source rocksdb build.

### Cross compilation

```bash
# Build everything the current host can attempt
./scripts/cross.sh

# List targets
./scripts/cross.sh --list

# Selected targets
./scripts/cross.sh --target linux-arm64 --target linux-arm64-musl
```

Linux cross targets use Docker-backed images configured in `Cross.toml` and
`docker/cross/*.Dockerfile`. On Apple Silicon, `cross 0.2.5` is known to fail
for some Linux targets; `cross.sh` skips them unless
`GNOSTR_CROSS_FORCE=1` / `--force-broken-cross` is set.

### Nix

```bash
nix develop
cargo build --release
```

## Test commands

Prefer the provided wrapper scripts because they silence noisy crates, use the
system rocksdb wrapper, and run related test subsets together.

```bash
# Full workspace test suite (also runs ngit notes tests and verifies nip44 vectors)
./scripts/gnostr-tests.sh --workspace

# Run a single workspace test by exact name
./scripts/gnostr-tests.sh --test blossom_remote_push_list_and_fetch_round_trip -- --nocapture

# Asyncgit / NIP-34 event matrix tests
./scripts/gnostr-asyncgit-tests.sh --nocapture

# ngit notes tests
./scripts/gnostr-ngit-tests.sh --notes --nocapture

# Full ngit suite
./scripts/gnostr-ngit-tests.sh --all --nocapture

# Workspace-only runner with variants
./scripts/cargo-test-workspace.sh
./scripts/cargo-test-workspace.sh ignored --nocapture
./scripts/cargo-test-workspace.sh all-features --package gnostr-bins --nocapture
./scripts/cargo-test-workspace.sh long-tests-ignored --nocapture

# Makefile equivalents
make cargo-test-workspace
make cargo-test--ignored
make cargo-test-nightly-workspace
```

Many network-dependent tests are marked `#[ignore]` and are skipped by the
default wrapper commands. The workspace runner explicitly skips a known list of
such tests when `--ignored` is requested.

## Code style and linting

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets -- -A warnings   # CI style
make cargo-clippy-workspace                              # nightly, -D warnings
```

- `asyncgit/rustfmt.toml` and `ngit/rustfmt.toml` exist; `ngit/rustfmt.toml`
  enables unstable rustfmt features and requires nightly for formatting.
- The root crate uses default rustfmt settings.
- Error handling: prefer `anyhow::Result` / `anyhow::Context` for application
  code and `thiserror` for library errors.
- Tests frequently use `pretty_assertions`, `mockall`, `serial_test`,
  `tempfile`, and `assert_cmd`.

## CI/CD and deployment

GitHub Actions workflows in `.github/workflows/`:

- `ci.yml` — `cargo fmt`, `cargo clippy`, `cargo test --workspace` on pushes
  to `master` and PRs.
- `gnostr-test-matrix.yml` — daily/matrix tests on macOS and Ubuntu with
  stable/nightly; runs the bins tests, asyncgit and ngit wrappers, and several
  CLI smoke tests.
- `build-artifact.yml` — cross builds release artifacts for Linux, macOS, and
  Windows; publishes Docker images to GHCR.
- `cross-docker.yml` / `cargo-cross.yml` — Docker-backed cross builds.
- `release.yml` — `cargo-dist` release pipeline (version tags like
  `release/v970410.1846.41651`).
- `ghcr.io.yml` — container image publishing.
- `gnostr-relay-release.yml`, `gnostr-js-release.yml`,
  `gnostr-asyncgit-release.yml`, `gnostr-chat.yml` — per-crate release flows.

Releases use `cargo-dist` configured in `dist-workspace.toml`:

- Targets: `aarch64-apple-darwin`, `aarch64-unknown-linux-gnu`,
  `x86_64-apple-darwin`, `x86_64-unknown-linux-gnu`, `x86_64-pc-windows-msvc`.
- Features for dist builds: `vendored-openssl`, `web`, `chat`, `p2p`,
  `crawler`, `git`, `legit`, `relay`.
- Installers: shell, powershell, homebrew. Homebrew tap:
  `gnostr-org/homebrew-gnostr-org`.

Docker images are defined in `docker/` and composed via `docker-compose.yml`
(`gnostr-node`, `nginx`, optional `tor`) and `docker-compose.blossom.yml`
(Blossom server stack). See `docker/Dockerfile.*`, `nginx.conf`, and
`DOCKER.md` / `CROSS.md` for details.

## Configuration and runtime

- Relay config template: `config/gnostr.toml`.
- Default relay data directory: `.gnostr/relay`; default listen address
  `127.0.0.1:8080`.
- App cache/config directory: OS config dir under `gnostr/` (macOS falls back
  to `~/.config/gnostr`).
- Logging: stderr plus `gnostr.log` in the app cache.
- Additional config files in `config/`: `relays.yaml`, `shitlist.yaml`.

## Security considerations

- The `gnostr` CLI defaults `--nsec` to a publicly known weak key. Treat it as
  a placeholder; never pass real private keys in CI logs, test code, or shell
  history.
- Tests and fixtures contain hardcoded keys and hashes; do not reuse them in
  production.
- The relay binds to localhost by default; change `host` in `gnostr.toml` only
  after reviewing auth, rate limiting, and NIP-42 settings.
- Use `vendor-openssl` / `vendored-openssl` for portable static binaries, but
  be aware it increases build times and attack surface.
- Remote helpers and P2P code interact with external networks and untrusted
  relays; input validation and timeout handling are important.

## Common tasks for agents

- **Adding a `gnostr` subcommand:** create `src/lib/sub_commands/<name>.rs`,
  add a variant to `GnostrCommands` in `src/lib/cli.rs`, and dispatch it in
  `src/main.rs`.
- **Adding a binary:** add an entry to `bins/Cargo.toml` with the required
  feature flag, then place the source in `bins/src/bin/`.
- **Changing dependencies:** check `[workspace.dependencies]`,
  `[patch.crates-io]`, and the relevant `vendor/` crate before bumping versions.
- **Changing formatting/clippy settings:** update `rustfmt.toml` in affected
  crates; the root crate has no custom rustfmt config.
- **Working in `ngit/`:** read `ngit/AGENTS.md` first; it has crate-specific
  conventions, URL handling rules, and test notes.
- **Running tests:** prefer `./scripts/gnostr-tests.sh`,
  `./scripts/gnostr-asyncgit-tests.sh`, and `./scripts/gnostr-ngit-tests.sh`
  over raw `cargo test` so logs and rocksdb linking are handled consistently.

## `gnostr-legit` usage

Stage the intended changes before mining a commit. From the repository root:

```bash
git add legit && git add types && \
  RUST_LOG=gnostr_legit=debug cargo run -p gnostr-legit -- \
    -m "feat(nostr): feature summary" \
    -m "commit body 1" \
    -m "commit body 2" \
    -m "etc..."
```

The first `-m` is the commit subject; subsequent `-m` values form the body.
Mining defaults to repository `.`, prefix `000`, and the available logical CPU
count unless `--repo`, `--prefix`, or `--threads` overrides them. Unstaged
changes must be staged or stashed before mining. A successful mined commit is
written locally and its NIP-34 patch event is published to the selected relays.

Add `--nsec "$NOSTR_PRIVATE_KEY"` after `--` to sign the event with a
64-character hex/SHA-256 or `nsec1...` private key. Without it, the signing key
is derived from the public commit hash and is not a secure identity. Never put
literal private keys in documentation, logs, or shell history.
Without `--nsec`, a kind-0 profile event with the gnostr avatar/banner is sent
before the NIP-34 patch, using the same commit-derived key and selected relays.
An explicit `--nsec` never triggers automatic profile metadata publication.
`gnostr_types::nostr::Metadata::default()` and `Metadata::new()` include the
default avatar and banner URLs; deserialization does not inject these defaults.
All legit-generated events carry the hashtag tag `["t","gnostr"]` (`#gnostr`).
After the patch is sent, a kind-1 note references its event ID as `nostr:note1...`
in the content and an `e` tag. The note uses the same signing key and relays,
includes `["t","gnostr"]`, and logs its own event ID.
Both patch and note include the parent commit-derived `p` tag. A cached or
relay-discovered parent patch ID adds an `e` tag marked `reply`; the note's own
patch reference is marked `mention`. Cache entries live in the Git directory at
`gnostr/patch-events/<commit hash>`. Missing parent events produce chain roots,
not fabricated event IDs. Parent public keys are derived deterministically from
zero-padded Git hashes; actual event IDs must be cached or discovered.

## Verification summary

- `cargo metadata --no-deps` parses successfully and lists 29 workspace
  packages.
- Rust toolchain on this machine: `cargo 1.99.0`, `rustc 1.99.0`.

### IPFS (Kubo) integration in `gnostr-web`

`gnostr-web` can embed a Kubo node when built with `--features ipfs`
(requires Go toolchain for `ipfs/ffi/rust`). See `web/AGENTS.md` for endpoints,
configuration (`GNOSTR_IPFS_REPO`, `GNOSTR_IPFS_ONLINE`), and interop details.
