# swift-gnostr-ffi-kitchensink

Umbrella Swift package for the FFI-backed gnostr stack.

### Dependency chain

`FFIKitchenSink` → `AsyncGit`, `Crawler`, `Relay`, `GnostrTypes`

### What it covers

- shared Nostr types and Rust-backed normalization helpers
- asyncgit NIP-34 helpers
- crawler relay discovery and query builders
- relay lifecycle/configuration helpers

### Runtime loading

Each subpackage keeps its own `GNOSTR_*_FFI_LIBRARY` lookup and local `Rust/.../target/...` fallback paths.

### Xcode app

Open `FFIKitchenSinkApp.xcodeproj` for the SwiftUI app target that runs the umbrella stack on iOS, macOS, and Mac Catalyst.
