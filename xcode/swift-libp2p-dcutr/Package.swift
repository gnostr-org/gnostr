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
    name: "swift-libp2p-dcutr",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "LibP2PDCUtR",
            targets: ["LibP2PDCUtR"]
        ),
    ],
    dependencies: [
        .package(path: "../swift-libp2p"),
        .package(url: "https://github.com/apple/swift-protobuf.git", .upToNextMajor(from: "1.29.0")),
    ],
    targets: [
        .target(
            name: "LibP2PDCUtR",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "SwiftProtobuf", package: "swift-protobuf"),
            ],
            resources: [
                .copy("Protobuf/HolePunch.proto"),
            ]
        ),
        .testTarget(
            name: "LibP2PDCUtRTests",
            dependencies: [
                .target(name: "LibP2PDCUtR"),
                .product(name: "LibP2P", package: "swift-libp2p"),
            ]
        ),
    ]
)
