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
    name: "swift-libp2p-fluent",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
        .watchOS(.v6),
        .tvOS(.v13),
    ],
    products: [
        .library(name: "Fluent", targets: ["Fluent"])
    ],
    dependencies: [
        .package(url: "https://github.com/vapor/fluent-kit.git", .upToNextMajor(from: "1.52.2")),
        .package(path: "../swift-libp2p"),
    ],
    targets: [
        .target(
            name: "Fluent",
            dependencies: [
                .product(name: "FluentKit", package: "fluent-kit"),
                .product(name: "LibP2P", package: "swift-libp2p"),
            ],
            swiftSettings: swiftSettings
        ),
        .testTarget(
            name: "FluentTests",
            dependencies: [
                .target(name: "Fluent"),
                .product(name: "XCTFluent", package: "fluent-kit"),
                .product(name: "LibP2PTesting", package: "swift-libp2p"),
            ],
            swiftSettings: swiftSettings
        ),
    ]
)

var swiftSettings: [SwiftSetting] {
    [
        .enableUpcomingFeature("ExistentialAny"),
        .enableUpcomingFeature("InternalImportsByDefault"),
        .enableUpcomingFeature("MemberImportVisibility"),
        .enableUpcomingFeature("InferIsolatedConformances"),
        .enableUpcomingFeature("NonisolatedNonsendingByDefault"),
        .enableUpcomingFeature("ImmutableWeakCaptures"),
    ]
}
