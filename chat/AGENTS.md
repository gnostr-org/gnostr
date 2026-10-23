# AGENTS.md — `gnostr-chat`

Guide for AI agents working on the `gnostr-chat` P2P chat and TUI crate.

## Overview

`gnostr-chat` provides a libp2p-based chat transport, a ratatui TUI, and a
headless mode. It is used by the root `gnostr chat` subcommand and by
feature-gated binaries in `gnostr-bins`.

## Source map

| File | Purpose |
|------|---------|
| `src/lib.rs` | Crate root; re-exports modules |
| `src/command.rs` | CLI args (`ChatSubCommands`), logging setup, mode dispatch (`run`, `run_oneshot`, headless) |
| `src/session.rs` | `ChatSession` handle; wires command/output channels, readiness watch, notifications |
| `src/p2p.rs` | Swarm construction, `evt_loop`, local relay service, mDNS/dial logic, message queueing, chunk reassembly |
| `src/event.rs` | Internal `ChatEvent` enum |
| `src/message.rs` | Re-export of `gnostr_p2p::message::*` (Nostr/git-note wire types) |
| `src/msg.rs` | Chat-specific `Msg` / `MsgKind` types and TUI line rendering |
| `src/tui.rs` | Ratatui TUI (`run_chat_tui`, `ChatTui`) |
| `src/tests/p2p_chat_integration_tests.rs` | Relay-probe and multi-peer connectivity tests (mostly `#[ignore]` / `long_tests`) |

## `ChatSession`

`ChatSession::connect(topic).await` (`src/session.rs:32`) is the main entry point:

1. Starts `spawn_local_p2p_relay_service_async()` (`src/p2p.rs:134`).
2. Creates an mpsc command channel (`command_tx` / `command_rx`).
3. Creates an mpsc output channel and a `broadcast::Sender<ChatNotification>`.
4. Creates a `watch::Sender<bool>` that becomes `true` on the first
   `PeerConnected` event.
5. Spawns `evt_loop(command_rx, output_tx, topic_handle)` owning the libp2p swarm.
6. Spawns a bridge task that converts internal `ChatEvent`s into public
   `ChatNotification`s.

Public methods:

- `subscribe() -> broadcast::Receiver<ChatNotification>` — receive messages,
  errors, connection info, and `MessagePublished` confirmations.
- `wait_for_connected(Duration)` — waits for the readiness watch to become true.
- `send_text(text)` — builds a `Msg` and sends `ChatEvent::ChatMessage`.
- `send_crawler_search(nip)` — sends `ChatEvent::CrawlerSearch`.

## Local P2P relay service

`LocalP2pRelayService` (`src/p2p.rs:102`) is an independent libp2p peer with
`RelayBehaviour` (relay client/server, autonat, dcutr, ping, identify). It
listens on ephemeral 127.0.0.1 TCP and UDP/QUIC ports and includes the Tor
transport from `gnostr_p2p`. The chat swarm includes a `relay::client::Behaviour`
so it can use this service for hole punching and circuit connections.

## Message send/receive

Send path:

- `ChatSession::send_text` → `ChatEvent::ChatMessage(Msg)` → `evt_loop` command channel.
- `publish_or_queue_chat_message` (`src/p2p.rs:43`) serializes the `Msg` to JSON
  and calls `gossipsub.publish(topic, payload)`.
- If Gossipsub returns an "insufficient peers" error, the message is queued in
  `pending_chat_messages` and retried every second (`flush_pending_chat_messages`,
  `src/p2p.rs:72`) and on each `ConnectionEstablished`.

Receive path:

- `SwarmEvent::Behaviour(MyBehaviourEvent::Gossipsub(...))` (`src/p2p.rs:701`).
- Deserializes the payload into `Msg`.
- If the message has chunking metadata, it is reassembled via `MessageReassembler`.
- Text wrapping is applied and `ChatEvent::ChatMessage(msg)` is sent to the
  output channel.

