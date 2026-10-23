# AGENTS.md — `gnostr-web` / `gnostr-js`

Guide for AI agents working on the web-server and embedded-asset crates.

## Overview

The workspace has two closely related web crates:

- **`gnostr-js`** — a library of embedded static assets and small warp-based web
  servers (embedded Nostr web app, relay launcher, crawler control, repo flattener).
- **`gnostr-web`** — a library + binary. The library provides the Git-backed
  `gnostr-gnit`/`gnostr-org` axum server; the binary (`web/src/main.rs`) delegates
  to `gnostr_js::web_app::run(port)`, so `gnostr-web` binary is actually the
  embedded Nostr web app.

Do not confuse these with the root `gnostr server` subcommand, which is a
Blossom blob-server wrapper and shares no code paths.

## Source map

### `gnostr-web` (`web/`)

| File | Purpose |
|------|---------|
| `src/main.rs` | Binary entry point (`gnostr-web`) → delegates to `gnostr_js::web_app::run` |
| `src/lib.rs` | Re-exports `app::*`, plus `css`, `images`, `theme` modules |
| `src/app/mod.rs` | Declares modules: `assets`, `startup`, `methods`, `database`, `git`, `layers`, ... |
| `src/app/startup.rs` | `Config`, `GnitArgs`, `open_database`, `run_indexer`, `init_static_asset_hashes`, `find_available_port` |
| `src/app/assets.rs` | `include_bytes!` for compiled CSS/JS bundle and SVG images |
| `src/app/methods/mod.rs` | Route handler modules: `index`, `gnostr`, `repo` |
| `src/app/methods/index.rs` | Askama `View` template + repo-list/SPA shell handlers |
| `src/app/methods/gnostr.rs` | `/gnostr` redirect handler |
| `src/app/methods/repo/mod.rs` | Custom fallback router for repo-scoped paths |
| `src/app/methods/repo/smart_git.rs` | Proxies Git smart-HTTP via `git http-backend` |
| `src/app/layers/logger.rs` | `LoggingMiddleware` tower service |
| `src/app/git.rs` | `Git` helper axum extension |
| `src/app/database/*` | RocksDB schema and indexer for scanned Git repos |
| `build.rs` | Compiles SCSS, builds `bundle.js` from `gnostr_js` assets |
| `templates/*.html` | Askama templates rendered server-side |

### `gnostr-js` (`js/`)

| File | Purpose |
|------|---------|
| `src/lib.rs` | Public modules; re-exports `embedded::*` helpers |
| `src/web_app.rs` | `run(port)` — main embedded warp web app; also starts relay and crawler |
| `src/relay_app.rs` | `run()` — launches `gnostr-relay` via `gnostr_relay::launcher` |
| `src/crawler_control.rs` | Detached crawler lifecycle helpers |
| `src/relay_control.rs` | Detached relay lifecycle helpers |
| `src/utils/detach.rs` | Unix-centric detached-process utilities |
| `src/bridge.rs` | `shell_html()`, `asset_content_type()`, `asset_response()` |
| `src/embedded.rs` | Re-exports bundle getters |
| `src/template_html.rs` | `TemplateHtml` wrapper around `template.html` |
| `src/websock_index_html.rs` | `WEBSOCKET_INDEX_HTML` for the chat demo |
| `src/flat/mod.rs`, `scan.rs`, `render.rs` | Repo flattening logic |
| `src/css/css_bundle.rs`, `src/js/js_bundle.rs`, `src/images/images_bundle.rs`, `src/pwa/pwa_bundle.rs` | Static-asset maps |
| `build.rs` | Writes NIP-89 app metadata |

## Web frameworks

| Framework | Used by |
|-----------|---------|
| **axum 0.7** | `gnostr-gnit`, `gnostr-org`, mock server in `src/lib/sub_commands/server.rs` |
| **warp 0.3** | `gnostr-js` embedded app, `websockets_chat` demo |
| **actix-web 4** | `gnostr-js` binary main, actual relay server implementation |

## Endpoints and route registration

### `gnostr-js` warp app (`js/src/web_app.rs`)

- `GET /` → redirect to `/nip`
- `GET /nip` → shell HTML
- `GET /messages`, `/gnostr`, `/relays`
- `GET /nip/34`, `/nip/34/<kind>`, `/nip/34/query`, `/nip/34/relays.{yaml,json,txt}`, `/nip/34/<file>`
- `GET /repository-details/<repo_id>`
- `GET /flat?repo=<url>&ref=<ref>&check/precheck=<bool>`
- `GET /api/relay/status`, `POST /api/relay/start`, `POST /api/relay/stop`
- `GET /api/relay/discovery`
- Static prefixes: `/js/*`, `/css/*`, `/images/*`, `/pwa/*`
- Catch-all → shell HTML

