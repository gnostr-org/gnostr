// swift-tools-version: 5.9
import PackageDescription

let package = Package(
    name: "KuboXcode",
    platforms: [.macOS(.v13), .iOS(.v16)],
    dependencies: [
        .package(path: "../swift"),
    ],
    targets: [
        .executableTarget(
            name: "XcodeApp",
            dependencies: [.product(name: "Kubo", package: "swift")],
            path: "Sources/XcodeApp",
            linkerSettings: [
                .unsafeFlags(["-L", "../swift", "-lkubo_ffi",
                             "-framework", "Security", "-framework", "CoreFoundation",
                             "-lresolv", "-lpthread", "-ldl"]),
            ]
        ),
    ]
)
