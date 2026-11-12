# AGENTS.md — `gnostr-bins`

Guide for AI agents working on the feature-gated binaries crate.

## Overview

`gnostr-bins` is not a library; it is a collection of binary entry points.
Every binary in `bins/src/bin/` is gated by a Cargo feature in
`bins/Cargo.toml`. The crate pulls in workspace crates as optional dependencies
and exposes standalone executables for chat, P2P, relay, web, crawler, git, and
remote-helper tools.

## Feature flags

`bins/Cargo.toml:337-349` defines the feature switches. Each `[[bin]]` entry
uses `required-features` to decide what is compiled.

| Feature | Enables dependency | Binary families |
|---------|--------------------|-----------------|
| `blossom` (default) | none extra; unlocks a large binary set | `gnostr-server`, `gnostr-client`, `gnostr-sniper`, git remote helpers, misc |
| `web` | `gnostr-web`, `gnostr-js` | `gnostr-org`, `gnostr-gnit`, `gnostr-js`, `embedded_js`, etc. |
| `relay` | `gnostr-relay` | `gnostr-relay`, `gnostr-relay-tui` |
| `chat` | `gnostr-chat` | `gnostr-chat`, chat TUI demos |
| `p2p` | `gnostr-p2p` + `vendored-openssl` | `gnostr-p2p`, `gnostr-p2p-client`, `gnostr-p2p-relay-server`, etc. |
| `crawler` | `gnostr-crawler`, `gnostr_filetreelist` | `gnostr_crawler`, `gnostr_master_detail`, filetree variants |
| `git` | `gnostr_ngit`, `git-version` | `gnostr-tui`, `git-tui`, `gnostr-nip34` |
| `legit` | `gnostr-legit` | `gnostr-legit`, `git-legit` |
| `nip34-syndicate` | `git` + `crawler` + `p2p` | feature combo, no dedicated binary |

Build examples:

```bash
# Default binaries (blossom feature set)
cargo build -p gnostr-bins

# Network-heavy feature set
cargo build -p gnostr-bins --features chat,p2p,crawler,blossom,relay,web,git

# Everything (slow)
cargo build -p gnostr-bins --all-features
```

## Binary inventory by network stack

### Chat / P2P binaries (`chat`, `p2p` features)

| Binary | Source | Network stack | Default bind / notes |
|--------|--------|---------------|----------------------|
| `gnostr-chat` | `bins/src/bin/gnostr-chat.rs` | `gnostr_chat` — libp2p/gossipsub | ephemeral TCP/QUIC; topic `gnostr-dev` |
| `gnostr-p2p` | `bins/src/bin/gnostr-p2p.rs` | `gnostr_p2p` — libp2p | `/ip4/0.0.0.0/tcp/<port>` (default `0`, examples use `4001`) |
| `gnostr-p2p-client` | `bins/src/bin/gnostr-p2p-client.rs` | `gnostr_p2p` | DHT / direct lookup client |
| `gnostr-p2p-relay-server` | `bins/src/bin/gnostr-p2p-relay-server.rs` | `libp2p` relay behaviour | `listen_default_addresses_relay` |
| `gnostr-p2p-rendezvous-server` | `bins/src/bin/gnostr-p2p-rendezvous-server.rs` | `libp2p` rendezvous behaviour | `listen_default_addresses_relay` |
| `gnostr-lookup` | `bins/src/bin/gnostr-lookup.rs` | root `gnostr::p2p` re-export | 20 s lookup timeout |
| `gnostr-kvs` | `bins/src/bin/gnostr-kvs.rs` | root `gnostr::p2p` re-export | listens on `/ip4/0.0.0.0/tcp/0` |

### Relay binaries (`relay` feature)

| Binary | Source | Network stack | Default bind |
|--------|--------|---------------|--------------|
| `gnostr-relay` | `bins/src/bin/gnostr-relay.rs` | `gnostr_relay` / actix-web | `127.0.0.1:8080` |
| `gnostr-relay-tui` | `bins/src/bin/gnostr-relay-tui.rs` | `gnostr_relay` / actix-web | `127.0.0.1:8080` |

