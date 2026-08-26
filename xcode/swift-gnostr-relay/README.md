## swift-gnostr-relay

Swift wrapper for relay lifecycle, relay discovery, and relay configuration defaults.

### Dependency chain

`Relay` → `Crawler` → `GnostrTypes`

`Relay` → `RustRelayBridge` → `relay-ffi` (`cdylib`) → `gnostr-relay` Rust crate

### What it covers

- relay `/api/relay/status`, `/start`, `/stop`, and `/discovery`
- relay process state payloads
- relay configuration defaults from the Rust relay CLI
- relay listen-endpoint normalization

### Runtime loading

The relay bridge looks for `GNOSTR_RELAY_FFI_LIBRARY`, then falls back to:

- `Rust/relay-ffi/target/debug/librelay_ffi.dylib`
- `Rust/relay-ffi/target/release/librelay_ffi.dylib`
