# AGENTS.md — `gnostr-relay`

Guide for AI agents working on the Nostr relay crate and its vendored
implementation.

## Overview

`gnostr-relay` is a thin wrapper around a vendored fork of `nostr-relay`
0.4.8. The wrapper adds gnostr-specific conveniences (endpoint file helpers,
launcher, CLI), while the vendored crate provides the actual Actix-web/WebSocket
relay implementation.

## Source map

| Path | Purpose |
|------|---------|
| `relay/Cargo.toml` | Crate manifest; depends on `nostr-relay` 0.4.8 and `actix-web` |
| `relay/src/lib.rs` | Public API: re-exports, endpoint-file helpers, server runners |
| `relay/src/launcher.rs` | Standalone `gnostr-relay` binary launcher |
| `relay/src/cli.rs` | `RelayCli` clap struct for the standalone binary |
| `relay/extensions/src/lib.rs` | Re-export: `pub use nostr_extensions::*;` |
| `relay/extensions/Cargo.toml` | Extension feature flags |
| `vendor/nostr-relay/src/app.rs` | `App` struct, routes, `App::create`, `create_web_app` |
| `vendor/nostr-relay/src/server.rs` | Actix `Server` actor; dispatches EVENT/REQ/CLOSE/AUTH/COUNT |
| `vendor/nostr-relay/src/session.rs` | Per-WebSocket `Session` actor; heartbeat, extension hooks |
| `vendor/nostr-relay/src/message.rs` | `IncomingMessage`, `OutgoingMessage`, `ClientMessage`, subscriptions |
| `vendor/nostr-relay/src/setting.rs` | `Setting`, `SettingWrapper`, config loading/watching, NIP-11 info |
| `vendor/nostr-relay/src/reader.rs` / `writer.rs` / `subscriber.rs` | DB read workers, event write worker, subscription dispatch |
| `vendor/nostr-relay/src/extension.rs` | `Extension` trait |

## Bind, listen, and WebSocket flow

1. `relay/src/lib.rs:run_app_with_endpoint(app_data)` reads
   `setting.network.host`/`port`.
2. Binds a `std::net::TcpListener` and writes the bound endpoint to:
   - `<data_path>/relay-endpoint`
   - `<ProjectDirs::data_local_dir()>/relay-endpoint`
3. Starts `actix_web::HttpServer` with `create_web_app(data)`.

WebSocket route (`vendor/nostr-relay/src/app.rs`):

- Root path `/` handles:
  - `Upgrade: websocket` → WebSocket session.
  - `Accept: application/nostr+json` → NIP-11 relay information document.
  - otherwise → redirect or description.
- Each connection becomes a `Session` actor registered with the central `Server`
  actor.

## Relay extensions

`gnostr-extensions` (`relay/extensions/`) is a re-export crate for
`nostr-extensions` 0.4.8. Extensions implement the `Extension` trait in
`vendor/nostr-relay/src/extension.rs`:

```rust
pub trait Extension: Send + Sync {
    fn name(&self) -> &'static str;
    fn setting(&mut self, setting: &SettingWrapper);
    fn config_web(&mut self, cfg: &mut ServiceConfig);
    fn connected(&self, session: &mut Session, ctx: ...);
    fn disconnected(&self, session: &mut Session, ctx: ...);
    fn message(&self, msg: ClientMessage, session: &mut Session, ctx: ...)
        -> ExtensionMessageResult;
}
```

Available extensions:

| Extension | Feature | Config key | Purpose |
|-----------|---------|------------|---------|
| Auth | always compiled | `[auth]` | NIP-42 authentication; IP/pubkey white/blacklists |
| Metrics | `metrics` (default) | `[metrics]` | Prometheus endpoint at `/metrics?auth=...` |
| Ratelimiter | `rate_limiter` (default) | `[rate_limiter]` | IP-based rate limiting per event kind |
| Count | `count` (default) | `[count]` | NIP-45 COUNT support |
| Search | `search` | `[search]` | Disabled due to `lindera-unidic` compatibility |

Add extensions via `App::add_extension`. See `bins/examples/gnostr_relay_ext_demo.rs`.

## Configuration

Relay server config template: `config/gnostr.toml`. Default runtime path:
`.gnostr/relay.toml`.

Key networking sections (`vendor/nostr-relay/src/setting.rs`):

