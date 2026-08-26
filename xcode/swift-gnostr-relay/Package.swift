// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "swift-gnostr-relay",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "Relay",
            targets: ["Relay"]
        ),
    ],
    dependencies: [
        .package(path: "../swift-gnostr-crawler"),
        .package(path: "../swift-gnostr-types"),
    ],
    targets: [
        .target(
            name: "Relay",
            dependencies: [
                .product(name: "Crawler", package: "swift-gnostr-crawler"),
                .product(name: "GnostrTypes", package: "swift-gnostr-types"),
            ]
        ),
        .testTarget(
            name: "RelayTests",
            dependencies: [
                "Relay",
                .product(name: "Crawler", package: "swift-gnostr-crawler"),
            ]
        ),
    ]
)
