# AGENTS.md — `gnostr-p2p`

Guide for AI agents working on the `gnostr-p2p` libp2p networking crate.

## Overview

`gnostr-p2p` wraps `libp2p` 0.54 into a reusable P2P layer used by chat,
crawler broadcast, git publishing, and the standalone P2P binaries. It also
contains an optional Tor transport (`arti-client`) and a BFT logical clock
for distributed time sync.

## Source map

| File | Purpose |
|------|---------|
| `src/lib.rs` | Public surface; re-exports, `keypair_from_seed`, feature gating |
| `src/swarm_builder.rs` | `build_swarm`, `build_behaviour` — main swarm constructor |
| `src/behaviour.rs` | Composite `NetworkBehaviour` used by the main swarm |
| `src/network_config.rs` | Network enum, bootnodes, protocol/topic env helpers |
| `src/cli.rs` | Shared CLI opts (`NodeOpts`, `NetworkOpts`, `LookupOpts`) |
| `src/embedded_network.rs` | In-process embedded network: start/stop, peer registry, chat encryption |
| `src/event_handler.rs` | Swarm event handler (mDNS, identify, kad, gossipsub, relay, rendezvous) |
| `src/command_handler.rs` | Interactive stdin commands (TOPIC, GET, PUT, etc.) |
| `src/time.rs` | BFT time sync: `SyncState`, `P2PClock`, `run_time_sync_daemon` |
| `src/tor.rs` / `src/tor/*.rs` | Optional Tor transport via `arti-client` |
| `src/lookup.rs` | `LookupClient` for direct DHT/address peer lookup |
| `src/crawler_broadcast.rs` | Relay bucket gossip + Nostr relay publishing |
| `src/relay_bridge.rs` | Nostr relay WebSocket bridge from the P2P side |
| `src/kvs.rs` | Request/response file-exchange demo over Kademlia |
| `src/fractal.rs`, `src/perfect_ip.rs` | Experimental QUIC repair/gossip protocols |
| `src/git_publisher.rs` | Publish git tags/commits/diffs to Kademlia + gossipsub |
| `src/utils.rs` | `multiaddr_with_peer_id`, logging init, key generation |

## libp2p stack

### Transports

Configured in `src/swarm_builder.rs:build_swarm`:

- TCP + Noise + Yamux
- QUIC
- DNS (except iOS/tvOS)
- WebSocket over TCP (except iOS/tvOS)
- Optional Tor transport (`tor` feature)
- Relay client transport

### Protocols / behaviours

Composite `Behaviour` in `src/behaviour.rs`:

- `relay_client` / `relay_server` — circuit relay
- `autonat` — NAT status probing
- `dcutr` — hole punching
- `gossipsub` — pub/sub (chat, presence, relay buckets, clock alerts)
- `kademlia` — primary DHT
- `ipfs` — secondary Kademlia on `/ipfs/kad/1.0.0`
- `identify` — peer info exchange
- `rendezvous_client` / `rendezvous` — namespace discovery
- `ping` — liveness/RTT
- `mdns` — local peer discovery

Gossipsub config uses 1s heartbeat, `Permissive` validation, and a custom
message-id hasher over `message.data`.

## Peer discovery and connection

1. **mDNS** — local LAN discovery; discovered peers are auto-dialed.
2. **Kademlia DHT** — bootstraps via IPFS bootnodes.
3. **Gossipsub presence** — `PeerPresence` JSON on `gnostr/p2p/presence`.
4. **Rendezvous** — namespace registration/discovery.
5. **AutoNAT** — external address probing and NAT classification.
6. **Identify** — adds observed external addresses.

Discovered addresses are normalized with `utils::multiaddr_with_peer_id`,
added to Kademlia/AutoNAT, and dialed.

## Tor support

Feature: `tor` (and `listen-onion-service` for hidden services).

- Uses `arti-client` 0.21 with `tokio`, `rustls`, `onion-service-client`.
- `TorTransport` implements libp2p `Transport`.
- `build_transport(keypair)` bootstraps Arti, wraps with Noise + Yamux.
- Address conversion modes: `DnsOnly` (`/dns*/*`, `/onion3/...`) and
  `IpAndDns` (also raw IPv4/IPv6 via `dangerous_extract`).
- Default Tor proxy for reqwest helpers is `socks5h://127.0.0.1:9050`.

All Tor code must be behind `#[cfg(feature = "tor")]`.

## BFT time sync

`src/time.rs` provides `P2PClock` and `SyncState`:

- Logical UTC clock with slew rate and monotonicity guarantee.
- `apply_bft_sync(estimates)` requires `2*f + 1` peer estimates.
- Persists checkpoint to disk as JSON.
- `run_time_sync_daemon()` runs a separate libp2p swarm on `/time/5.0` and
  `clock-alerts` gossip topic.

Default fault budget `f=1` means at least 3 peer estimates are needed.

## Key public APIs

- `gnostr_p2p::keypair_from_seed(Option<String>)` — deterministic Ed25519 keypair.
- `gnostr_p2p::swarm_builder::build_swarm(keypair)` — builds `Swarm<Behaviour>`.
- `gnostr_p2p::embedded_network::{start, stop, status, logs, peers, send_chat_message, register_chat_topic}`.
- `gnostr_p2p::time::{P2PClock, SyncState, run_time_sync_daemon}`.
- `gnostr_p2p::tor::{build_transport, TorTransport}`.
- `gnostr_p2p::crawler_broadcast::{RelayBucket, broadcast_event_to_crawler_relays}`.
- `gnostr_p2p::relay_bridge::{NostrRelayConnection, RelayBridgeSession}`.
- `gnostr_p2p::lookup::LookupClient`.