```toml
[network]
host = "127.0.0.1"   # bind interface; 0.0.0.0 for all
port = 8080
heartbeat_interval = "1m"
index_redirect_to = "https://gnostr.org"

[thread]
http = 0       # 0 == num_cpus
reader = 0

[limitation]
max_message_length = 524288
max_subscriptions = 20
max_filters = 10
max_limit = 300
max_subid_length = 100
min_prefix = 10
max_event_tags = 5000
max_event_time_older_than_now = 94608000
max_event_time_newer_than_now = 900
```

Extensions have their own sections (`[metrics]`, `[auth]`, `[rate_limiter]`,
`[count]`).

Other config files in `config/`:

- `relays.yaml` — bootstrap relay list used by the **crawler**, not the relay.
- `shitlist.yaml` — blocklist consumed by the crawler.

## Root CLI `gnostr relay`

Files:

- `src/lib/cli.rs:294` — `GnostrCommands::Relay(relay::RelaySubCommand)`
- `src/lib/sub_commands/relay.rs` — full implementation
- `src/main.rs:477` — dispatch

`RelaySubCommand` args:

- `--config` / `-c` — config path
- `--data` / `-d` — data dir (default `.gnostr/relay`)
- `--watch` / `-w` — watch config for changes
- `--logging` — log level
- `--detach` — daemon mode (Unix only)

Behavior:

1. Resolves config: explicit `--config` → `.gnostr/relay.toml` if present → defaults.
2. Creates `gnostr_relay::App` via `create_relay_app`.
3. Runs inside an `actix_web::rt::System` on a `tokio::task::spawn_blocking` thread.
4. Detects LMDB version mismatch (`MDB_VERSION_MISMATCH`), backs up stale `events/`
   dir, and retries.

Standalone binary:

- `bins/src/bin/gnostr-relay.rs` uses `gnostr_relay::launcher::run(...)` and is
  gated by the `relay` feature in `gnostr-bins`.

## Key public APIs

| Name | Location | Purpose |
|------|----------|---------|
| `gnostr_relay::App` | `vendor/nostr-relay/src/app.rs` | Core relay application state |
| `App::create(setting_path, watch, env_prefix, data_path)` | `app.rs` | Load config, open LMDB, spawn actors |
| `App::add_extension` | `app.rs` | Chainable extension registration |
| `gnostr_relay::run_app_with_endpoint` | `relay/src/lib.rs` | Bind, write endpoint file, run server |
| `gnostr_relay::spawn_app_with_endpoint` | `relay/src/lib.rs` | Same but returns `ServerHandle` |
| `gnostr_relay::Setting` | `vendor/nostr-relay/src/setting.rs` | Deserializable configuration struct |
| `gnostr_relay::Extension` | `vendor/nostr-relay/src/extension.rs` | Extension trait |
| `gnostr_relay::Session` | `vendor/nostr-relay/src/session.rs` | Per-connection actor |
| `gnostr_relay::Server` | `vendor/nostr-relay/src/server.rs` | Central actor routing messages |

Endpoint discovery helpers:

- `types/src/nostr/client.rs:local_relay_endpoint_paths()` checks
  `GNOSTR_RELAY_ENDPOINT_FILE`, project data dir, `data/relay-endpoint`,
  `.gnostr/relay/relay-endpoint`.

## Common tasks for agents

### Adding a new relay extension

1. Create a type implementing `gnostr_relay::Extension`.
2. Add it with `app_data.add_extension(MyExt)`.
3. If it needs NIP-11 info/limitation entries, call
   `setting.add_information(...)` / `setting.add_limitation(...)`.
4. For HTTP routes, implement `config_web`.
5. See `bins/examples/gnostr_relay_ext_demo.rs`.

### Changing the listen address or port

- Edit `[network] host` / `port` in the relay config.
- Requires a restart; only in-memory config is hot-reloaded via `SettingWrapper`.

### Handling LMDB version mismatch

- `src/lib/sub_commands/relay.rs` detects `MDB_VERSION_MISMATCH` and renames
  the stale `<data>/events` dir to `events.mdb-version-mismatch-<timestamp>`.
- Do not assume an existing `.gnostr/relay/events` is compatible across
  `nostr-db` upgrades.

## Pitfalls

- The root `gnostr relay` subcommand runs the actix system on a blocking thread
  inside the existing tokio runtime. Do not mix tokio and actix runtimes
  carelessly.
- Default bind is `127.0.0.1:8080`; change to `0.0.0.0` only after reviewing
  auth/rate limits.
- `nostr-relay` is vendored under `vendor/nostr-relay` and patched in root
  `Cargo.toml`. Do not bump the version without updating the vendored copy and
  verifying `nostr-db` / `nostr-lmdb` compatibility.
- Relay tests use temporary data dirs and `tokio_tungstenite` clients.
