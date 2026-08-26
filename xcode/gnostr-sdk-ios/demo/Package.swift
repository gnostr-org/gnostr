let package = Package(
    name: "GnostrSDKDemo",
    dependencies: [
        .package(path: "../")
    ],
    targets: [
        .target(
            name: "GnostrSDKDemo",
            dependencies: [
                "GnostrSDK",
                .product(name: "ContextAwareToolbar", package: "GnostrSDK")
            ])
    ]
)