## mDNS discovery and dialing

The chat swarm enables `libp2p::mdns::tokio::Behaviour`. On `Mdns::Discovered`,
it:

1. Adds the peer as a Gossipsub explicit peer.
2. Appends `/p2p/<peer_id>` via `gnostr_p2p::utils::multiaddr_with_peer_id`.
3. Registers it as an AutoNAT server.
4. Calls `swarm.dial(address_with_p2p)`.

On `Mdns::Expired`, the peer is removed from Gossipsub explicit peers and
AutoNAT servers.

## Runtime modes

### `--oneshot`

`run_oneshot` (`src/command.rs:199`):

- Refused if `--headless` is also passed.
- Waits up to 30 seconds for a connected peer.
- Records a P2P clock timestamp.
- Sends the message and waits up to 30 seconds for a
  `ChatNotification::MessagePublished` confirmation.
- Sleeps 500 ms to let Gossipsub propagate, then exits.

Useful for scripts like `scripts/gnostr-chat-one-shot.sh`.

### `--headless`

Headless mode (`src/command.rs:156`) sets the process title, spawns a
notification printer, and blocks on `tokio::signal::ctrl_c()`. Intended for
long-running background peers / Docker containers.

### TUI mode

`run_chat_tui` (`src/tui.rs:30`) enters raw mode, polls crossterm events every
75 ms, drains notifications into the message list, and supports composer/timeline
focus, sending messages, `/crawler <nip>` search, and a help overlay.

## Message chunking

`Msg` carries optional chunking metadata (`message_id`, `sequence_num`,
`total_chunks`). `MessageReassembler` (`src/p2p.rs:231`) buffers chunks by
`message_id` and returns the reassembled message when all chunks arrive.

Important: the send path does **not** currently split messages into chunks.
Chunk reassembly is ready for future large-payload support but is not exercised
by normal chat sends.

## Common tasks for agents

### Adding a new chat event type

1. Add the variant to `ChatEvent` in `src/event.rs`.
2. Handle it in `evt_loop` (`src/p2p.rs`) if it is a command.
3. Convert it to a `ChatNotification` in `forward_updates` (`src/session.rs:119`).
4. Handle the notification in `spawn_notification_printer` (`src/command.rs:231`)
   and the TUI's `drain_incoming` (`src/tui.rs:124`).

### Changing how messages are published

- Edit `publish_or_queue_chat_message` and `flush_pending_chat_messages` in
  `src/p2p.rs`.
- The 1-second flush interval is set in `evt_loop` (`src/p2p.rs`).

### Changing TUI rendering

- `MsgKind` variants render in `src/msg.rs` (`impl From<&Msg> for Line`).
- TUI screens and widgets are in `src/tui.rs`.

## Pitfalls

- "queued chat message until a peer connects" means no peer was subscribed to
  the topic yet. Ensure a receiver is running and discovered via mDNS.
- mDNS only works on the same LAN; different network segments need relay
  dialing or a bootstrap peer.
- `--oneshot` depends on at least one peer connecting within 30 seconds.
- The notification printer's `JoinHandle` is explicitly aborted in oneshot mode;
  in other modes it detaches if only dropped.
- Gossipsub message IDs are hashed from payload data; identical content from the
  same sender may be deduplicated.
- Many chat integration tests are network-dependent and marked `#[ignore]`.

## IPFS integration

Chat can store attachments in IPFS when built with `--features ipfs`.

- `Msg.attachments: Vec<Attachment>` carries content-addressed files.
- `crate::ipfs::attach_file(path)` adds a file to an embedded Kubo node.
- `ChatSession::send_text_with_attachment(text, path)` sends a message with an
  attachment (used by `--oneshot --attach <path>`).
- Tests: `cargo test -p gnostr-chat --features ipfs` (ignored tests require
  the Go toolchain).
