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
    name: "swift-multiaddr",
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "Multiaddr",
            targets: ["Multiaddr"]
        )
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        // .package(url: /* package url */, from: "1.0.0"),
        .package(path: "../swift-varint"),
        .package(path: "../swift-multicodec"),
        .package(path: "../swift-multibase"),
        .package(path: "../swift-multihash"),
        .package(path: "../swift-cid"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "Multiaddr",
            dependencies: [
                .product(name: "VarInt", package: "swift-varint"),
                .product(name: "Multicodec", package: "swift-multicodec"),
                .product(name: "Multibase", package: "swift-multibase"),
                .product(name: "Multihash", package: "swift-multihash"),
                .product(name: "CID", package: "swift-cid"),
            ]
        ),
        .testTarget(
            name: "MultiaddrTests",
            dependencies: ["Multiaddr"]
        ),
    ]
)
