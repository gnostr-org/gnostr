# LibP2PPubSub

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-pubsub/actions/workflows/build+test.yml/badge.svg)

> A WIP implementation of FloodSub and GossipSub Routers for swift-libp2p

> **Warning**
> This is a WIP. It probably won't work the way it's supposed to. Please file issues accordingly.

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
This repo contains the pubsub implementation for swift-libp2p. We currently provide two message routers:

- Floodsub, which is the baseline flooding protocol.
- Gossipsub, which is a more advanced router with mesh formation and gossip propagation.

> **Note**
> For more information check out the [Libp2p PubSub Spec](https://github.com/libp2p/specs/tree/master/pubsub) 

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(name: "LibP2PPubSub", url: "https://github.com/swift-libp2p/swift-libp2p-pubsub.git", .upToNextMinor(from: "0.2.0"))
    ],
    
    ...
)
```

## Usage

### Example 
check out the [tests]() for more examples

```Swift

import LibP2PPubSub

/// Configure libp2p
let app = Application(...)
// We have to use the `BasicConnectionLight` `Connection` implementation here! (the default `ARCConnection`, doesn't work due to the agressive timeout behaviour...)
app.connectionManager.use(connectionType: BasicConnectionLight.self)
// Tell it to use Floodsub
app.pubsub.use(.floodsub)
// Or tell it to use Gossipsub
app.pubsub.use(.gossipsub)

/// Subscribe
let subscription = try app.pubsub.gossipsub.subscribe(.init(topic: "news", signaturePolicy: .strictSign, validator: .acceptAll, messageIDFunc: .hashSequenceNumberAndFromFields))
subscription.on = { event -> EventLoopFuture<Void> in
    switch event {
    case .newPeer(let peer):
        ...
        
    case .data(let pubSubMessage):
        ...
        
    case .error(let error):
        ...
    }
    return app.eventLoopGroup.any().makeSucceededVoidFuture()
}


/// Publish Message
// Using a subscription
subscription.publish("Hello".data(using: .utf8))
// Without a subscription
app.pubsub.publish("Hello".data(using: .utf8), to: "topic")


/// Unsubscribe
// Using a subscription
subscription.unsubscribe()
// Without a subscription
app.pubsub.unsubscribe("topic")


/// Stop the app
app.shutdown()

```

### API
```Swift
/// Initializers

/// Properties

/// Methods

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [PubSub Spec](https://github.com/libp2p/specs/tree/master/pubsub) 
- [The JS PubSub implementation](https://github.com/libp2p/js-libp2p-interfaces) 
- [The GO PubSub implementation](https://github.com/libp2p/go-libp2p-pubsub)

## License

[MIT](LICENSE) © 2022 Breth Inc.
