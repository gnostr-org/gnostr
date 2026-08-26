// swift-tools-version: 6.0

import Foundation
import PackageDescription

let package = Package(
    name: "swift-cross-ui-p2p",
    platforms: [
        .macOS(.v11),
        //.iOS(.v13),
    ],
    products: [
        .executable(name: "SwiftCrossUIP2P", targets: ["SwiftCrossUIP2P"]),
    ],
    dependencies: {
        var dependencies: [Package.Dependency] = [
            .package(path: "../swift-cross-ui"),
            .package(path: "../swift-libp2p"),
            .package(path: "../swift-libp2p-noise"),
            .package(path: "../swift-libp2p-yamux"),
            .package(path: "../swift-libp2p-dcutr"),
            .package(path: "../swift-libp2p-kad-dht"),
            .package(name: "GnostrGit", path: "../Git"),
        ]

#if os(macOS) || os(iOS)
        dependencies.append(.package(path: "../swift-libp2p-mdns"))
#endif

        return dependencies
    }(),
    targets: [
        .executableTarget(
            name: "SwiftCrossUIP2P",
            dependencies: {
                var dependencies: [Target.Dependency] = [
                    .product(name: "SwiftCrossUI", package: "swift-cross-ui"),
                    .product(name: "DefaultBackend", package: "swift-cross-ui"),
                    .product(name: "LibP2P", package: "swift-libp2p"),
                    .product(name: "LibP2PNoise", package: "swift-libp2p-noise"),
                    .product(name: "LibP2PYAMUX", package: "swift-libp2p-yamux"),
                    .product(name: "LibP2PDCUtR", package: "swift-libp2p-dcutr"),
                    .product(name: "LibP2PKadDHT", package: "swift-libp2p-kad-dht"),
                    .product(
                        name: "GnostrGit",
                        package: "GnostrGit",
                        condition: .when(platforms: [.iOS])
                    ),
                    .product(
                        name: "XGit",
                        package: "GnostrGit",
                        condition: .when(platforms: [.iOS])
                    ),
                ]

#if os(macOS) || os(iOS)
                dependencies.append(.product(name: "LibP2PMDNS", package: "swift-libp2p-mdns"))
#endif

                return dependencies
            }()
        ),
    ]
)
