# LibP2PMPLEX

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-mplex/actions/workflows/build+test.yml/badge.svg)

> A LibP2P Stream Multiplexer protocol

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
Mplex is a Stream Multiplexer protocol. The origins of this protocol are based in multiplex, the JavaScript-only Stream Multiplexer.

Mplex is a very simple protocol that does not provide many features offered by other stream multiplexers. Notably, mplex does not provide backpressure at the protocol level.

#### Note:
- For more information check out the [MPLEX Spec](https://github.com/libp2p/specs/blob/master/mplex/README.md)

## Install

Include the following dependency in your Package.swift file
``` swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-mplex.git", .upToNextMinor(from: "0.2.0"))
    ],
        ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PMPLEX", package: "swift-libp2p-mplex"),
            ]),
    ...
)
```

## Usage

### Example 
``` swift

import LibP2PMPLEX

/// Tell libp2p that it can use mplex...
app.muxers.use( .mplex )

```

### API
``` swift
Not Applicable
```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits
This repo is just a gnarly fork of the beautiful http2 code by the swift nio team below...
- [Swift NIO HTTP/2](https://github.com/apple/swift-nio-http2.git)
- [MPLEX Spec](https://github.com/libp2p/specs/blob/master/mplex/README.md) 

## License

[MIT](LICENSE) © 2022 Breth Inc.
