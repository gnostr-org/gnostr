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
    name: "swift-libp2p-mdns",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "LibP2PMDNS",
            targets: ["LibP2PMDNS"]
        )
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.

        // LibP2P Core Modules
        .package(path: "../swift-libp2p"),

        // DNS Support for mDNS
        .package(url: "https://github.com/Bouke/DNS.git", .upToNextMajor(from: "1.2.0")),

        // LibP2P MPLEX
        .package(path: "../swift-libp2p-mplex"),

        // LibP2P PlaintextV2
        .package(path: "../swift-libp2p-plaintext"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "LibP2PMDNS",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "DNS", package: "DNS"),
            ]
        ),
        .testTarget(
            name: "LibP2PMDNSTests",
            dependencies: [
                "LibP2PMDNS",
                .product(name: "LibP2PMPLEX", package: "swift-libp2p-mplex"),
                .product(name: "LibP2PPlaintext", package: "swift-libp2p-plaintext"),
            ]
        ),
    ]
)
