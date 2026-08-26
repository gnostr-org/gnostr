# LibP2PYAMUX

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-yamux/actions/workflows/build+test.yml/badge.svg)

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
Yamux is a Stream Multiplexer protocol.

Yamux uses a single streaming connection underneath, but imposes message framing so that it can be shared between many logical streams. These logical streams support windowing which provides a soft version of backpressure.

#### Note:
- For more information check out the [YAMUX Spec](https://github.com/libp2p/specs/blob/master/yamux/README.md)

## Install

Include the following dependency in your Package.swift file
``` swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-yamux.git", .upToNextMinor(from: "0.2.0"))
    ],
        ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PYAMUX", package: "swift-libp2p-yamux"),
            ]),
    ...
)
```

## Usage

### Example 
``` swift

import LibP2PYAMUX

/// Tell libp2p that it can use yamux...
app.muxers.use( .yamux )

```

### API
``` swift
Not Applicable
```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits
This repo is just a modified fork of the beautiful swift-nio-ssh repo...
- [Swift NIO SSH](https://github.com/apple/swift-nio-ssh)
- [YAMUX Spec](https://github.com/libp2p/specs/blob/master/yamux/README.md) 

## License

[MIT](LICENSE) © 2025 Breth Inc.
