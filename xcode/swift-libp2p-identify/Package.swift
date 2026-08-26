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
    name: "swift-libp2p-identify",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "LibP2PIdentify",
            targets: ["LibP2PIdentify"]
        )
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(path: "../swift-libp2p"),

        // Test Dependencies
        .package(path: "../swift-libp2p-yamux"),
        .package(path: "../swift-libp2p-noise"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "LibP2PIdentify",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p")
            ],
            resources: [
                .copy("Protobufs/Identify.proto")
            ]
        ),
        .testTarget(
            name: "LibP2PIdentifyTests",
            dependencies: [
                "LibP2PIdentify",
                .product(name: "LibP2PNoise", package: "swift-libp2p-noise"),
                .product(name: "LibP2PYAMUX", package: "swift-libp2p-yamux"),
            ]
        ),
    ]
)