### Web binaries (`web` feature)

| Binary | Source | Network stack | Default bind |
|--------|--------|---------------|--------------|
| `gnostr-org` | `bins/src/bin/gnostr-org.rs` | `gnostr_web` / axum | `127.0.0.1:3333` |
| `gnostr-gnit` | `bins/src/bin/gnostr-gnit.rs` | `gnostr_web` / axum | `127.0.0.1:3333` |
| `gnostr-js` | `bins/src/bin/gnostr-js.rs` | `gnostr_js` / warp + actix | subcommands: web (`3030`), relay (`8080`), crawler (`3000`) |
| `gnostr-js-relay` | `bins/src/bin/gnostr-js-relay.rs` | `gnostr_js::relay_app` | `127.0.0.1:8080` |
| `embedded_js` | `bins/src/bin/embedded_js.rs` | `gnostr_js::web_app` | `127.0.0.1:3030` |
| `embedded_main_js` | `bins/src/bin/embedded_main_js.rs` | `gnostr_js::web_app` | `127.0.0.1:3030` |
| `websockets_chat` | `bins/src/bin/websockets_chat.rs` | warp WebSocket | `127.0.0.1:3333` |
| `flat` | `bins/src/bin/flat.rs` | `gnostr_js::flat` | — |

### Crawler binaries (`crawler` feature)

| Binary | Source | Network stack | Default bind |
|--------|--------|---------------|--------------|
| `gnostr_crawler` | `bins/src/bin/gnostr_crawler.rs` | `gnostr_crawler` | API server `127.0.0.1:3000` |
| `gnostr_master_detail` | `bins/src/bin/gnostr_master_detail.rs` | `gnostr_crawler` + `gnostr_filetreelist` | local TUI over saved files |
| `gnostr_crawler_filetree-*` | `bins/src/bin/gnostr_crawler_filetree-*.rs` | `gnostr_crawler` + `gnostr_filetreelist` | local TUI |

### Git helpers and TUI binaries (`blossom`, `git`, `legit` features)

| Binary | Source | Network stack | Notes |
|--------|--------|---------------|-------|
| `gnostr-server` | `bins/src/bin/gnostr-server.rs` | Blossom server wrapper | `0.0.0.0:3000` |
| `gnostr-client` | `bins/src/bin/gnostr-client.rs` | `gnostr::nostr_client`, `reqwest` | connects to `wss://relay.damus.io` |
| `gnostr-sniper` | `bins/src/bin/gnostr-sniper.rs` | `gnostr::sub_commands::sniper`, `reqwest` | fetches NIP-11 metadata |
| `git-remote-blossom` | `bins/src/bin/git-remote-blossom.rs` | `gnostr_git_helpers::blossom_backend` | HTTPS Blossom |
| `git-remote-nostr` | `bins/src/bin/git-remote-nostr.rs` | `gnostr_git_helpers::nostr_backend` | relay + git smart HTTP |
| `git-remote-ipfs` | `bins/src/bin/git-remote-ipfs.rs` | `gnostr_git_helpers::ipfs_backend` | Kubo RPC |
| `git-remote-pkarr` | `bins/src/bin/git-remote-pkarr.rs` | `gnostr_git_helpers::pkarr_backend` | PKARR + Blossom |
| `git-remote-tor` | `bins/src/bin/git-remote-tor.rs` | `gnostr_git_helpers::tor_backend` | Tor SOCKS5 + Blossom |
| `git-remote-gnostr-node` | `bins/src/bin/git-remote-gnostr-node.rs` | `nostr::prelude` | stub helper |
| `gnostr-tui` / `git-tui` / `git_tui` | `bins/src/bin/gnostr-tui.rs`, etc. | `gnostr_ngit`, `gnostr_asyncgit` | local git TUI |
| `gnostr-nip34` | `bins/src/bin/gnostr-nip34.rs` | `gnostr_asyncgit` + `gnostr_crawler` | NIP-34 navigator |
| `gnostr-legit` / `git-legit` | `bins/src/bin/gnostr-legit.rs`, etc. | `gnostr_legit` | local PoW mining |

