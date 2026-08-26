# swift-gnostr-ffi-relay

Swift wrapper for the Rust relay crate using FFI only.

### Dependency chain

`FFRelay` → `relay-ffi` (`cdylib`) → `gnostr-relay`

`RelayGUI` → `FFRelay`

### What it covers

- relay configuration defaults from the Rust relay CLI
- relay listen-endpoint normalization
- cross-platform SwiftUI dashboard for iOS, iPadOS, and Mac Catalyst

### Runtime loading

The bridge looks for `GNOSTR_RELAY_FFI_LIBRARY`, then falls back to:

- `Rust/relay-ffi/target/debug/librelay_ffi.dylib`
- `Rust/relay-ffi/target/release/librelay_ffi.dylib`
