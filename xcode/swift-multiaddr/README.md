# Multiaddr

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)](https://github.com/multiformats/multiformats)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-multiaddr/actions/workflows/build+test.yml/badge.svg)

> Composable and future-proof network addresses 

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
Multiaddr aims to make network addresses future-proof, composable, and efficient.

Current addressing schemes have a number of problems.
- They hinder protocol migrations and interoperability between protocols.
- They don't compose well. There are plenty of X-over-Y constructions, but only few of them can be addressed in a classic URI/URL or host:port scheme.
- They don't multiplex: they address ports, not processes.
- They're implicit, in that they presume out-of-band values and context.
- They don't have efficient machine-readable representations.

Multiaddr solves these problems by modelling network addresses as arbitrary encapsulations of protocols.
- support addresses for any network protocol.
- are self-describing.
- conform to a simple syntax, making them trivial to parse and construct.
- have human-readable and efficient machine-readable representations.
- encapsulate well, allowing trivial wrapping and unwrapping of encapsulation layers.

#### Note:
For more info check out the [Multiformats / Multiaddr Spec](https://github.com/multiformats/multiaddr)


## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-multiaddr.git", .upToNextMajor(from: "0.0.1"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "Multiaddr", package: "swift-multiaddr"),
            ]),
    ...
)
```

## Usage

### Example 
check out the [tests](https://github.com/SwiftEthereum/Multiaddr/blob/main/Tests/MultiaddrTests/MultiaddrTests.swift) for more examples

```Swift

import Multiaddr

/// Instantiate a Multiaddr from a String describing your protocol stack 
let addr = try Multiaddr("/dns6/foo.com/tcp/443/https")

/// - Note: This Multiaddr indicates that the server is reachable over `DNS` at `foo.com` and is running an `https` server over `tcp` on port 443
/// - Note: This lets a libp2p node know what protocol stack is necessary for communication with this server.

dump(addr)

// ▿ /dns6/foo.com/tcp/443/https
//   ▿ addresses: 3 elements
//     ▿ /dns6/foo.com
//       - addrProtocol: Multicodec.Codecs.dns6
//       ▿ address: Optional("foo.com")
//         - some: "foo.com"
//     ▿ /tcp/443
//       - addrProtocol: Multicodec.Codecs.tcp
//       ▿ address: Optional("443")
//         - some: "443"
//     ▿ /https
//       - addrProtocol: Multicodec.Codecs.https
//       - address: nil


/// Address Encapsulation
let ip = try! Multiaddr("/ip4/127.0.0.1")
let proto = try! Multiaddr("/udt")

ip.encapsulate(proto) // -> "/ip4/127.0.0.1/udt"

/// Address Decapsulation
let full = try! Multiaddr("/ip4/1.2.3.4/tcp/80")
let port = try! Multiaddr("/tcp/80")

full.decapsulate(port) // -> "/ip4/1.2.3.4"

```

### API
```Swift

/// Initializers
Multiaddr.init(_ string: String) throws    
Multiaddr.init(_ bytes: Data) throws 
Multiaddr.init(_ proto: MultiaddrProtocol, address: String?) throws 


/// Methods
/// Data representation of the `Multiaddr`
Multiaddr.binaryPacked() throws -> Data
    
/// Returns a list of `Protocol` elements contained by this `Multiaddr`, ordered from left-to-right.
Multiaddr.protocols() -> [MultiaddrProtocol] 

/// Returns a list of `Protocol` elements as Multicodec Names contained by this `Multiaddr`, ordered from left-to-right.
Multiaddr.protoNames() -> [String]

/// Returns a list of `Protocol` elements as Multicodec Codes contained by this `Multiaddr`, ordered from left-to-right.
Multiaddr.protoCodes() -> [UInt64] 

/// Encapsulation
Multiaddr.encapsulate(_ other: Multiaddr) -> Multiaddr 
Multiaddr.encapsulate(_ other: String) throws -> Multiaddr
Multiaddr.encapsulate(proto: MultiaddrProtocol, address:String?) throws -> Multiaddr

/// Decapsulation
Multiaddr.decapsulate(_ other: Multiaddr) -> Multiaddr
Multiaddr.decapsulate(_ other: String) -> Multiaddr
Multiaddr.decapsulate(_ other: MultiaddrProtocol) -> Multiaddr

/// Other Methods
/// Removes and returns the last `Address` of this `Multiaddr`.
Multiaddr.pop() -> Address?

/// Extracts a PeerID from the Multiaddress if one exists, otherwise returns nil
Multiaddr..getPeerID() -> String? 

/// Extracts a Unix Path from the Multiaddress if one exists, otherwise returns nil
Multiaddr.getPath() -> String?

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critques, are welcome! 

Let's make this code better together! 🤝

## Credits

- Major credit to the work done by [lukereichold - swift-multiaddr](https://github.com/lukereichold/swift-multiaddr), this repo is a slightly modified fork of his project
- [NeoTeo & Richard Litt - SwiftMultiaddr ](https://github.com/multiformats/SwiftMultiaddr) 
- [krzyzanowskim - CryptoSwift](https://github.com/krzyzanowskim/CryptoSwift)

## License

[MIT](LICENSE) © 2022 Breth Inc.























