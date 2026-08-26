// swift-tools-version:6.0
//===----------------------------------------------------------------------===//
//
// This source file is part of the swift-libp2p open source project
//
// Copyright (c) 2022-2025 swift-libp2p project authors
// Licensed under MIT
//
// See LICENSE for license information
// See CONTRIBUTORS for the list of swift-libp2p project authors
//
// SPDX-License-Identifier: MIT
//
//===----------------------------------------------------------------------===//

import PackageDescription

let package = Package(
    name: "swift-libp2p-queues-redis-driver",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "QueuesRedisDriver",
            targets: ["QueuesRedisDriver"]
        )
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        // LibP2P
        .package(path: "../swift-libp2p"),
        // Queues
        .package(path: "../swift-libp2p-queues"),
        // Redis
        .package(path: "../swift-libp2p-redis"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "QueuesRedisDriver",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "Queues", package: "swift-libp2p-queues"),
                .product(name: "Redis", package: "swift-libp2p-redis"),
            ],
            swiftSettings: swiftSettings
        ),
        // Testing
        .testTarget(
            name: "QueuesRedisDriverTests",
            dependencies: [
                .target(name: "QueuesRedisDriver"),
                .product(name: "LibP2PTesting", package: "swift-libp2p"),
                .product(name: "QueuesTesting", package: "swift-libp2p-queues"),
            ],
            swiftSettings: swiftSettings
        ),
    ]
)

var swiftSettings: [SwiftSetting] {
    [
        //.enableUpcomingFeature("ExistentialAny"),
        //.enableUpcomingFeature("InternalImportsByDefault"),
        //.enableUpcomingFeature("MemberImportVisibility"),
        //.enableUpcomingFeature("InferIsolatedConformances"),
        //.enableUpcomingFeature("NonisolatedNonsendingByDefault"),
        //.enableUpcomingFeature("ImmutableWeakCaptures"),
    ]
}
