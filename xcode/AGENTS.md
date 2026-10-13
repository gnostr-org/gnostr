# AGENTS.md

Guide for AI agents and automated tools working in the `xcode` workspace.

## Project overview

This directory is an umbrella Xcode/SwiftPM monorepo that collects experiments,
sample apps, reusable libraries, and FFI wrappers around three main themes:

- **Nostr / gnostr** – native Swift Nostr SDKs, Rust-backed FFI bindings, relay
  and crawler wrappers, and the Damus-derived `gnostr-app`.
- **libp2p in Swift** – a port of the libp2p networking stack to Swift/SwiftNIO,
  plus iOS/macOS sample apps that run TCP + Noise + Yamux, mDNS, Kademlia DHT,
  DCUtR, and Gossipsub.
- **Git tooling** – Swift/Objective-C wrappers around `libgit2` (MiniGit,
  GnostrGit), iOS/Mac sample apps, and helper scripts to build `libgit2`
  XCFrameworks.

The top-level `Gnostr-Xcode.xcworkspace` is intentionally minimal and currently
only references `MiniP2PSample`. Most work happens inside the individual Swift
packages and `.xcodeproj` directories, not through the workspace file.

Many subdirectories are forks or vendored copies of upstream projects (e.g.
`swift-libp2p/*`, `gnostr-sdk-ios`, `gnostr-sdk-ffi`, `swift-cross-ui`,
`swift-bundler`, `Git`/`LibGit2-iOS`). Treat each package as the source of truth
for its own build requirements.

## Key configuration files

| File | Purpose |
|------|---------|
| `Makefile` | Generates a `make <script>` target for every `scripts/*.sh`. `make help` lists them. |
| `Gnostr-Xcode.xcworkspace/contents.xcworkspacedata` | Minimal Xcode workspace definition. |
| `Gnostr-Xcode.xcworkspace/xcshareddata/swiftpm/Package.resolved` | Shared SwiftPM resolution pins used by workspace-resolved packages. |
| `clone-org.sh` | Clones every repo in the `swift-libp2p` GitHub org as a sibling directory. |
| `.gitignore` | Ignores `.build`, `Packages`, `xcuserdata`, `.swiftpm`, `Package.resolved`, `dump.rdb`, `*.xcframework` build artifacts, etc. |
| `.env`, `.env.development`, `.env.testing` | Exist at the repo root. **Do not commit secrets here**; `.gitignore` does not ignore them. |
| `*/Package.swift` | Each Swift package defines its own targets, dependencies, and tools version. |
| `gnostr-sdk-ffi/Cargo.toml` | Rust/uniffi crate that produces `libnostr_sdk_ffi` for Swift/Python/JVM/KMP. |
| `swift-gnostr-*/Rust/*/Cargo.toml` | Small Rust FFI crates behind the Swift `gnostr-*` wrappers. |
| `*/.github/workflows/*.yml` | Per-package CI; reusable templates live in `github-workflows/.github/workflows/`. |

## Technology stack

- **Languages:** Swift (5.10 / 6.0+), Objective-C/C++ for `libgit2` bindings,
  Rust (edition 2024) for FFI crates, JavaScript/Node for `gnostore` extension
  assets.
- **Build tools:** Xcode, `swift build`/`swift test`, `cargo`, `just`, `cmake`,
  `ninja`, `npm`/`esbuild`/`tailwindcss`.
- **Networking:** SwiftNIO, Vapor `console-kit`/`routing-kit`, `async-kit`.
- **Cryptography:** Apple `CryptoKit`, `swift-crypto`, `secp256k1.swift`,
  `CryptoSwift`.
- **Cross-platform UI:** SwiftCrossUI with AppKit/UIKit/Gtk/WinUI backends.
- **FFI:** UniFFI 0.29.4 (`gnostr-sdk-ffi`) and hand-rolled cdylib loading for
  the `swift-gnostr-*` packages.
- **Native C/C++ libraries:** `libgit2` 1.3.1, OpenSSL 3.0.4, libssh2 1.10.0,
  PCRE 8.45, cpp-libp2p 0.1.37.

## Code organization

Top-level groups:

