# Multicodec

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)](https://github.com/multiformats/multiformats)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-multicodec/actions/workflows/build+test.yml/badge.svg)

> Swift implementation of the multicodec specification

## Table of Contents

- [Install](#install)
- [Usage](#usage)
  - [Example](#example)
  - [API](#api)
- [Updating the Codec Values](#updating-the-codec-values)
- [Contributing](#contributing) 
- [Credits](#credits)
- [License](#license)

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-multicodec.git", .from("0.0.1"))
    ],
    ...
    .target(
        name: "...",
        dependencies: [
            ...
            .product(name: "Multicodec", package: "swift-multicodec"),
        ]),
)
```

## Usage

### Example

```swift

import Multicodec

let prefixedProtobuf = addPrefix(.protobuf, protobuf)
// prefixedProtobuf 0x50...

// The multicodec codec values can be accessed directly:
print(Codecs.DAG_CBOR.code) //113

// To get the string representation and description of a codec (e.g. for error messages):
print(Codecs(113).name)        // dag-cbor
print(Codecs(113).description) // optional("MerkleDAG cbor")
```

### API

This package conforms to the [JS-Multicodec API outlined here](https://multiformats.github.io/js-multicodec/)

The ground truth for codec values is the [multicodec default table](https://github.com/multiformats/multicodec/blob/master/table.csv)

## Updating the Codec Values

Updating the Codec enum is done by running the following command at the projects root directory...

    swift run update-codecs

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

Big thanks to work done by the [js-multicodec](https://github.com/multiformats/js-multicodec) team for writing clear code with documentation and tests that made porting this library to Swift relatively painless.

## License

[MIT](LICENSE) © 2022 Breth Inc.
