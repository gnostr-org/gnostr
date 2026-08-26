# LibP2PMSS (Embedded)

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)](https://github.com/multiformats/multiformats)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos)](https://github.com/swift-libp2p/swift-libp2p-mss/actions/workflows/build+test.yml/badge.svg)

> MultistreamSelect (MSS) is a Libp2p Transport Upgrader. A Transport Upgrader is responsible for negotiating the protocols that we'll speak when interracting with a remote libp2p Peer. 

#### Note: 
- For more info check out the [MSS Spec](https://github.com/multiformats/multistream-select)

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
Some protocols have sub-protocols or protocol-suites. Often, these sub protocols are optional extensions. Selecting which protocol to use -- or even knowing what is available to chose from -- is not simple.

What if there was a protocol that allowed mounting or nesting other protocols, and made it easy to select which protocol to use?

At the operating system level, protocol selection is usually accomplished using numbered ports, with some out-of-band agreement on the mapping of port numbers to protocols. For example, HTTP requests will use TCP port 80 unless otherwise specified.

multistream-select brings this concept into the "protocol level", which gives you the flexibility to evolve a mapping of human-readable protocol names to the semantics you need, which may change over time.

This is especially useful in environments where connections to arbitrary OS ports is difficult or impossible, for example because one or more parties is behind a NAT. In such cases, a single connection with a stream multiplexer will allow you to open many independent communication channels, but you'll need some mechanism for deciding what protocol to use for each channel. If you support multiple stream multiplexers, you can even use multistream-select to decide which one to use in the first place.

## Install
#### Heads up ‼️
- This package is embedded into swift-libp2p, there's no need to explicitly include this package in your swift-libp2p project.
Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-mss.git", .upToNextMinor(from: "0.1.0"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PMSS", package: "swift-libp2p-mss"),
            ]),
    ...
)
```

## Usage
#### Heads up ‼️
- This package is embedded into swift-libp2p, there's no need to explicitly include this package in your swift-libp2p project.
### Example 
TODO

```Swift

import LibP2PMSS

/// When you configure your app
app.transportUpgraders.use(.mss)

```

### API
```Swift
N/A
```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [MSS Spec](https://github.com/multiformats/multistream-select)

## License

[MIT](LICENSE) © 2022 Breth Inc.