### `gnostr-gnit` / `gnostr-org` axum app

- `GET /` → repo-list
- `GET /gnostr` → redirect
- SPA shell routes: `/search`, `/thread/:thread_id`, `/messages`, `/dm`, ...
- Hashed static assets: `/style-{hash}.css`, `/js-{hash}.js`, favicons
- Fallback chain: `.fallback(methods::index::handle_spa).fallback(methods::repo::service)`
- Layers: `TimeoutLayer`, `LoggingMiddleware`, `Extension(Git)`, `Extension(db)`, `CorsLayer`

### Repo paths (`web/src/app/methods/repo/mod.rs`)

`service()` is a custom fallback handler that walks URI segments, finds a repo
name on disk and in RocksDB, then dispatches:

- `about`, `refs`, `log`, `tree`, `commit`, `diff`, `patch`, `tag`,
  `snapshot`, `thread`, `git-upload-pack`

`git-upload-pack` is proxied through `git http-backend` in `smart_git.rs`.

### `websockets_chat` demo

- `GET /chat` → warp WebSocket upgrade (broadcast chat)
- `GET /` → `WEBSOCKET_INDEX_HTML`

## Static assets

### `gnostr-js`

- Asset maps built with `include_bytes!` in `js/src/{css,js,images,pwa}/*_bundle.rs`.
- `template.html` embedded in `src/template_html.rs`.
- `src/bridge.rs::asset_response()` serves bytes with MIME types.

### `gnostr-web`

- `web/build.rs` compiles SCSS and concatenates JS into `OUT_DIR`.
- `web/src/app/assets.rs` embeds built artifacts via `include_bytes!(concat!(env!("OUT_DIR"), ...))`.
- Hashed filenames generated at startup by `init_static_asset_hashes()` using
  `xxhash-rust`.

## Binding and listening

| Server | Bind code | Default address |
|--------|-----------|-----------------|
| `gnostr-js` web app | `warp::serve(routes).run(([127,0,0,1], port)).await` | `127.0.0.1:3030` |
| `gnostr-gnit` / `gnostr-org` | `TcpListener::bind(...)` then `axum::serve(...)` | `127.0.0.1:3333` (env `GNOSTR_GNIT_BIND_PORT`) |
| `websockets_chat` | `warp::serve(routes).run(([127,0,0,1], port)).await` | `127.0.0.1:3333` |
| `gnostr-relay` (launched by `relay_app`) | `actix-web` `HttpServer` inside `gnostr-relay` | `127.0.0.1:8080` |

All warp-based servers call `webbrowser::open()` to launch a browser tab.

## Key public APIs

- `gnostr_js::web_app::run(port)`
- `gnostr_js::relay_app::run()`
- `gnostr_js::crawler_control::{start_crawler, crawler_status}`
- `gnostr_js::relay_control::{start_relay, stop_relay, relay_status}`
- `gnostr_js::flat::{run, build_html}`
- `gnostr_js::embedded::{get_css_assets, get_js_assets, get_images_assets, get_pwa_assets, shell_html}`
- `gnostr_web::app::{Config, GnitArgs, open_database, run_indexer, init_static_asset_hashes, into_response, methods, Git}`
- `gnostr_web::app::methods::repo::service`
- `gnostr_web::app::layers::logger::LoggingMiddleware`

## Feature flags

- `ipfs` (optional, `gnostr-web`): enables the embedded Kubo (IPFS) node via
  `ipfs/ffi/rust` (`kubo-ffi`). Off by default so a plain
  `cargo build -p gnostr-web` does not require a Go toolchain. Enable with
  `--features ipfs`. When enabled, the server starts a Kubo node + gateway and
  exposes `/api/ipfs/*`.
- Root `gnostr` crate: `blossom` enables the `server` subcommand (Blossom), not
  the web UI.
- `gnostr-bins` crate:
  - `web` → enables `gnostr-web` and `gnostr-js` optional deps.
  - `relay` → `gnostr-relay`
  - `crawler` → `gnostr-crawler`
  - The `gnostr-js` binary additionally requires both `relay` and `crawler`
    features.

## Binaries

