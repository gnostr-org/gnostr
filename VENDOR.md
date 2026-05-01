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

## curve25519-dalek

`vendor/curve25519-dalek` is a local patch of the `5.0.0-pre.1` prerelease
snapshot. It is needed because the workspace resolves both `pkarr` and
`iroh-base` through that prerelease line, and the upstream snapshot still used
the older `digest::crypto_common` path that fails on the current toolchain.

## rpassword

`rpassword` is pinned to `=7.4.0` in the workspace so `cargo install --path .`
stays on the macOS-compatible release instead of selecting `7.5.0`.

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

## actix test crates

`vendor/actix-http-test`, `vendor/actix-test`, and `vendor/actix-http` carry
local fixes so the vendored test suite builds cleanly. The `actix-http-test`
helper now sets `SO_REUSEADDR` before bind, `actix-test` enables the
`actix-web` `macros` feature for its doctests, and `actix-http` keeps its test
targets trimmed to the upstream manifest shape.

`vendor/actix-web` keeps `tests/test_server.rs` disabled because it keeps
failing in this workspace, but `tests/test_server_openssl.rs` provides a much
smaller OpenSSL-backed smoke test for the server setup.

## cargo-test scripts

`scripts/cargo-test-vendor.sh` and `scripts/cargo-test-workspace.sh` now accept
`--target-dir` and a shared `--target-tmpdir` mode. Vendored runs copy the tree
into `$(TMPDIR)/cargo-test-vendor`, and `--target-tmpdir-clean` removes that
shared temp root first.

## host-specific tests

For tests that depend on local machine setup, prefer a runtime preflight check
with a clear skip message over a hard failure. The `actix-tls` local-address
test now uses that pattern when `127.0.0.3` is not configured on macOS.
