# QueuesRedisDriver

[![](https://img.shields.io/badge/made%20by-Breth-blue.svg?style=flat-square)](https://breth.app)
[![](https://img.shields.io/badge/project-libp2p-yellow.svg?style=flat-square)](http://libp2p.io/)
[![Swift Package Manager compatible](https://img.shields.io/badge/SPM-compatible-blue.svg?style=flat-square)](https://github.com/apple/swift-package-manager)
![Build & Test (macos and linux)](https://github.com/swift-libp2p/swift-libp2p-queues-redis-driver/actions/workflows/build+test.yml/badge.svg)

> A Queues driver, powered by Redis, for your swift-libp2p app

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
This repo contains the a Redis backed Queues driver implementation. If you have Redis installed on your machine, this package will allow you to dispatch
asynchronous jobs and create scheduled jobs (similar to chron) from your swift-libp2p app. 

## Install

Include the following dependency in your Package.swift file
```Swift
let package = Package(
    ...
    dependencies: [
        ...
        .package(url: "https://github.com/swift-libp2p/swift-libp2p-queues-redis-driver.git", .upToNextMinor(from: "0.0.1"))
    ],
    ...
        .target(
            ...
            dependencies: [
                ...
                .product(name: "QueuesRedisDriver", package: "swift-libp2p-queues-redis-driver"),
            ]),
    ...
)
```

## Usage

### Vapor Queues Documentation
check out Vapor's [Queues documentation](https://docs.vapor.codes/advanced/queues/) for detailed usage info

### Example 
check out the [tests]() for more examples

```Swift
import LibP2P
import QueuesRedisDriver

// Configure your app's queues to use your redis endpoint
try app.queues.use(.redis(url: "redis://127.0.0.1:6379"))

// Register jobs
let emailJob = EmailJob()
app.queues.add(emailJob)

// To run a worker in the same process as your application
try app.queues.startInProcessJobs(on: .default)

// Dispatch a job using either a request object in a route handler
try await req.queue.dispatch(
    EmailJob.self, 
    .init(to: "email@email.com", message: "message")
)

// Or at any time throughout your apps lifecycle using the app itself
try await app.queue.dispatch(
    EmailJob.self, 
    .init(to: "email@email.com", message: "message")
)

// You can also schedule jobs to be run periodically 
app.queues.schedule(CleanupJob())
    .yearly()
    .in(.may)
    .on(23)
    .at(.noon)

// Once you schedule your jobs, start them with the following call
try app.queues.startScheduledJobs()
```

### API
```Swift

```

## Contributing

Contributions are welcomed! This code is very much a proof of concept. I can guarantee you there's a better / safer way to accomplish the same results. Any suggestions, improvements, or even just critiques, are welcome! 

Let's make this code better together! 🤝

## Credits

- [Vapor Queues](https://github.com/vapor/queues)
- [Queues Redis Driver](https://github.com/vapor/queues-redis-driver)

## License

[MIT](LICENSE) © 2026 Breth Inc.


