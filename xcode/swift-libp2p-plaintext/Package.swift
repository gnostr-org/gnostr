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
    name: "swift-libp2p-plaintext",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "LibP2PPlaintext",
            targets: ["LibP2PPlaintext"]
        )
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.

        // LibP2P Modules
        .package(path: "../swift-libp2p"),

        // YAMUX for testing
        .package(path: "../swift-libp2p-yamux"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "LibP2PPlaintext",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p")
            ],
            resources: [
                .copy("Protobuf/Exchange.proto")
            ]
        ),
        .testTarget(
            name: "LibP2PPlaintextTests",
            dependencies: [
                "LibP2PPlaintext",
                .product(name: "LibP2PYAMUX", package: "swift-libp2p-yamux"),
            ]
        ),
    ]
)
