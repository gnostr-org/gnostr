// swift-tools-version: 6.0
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "test-fluent-driver",
    platforms: [
        .macOS(.v13)
    ],
    dependencies: [
        // Dependencies declare other packages that this package depends on.
        .package(path: "../swift-libp2p"),
        .package(path: "../swift-libp2p-fluent"),
        .package(url: "https://github.com/vapor/fluent-sqlite-driver.git", from: "4.0.0"),
        .package(url: "https://github.com/vapor/fluent-kit.git", .upToNextMajor(from: "1.52.2")),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .executableTarget(
            name: "App",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "Fluent", package: "swift-libp2p-fluent"),
                .product(name: "FluentSQLiteDriver", package: "fluent-sqlite-driver"),
            ],
            swiftSettings: swiftSettings),
        .testTarget(
            name: "AppTests",
            dependencies: [
                .target(name: "App"),
                .product(name: "LibP2PTesting", package: "swift-libp2p"),
                .product(name: "XCTFluent", package: "fluent-kit"),
                .product(name: "FluentSQLiteDriver", package: "fluent-sqlite-driver"),
            ],
            swiftSettings: swiftSettings),
    ]
)

var swiftSettings: [SwiftSetting] { [
    .enableUpcomingFeature("ExistentialAny"),
] }
