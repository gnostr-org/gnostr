// swift-tools-version: 6.0
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
    name: "swift-libp2p-relay",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "LibP2PRelay",
            targets: ["LibP2PRelay"]
        ),
    ],
    dependencies: [
        .package(path: "../swift-libp2p"),
        .package(path: "../swift-multiaddr"),
    ],
    targets: [
        .target(
            name: "LibP2PRelay",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "Multiaddr", package: "swift-multiaddr"),
            ]
        ),
        .testTarget(
            name: "LibP2PRelayTests",
            dependencies: [
                .target(name: "LibP2PRelay"),
                .product(name: "LibP2P", package: "swift-libp2p"),
            ]
        ),
    ]
)
