# LibP2PWebSocket

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-websocket/actions/workflows/build+test.yml/badge.svg)

> A WebSocket Transport module for libp2p

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
This repo adds support for WebSockets over HTTP1 for swift-libp2p applications. WebSockets are realtime full-duplex streams over tcp. This module doesn't support WebSocket Secure (WSS) connections. Instead it encrypts traffic using a libp2p security module such as [Noise](https://github.com/swift-libp2p/swift-libp2p-noise.git).


## Install

Include the following dependency in your Package.swift file
``` swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-websocket.git", .upToNextMinor(from: "0.2.0"))
    ],
        ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PWebSocket", package: "swift-libp2p-websocket"),
            ]),
    ...
)
```

## Usage

### Example 
``` swift

import LibP2PWebSocket

/// Tell libp2p that it should listen for WS connections on the following ip:port...
app.servers.use( .ws(host: "127.0.0.1", port: 10000) )
/// Tell libp2p that it can dial peers using WebSockets `ws`
app.transports.use( .ws )

```

### API
``` swift
Not Applicable
```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits
- [Swift NIO](https://github.com/apple/swift-nio.git)

## License

[MIT](LICENSE) © 2022 Breth Inc.

