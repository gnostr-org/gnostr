## swift-gnostr-asyncgit

Swift asyncgit wrapper that reuses `swift-gnostr-types` and calls into Rust when the FFI dylibs are available.

### Current dependency chain

`AsyncGit` → `GnostrTypes` → `RustGnostrTypesBridge` → `gnostr-types-ffi` → `gnostr-types`

`AsyncGit` also keeps a small Rust bridge for asyncgit-specific helpers:

`AsyncGit` → `RustAsyncGitBridge` → `asyncgit-ffi` (`cdylib`) → `gnostr-asyncgit` Rust crate

### What lives where

- `swift-gnostr-types`
  - shared Nostr data types
  - Rust-backed NIP-34 helpers
- `swift-gnostr-asyncgit`
  - repo-state helpers
  - asyncgit-specific wrappers
  - uses `GnostrTypes` for shared models and NIP-34 event generation

### Runtime loading

The asyncgit bridge looks for `ASYNCGIT_FFI_LIBRARY`, then falls back to:

- `Rust/asyncgit-ffi/target/debug/libasyncgit_ffi.dylib`
- `Rust/asyncgit-ffi/target/release/libasyncgit_ffi.dylib`

The shared types bridge looks for `GNOSTR_TYPES_FFI_LIBRARY`, then falls back to:

- `Rust/gnostr-types-ffi/target/debug/libgnostr_types_ffi.dylib`
- `Rust/gnostr-types-ffi/target/release/libgnostr_types_ffi.dylib`
