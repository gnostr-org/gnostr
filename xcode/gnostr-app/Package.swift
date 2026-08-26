// swift-tools-version:5.5
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "gnostr",
    platforms: [
        .iOS(.v15),
        .macOS(.v12)
    ],
    products: [
        .library(
            name: "gnostr",
            targets: ["gnostr"])
    ],
    dependencies: [
        .package(url: "https://github.com/daltoniam/Starscream.git", .upToNextMajor(from: "4.0.0")),
        .package(url: "https://github.com/jb55/secp256k1.swift.git", branch: "main"),
        .package(url: "https://github.com/onevcat/Kingfisher.git", .upToNextMajor(from: "7.0.0")),
        .package(url: "https://github.com/joshuajhomann/Shimmer", branch: "master"),
        .package(url: "https://github.com/SparrowTek/Vault", .upToNextMajor(from: "1.0.0")),
        .package(path: "Git")
    ],
    targets: [
        .target(
            name: "gnostr",
            dependencies: [
                .product(name: "Starscream", package: "Starscream"),
                .product(name: "secp256k1", package: "secp256k1.swift"),
                .product(name: "Kingfisher", package: "Kingfisher"),
                .product(name: "Shimmer", package: "Shimmer"),
                .product(name: "Vault", package: "Vault"),
                .product(name: "GnostrGit", package: "GnostrGit")
            ],
            path: "damus",
            exclude: ["Preview Content"]
        ),
        .testTarget(
            name: "gnostrTests",
            dependencies: ["gnostr"],
            path: "damusTests"
        ),
        .testTarget(
            name: "gnostrUITests",
            dependencies: ["gnostr"],
            path: "damusUITests"
        )
    ]
)
