# SwiftCrossUI P2P

Kitchen-sink demo space for `swift-cross-ui` + `swift-libp2p`.

The first pass includes:

- a cross-platform SwiftCrossUI shell
- a deterministic libp2p node
- TCP + Noise + Yamux
- mDNS discovery
- KadDHT / DCUtR wiring
- a local git repository viewer backed by `Git` (iOS only; the bundled
  `Clibgit2` xcframework does not include a Mac Catalyst library slice)

Run it with:

```sh
cd swift-cross-ui-p2p
swift run SwiftCrossUIP2P
```

## iPad and Mac Catalyst

The package supports iOS 14+, macOS 11+, and Mac Catalyst 14+. When building
for iPad or Mac Catalyst, the app uses `UIKitBackend` directly so that the UI
is rendered with the iPad-style appearance on both devices. On macOS it falls
back to `DefaultBackend` (AppKit), and on Linux it uses `DefaultBackend` (Gtk).

Build for Mac Catalyst from the command line with `xcodebuild`:

```sh
xcodebuild -scheme swift-cross-ui-p2p \
  -destination 'platform=macOS,variant=Mac Catalyst' \
  -derivedDataPath .build/derived-data
```

> Note: `swift build` for the `macabi` target currently hits a SwiftPM linker
> issue with macro plugins pulled in by upstream dependencies. Use `xcodebuild`
> until that is resolved.
