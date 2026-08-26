// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "swift-gnostr-asyncgit",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "AsyncGit",
            targets: ["AsyncGit"]
        ),
    ],
    dependencies: [
        .package(path: "../swift-gnostr-types"),
    ],
    targets: [
        .target(
            name: "AsyncGit",
            dependencies: [
                .product(name: "GnostrTypes", package: "swift-gnostr-types"),
            ]
        ),
        .testTarget(
            name: "AsyncGitTests",
            dependencies: ["AsyncGit"]
        ),
    ]
)
