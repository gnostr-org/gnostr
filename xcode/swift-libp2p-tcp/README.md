# LibP2PTCP (Embedded)

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)

> TCP Client and Server Transport for Libp2p

#### Note: 
- For more info check out the [TCP Wiki](https://en.wikipedia.org/wiki/Transmission_Control_Protocol)

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
TCP is one of the most widely used internet transport protocols. It's what protocols such as HTTP1/2 and Websockets are built upon. This package lets you have access to both a TCP Client, in order to make outbound TCP requests, and a TCP Server, in order to respond to inbound TCP Requests. 

## Install
#### Heads up ‼️
- This package is embedded into swift-libp2p, there's no need to explicitly include this package in your swift-libp2p project.
Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-tcp.git", .upToNextMinor(from: "0.1.0"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "LibP2PTCP", package: "swift-libp2p-tcp"),
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

import LibP2PTCP

/// When you configure your app
app.transports.use(.tcp)
app.servers.use(.tcp)
app.clients.use(.tcp)
```

### API
```Swift
N/A
```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [swift-nio](https://github.com/apple/swift-nio)

## License

[MIT](LICENSE) © 2022 Breth Inc.
