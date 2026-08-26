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
    name: "swift-multibase",
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "Multibase",
            targets: ["Multibase"]
        )
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(path: "../swift-bases")
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "Multibase",
            dependencies: [
                .product(name: "Base2", package: "swift-bases"),
                .product(name: "Base8", package: "swift-bases"),
                .product(name: "BaseX", package: "swift-bases"),
                .product(name: "Base32", package: "swift-bases"),
                .product(name: "Base64", package: "swift-bases"),
            ]
        ),
        .testTarget(
            name: "MultibaseTests",
            dependencies: ["Multibase"]
        ),
    ]
)
