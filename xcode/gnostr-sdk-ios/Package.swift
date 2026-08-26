// swift-tools-version: 5.10
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "GnostrSDK",
    platforms: [.macOS(.v14), .iOS(.v15)],
    products: [
        // Products define the executables and libraries a package produces,
        // and make them visible to other packages.
        // origin https://github.com/nostr-sdk/nostr-sdk-ios
        .library(
            name: "ContextAwareToolbar",
            targets: ["ContextAwareToolbar"]),
        .library(
            name: "GnostrSDK",
            targets: ["GnostrSDK"]),
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        // .package(url: /* package url */, from: "1.0.0"),
        .package(url: "https://github.com/apple/swift-docc-plugin.git", .upToNextMajor(from: "1.4.3")),
        .package(url: "https://github.com/GigaBitcoin/secp256k1.swift", exact: "0.12.2"),
        .package(url: "https://github.com/krzyzanowskim/CryptoSwift.git", .upToNextMajor(from: "1.8.4")),
        .package(url: "https://github.com/apple/swift-collections.git", .upToNextMajor(from: "1.1.4")),
        .package(url: "https://github.com/ibrahimcetin/SwiftGitX.git", .upToNextMajor(from: "0.4.0"))
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "ContextAwareToolbar",
            dependencies: [],
            path: "Sources/ContextAwareToolbar"
        ),
        .target(
            name: "GnostrSDK",
            dependencies: [
                .product(name: "secp256k1", package: "secp256k1.swift"),
                "CryptoSwift",
                .product(name: "OrderedCollections", package: "swift-collections")
            ],
            path: "Sources/GnostrSDK"
        ),
        .testTarget(
            name: "GnostrSDKTests",
            dependencies: ["GnostrSDK"],
            path: "Tests/GnostrSDKTests",
            resources: [.copy("Fixtures")]
        )
    ]
)
