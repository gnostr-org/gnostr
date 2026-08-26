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
    name: "swift-libp2p-mss",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "LibP2PMSS",
            targets: ["LibP2PMSS"]
        )
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.

        // LibP2P Core Modules
        .package(path: "../swift-libp2p"),

        // NIO Extras
        .package(url: "https://github.com/apple/swift-nio-extras.git", .upToNextMajor(from: "1.25.0")),

        // SwiftState for state machines
        .package(url: "https://github.com/ReactKit/SwiftState.git", .upToNextMajor(from: "6.0.0")),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "LibP2PMSS",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "SwiftState", package: "SwiftState"),
                .product(name: "NIOExtras", package: "swift-nio-extras"),
            ]
        ),
        .testTarget(
            name: "LibP2PMSSTests",
            dependencies: ["LibP2PMSS"]
        ),
    ]
)