| Group | Directories | Purpose |
|-------|-------------|---------|
| **libp2p Swift stack** | `swift-libp2p`, `swift-libp2p-core`, `swift-libp2p-crypto`, `swift-libp2p-*` transports, `swift-multiaddr`, `swift-multihash`, `swift-peer-id`, `swift-noise`, `swift-quic`, etc. | Swift implementation of libp2p primitives and protocols. |
| **gnostr Swift/FFI stack** | `swift-gnostr-types`, `swift-gnostr-asyncgit`, `swift-gnostr-crawler`, `swift-gnostr-relay`, `swift-gnostr-ffi-relay`, `swift-gnostr-ffi-kitchensink` | Shared Nostr types and optional Rust FFI bridges for git, crawler, and relay. |
| **Nostr SDKs** | `gnostr-sdk-ios` (pure Swift), `gnostr-sdk-ffi` (Rust/uniffi multi-bindings) | Reusable Nostr client libraries. |
| **Sample / template apps** | `MiniP2PSample`, `libp2p-app-template`, `ChatAppExample-iOS`, `MiniGitSample`, `gnostr-chat`, `gnostr-git-mac-ios`, `.GnostrP2P-KitchenSink` | Xcode projects demonstrating P2P, chat, and Git workflows. |
| **Rust↔Swift examples** | `xcode-rust-example`, `xcode-gnostr-relay`, `xcode-gnostr-asyncgit`, `swift-universal` | Minimal Xcode apps that embed Rust static/dynamic libraries. |
| **Cross-platform UI** | `swift-cross-ui`, `swift-cross-ui-p2p`, `swift-bundler` | SwiftUI-like framework and demo apps. |
| **Git / libgit2** | `Git`, `LibGit2-iOS`, `gnostr-app` (Damus tree) | Swift git helpers and iOS git client experiments. |
| **Safari extension** | `gnostore` | NIP-07 Safari extension with web build pipeline. |
| **Icons / assets** | `icons` | App icon and banner generation Swift app. |
| **Specs / interop** | `specs`, `test-plans` | libp2p specs and upstream interop test plans. |
| **Build scripts** | `scripts/` | Convenience wrappers for SwiftPM, Xcode, Redis, libgit2, libp2p. |

## Build process

### Swift packages

Build any local package:

```bash
swift build --package-path swift-libp2p
# or
./scripts/swift-build-package.sh swift-libp2p
```

Run tests for every `Package.swift` in the repo:

```bash
./scripts/swift-test-all.sh
```

This script skips `LibGit2-iOS` (binary-only), starts Redis when testing the
redis-related packages, and passes through any extra `swift test` arguments.

### Xcode projects

Each `.xcodeproj` has a wrapper script under `scripts/` that sets the right
`-project`, `-scheme`, `-sdk`, and `-destination` defaults and rebuilds
native XCFrameworks if needed:

```bash
./scripts/xcodebuild-minigit-sample.sh
./scripts/xcodebuild-libp2p-app-template.sh
```

### Native XCFrameworks

- **libgit2 + dependencies:** `scripts/libgit2/build-libgit2-framework.sh`
  downloads OpenSSL/libssh2/libgit2/PCRE, verifies checksums/signatures, builds
  static libraries for iOS/macOS/Mac Catalyst, and produces
  `LibGit2-iOS/libgit2.xcframework` and `LibGit2-iOS/Clibgit2.xcframework`.
- **cpp-libp2p:** `scripts/libp2p/build-libp2p-framework.sh` builds the vendored
  `LibP2P-iOS/cpp-libp2p-0.1.37` source for macOS/iOS simulators and produces
  `LibP2P-iOS/LibP2P.xcframework`. QUIC is disabled.

### Cross-platform SwiftCrossUI

On Linux/macOS with Homebrew:

```bash
./scripts/build-swift-cross-ui.sh
```

The script installs GTK4 and pkg-config if missing, then builds the
`SwiftCrossUI` target.

### Rust FFI

`gnostr-sdk-ffi` is a UniFFI crate. Typical workflow:

```bash
cd gnostr-sdk-ffi
just precommit          # fmt + check + clippy + test
cargo build --release   # builds libnostr_sdk_ffi
./swift/assemble.sh     # repackages Swift bindings + static libs into nostr_sdkFFI.xcframework
```

The `swift-gnostr-*` packages load Rust cdylibs at runtime. Each bridge looks
for an environment variable first (e.g. `GNOSTR_TYPES_FFI_LIBRARY`), then falls
back to `Rust/*/target/debug/*.dylib` and `Rust/*/target/release/*.dylib`.

### gnostore Safari extension

```bash
cd gnostore
npm install
npm run build
npm run watch
npm run watch-tailwind
./build.sh              # macOS + iOS Xcode build
```

`gnostore/.tool-versions` pins Node.js 16.20.2.

## Runtime architecture

