# Kubo Xcode

A SwiftPM package (openable directly in Xcode) demonstrating the Kubo FFI
with a SwiftUI GUI, node lifecycle, peer networking, and an embedded WebUI.

## Open in Xcode (recommended for signing)

### Option A: SwiftPM package (ffi/xcode/Package.swift)

1. Open Xcode → **File → Open…** → select `ffi/xcode/Package.swift`.
2. Xcode resolves the local `../swift` package automatically.
3. Select the `XcodeApp` target, set your **Signing & Capabilities** team,
   and build/run (⌘R).

### Option B: Your own Xcode project (kubo-ffi-example.xcodeproj)

1. Open `ffi/xcode/kubo-ffi-example.xcodeproj` in Xcode.
2. **File → Add Package Dependencies…** → **Add Local…** → select
   `ffi/swift` (the `KuboFFI` package).
3. Link the `Kubo` product to your app target.
4. Build the Go archive first: `cd ffi/swift && make build`
   (produces `libkubo_ffi.a` in `ffi/swift/`).
5. Add `ffi/swift/libkubo_ffi.a` to **Link Binary With Libraries** and set
   the header search path to `ffi/swift/Sources/CKubo`.
6. Set your **Signing & Capabilities** team and build/run (⌘R).

This gives you full access to Xcode's signing, entitlements, and debugger.

## Build from the command line

```sh
cd ffi/swift && make build          # build the Go FFI archive
cd ../xcode && make build          # swift build
make app                           # build a signed .app bundle
make run                           # build + open the .app
```

Or with xcodebuild directly:

```sh
cd ffi/swift && make build
cd ../xcode && xcodebuild -scheme KuboXcode -destination 'platform=macOS' build
```

## Entitlements

`Entitlements.plist` requests network client/server access and disables
the app sandbox. Adjust in Xcode's Signing & Capabilities tab as needed.

## Notes

- The archive `libkubo_ffi.a` is produced by `ffi/swift/Makefile` and is
  gitignored; rebuild it after changing Go FFI sources.
- The package depends on `../swift` (relative path), so keep the directory
  layout intact.
- The WebContent process (WKWebView) may show sandbox warnings on macOS 26
  with ad-hoc signing; open the logged WebUI URL in a browser if needed.
  EOF
  echo done