| Binary | Crate / source | Framework | Notes |
|--------|----------------|-----------|-------|
| `gnostr-web` | `web/src/main.rs` | warp (via `gnostr-js`) | delegates to `web_app::run` |
| `gnostr-gnit` | `bins/src/bin/gnostr-gnit.rs` | axum | Git repo browser/server |
| `gnostr-org` | `bins/src/bin/gnostr-org.rs` | axum | identical code to `gnostr-gnit` |
| `gnostr-js` | `bins/src/bin/gnostr-js.rs` | warp + actix-web | web / relay / crawler subcommands |
| `gnostr-js-relay` | `bins/src/bin/gnostr-js-relay.rs` | actix-web | launches relay only |
| `embedded_js`, `embedded_main_js` | `bins/src/bin/embedded_js.rs`, `embedded_main_js.rs` | warp | thin wrappers |
| `websockets_chat` | `bins/src/bin/websockets_chat.rs` | warp | chat demo |
| `flat` | `bins/src/bin/flat.rs` | — | repo flattener |

## Common tasks for agents

### Adding a route to the embedded warp app

Edit `js/src/web_app.rs`, add a `warp::path(...)` filter, and include it in the
final `.or(...)` chain.

### Adding a route to the axum Git/SPA app

Edit `bins/src/bin/gnostr-gnit.rs` **and** `bins/src/bin/gnostr-org.rs` (they
are currently exact copies), or refactor them to share server-building code.

### Adding static assets

1. Add the file to the appropriate `js/src/<type>/..._bundle.rs` or
   `web/src/app/assets.rs`.
2. For `gnostr-gnit`/`gnostr-org`, register a hashed route in both binaries.
3. Update `web/build.rs` if SCSS compilation or JS bundling is needed.
4. Emit `cargo:rerun-if-changed` in the relevant `build.rs`.

## Pitfalls

- `gnostr-gnit.rs` and `gnostr-org.rs` are identical; update both or refactor.
- The axum router uses `.fallback(...).fallback(...)`; the second fallback
  overwrites the first.
- `gnostr-js` web app auto-starts a relay on `127.0.0.1:8080` and a crawler on
  `127.0.0.1:3000`; these ports can conflict.
- Detached helpers are Unix-oriented (pid files, `lsof`, `setsid`, `kill`).
- `webbrowser::open()` is called unconditionally in warp servers; this can break
  headless CI.
- `web/build.rs` depends on `gnostr_js` as a build dependency; ensure `js`
  compiles before `web` when adding assets.
- Do not confuse `gnostr server` (Blossom) with `gnostr-web`/`gnostr-js`.

## Testing

- Run IPFS-related tests with `--features ipfs`:
  `cargo test -p gnostr-web --features ipfs`
- The `node_roundtrip` test is `#[ignore]`d because it requires the Go toolchain to build `kubo-ffi`; run it with `-- --ignored`.

## IPFS / Helia interop

`gnostr-web` can embed a Kubo (IPFS) node when built with `--features ipfs`.
The node starts at process startup and exposes:

- `GET /api/ipfs/status` — `{ enabled, gateway, error }`
- `POST /api/ipfs/add` — add bytes, returns `{ cid }`
- `POST /api/ipfs/add-batch` — add many items, returns `{ cids }`
- `GET /api/ipfs/cat/:cid` — fetch stored bytes
- `POST /api/ipfs/pin/:cid` — pin a CID
- `GET /api/ipfs/ls` — list pinned CIDs
- `POST /api/ipfs/manifest` — publish a manifest JSON, returns `{ cid }`
- `GET /api/ipfs/peers` — list swarm peers
- `GET /ipfs/:cid` — redirect to the Kubo gateway

The browser Helia node talks to Kubo via these endpoints and the local gateway
URL returned by `/api/ipfs/status`. CIDs are linked to the gateway so content
can be retrieved directly.

Configuration (only meaningful with `--features ipfs`):

- `--ipfs-repo <path>` / `GNOSTR_IPFS_REPO` — Kubo repository location
  (default: OS app data dir `…/gnostr/ipfs`).
- `--ipfs-online` / `GNOSTR_IPFS_ONLINE=true` — enable swarm participation
  (default: offline, local-only).

The Kubo FFI lives in `../ipfs/ffi/rust` (`kubo-ffi`) and requires a Go
toolchain at build time. The `ipfs` feature is off by default so a plain
`cargo build -p gnostr-web` does not need Go.
