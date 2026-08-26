# LibP2PPlaintext

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-plaintext/actions/workflows/build+test.yml/badge.svg)

> A LibP2P Stream Faux-Cryption protocol

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
⚠️ Plaintext is intended only for debugging and interoperability testing purposes. ⚠️

> Secure communications are a key feature of libp2p, and encrypted transport is configured by default in libp2p implementations to encourage security for all production traffic. However, there are some use cases such as testing in which encryption is unnecessary. For such cases, the plaintext "security" protocol can be used. By conforming to the same interface as real security adapters like SECIO and TLS, the plaintext module can be used as a drop-in replacement when encryption is not needed.

#### Note:
- For more information check out the [Plaintext Spec](https://github.com/libp2p/specs/blob/master/plaintext/README.md)

## Install

Include the following dependency in your Package.swift file
``` swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-plaintext.git", .upToNextMinor(from: "0.2.0"))
    ],
        ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PPlaintext", package: "swift-libp2p-plaintext"),
            ]),
    ...
)
```

## Usage

### Example 
``` swift

import LibP2PPlaintext

/// Tell libp2p that it can use plaintext...
app.security.use( .plaintextV2 )

```

### API
``` swift
Not Applicable
```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝


## Credits

- [Plaintext Spec](https://github.com/libp2p/specs/blob/master/plaintext/README.md)

## License

[MIT](LICENSE) © 2022 Breth Inc.

