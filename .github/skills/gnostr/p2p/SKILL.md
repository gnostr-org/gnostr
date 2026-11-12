---
name: gnostr-p2p
description: Work with the P2P networking crate.
---

# p2p

Use this skill for the repo's P2P networking crate and peer-related helpers.

## Verified notes

- `gnostr -V` should be checked when behavior differs across environments.
- `gnostr-p2p` provides the repo's P2P support crate.
- `p2p/src/bridge.rs` and `p2p/src/relay_bridge.rs` provide the nostr compatibility bridge.
- `p2p/src/lib.rs` keeps a legacy `crate::p2p::...` namespace for older module paths.

## Common commands

```bash
cargo test -p gnostr-p2p -- --nocapture
cargo check -p gnostr-p2p
cargo test -p gnostr-p2p relay_bridge::tests::relay_bridge_round_trips_messages -- --nocapture
```

## Rules

- Keep peer messaging behavior explicit.
- Do not introduce hidden network side effects.
- Keep bridge changes compatible with the legacy module namespace.
