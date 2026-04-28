# Vendored dependency notes

## ed25519-dalek

`ed25519-dalek` is patched to the local copy in `vendor/ed25519-dalek`.
This keeps the workspace building against the `pkarr`/`blossom-rs` chain
without pulling the broken upstream `3.0.0-pre.1` PKCS#8 handling directly
from crates.io.

The local patch changes `src/signing.rs` so PKCS#8 validation uses
`pkcs8::Error::KeyMalformed(KeyError::Invalid)` for malformed public-key
material.

## pkarr

`git-helpers` uses `pkarr` for `git-remote-pkarr` / PKARR DHT resolution.
The current workspace keeps that dependency on the vendored path so the
`ed25519-dalek` patch above is applied consistently during builds.

## actix-web-actors

`vendor/actix-web-actors` is a vendored snapshot of the registry package, not a
local source fork. The tracked history shows it was added by
`5dd417ba93 vendor/actix-web-actors:patch`, and the only difference from the
registry copy is a normalized dependency constraint in `Cargo.toml`.

## asn1-rs

`vendor/asn1-rs` is also a vendored snapshot, and the current worktree adds
`src/tostatic.rs` plus `tests/to_static.rs`,
`tests/toder_sequence_lifetime.rs`, and `tests/toder_sequence_simple.rs`.
`.cargo-checksum.json` is the Cargo-generated vendor checksum for that crate.