### Notable `blossom`-gated misc binaries

| Binary | Network behavior |
|--------|------------------|
| `warp_filters` | Starts a Warp server on `127.0.0.1:3030` |
| `gnostr-repo-flattener` | `git clone` over SSH/HTTP depending on URL |

## Cross-reference: workspace crate → bins that use it

| Crate | Bins that use it |
|-------|------------------|
| `gnostr-chat` | `gnostr-chat` |
| `gnostr-p2p` | `gnostr-p2p`, `gnostr-p2p-client`, `gnostr-p2p-relay-server`, `gnostr-p2p-rendezvous-server` |
| `gnostr-crawler` | `gnostr_crawler`, `gnostr_master_detail`, filetree variants, `gnostr_chat_tui_demo*`, `gnostr-js` crawler subcommand |
| `gnostr-relay` | `gnostr-relay`, `gnostr-relay-tui`, `gnostr-js` relay subcommand |
| `gnostr-web` | `gnostr-org`, `gnostr-gnit` |
| `gnostr-js` | `gnostr-js`, `gnostr-js-relay`, `embedded_js`, `embedded_main_js`, `flat`, `websockets_chat` |
| `gnostr-git-helpers` | `git-remote-blossom`, `git-remote-nostr`, `git-remote-ipfs`, `git-remote-pkarr`, `git-remote-tor`, `git-remote-gnostr-node` |

Note: `gnostr-lookup` and `gnostr-kvs` use the root crate's `gnostr::p2p`
re-export rather than depending on `gnostr-p2p` directly.

## Examples

`bins/examples/` contains network demos:

| Example | Network behavior |
|---------|------------------|
| `gnostr_relay_demo.rs` | Starts a Nostr relay via `gnostr_relay::App` |
| `gnostr_relay_ext_demo.rs` | Relay with `Metrics`/`Auth`/`Ratelimiter` extensions |
| `gnostr_crawler.rs` | Crawler CLI using a `reqwest` client |
| `embedded_js.rs`, `gnostr_welcome.rs` | Warp server serving embedded JS/CSS/images on `127.0.0.1:3030` |
| `fetch_metadata.rs`, `fetch_by_id*.rs`, `fetch_by_kind_and_author*.rs`, `fetch_relay_list.rs`, `fetch_nip11.rs`, `dump_relay*.rs`, `post_event.rs`, `post_from_files.rs`, `test_nip46.rs` | Connect to a Nostr relay with `gnostr::Probe` / `gnostr::fetch_by_filter` |

## Common tasks for agents

### Adding a new binary

1. Add the source file in `bins/src/bin/<name>.rs`.
2. Add an `[[bin]]` entry in `bins/Cargo.toml` with `required-features = [...]`.
3. Add any new optional dependencies to `[dependencies]` and wire them through
   the relevant feature.
4. Build with the feature enabled: `cargo build -p gnostr-bins --features <feat>`.

### Adding a feature that unlocks several binaries

1. Define the feature in `bins/Cargo.toml:337`.
2. List the optional crates it pulls in.
3. Set `required-features = ["<feat>"]` on each related `[[bin]]`.

### Changing default bind addresses

- Most defaults live in the underlying crate (`gnostr-chat`, `gnostr-crawler`,
  `gnostr-web`, `gnostr-relay`, etc.), not in `bins/`. Update the crate, then
  document the change here.

## Pitfalls

- `gnostr-gnit.rs` and `gnostr-org.rs` are currently identical; update both or
  refactor them to share server-building code.
- `gnostr-js` requires all three of `web`, `relay`, and `crawler` features to
  compile.
- Port collisions are common in development: `3030` (gnostr-js/embedded),
  `3333` (gnostr-gnit/org/websockets_chat), `8080` (relay), `3000` (crawler/Blossom).
- Several binaries call `webbrowser::open()` unconditionally; this breaks in
  headless CI.
- Many network tests are ignored; run with `--ignored` only when you intend to
  hit the live network.
