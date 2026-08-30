// swift-tools-version:5.9
import PackageDescription

let package = Package(
    name: "libp2p-app-template",
    platforms: [.macOS(.v13), .iOS(.v15)],
    products: [
        .library(name: "LibP2PAppTemplateCore", targets: ["LibP2PAppTemplateCore"]),
    ],
    dependencies: [
        .package(path: "../swift-libp2p"),
        .package(path: "../swift-libp2p-noise"),
        .package(path: "../swift-libp2p-yamux"),
        .package(path: "../swift-libp2p-dcutr"),
        .package(path: "../swift-libp2p-mdns"),
        .package(path: "../swift-libp2p-kad-dht"),
        .package(path: "../swift-libp2p-pubsub"),
        .package(path: "../swift-libp2p-relay"),
        .package(path: "../swift-libp2p-autonat"),
        .package(path: "../swift-multihash"),
    ],
    targets: [
        .target(
            name: "LibP2PAppTemplateCore",
            dependencies: [
                .product(name: "LibP2P", package: "swift-libp2p"),
                .product(name: "LibP2PNoise", package: "swift-libp2p-noise"),
                .product(name: "LibP2PYAMUX", package: "swift-libp2p-yamux"),
                .product(name: "LibP2PDCUtR", package: "swift-libp2p-dcutr"),
                .product(name: "LibP2PMDNS", package: "swift-libp2p-mdns"),
                .product(name: "LibP2PKadDHT", package: "swift-libp2p-kad-dht"),
                .product(name: "LibP2PPubSub", package: "swift-libp2p-pubsub"),
                .product(name: "LibP2PRelay", package: "swift-libp2p-relay"),
                .product(name: "LibP2PAutoNAT", package: "swift-libp2p-autonat"),
                .product(name: "Multihash", package: "swift-multihash"),
            ],
            path: "LibP2PAppTemplate",
            exclude: ["Preview Content", "Assets.xcassets", "LibP2PAppTemplate.entitlements", "LibP2PAppTemplateApp.swift"]
        ),
        .testTarget(
            name: "LibP2PAppTemplateTests",
            dependencies: ["LibP2PAppTemplateCore"]
        ),
    ]
)
