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
    name: "swift-libp2p-autonat",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "LibP2PAutoNAT",
            targets: ["LibP2PAutoNAT"]
        ),
    ],
    dependencies: [
        .package(path: "../swift-libp2p"),
        .package(path: "../swift-multiaddr"),
        .package(path: "../swift-varint"),
    ],
    targets: [
        .target(
            name: "LibP2PAutoNAT",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "Multiaddr", package: "swift-multiaddr"),
                .product(name: "VarInt", package: "swift-varint"),
            ]
        ),
        .testTarget(
            name: "LibP2PAutoNATTests",
            dependencies: ["LibP2PAutoNAT"]
        ),
    ]
)
