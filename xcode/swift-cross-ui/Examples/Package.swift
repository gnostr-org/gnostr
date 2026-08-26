// swift-tools-version: 5.10

import Foundation
import PackageDescription

let env = ProcessInfo.processInfo.environment

var exampleDependencies: [Target.Dependency] = [
    .product(name: "SwiftCrossUI", package: "swift-cross-ui"),
    .product(name: "DefaultBackend", package: "swift-cross-ui"),
]
var hotReloadingDependencies: [Package.Dependency] = []
let hotReloadingEnabled = env["SWIFT_BUNDLER_HOT_RELOADING"] == "1"

#if compiler(>=6.0)
if hotReloadingEnabled {
    hotReloadingDependencies = [
        .package(name: "swift-bundler", path: "../../swift-bundler")
    ]
    exampleDependencies.append(
        .product(
            name: "SwiftBundlerRuntime",
            package: "swift-bundler",
            condition: .when(platforms: [.macOS, .linux])
        )
    )
}
#endif

let package = Package(
    name: "Examples",
    platforms: [.macOS(.v10_15), .iOS(.v13), .tvOS(.v13), .macCatalyst(.v13), .visionOS(.v1)],
    dependencies: [
        .package(name: "swift-cross-ui", path: ".."),
        .package(
            url: "https://github.com/stackotter/swift-miniaudio",
            .upToNextMinor(from: "0.1.2")
        ),
    ] + hotReloadingDependencies,
    targets: [
        .executableTarget(
            name: "ControlsExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "CounterExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "RandomNumberGeneratorExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "WindowingExample",
            dependencies: exampleDependencies,
            resources: [.copy("Banner.png")]
        ),
        .executableTarget(
            name: "GreetingGeneratorExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "NavigationExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "SplitExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "SpreadsheetExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "StressTestExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "NotesExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "PathsExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "WebViewExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "HoverExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "ForEachExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "AdvancedCustomizationExample",
            dependencies: exampleDependencies,
            resources: [.copy("Banner.png")]
        ),
        .executableTarget(
            name: "ColorsExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "GradientsExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "MusicPlayerExample",
            dependencies: [
                .product(name: "MiniAudio", package: "swift-miniaudio")
            ] + exampleDependencies
        ),
        .executableTarget(
            name: "FontsExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "TapGesturesExample",
            dependencies: exampleDependencies
        ),
        .executableTarget(
            name: "DesktopExample",
            dependencies: exampleDependencies
        )
    ]
)
