# Multihash

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)](https://github.com/multiformats/multiformats)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-multihash/actions/workflows/build+test.yml/badge.svg)

> Swift implementation of Multihash -> Self identifying hashes

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
Multihash is a protocol for differentiating outputs from various well-established cryptographic hash functions, addressing size + encoding considerations.

#### For more details see 
- [https://github.com/multiformats/multihash](https://github.com/multiformats/multihash)

#### Note:
This package currently doesn't support Blake2b, Blake2s & Blake3. If you're up for the challenge, please feel free to add support!

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-multihash.git", .upToNextMajor(from: "0.0.1"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "Multihash", package: "swift-multihash"),
            ]),
    ...
)
```

## Usage

### Example

```Swift

import Multihash

/// Multihash Format 
/// <varint hash function code><varint digest size in bytes><hash function output>

Multihash.supportedHashAlgorithms
/// [.md5, .sha1, .sha2_256, .sha2_512, .sha3_224, .sha3_256, .sha3_384, .sha3_512, .keccak_224, .keccak_256, .keccak_384, .keccak_512]

/// Encoding a Multihash
let multihash = try Multihash(raw: "multihash", hashedWith: .sha1)
multihash.asString(base: .base16)         // -> "111488c2f11fb2ce392acb5b2986e640211c4690073e"
multihash.asString(base: .base32PadUpper) // -> "CEKIRQXRD6ZM4OJKZNNSTBXGIAQRYRUQA47A===="
multihash.asString(base: .base58btc)      // -> "5dsgvJGnvAfiR3K6HCBc4hcokSfmjj"
multihash.asString(base: .base64Pad)      // -> "ERSIwvEfss45KstbKYbmQCEcRpAHPg=="

let multihash = try Multihash(raw: "multihash", hashedWith: .sha2_256)
multihash.asString(base: .base16)         // -> "12209cbc07c3f991725836a3aa2a581ca2029198aa420b9d99bc0e131d9f3e2cbe47"
multihash.asString(base: .base32PadUpper) // -> "CIQJZPAHYP4ZC4SYG2R2UKSYDSRAFEMYVJBAXHMZXQHBGHM7HYWL4RY="
multihash.asString(base: .base58btc)      // -> "QmYtUc4iTCbbfVSDNKvtQqrfyezPPnFvE33wFmutw9PBBk"
multihash.asString(base: .base64Pad)      // -> "EiCcvAfD+ZFyWDajqipYHKICkZiqQgudmbwOEx2fPiy+Rw=="

/// Decoding a Multihash

/// Given a Multibase compliant Multihash String
/// 
/// Example
/// "f111488c2f11fb2ce392acb5b2986e640211c4690073e"
///     f       11      14     88c2f11fb2ce392acb5b2986e640211c4690073e
/// <base16> <sha1> <20 bits> <sha1 digest>

let mh = try Multihash(multihash: "f111488c2f11fb2ce392acb5b2986e640211c4690073e")
print(mh.name) // -> "sha1"
print(mh.code) // ->  0x11
print(mh.digest.hexString) // -> "88c2f11fb2ce392acb5b2986e640211c4690073e"

/// Or use the try decodeMultihashBuffer() method

let multihashBuffer = Data(...)           // -> 111488c2f11fb2ce392acb5b2986e640211c4690073e
let decoded = try decodeMultihashBuffer(multihashBuffer)
decoded.name                              // -> "sha1"
decoded.code                              // -> 0x11
decoded.digest                            // -> 88c2f11fb2ce392acb5b2986e640211c4690073e (as hex)
decoded.length                            // -> 20

```

### API
```Swift

/// Initializers
Multihash(multihash:String) throws 
Multihash(:[UInt8]) throws
Multihash(hexString str:String) throws
Multihash(b58String str:String) throws
Multihash(multibase:String, codec:Codecs) throws


/// Properties
Multihash.code:Int?
Multihash.algorithm:Codecs?
Multihash.name:String?
Multihash.length:Int?
Multihash.digest:[UInt8]?

Multihash.asMultibase(_ base: BaseEncoding) -> String
Multihash.asString(base: BaseEncoding) -> String

Multihash.hexString:String
Multihash.b58String:String

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [Multiformats / Multihash](https://github.com/multiformats/multihash)
- [CryptoSwift](https://github.com/krzyzanowskim/CryptoSwift)

## License

[MIT](LICENSE) © 2022 Breth Inc.
