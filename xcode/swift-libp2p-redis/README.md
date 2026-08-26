# Redis

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-redis/actions/workflows/build+test.yml/badge.svg)

> A Redis interop add on for your swift-libp2p app

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
This repo contains the code necessary for a swift-libp2p app to comunicate with a local redis server. 
- It enables simple key-value stores 
- Provides a persistence layer for swift-libp2p caches
- Can act as a local bridge between two or more swift-libp2p instances
- Be used to communicate with other processes

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-redis.git", .upToNextMinor(from: "0.0.1"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "Redis", package: "swift-libp2p-redis"),
            ]),
    ...
)
```

## Usage

### Example 
check out the [tests]() for more examples

```Swift

import Redis

/// Configure libp2p
let app = try await Application.make(...)

/// Configure Redis
let redisConfig = try RedisConfiguration(
    hostname: Environment.get("REDIS_HOSTNAME") ?? "localhost",
    port: Environment.get("REDIS_PORT")?.int ?? 6379,
    pool: .init(connectionRetryTimeout: .milliseconds(100))
)
app.redis.configuration = redisConfig 

// Start the app
try await app.startup()

struct Hello: Codable {
    var message: String
    var array: [Int]
    var dict: [String: Bool]
}

let hello = Hello(
    message: "world",
    array: [1, 2, 3],
    dict: ["yes": true, "false": false]
)
try await app.redis.set("hello", toJSON: hello)

let get = try await app.redis.get("hello", asJSON: Hello.self)
get.message // "world"
get.array.first // 1
get.array.last // 3
get.dict["yes"] // true
get.dict["false"] // false

let _ = try await app.redis.delete(["hello"])
try await app.redis.get("hello", asJSON: Hello.self) // nil

/// Stop the app
try await app.asyncShutdown()

```

### API
```Swift

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [Vapor Redis](https://github.com/vapor/redis) 
- [RediStack](https://github.com/swift-server/RediStack) 
- [Redis](https://redis.io)

## License

[MIT](LICENSE) © 2026 Breth Inc.
