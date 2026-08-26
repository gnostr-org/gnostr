# Multibase

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)](https://github.com/multiformats/multiformats)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-multibase/actions/workflows/build+test.yml/badge.svg)


> Self identifying base encodings using the Multibase spec

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
Multibase is a protocol for disambiguating the encoding of base-encoded (e.g.,
base32, base36, base64, base58, etc.) binary appearing in text.

When text is encoded as bytes, we can usually use a one-size-fits-all encoding
(UTF-8) because we're always encoding to the same set of 256 bytes (+/- the NUL
byte). When that doesn't work, usually for historical or performance reasons, we
can usually infer the encoding from the context.

However, when bytes are encoded as text (using a base encoding), the base choice
of base encoding is often restricted by the context. Worse, these restrictions
can change based on where the data appears in the text. In some cases, we can
only use `[a-z0-9]`. In others, we can use a larger set of characters but need a
compact encoding. This has lead to a large set of "base encodings", one for
every use-case. Unlike when encoding text to bytes, we can't just standardize
around a single base encoding because there is no optimal encoding for all
cases.

Unfortunately, it's not always clear *what* base encoding is used; that's where
multibase comes in. It answers the question:

> Given data d encoded into text s, what base is it encoded with?

#### Heads up ‼️
- This library was built quickly and dirty as part of a larger project.
- This library hasn't been extensively tested! 
- I wouldn't use this in production until you've vetted it yourself! 

#### For more details see 
- [Multiformats / Mulitbase Spec](https://github.com/multiformats/multibase)


## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-multibase.git", .upToNextMajor(from: "0.0.1"))
    ],
    ...
    targets: [
        .target(
            ...
            dependencies: [
                ...
                .product(name: "Multibase", package: "swift-multibase"),
            ]),
    ]
    ...
)
```

## Usage

### Example

```Swift
import Multibase

/// The Multibase format is:
/// <base-encoding-character><base-encoded-data>
/// Where `<base-encoding-character>` is used according to the multibase table.

// To Encode a human readable string into a certain base encoding...
"Decentralize Everything!!".encode(as: .base8) // -> "72106254331267164344605543227514510062566312711713506415133463441102" -- Note the '7' multibase prefix tag
"Decentralize Everything!!".encode(as: .base64) // -> "mRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ" -- Note the 'm' multibase prefix tag

// To Decode a Multibase encoded string (one that has the proper base prefix prepended)...
try BaseEncoding.decodeIntoString("72106254331267164344605543227514510062566312711713506415133463441102") // -> (base: .base8, string: "Decentralize Everything!!")
try BaseEncoding.decodeIntoString("mRGVjZW50cmFsaXplIGV2ZXJ5dGhpbmchIQ") // -> (base: .base64, string: "Decentralize Everything!!")

/// String/Data/[UInt8] extensions (NOTE: all extensions default to no Multibase prefix!)
"Hello World".data(using: .utf8)!.asString(base: .base16)                              // ->  48656c6c6f20576f726c64 
"Hello World".data(using: .utf8)!.asString(base: .base16, withMultibasePrefix: true)   // -> f48656c6c6f20576f726c64

```

### API
```Swift

/// BaseEncoding Enum
let multibase = try! BaseEncoding.decodeIntoString("t8him6pbeehp62r39f9ii0pbmclp7it38d5n6e891")
multibase2.base   // -> .base32HexPad
multibase2.string // -> "Decentralize everything!!"


/// String Extensions
String(decoding:String, as:BaseEncoding, using:String.Encoding = .utf8) throws
String.baseEncoding -> BaseEncoding
String.encode(as: BaseEncoding, using:String.Encoding = .ascii) -> String


/// Data Extensions
Data.init(decoding:String, as:BaseEncoding) throws
Data.asString(base:BaseEncoding, withMultibasePrefix:Bool = false) -> String


/// Array<UInt8> Extensions
Array<UInt8>(decoding:String, as:BaseEncoding) throws
Array<UInt8>.asString(base:BaseEncoding, withMultibasePrefix:Bool = false) -> String

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [Multiformat / Multibase Spec](https://github.com/multiformats/multibase)


## License

[MIT](LICENSE) © 2022 Breth Inc.
