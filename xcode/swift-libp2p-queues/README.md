# Queues

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-queues/actions/workflows/build+test.yml/badge.svg)

> A Vapor-esque Queues implementation for your swift-libp2p app

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
This repo contains the code necessary for a swift-libp2p app to create, schedule, dispatch and execute jobs / services. 
- It is designed to be used with a Queues driver (ex: redis queues driver)
- See the Vapor Queues documentation for further information and examples on how to use Queues in your app.

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-queues.git", .upToNextMinor(from: "0.0.1"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "Queues", package: "swift-libp2p-queues"),
            ]),
    ...
)
```

## Usage

### Example 
check out the [tests]() for more examples

```Swift

```

### API
```Swift

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [Vapor Queues](https://github.com/vapor/queues)

## License

[MIT](LICENSE) © 2026 Breth Inc.

