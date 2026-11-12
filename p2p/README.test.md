## gnostr-p2p

`p2p` is the libp2p networking crate for gnostr. It owns swarm construction, peer lookup helpers, relay and crawler broadcast wiring, deterministic identity generation, and the attestation syndication path used by the tests.

## What it can do

- Build a libp2p swarm with TCP, QUIC, DNS, WebSocket, relay, mDNS, kademlia, identify, ping, rendezvous, autonat, and dcutr.
- Add the local `libp2p-tor` transport with the `tor` feature.
- Expose browser-side assets and bridge helpers with the `js` feature.
- Handle interactive swarm commands such as `TOPIC`, `GET`, `GET_PROVIDERS`, `PUT`, `PUT_PROVIDER`, and `CRAWLER_BUCKETS`.
- Publish crawler-discovered relay buckets and public attestation events through the relay bridge.

## Features

- `js` keeps the browser-side bundle enabled by default.
- `tor` adds the local `libp2p-tor` transport to the swarm builder.

## Testing help

- Use `--nocapture` so the printed attestation payload and relay destinations stay visible.
- `pretty_print_attestations` keeps the same commit → event → note shape as `asyncgit`.
- Real-event coverage must not assume buckets are pre-primed; the test should fetch the live crawler relay list and seed a temp bucket from that output before broadcasting.
- If the crawler server is not already running, start it first:

```sh
gnostr crawler serve --port 8080 --detach
```

- The live crawler relay list is served from `http://127.0.0.1:8080/relays.yaml`.
