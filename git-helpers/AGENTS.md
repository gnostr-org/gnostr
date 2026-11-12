# AGENTS.md — `gnostr-git-helpers`

Guide for AI agents working on the standalone git remote helpers.

## Overview

`gnostr-git-helpers` implements git's remote-helper protocol for several
decentralized storage and naming backends. Each helper is exposed as both a
native crate binary (`git-helpers/src/bins/`) and a logging-enabled re-export
in `gnostr-bins` (built with `--features blossom`).

The helpers translate git's stdin/stdout remote-helper commands into network
calls, then read/write git objects from the remote.

## Source map

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root; re-exports all backends and the protocol driver |
| `src/protocol.rs` | `RemoteHelper` trait and `run_helper` stdin/stdout driver |
| `src/blossom_backend.rs` | `BlossomRemote` — Blossom BUD-01/02 over HTTPS/HTTP |
| `src/nostr_backend.rs` | `NostrRemote` — resolves `nostr://` repos via relays, then delegates to git smart HTTP |
| `src/nostr_relay.rs` | WebSocket relay query helpers for the nostr backend |
| `src/ipfs_backend.rs` | `IpfsRemote` — Kubo MFS read/write over HTTP RPC |
| `src/pkarr_backend.rs` | `PkarrRemote` — PKARR `_blossom` TXT resolution, then Blossom |
| `src/tor_backend.rs` | `TorRemote` — Blossom over a Tor SOCKS5 proxy |
| `src/auth.rs` | Nostr event signing for Blossom auth and HTTP `Authorization` headers |
| `src/bins/` | Minimal `main()` wrappers for each helper binary |

## `RemoteHelper` protocol driver

`src/protocol.rs:70` defines the trait:

```rust
pub trait RemoteHelper {
    fn capabilities(&self) -> &[&'static str];
    fn list(&mut self, for_push: bool) -> Result<Vec<GitRef>>;
    fn fetch(&mut self, cmds: Vec<FetchCmd>) -> Result<()>;
    fn push(&mut self, specs: Vec<PushSpec>) -> Result<Vec<PushResult>>;
}
```

`run_helper` (`src/protocol.rs:146`) speaks git's remote-helper protocol:

- Advertises `fetch`, `push`, `option`.
- Parses `capabilities`, `list [for-push]`, `fetch <oid> <ref>`,
  `push [+]<src>:<dst>`, and `option` lines.
- Keeps stdout clean for git; diagnostics go to stderr.

## Backends and network transports

| Helper | Binary | Backend file | Network protocol |
|--------|--------|--------------|------------------|
| `git-remote-blossom` | `bins/src/bin/git-remote-blossom.rs` | `blossom_backend.rs` | HTTPS/HTTP `reqwest` (blocking); BUD-01 upload, BUD-02 list |
| `git-remote-nostr` | `bins/src/bin/git-remote-nostr.rs` | `nostr_backend.rs` + `nostr_relay.rs` | WebSocket relay query (`tokio-tungstenite`), then HTTPS git smart protocol |
| `git-remote-ipfs` | `bins/src/bin/git-remote-ipfs.rs` | `ipfs_backend.rs` | Kubo HTTP RPC (`reqwest`) |
| `git-remote-pkarr` | `bins/src/bin/git-remote-pkarr.rs` | `pkarr_backend.rs` | PKARR DHT/relays (`pkarr` crate), then HTTPS Blossom |
| `git-remote-tor` | `bins/src/bin/git-remote-tor.rs` | `tor_backend.rs` | Tor SOCKS5 proxy + HTTP Blossom |

### URL schemes

| Backend | URL schemes |
|---------|-------------|
| Blossom | `blossom://<host>/<pubkey>/<repo>`, `blossom+https://...`, `blossom+http://...` (`blossom_backend.rs:447`) |
| Nostr | `nostr://<npub>/<repo>`, `nostr://<npub>/<relay>/<repo>`, `nostr+wss://...`, `nostr+ws://...` (`nostr_backend.rs:249`) |
| IPFS | `ipfs://<repo>`, `ipfs+api://<host>:<port>/<repo>` (`ipfs_backend.rs:359`) |
| PKARR | `pkarr://<zbase32-key>/<repo>` (`pkarr_backend.rs:146`) |
| Tor | `blossom+onion://<host.onion>/<pubkey>/<repo>`, `tor://...` (`tor_backend.rs:94`) |

## Feature flags

- `git-helpers/Cargo.toml` has **no feature flags** — all backends compile together.
- `bins/Cargo.toml` gates every remote-helper binary with `required-features = ["blossom"]`.
- `blossom` is the default feature of `gnostr-bins` and simply unlocks the helper binary set.
- Root `Cargo.toml` includes `gnostr-git-helpers` unconditionally.

## Configuration and environment

| Variable | Used by | Default |
|----------|---------|---------|
| `NOSTR_NSEC` | blossom, nostr, tor, pkarr | none — uploads/pushes unauthenticated if missing |
| `NOSTR_RELAY` | nostr | none |
| `GRASP_SERVER` | nostr | none — skips relay lookup if set |
| `IPFS_API` | ipfs | `http://127.0.0.1:5001` |
| `SOCKS5_PROXY` | tor | `socks5h://127.0.0.1:9050` |
| `NOSTR_PUBKEY` | pkarr | none — overrides pubkey derived from `NOSTR_NSEC` |

### Timeouts

- Nostr relay WebSocket query: 10 s read timeout, 3 retries with exponential
  backoff up to 30 s (`nostr_relay.rs:111-117`, `144`).
- Other backends rely on `reqwest` defaults (no explicit timeout set).

### Authentication

- Blossom uploads sign a kind:24242 auth event (`auth.rs:13`).
- Nostr push injects a kind:27235 `Authorization: Nostr <base64>` header via
  `GIT_HTTP_EXTRA_HEADER` (`nostr_backend.rs:170-171`, `auth.rs:71`).
- Tor routes all Blossom HTTP traffic through the configured SOCKS5 proxy
  (`tor_backend.rs:51-58`).

## Common tasks for agents

### Adding a new remote-helper backend

1. Implement `RemoteHelper` in a new backend module.
2. Add a `src/bins/git-remote-<name>.rs` wrapper.
3. Add a re-export binary in `bins/src/bin/git-remote-<name>.rs` with tracing setup.
4. Register the binary in `bins/Cargo.toml` under the `blossom` feature.
5. Update `src/lib.rs` re-exports.

### Changing how `nostr://` URLs resolve

- Edit `src/nostr_backend.rs` and `src/nostr_relay.rs`.
- `NostrRemote` delegates object transport to git smart HTTP after resolving the
  repo announcement event.

### Adding auth to a backend

- Use helpers in `src/auth.rs` to sign Nostr events.
- For HTTP-based auth, set `GIT_HTTP_EXTRA_HEADER` before shelling out to git.

## Pitfalls

- All helpers use blocking `reqwest`; long operations may block the git process.
- Missing `NOSTR_NSEC` does not fail pushes silently — uploads proceed
  unauthenticated and may be rejected by the server.
- `git-remote-nostr` depends on an external git smart-HTTP server; the relay only
  resolves the repo metadata.
- Tor bootstrap is external; the helper only proxies HTTP through SOCKS5.
- Backends share no connection pool; each helper process opens its own sockets.
