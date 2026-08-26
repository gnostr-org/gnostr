## swift-gnostr-crawler

Swift client and query-builder for the crawler HTTP surface.

### Dependency chain

`Crawler` → `GnostrTypes`

`Crawler` → `RustCrawlerBridge` → `crawler-ffi` (`cdylib`) → `gnostr-crawler`

### What it covers

- relay discovery payloads
- relay process state (`/api/relay/status`, `/start`, `/stop`)
- crawler query URL building (`/query`, `/:nip/query`)
- Nostr REQ wire-message generation for crawler queries
- Rust-backed normalization helpers for crawler metadata when the dylib is present
- live crawler/p2p network snapshots for SwiftUI dashboards

### Runtime loading

The crawler bridge looks for `GNOSTR_CRAWLER_FFI_LIBRARY`, then falls back to:

- `Rust/crawler-ffi/target/debug/libcrawler_ffi.dylib`
- `Rust/crawler-ffi/target/release/libcrawler_ffi.dylib`

### Network dashboard

`CrawlerNetworkStore` polls the crawler runtime, crawl runtime, relay discovery,
and the local `~/.config/gnostr/{crawler,p2p}` relay bucket trees.

`CrawlerNetworkDashboard` renders that snapshot as a SwiftUI list for embedding
in an app.