## Feature flags and environment variables

Cargo features (`p2p/Cargo.toml`):

- `default = ["js"]`
- `js` — enables `gnostr-js` dependency and browser bridge modules.
- `tor` — enables `arti-client` and Tor transport.
- `listen-onion-service` — hidden-service listening (requires `tor`).

Environment variables:

| Variable | Effect |
|----------|--------|
| `GNOSTR_P2P_PROTOCOL` | Override Kademlia protocol name |
| `GNOSTR_P2P_PROTOCOL_VERSION` | Replace last protocol segment |
| `GNOSTR_P2P_DISCOVERY_TOPIC` | Override presence topic |
| `GNOSTR_P2P_CHAT_TOPIC` | Default chat topic |
| `GNOSTR_NSEC` | P2P identity seed; also chat-topic encryption seed |
| `GNOSTR_RELAY_KIND` | Deterministic relay identity: `blockheight`/`weeble`/`wobble` |
| `HOME` / `XDG_CONFIG_HOME` | Config dir for relay buckets and chat topics |
| `RUST_LOG` | Tracing/log filter |

## Common tasks for agents

### Adding a new behaviour to the main swarm

1. Add the field to `Behaviour` in `src/behaviour.rs`.
2. Initialize it in `src/swarm_builder.rs:build_behaviour`.
3. Handle its events in `src/event_handler.rs`.
4. Update any platform-specific swarm builders (e.g., `embedded_network_tvos.rs`).

### Changing the Kademlia protocol name

- Update `src/network_config.rs:active_protocol_name()`.
- Keep `src/lookup.rs` protocol resolution in sync.

### Adding a relay-bucket broadcast target

- Buckets are loaded from `get_config_dir_path()/<nip>/relays.{yaml,json}`.
- Use `broadcast_event_to_crawler_relays`.
- Invalid/private hosts are filtered; loopback allowed only via `is_loopback_relay`.

### Working with the embedded network

- `start()` spawns a thread with its own Tokio runtime; do not call from inside
  another Tokio runtime without expecting nesting.
- Chat topics are encrypted to `GNOSTR_NSEC` and stored in platform config dir.

## Pitfalls

- `keypair_from_seed` is deterministic; tests often rely on the default weak
  gnostr key. Do not use real private keys in tests or CI logs.
- Gossipsub validation is `Permissive`; do not assume cryptographic validation.
- `SyncState` uses `Arc<RwLock<>>`; keep lock scopes short to avoid blocking
  `now_utc`.
- Tor bootstrap can be slow and may fail if no Tor network is reachable.
- Many P2P tests are network-dependent and marked `#[ignore]`.
- `p2p/` uses default rustfmt; run `cargo fmt --all --check`.

## IPFS (Kubo) interop (Phase 1)

`gnostr-p2p` can derive a shared swarm key from a Nostr nsec when built with
`--features ipfs` (requires Go toolchain for `ipfs/ffi/rust`):

- `gnostr_p2p::swarm_key_from_nsec(nsec_or_hex)` returns an IPFS PSK string.
- `NodeOpts`/`LookupOpts` accept `--swarm-key` / `GNOSTR_P2P_SWARM_KEY`.

This is the first step toward a shared identity/swarm between the libp2p and
Kubo planes; full peer/content bridging is planned in later phases.

## IPFS (Kubo) interop (Phase 2)

`gnostr-p2p` can host an embedded Kubo node when built with `--features ipfs`
(requires Go toolchain for `ipfs/ffi/rust`):

- `gnostr_p2p::ipfs::init_global(repo_path, online)` starts the node and
  stores a process-global `Arc<IpfsState>`; `gnostr_p2p::ipfs::global()`
  retrieves it.
- `NodeOpts`/`LookupOpts` accept `--ipfs-repo` / `GNOSTR_P2P_IPFS_REPO` and
  `--ipfs-online` / `GNOSTR_P2P_IPFS_ONLINE`.

This shares the Kubo node handle within the p2p process; cross-process sharing
will be addressed in later phases.

## IPFS (Kubo) interop (Phase 3)

`gnostr-p2p` provides bridge helpers (`ipfs_bridge` module, cfg `ipfs`) to
exchange peer info with an embedded Kubo node:

- `kubo_peer_id`, `kubo_listening_addrs`, `kubo_swarm_peers`
- `connect_kubo(state, addr)` — dial an address from Kubo's side
- `kubo_bootstrap_info(state)` — (peer_id, addrs) for seeding the swarm

These allow p2p code to bootstrap from and dial into a Kubo node.

## IPFS (Kubo) interop (Phase 4)

Content-exchange helpers in `ipfs_bridge` (cfg `ipfs`):

- `fetch_block(state, cid)` / `provide_block(state, data)` — blockstore access.
- `fetch_via_gateway(gateway_url, cid)` — async HTTP fetch from a Kubo gateway.
- `CID_ANNOUNCE_TOPIC` and `announce_message(cid, peer_id)` — gossipsub
  announce format for CID discovery.

Subscribers can announce CIDs over gossipsub and fetch blocks via Kubo.
