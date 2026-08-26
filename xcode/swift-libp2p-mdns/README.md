# LibP2PMDNS

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos)](https://github.com/swift-libp2p/swift-libp2p-mdns/actions/workflows/build+test.yml/badge.svg)

> An mDNS Discovery Mechanism for registering, and discovering, libp2p services, and peers, on a local area network. 

### Note: 
- This implementation uses the `dnssd` library on mac/ios systems.  
- This implementation is not compatible with GETH / JS libp2p mdns packages.
- This implementation does not work on linux at the moment.

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
mDNS helps you discover libp2p peers on a local network without pre-configuring ip addresses and port numbers. 

#### Heads up ‼️
- This package uses the `dnssd` library and doesn't work on Linux at the moment.

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-mdns.git", .upToNextMinor(from: "0.1.0"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PMDNS", package: "swift-libp2p-mdns"),
            ]),
    ...
)
```

## Usage

### Example 
TODO

```Swift

import LibP2PMDNS

/// When you configure your app
app.discovery.use(.mdns)

```

### API
```Swift
N/A
```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [Bouke Haarsma](https://twitter.com/BoukeHaarsma) for the DNS library

## License

[MIT](LICENSE) © 2022 Breth Inc.
