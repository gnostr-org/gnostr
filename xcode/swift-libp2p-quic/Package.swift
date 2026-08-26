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
    name: "swift-libp2p-quic",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "LibP2PQUIC",
            targets: ["LibP2PQUIC"]
        ),
    ],
    dependencies: [
        .package(path: "../swift-libp2p"),
        .package(path: "../swift-multiaddr"),
        .package(path: "../swift-multicodec"),
        .package(url: "https://github.com/apple/swift-nio.git", .upToNextMajor(from: "2.0.0")),
    ],
    targets: [
        .target(
            name: "LibP2PQUIC",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "Multiaddr", package: "swift-multiaddr"),
                .product(name: "Multicodec", package: "swift-multicodec"),
                .product(name: "NIOCore", package: "swift-nio"),
            ]
        ),
        .testTarget(
            name: "LibP2PQUICTests",
            dependencies: ["LibP2PQUIC"]
        ),
    ]
)
