# AGENTS.md — `gnostr-crawler`

Guide for AI agents working on the Nostr relay crawler and its API server.

## Overview

`gnostr-crawler` discovers, probes, and catalogs Nostr relays. It maintains a
relay pool via `nostr_sdk::Client`, fetches relay metadata (NIP-11), crawls
relay lists from events, and serves the collected data through an Axum HTTP API.

It is used by the root `gnostr crawler` subcommand and by several binaries in
`gnostr-bins` (under the `crawler` feature).

## Source map

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root; re-exports modules |
| `src/cli.rs` | `Cli` / `Commands` — `sniper`, `watch`, `nip34`, `crawl`, `serve` |
| `src/relay_manager.rs` | `RelayManager` — relay pool, discovery, subscriptions, status monitoring |
| `src/relay_manager/sources.rs` | Bootstrap relay sources and shitlist filtering |
| `src/relay_manager/active.rs` | Thread-safe `ActiveRelayList` |
| `src/processor.rs` | Event processor; buckets relays by NIP |
| `src/relay_fetch.rs` | Concurrent NIP-11 metadata fetch over HTTP(S) |
| `src/relays.rs` | Relay file I/O, liveness checks, config paths |
| `src/relay_io.rs` | Low-level relay read/write helpers |
| `src/relays_set.rs` | Relay URL set utilities |
| `src/query/mod.rs` | Generic `REQ` query builder and sender over WebSocket |
| `src/api.rs` | Axum API server routes and startup |
| `src/commands/sniper.rs` | `sniper` command implementation |
| `src/commands/watch.rs` | `watch` command implementation |
| `src/commands/nip34.rs` | `nip34` command implementation |
| `src/commands/crawl.rs` | `crawl` command implementation |
| `tests/crawler_helpers.rs` | Integration / network matrix tests |

## Relay discovery and `RelayManager`

`RelayManager` (`src/relay_manager.rs:30`) wraps `nostr_sdk::Client`, an
`ActiveRelayList`, and a `Processor`.

- **Bootstrap sources** (`src/relay_manager/sources.rs:6`):
  - Embedded `relays.yaml` baked in via `include_bytes!`.
  - User config file `relays.yaml` in the crawler config dir.
  - Online lists fetched over HTTPS (`permissionlesstech/bitchat` CSV,
    `sesseor/nostr-relays-list` TXT).
  - Hard-coded `BOOTSTRAP_RELAYS` static.
- **Active cap** — configurable via `--max-active-relays` (default `50`) or the
  `GNOSTR_CRAWLER_MAX_ACTIVE_RELAYS` environment variable (`src/relay_manager.rs`).
- **Subscriptions** — subscribes to `ContactList` + `RecommendRelay` for the past
  6 hours.
- **Relay discovery from events** — extracts relay URLs from `RelayList` content,
  `ContactList` `p` tags, and `RecommendRelay` event content.
- **Status monitor** — background task polls every 30 s (`monitor_relays`).
- **Stop condition** — stops when all connected/connecting relays send `EOSE` or
  after 20 s of inactivity.

`ActiveRelayList` (`src/relay_manager/active.rs:6`) is an
`Arc<Mutex<Vec<RelayUrl>>>` updated by status polling.

## Network protocols

| Protocol | Usage | Location |
|----------|-------|----------|
| WebSocket (Nostr client) | Subscriptions, event ingestion, `REQ` queries | `relay_manager.rs:207-232`; `query/mod.rs:191-268` |
| HTTP(S) GET | Fetch NIP-11 metadata (`application/nostr+json`) | `relay_fetch.rs:14-59` |
| HTTP(S) HEAD | Liveness checks | `relays.rs:438-467` |
| HTTP REST API server | Axum crawler API | `api.rs:1729-1773` |

`query/mod.rs` uses `tokio_tungstenite` directly for one-shot queries, while
`RelayManager` uses `nostr_sdk::Client`'s built-in pool.

## Querying, filtering, and storing events

### Query builder

`build_gnostr_query` (`src/query/mod.rs:270`) constructs a
`["REQ","gnostr-query",filter]` JSON message. It supports authors, ids, kinds,
generic `#` tags, hashtag `#t`, mentions `#p`, references `#e`, and limit. It
strips prefixes like `kind:`, `nip=`, etc.

### Query sender

`send(query, relays, limit)` (`src/query/mod.rs:140`):

1. Iterates over the supplied relays.
2. Opens a WebSocket with a **1 s connect timeout**.
3. Sends `REQ`, collects `EVENT` frames until `EOSE` or timeout.
4. Falls back to the next relay if one returns empty or fails.

### Filtering in the API

`src/api.rs:1340` performs client-side substring search on raw JSON results
(splitting on whitespace and comma).

### Storage

Config dir: `ProjectDirs::from("org","gnostr","gnostr/crawler")` (`src/relays.rs:21`).

Written files include:

- `/relays.yaml`, `/relays.json`, `/relays.txt`
- `/<nip>/relays.yaml`, `relays.json`, `relays.txt`
- `/<nip>/<host>.json` per-relay metadata
- `kinds.txt`, `kinds.json`, `index.html`, `filetree-favorites.txt`

Helpers: `src/relays.rs:93-421`, `src/commands/sniper.rs:7-32`.

## CLI commands

`src/cli.rs:10` defines:

- `sniper <nip> [--shitlist <path>]` — find relays advertising a NIP.
- `watch [--shitlist <path>]` — crawl bootstrap relays, fetch metadata, bucket by NIP.
- `nip34 [--shitlist <path>]` — print relays advertising NIP-34.
- `crawl [ARGS]` — run the main `RelayManager` crawl.
- `serve [--port 3000] [--detach] [--no-watch] [--no-sniper] [--shitlist <path>]` — start the API server.

