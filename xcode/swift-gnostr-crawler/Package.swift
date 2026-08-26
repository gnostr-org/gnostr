// swift-tools-version: 6.0
import PackageDescription

let package = Package(
    name: "swift-gnostr-crawler",
    platforms: [
        .macOS(.v10_15),
        .iOS(.v13),
    ],
    products: [
        .library(
            name: "Crawler",
            targets: ["Crawler"]
        ),
    ],
    dependencies: [
        .package(path: "../swift-gnostr-types"),
    ],
    targets: [
        .target(
            name: "Crawler",
            dependencies: [
                .product(name: "GnostrTypes", package: "swift-gnostr-types"),
            ]
        ),
        .testTarget(
            name: "CrawlerTests",
            dependencies: [
                "Crawler",
                .product(name: "GnostrTypes", package: "swift-gnostr-types"),
            ]
        ),
    ]
)
