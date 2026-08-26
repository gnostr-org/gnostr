// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "swift-gnostr-ffi-kitchensink",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "FFIKitchenSink",
            targets: ["FFIKitchenSink"]
        ),
    ],
    dependencies: [
        .package(path: "../swift-gnostr-asyncgit"),
        .package(path: "../swift-gnostr-crawler"),
        .package(path: "../swift-gnostr-relay"),
        .package(path: "../swift-gnostr-types"),
    ],
    targets: [
        .target(
            name: "FFIKitchenSink",
            dependencies: [
                .product(name: "AsyncGit", package: "swift-gnostr-asyncgit"),
                .product(name: "Crawler", package: "swift-gnostr-crawler"),
                .product(name: "Relay", package: "swift-gnostr-relay"),
                .product(name: "GnostrTypes", package: "swift-gnostr-types"),
            ]
        ),
        .testTarget(
            name: "FFIKitchenSinkTests",
            dependencies: [
                "FFIKitchenSink",
                .product(name: "AsyncGit", package: "swift-gnostr-asyncgit"),
                .product(name: "Crawler", package: "swift-gnostr-crawler"),
                .product(name: "Relay", package: "swift-gnostr-relay"),
                .product(name: "GnostrTypes", package: "swift-gnostr-types"),
            ]
        ),
    ]
)
