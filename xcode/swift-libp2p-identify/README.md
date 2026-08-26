# Identify (Embedded)

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-identify/actions/workflows/build+test.yml/badge.svg)

> A core set of protocols used for `Identify`ing peers within libp2p

## Table of Contents

- [Overview](#overview)
- [Install](#install)
- [Usage](#usage)
  - [Example](#example)
  - [API](#api)
- [Contributing](#contributing)
- [Credits](#credits)
- [License](#license)

## Overview
Identify consists of a set of protocols used to help identify remote peers within the libp2p ecosystem. The protocols included within Identify are...

- /ipfs/id 
   - This protocol exchanges Signed Peer Records used for confirming a remote Peer is who they claim to be, the addresses they're listening on and the protocols they speak.
- /ipfs/id/push
   - An extension of `/ipfs/id` that pushes changes to a Peers listening addresses and protocols as they update in realtime.
- /ipfs/id/delta
   - An extension of `/ipfs/id` that pushes `delta` (partial) Peer Records that contain changes since an initial `/ipfs/id` Peer Record exchange
- /ipfs/ping
   - A ping protocol that allows for quick and easy pings to libp2p Peers for measuring latency, testing liveness/connectivity, etc.

#### For more details see 
- [Multiformats / Mulitbase Spec](https://github.com/multiformats/multibase/blob/master/README.md)


## Install 
> [!CAUTION]
> This package is embedded into [swift-libp2p](https://github.com/swift-libp2p/swift-libp2p) . There's no need to include this package as a dependency in your swift-libp2p project.

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-identify.git", .upToNextMinor(from: "0.1.0"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PIdentify", package: "swift-libp2p-identify"),
            ]),
    ...
)
```

## Usage
> [!CAUTION]
> This package is embedded into [swift-libp2p](https://github.com/swift-libp2p/swift-libp2p) . There's no need to include this package as a dependency in your swift-libp2p project.

```Swift

/// Tell swift-libp2p to use Identify (the standard swift-libp2p implementation does this automatically)
app.identityManager.use(.default)

```


### Example

```Swift

/// Manually ping a peer
app.identify.ping(peer: peerID).map { 
    print("Latency: \($0)")
}
// or asynchronously
let ping = try await app.identify.ping(peer: peerID)

/// Manually ping a multiaddr
app.identify.ping(addr: multiaddr).map { 
    print("Latency: \($0)")
}
// or asynchronously
let ping = try await app.identify.ping(peer: multiaddr)

```

### API
```Swift

/// Attempts to Ping a Peer, returns the time it took to receive a response.
public func ping(peer: PeerID) -> EventLoopFuture<TimeAmount> 
public func ping(peer: PeerID) async throws -> TimeAmount 

/// Attempts to Ping a Multiaddr, returns the time it took to receive a response.
public func ping(addr: Multiaddr) -> EventLoopFuture<TimeAmount>
public func ping(addr: Multiaddr) async throws -> TimeAmount

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [Identify Spec](https://github.com/libp2p/specs/blob/master/identify/README.md)

## License

[MIT](LICENSE) © 2022 Breth Inc.
