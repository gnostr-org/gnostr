// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "swift-gnostr-types",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "GnostrTypes",
            targets: ["GnostrTypes"]
        ),
    ],
    dependencies: [],
    targets: [
        .target(
            name: "GnostrTypes"
        ),
        .testTarget(
            name: "GnostrTypesTests",
            dependencies: ["GnostrTypes"]
        ),
    ]
)
