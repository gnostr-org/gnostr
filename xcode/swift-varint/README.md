# VarInt encoding of 64-bit integers

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)](https://github.com/multiformats/multiformats)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-varint/actions/workflows/build+test.yml/badge.svg)

> Swift implementation of the Google Protocol Buffers VarInt specification

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
It is based on the Go implementation of the Google Protocol Buffers varint specification.
- Varints are a method of serializing integers using one or more bytes.
- Smaller numbers take a smaller number of bytes.

#### For more details see 
- [https://github.com/multiformats/go-varint](https://github.com/multiformats/go-varint)
- [ipfs/QmXJXJMai4p88HMsp2TPP1EtZxfSZQ1vyRtN5dGKvQ6MCw](ipfs/QmXJXJMai4p88HMsp2TPP1EtZxfSZQ1vyRtN5dGKvQ6MCw)

#### The encoding rules are:
-   Unsigned integers are serialized 7 bits at a time, starting with the least significant bits.
-   The most significant bit (msb) in each output byte indicates if there is a continuation byte.
-   Signed integers are mapped to unsigned integers using "zig-zag" encoding:
    - Positive values x are written as 2*x + 0,
    - Negative values x are written as 2*(~x) + 1
    - So, negative values are complemented and whether to complement is encoded in bit 0.


## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-varint.git", .upToNextMajor(from: "0.0.1"))
    ],
    
    ...
)
```

## Usage

### Example

```Swift

import VarInt

/// Create some arbitrary data
let bytes = [UInt8]("Hello World".data(using: .utf8)!)

/// Prefix the bytes with their length so we can recover the data later
let uVarIntLengthPrefixedBytes = putUVarInt(UInt64(bytes.count)) + bytes

/// ... send the data across a network or something ...

/// Read the length prefixed data to determine the length of the payload
let lengthPrefix = uVarInt(uVarIntLengthPrefixedBytes)
print(lengthPrefix.value)     // 11 -> `Hello World` == 11 bytes
print(lengthPrefix.bytesRead) // 1  -> The value `11` fits into 1 byte

/// So dropping the first byte will result in our original data again...
let recBytes = [UInt8](uVarIntLengthPrefixedBytes.dropFirst(lengthPrefix.bytesRead))

/// The original bytes and the recovered bytes are equal
print(bytes == recBytes) // True

```

### API
```Swift
/// Signed VarInts
public func putVarInt(_ value: Int64) -> [UInt8] 
public func varInt(_ buffer: [UInt8]) -> DecodedVarInt   // (value:  Int64, bytesRead: Int)

/// Unsigned VarInts
public func putUVarInt(_ value: UInt64) -> [UInt8]
public func uVarInt(_ buffer: [UInt8]) -> DecodedUVarInt // (value: UInt64, bytesRead: Int)
```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critques, are welcome! 

Let's make this code better together! 🤝

## Credits

[https://github.com/multiformats/go-varint](https://github.com/multiformats/go-varint)

## License

[MIT](LICENSE) © 2022 Breth Inc.
