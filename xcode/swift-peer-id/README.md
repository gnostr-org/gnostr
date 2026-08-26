# PeerID

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-multiformats-blue.svg?style=flat-square)](https://github.com/multiformats/multiformats)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-peer-id/actions/workflows/build+test.yml/badge.svg)

> An API / abstraction for managing libp2p public/private key pairs and identities

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
Libp2p uses cryptographic key pairs to sign & verify messages and derive unique peer identities (Peer ID's). This library wraps a public / private key pair in a PeerID object that exposes certain functionality for use with Libp2p nodes.

A Peer ID is the SHA-256 multihash of a public key.

The public key is a base64 encoded string of a protobuf containing an RSA DER buffer.

#### Note:
- For more information check out the [PeerID Spec](https://github.com/libp2p/specs/blob/master/peer-ids/peer-ids.md)

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-peer-id.git", .upToNextMinor(from: "0.2.0"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "PeerID", package: "swift-peer-id"),
            ]),
    ...
)
```

## Usage

### Example 
check out the [tests](https://github.com/swift-libp2p/swift-peer-id/blob/main/Tests/PeerIDTests/PeerIDTests.swift) for more examples

```Swift

import PeerID

/// Generate a new PeerID
let peerID = try PeerID(.Ed25519)

peerID.b58String                        // -> QmVJHUwJBshjMa7Ctngt34MXgXMTDeM5RjvgQNGqsiPLzB (libp2p PeerID standard)
peerID.keyPair                          // Access to the underlying key pair
peerID.keyPair?.keyType == .ed25519     // The type of Key
peerID.keyPair?.privateKey              // Access to the private key (for signing)
peerID.keyPair?.publicKey               // Access to the public key (for verifying signatures)

/// If you want to reuse the same PeerID between sessions, you can... 
        
/// Export a PeerID as an Encrypted PEM String that you can store... 
let encryptedPEM = try peerID.exportKeyPair(as: .privatePEMString(encryptedWithPassword: "mypassword"))

/// And then load the PeerID from and encrypted PEM String later
let peerID = try PeerID(pem: "ENCRYPTED_PEM_String", password: "mypassword")
```

### API
```Swift
/// Initializers
/// Generate a new PeerID with an underlying Key Pair (defaults to 2048 bit RSA)
PeerID.init(_ keyType:LibP2PCrypto.Keys.KeyPairType = .RSA(bits: .B2048)) throws

/// Use an existing Key Pair to instantiate a PeerID
PeerID.init(keyPair:LibP2PCrypto.Keys.KeyPair) throws

/// Inits a `PeerID` based solely on an ID value with no underlying `KeyPair`
PeerID.init(fromHexID hex:String) throws

/// Inits a `PeerID` based solely on an ID value with no underlying `KeyPair`
PeerID.init(fromBytesID bytes:[UInt8]) throws

/// Inits a `PeerID` from a v0 dag-pb or v1 libp2p-key CID complient string
PeerID.init(cid:String) throws

/// Inits a `PeerID` from a v0 dag-pb or v1 libp2p-key CID
PeerID.init(cid:CID) throws 

/// Inits a `PeerID` from a marshaled `PeerID` string
PeerID.init(marshaledPeerID:String, base: BaseEncoding? = nil) throws

/// Inits a `PeerID` from a marshaled `PeerID`
PeerID.init(marshaledPeerID data:Data) throws

/// Inits a `PeerID` from a marshaled public key string
PeerID.init(marshaledPublicKey str:String, base:BaseEncoding) throws

/// Inits a `PeerID` from a marshaled public key
PeerID.init(marshaledPublicKey key:Data) throws

/// Inits a `PeerID` from a marshaled private key string
PeerID.init(marshaledPrivateKey str:String, base:BaseEncoding) throws

/// Inits a `PeerID` from a marshaled private key
PeerID.init(marshaledPrivateKey data:Data) throws

/// Inits a `PeerID` from a PEM String
PeerID.init(pem: String, withPassword: String? = nil) throws

/// Properties
/// Returns the PeerID's id as a base58 string (multihash/CIDv0).
PeerID.b58String:String

/// Returns the PeerID's id as a hex string.
PeerID.hexString:String

/// A base32 encoded, version 1 CID, representing this PeerID
PeerID.cidString:String

/// A simple way of checking a PeerID's type (id only, public key & id, or private key, public key and id)
PeerID.type:PeerType

/// Returns the id of this PeerID as bytes
PeerID.bytes:[UInt8] 


/// Methods
/// Returns a protocol-buffers encoded version of the id, public key and, if `includingPrivateKey` is set to `true`, the private key.
PeerID.marshal(includingPrivateKey:Bool = false) throws -> [UInt8] 

/// Exports our PeerID as a JSON object
PeerID.toJSON(includingPrivateKey:Bool = false) throws -> Data

/// Exports our PeerID as a JSON string
PeerID.toJSONString(includingPrivateKey:Bool = false) throws -> String?

/// Exports our PeerID as a PEM String
PeerID.exportKeyPair(as: PeerID.ExportType) throws -> String

/// Signing and Verifying
// Signs data using this PeerID's private key. This signature can then be verified by a remote peer using this PeerID's public key
PeerID.signature(for msg:Data) throws -> Data

// Using this PeerID's public key, this method checks to see if the signature data was in fact signed by this peer and is a valid signature for the expected data
PeerID.isValidSignature(_ signature:Data, for expectedData:Data) throws -> Bool 

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [The JS PeerID implementation](https://github.com/libp2p/js-peer-id) 
- [PeerID Spec](https://github.com/libp2p/specs/blob/master/peer-ids/peer-ids.md) 

## License

[MIT](LICENSE) © 2022 Breth Inc.
























