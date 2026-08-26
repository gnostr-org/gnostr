// swift-tools-version:5.10
import PackageDescription

let package = Package(
    name: "RustyLib",
    platforms: [
        .iOS(.v15),
        .macCatalyst(.v15)
    ],
    products: [
        .library(
            name: "RustyLib",
            targets: ["RustyLib"])
    ],
    dependencies: [
    ],
    targets: [
        .target(
            name: "RustyLib",
            dependencies: [
                .byName(name: "RustyCore")
            ],
            path: "Sources/"
        ),
        .binaryTarget(
            name: "RustyCore",
            path: "artifacts/RustyCore.xcframework"
        ),
    ]
)