- Most Apple apps are SwiftUI targets for iOS, iPadOS, macOS, and Mac Catalyst.
- `swift-libp2p` apps build an `Application` object, configure transports
  (TCP/QUIC/WebSocket), security (Noise), muxers (Yamux/Mplex), discovery
  (mDNS/KadDHT), and PubSub (Gossipsub/Floodsub). Sample apps use deterministic
  per-profile listen ports (`10000`–`10004`) so iPhone, iPad, macOS, and
  Catalyst can run side by side.
- Nostr code is split between the pure-Swift `gnostr-sdk-ios` (`GnostrSDK`)
  and the Rust-backed `gnostr-sdk-ffi` (`NostrSDK`).
- Git code is split between `Git`/`MiniGit` (Objective-C/Swift on `libgit2`) and
  the `swift-gnostr-asyncgit` Rust FFI bridge.
- SwiftCrossUI apps use a backend abstraction; on macOS they default to
  AppKit, on Linux to GTK4, on iOS/tvOS to UIKit, and on Windows to WinUI.

## Development conventions

- Each Swift package owns its `Package.swift`, tools version, and platform pins.
  Common minimums are macOS 10.15 / iOS 13; some require macOS 14 / iOS 15.
- Local dependencies are referenced by relative path, e.g.
  `.package(path: "../swift-libp2p-core")`.
- `swift-libp2p` packages use license headers and the shared
  `github-workflows/.github/workflows/pull_request.yml` soundness checks,
  including unacceptable-language and license-header checks.
- Swift formatting is package-local: `gnostr-sdk-ios` uses SwiftLint;
  `xcode-gnostr-asyncgit` has a `.swiftformat`; `gnostr-sdk-ffi` uses nightly
  `cargo fmt` via `just fmt`.
- Build scripts set `GIT_CONFIG_COUNT=1`, `GIT_CONFIG_KEY_0=safe.bareRepository`,
  `GIT_CONFIG_VALUE_0=all` so SwiftPM can resolve bare repositories safely.
- Compile-time environment variables used by SwiftCrossUI:
  `SCUI_DEFAULT_BACKEND`, `SCUI_LIBRARY_TYPE`, `SCUI_HOT_RELOADING`,
  `SCUI_TEST_GTK3BACKEND`, `SCUI_BENCHMARK_VIZ`.

## Testing strategies

- **Swift packages:** `swift test --package-path <dir>` or the repo-wide
  `./scripts/swift-test-all.sh`.
- **Xcode projects:** wrapper scripts under `scripts/` run `xcodebuild` with
  sensible defaults.
- **Redis-backed packages:** `swift-test-all.sh` starts Redis with the package's
  `.github/redis*.conf` before running tests.
- **Rust FFI:** `cargo test --all` inside `gnostr-sdk-ffi` or each
  `swift-gnostr-*/Rust/*` crate.
- **Interop:** `test-plans/` contains upstream libp2p transport, gossipsub, and
  hole-punch interop specs (Go/Rust/Python).

## CI / deployment

There is no single root CI pipeline. Each package has its own
`.github/workflows/` that usually calls the reusable workflows in
`github-workflows/.github/workflows/`:

- `build+test.yml` – matrix of Ubuntu/macOS × Swift 6.0/6.1/6.2, debug/release.
- `pull_request.yml` – soundness checks + Linux/macOS test matrix.
- `providers.yml` – dependency-graph tests gated by a `dependency-check` label.

Deployment is project-specific:

- Xcode apps are archived/exported through Xcode or `xcodebuild`.
- `gnostr-sdk-ffi` produces Python wheels, JVM/KMP/Android artifacts, and a
  Swift `nostr_sdkFFI.xcframework`.
- `LibGit2-iOS`/`LibP2P-iOS` produce XCFrameworks consumed by sample apps.
- `gnostore` is a Safari extension published to the Mac App Store.

## Security considerations

- `.env`, `.env.development`, and `.env.testing` exist at the repo root.
  `.gitignore` does **not** ignore them, so review their contents before
  committing and never store production private keys or API tokens in tracked
  env files.
- FFI bridges load dynamic libraries from environment variables or local
  `Rust/*/target/{debug,release}` paths. Validate those paths when running in
  untrusted environments.
- Nostr sample code and tests may use hardcoded keys; do not reuse them for
  real accounts.
- P2P sample apps bind to `0.0.0.0` and discover peers on the LAN. Do not run
  them on untrusted networks without reviewing firewall and relay settings.
- The libgit2 build script downloads tarballs from upstream sources and
  verifies SHA-256 checksums and GPG signatures; the libp2p C++ build disables
  QUIC because the lsquic dependency is not integrated.