`dispatch_cli_command` passes a `reqwest::Client` to network commands
(`src/cli.rs:103`).

## Embedded API server (Axum)

Entry points (`src/api.rs`):

- `run_api_server(options)` — foreground.
- `run_api_server_detached(command_prefix, options)` — spawns the current exe with `serve`.
- `ServeOptions { port, no_watch, no_sniper, shitlist }` controls startup behavior.

Startup sidecars (`src/api.rs`):

- Prepares relay serve files, kinds files, `index.html`.
- Spawns `run_watch` and the sniper service as background tasks (each can be disabled with `--no-watch` / `--no-sniper`).

Routes (`src/api.rs:1729`):

```
GET /                         -> index HTML
GET /query                    -> generic nostr query form + results
GET /api/buckets              -> directory listing (JSON)
GET /api/buckets/file?path=.. -> file contents
GET /api/health               -> health check (`{"status":"ok"}`)
GET /api/stats                -> relay count, live NIPs and kinds
GET /api/sniper/logs          -> recent sniper service logs
GET /relays.{yaml,json,txt}
GET /:nip                     -> NIP index HTML
GET /:nip/query               -> NIP-specific query
GET /:nip/relays.{yaml,json,txt}
GET /:nip/:relay.json
GET /nip/:nip/...             -> alias routes
```

Bind address: `127.0.0.1:<port>` (`src/api.rs:1755`). Default port: `3000`
(`src/cli.rs:44`).

## Proxy, timeout, rate-limit, shitlist

### Shitlist

- Embedded `shitlist.yaml` loaded into `SHITLIST_RELAYS`
  (`src/processor.rs:41`).
- URL substring matching in `sources::is_shitlisted`
  (`src/relay_manager/sources.rs:62`).
- Applied in `run_sniper`, `run_watch`, `run_nip34`, and bootstrap seeding.

### Timeouts

- WebSocket query connect/read timeout: **5 s** default, configurable via
  `--query-timeout-secs` or `GNOSTR_CRAWLER_QUERY_TIMEOUT_SECS` (`src/query/mod.rs`).
- Relay liveness HEAD check: **5 s** (`src/relays.rs`).
- Relay metadata `reqwest` fetch uses default timeouts.

### Concurrency

- Metadata fetch uses `buffer_unordered(CONCURRENT_REQUESTS = 16)`
  (`src/relay_fetch.rs:7`, `56`).

### Proxy

HTTP/HTTPS proxy support for metadata fetching is enabled via `reqwest`:
set `GNOSTR_CRAWLER_PROXY`, `ALL_PROXY`, or `HTTPS_PROXY`. The crawler also
exposes `gnostr_crawler::api::build_http_client()` to build a proxied client.

## Feature flags

`bins/Cargo.toml:346`:

```toml
crawler = ["dep:gnostr-crawler", "dep:gnostr_filetreelist"]
```

Binaries gated by `crawler`:

- `gnostr_crawler`
- `gnostr_master_detail`
- `gnostr_crawler_filetree-*` variants

Other consumers:

- `gnostr-nip34` (`git` / `nip34-syndicate` feature) calls
  `gnostr_crawler::relays::bootstrap_relays()` for syndication.
- `gnostr-js` (`web`+`relay`+`crawler`) calls
  `gnostr_crawler::run_api_server(port)` for its crawler subcommand.

## Common tasks for agents

### Adding a new crawl source

1. Add the source URL/parser in `src/relay_manager/sources.rs`.
2. Update `BOOTSTRAP_RELAYS` or the online-fetch list as appropriate.
3. Ensure shitlist filtering is applied.

### Changing the API server port or bind address

- Edit `src/cli.rs` for CLI defaults.
- Edit `src/api.rs:1755` for the actual bind call.

### Adding an API route

1. Add the route in `src/api.rs:1729`.
2. Update the `index.html` / shell if the route is user-facing.
3. Update `bins/examples/gnostr_crawler.rs` if there is a relevant example.

### Adjusting relay concurrency

- `--max-active-relays` / `GNOSTR_CRAWLER_MAX_ACTIVE_RELAYS` (`src/relay_manager.rs`).
- `CONCURRENT_REQUESTS` in `src/relay_fetch.rs:7`.

## Pitfalls

- The default 5 s WebSocket query timeout may still be tight on very slow relays;
  raise it with `--query-timeout-secs` or `GNOSTR_CRAWLER_QUERY_TIMEOUT_SECS`.
- The default active relay cap is 50; raise or lower it with `--max-active-relays`
  or `GNOSTR_CRAWLER_MAX_ACTIVE_RELAYS`.
- The embedded `relays.yaml` is baked in at compile time; update the source file
  and rebuild to refresh defaults.
- Many crawler tests are network-dependent and marked `#[ignore]`.
- The API server auto-starts sidecar tasks; ensure port `3000` is free or use
  `--port`.

## IPFS / P2P awareness

The crawler can discover p2p nodes via a shared peer registry file
(`$DATA_DIR/gnostr/p2p-peers.json`, written by `gnostr-p2p`) and exposes them
at `GET /api/p2p/peers`. It also parses `ipfs://<cid>` references from text
via `nostr_envelope::extract_ipfs_cids`.

Environment/config:
- `GNOSTR_CRAWLER_IPFS` (planned) to enable richer IPFS integration.
