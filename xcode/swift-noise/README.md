# Noise

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-noise/actions/workflows/build+test.yml/badge.svg)

> A swift implementation of the [Noise Protocol Framework](https://noiseprotocol.org/noise.html). 

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
Noise is a framework for building crypto protocols. Noise protocols support mutual and optional authentication, identity hiding, forward secrecy, zero round-trip encryption, and other advanced features.

### ⚠️ Warning
This package has **NOT** been extensively tested in real world applications and **should NOT be used in production environments**. Although the actual cryptography is handled by swift-crypto, the handshake logic could, and probably does, contain a myriad of bugs. Please feel free to look over the code and submit improvements where you see fit.  

#### Note:
- For more information check out the [Noise Protocol Spec](https://noiseprotocol.org/noise.html)

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(name: "Noise", url: "https://github.com/swift-libp2p/swift-noise.git", .upToNextMinor(from: "0.1.0"))
    ],
        ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "Noise", package: "swift-noise"),
            ]),
        ...
    ...
)
```

## Usage

### Example 
check out the [tests](https://github.com/SwiftEthereum/Noise/tree/main/Tests/NoiseTests) for more examples

```Swift

import Noise

/// Instantiate an instance on the client side
let initiator = try Noise.HandshakeState(config:
    Noise.Config(
        cipherSuite: Noise.CipherSuite(
            keyCurve: .ed25519,
            cipher: .ChaChaPoly1305,
            hashFunction: .sha256
        ),
        handshakePattern: Noise.Handshakes.XX,
        initiator: true,
        prologue: [],
        presharedKey: nil,
        staticKeypair: initiatorsStatic,
        ephemeralKeypair: initiatorsEphemeral
    )
)
   
/// Instantiate an instance on the host side
let responder = try Noise.HandshakeState(config:
    Noise.Config(
        cipherSuite: Noise.CipherSuite(
            keyCurve: .ed25519,
            cipher: .ChaChaPoly1305,
            hashFunction: .sha256
        ),
        handshakePattern: Noise.Handshakes.XX,
        initiator: false,
        prologue: [],
        presharedKey: nil,
        staticKeypair: respondersStatic,
        ephemeralKeypair: respondersEphemeral
    )
)


/// On the client / initiator side

// Kick off the handshake by generating the first message
let (msgInit1, _, _) = try initiator.writeMessage(payload: []) // -> Yeilds the first payload to be sent to the host / responder

// ... wait for the response, then consume it
let (decryptedMessage2Payload, _, _) = try initiator.readMessage(responseFromHost) // -> Yeilds a decrypted payload if one was sent...

// Write the next message
let (msgInit3, initCS1, initCS2) = try initiator.writeMessage(payload: []) // -> Depending on the handshake chosen, you'll have your CipherStates available at this point

/// Handshake is completed on initiators end at this point...
/// After the handshake is completed, you'll have access to two CipherStates (one for inbound decryption and one for outbound encryption)

// You can start sending encrypted messages now...
let secureMessage1 = try initCS1!.encrypt(plaintext: "My Message".data(using: .utf8)!)

// And decrypting inbound messages using the second CipherState
let decryptedInboundMessage = try initCS2!.decrypt(ciphertext: encryptedInboundMessage)



/// On the host / responder side

// Read the first message from the initiator...
let (decryptedMessage1Payload, _, _) = try responder.readMessage(firstMessageFromInitiator) // -> Yeilds a decrypted payload, if one was sent... 

// Respond
let (msgResp2, _, _) = try responder.writeMessage(payload: []) // -> Yeilds the response message to be sent back to the initiator

// ... wait for the response from the initiator, then consume it
let (decryptedMessage3Payload, respCS1, respCS2) = try responder.readMessage(responseFromInitiator) // -> Depending on the handshake chosen, you'll have your CipherStates available at this point

/// Handshake is complete at this point...
/// After the handshake is completed, you'll have access to two CipherStates (one for inbound decryption and one for outbound encryption)

// You can start sending encrypted messages now...
let secureMessage1 = try respCS1!.encrypt(plaintext: "My Message".data(using: .utf8)!)

// And decrypting inbound messages using the second CipherState
let decryptedInboundMessage = try respCS2!.decrypt(ciphertext: encryptedInboundMessage)


```

### API
```Swift
/// Noise.HandshakeState
/// Initializers
Noise.HandshakeState(config: Noise.Config)

/// Methods
Noise.HandshakeState.writeMessage(payload:[UInt8]) throws -> (buffer:[UInt8], c1:CipherState?, c2:CipherState?)
Noise.HandshakeState.readMessage(_ inboundMessage:[UInt8]) throws -> (payload:[UInt8], c1:CipherState?, c2:CipherState?)

Noise.HandshakeState.shouldWrite() -> Bool
Noise.HandshakeState.shouldRead() -> Bool

Noise.HandshakeState.peerStatic() throws -> Curve25519.KeyAgreement.PublicKey
Noise.HandshakeState.peerEphemeral() throws -> Curve25519.KeyAgreement.PublicKey
Noise.HandshakeState.localEphemeral() throws -> Curve25519.KeyAgreement.PrivateKey

/// Noise.CipherState
/// Encryption / Decryption Post Handshake
CipherState.encrypt(plaintext:[UInt8]) throws -> [UInt8] 
CipherState.decrypt(ciphertext:[UInt8]) throws -> [UInt8]

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [Noise Protocol Spec](https://noiseprotocol.org/noise.html)
- [swift-crypto](https://github.com/apple/swift-crypto.git) 


## License

[MIT](LICENSE) © 2022 Breth Inc.
























