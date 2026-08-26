## swift-gnostr-types

Shared Swift Nostr types with an optional Rust FFI bridge.

### Current dependency chain

`GnostrTypes` → `RustGnostrTypesBridge` → `gnostr-types-ffi` (`cdylib`) → `gnostr-types` Rust crate

### What the chain provides

- Swift core models: `EventKind`, `Id`, `PublicKey`, `Signature`, `Tag`, `Event`, `GitNote`, `RepoRef`, `RepoState`
- Rust-backed NIP-34 helpers when the dylib is present:
  - `gitNoteEventID`
  - `gitNoteTags`
  - `generateGitNoteEvent`
- Rust-backed round-trips for shared core Nostr values:
  - `Event`
  - `PreEvent`
  - `Tag`
  - `NAddr`

### Runtime loading

The Swift bridge looks for `GNOSTR_TYPES_FFI_LIBRARY` first, then falls back to:

- `Rust/gnostr-types-ffi/target/debug/libgnostr_types_ffi.dylib`
- `Rust/gnostr-types-ffi/target/release/libgnostr_types_ffi.dylib`

If the dylib is missing, the Swift package still builds and uses its pure-Swift models.
