# gnostr-chat

`gnostr-chat` provides the chat transport crate used by the gnostr CLI.
The standalone binary is `gnostr-chat`; the root CLI subcommand remains `gnostr chat`.

It starts an auxiliary local relay peer for session bootstrap, and that peer now supports relay-client hole punching and Tor transport.

## Shared types and repair APIs

Chat re-exports the canonical implementations without introducing new wire types:

- `gnostr_chat::types` exposes `gnostr-types`, including Nostr types under
  `types::nostr`, `FileSliceReader`, and packet primitives under `types::perfect_ip`.
- `gnostr_chat::fractal` exposes the P2P Fractal swarm, slice types, and engine
  (including the shutdown-aware engine).
- `gnostr_chat::perfect_ip_core` exposes shared packetization, parity,
  reconstruction, and the JSON repair swarm.
- `gnostr_chat::perfect_ip` preserves the existing P2P demo API.

For example, packets produced by `types::perfect_ip::packetize` can be passed
directly to `perfect_ip_core::reconstruct_payload`; both use the same
`ProtocolSlice` type. These exports do not automatically start a repair swarm
or change the chat message transport.
