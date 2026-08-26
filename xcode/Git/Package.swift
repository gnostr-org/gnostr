// swift-tools-version:5.3
// The swift-tools-version declares the minimum version of Swift required to build this package.

import PackageDescription

let package = Package(
    name: "GnostrGit",
    platforms: [
        .iOS(.v14),
        .macOS(.v11)
    ],
    products: [
        // Products define the executables and libraries a package produces, and make them visible to other packages.
        .library(
            name: "XGit",
            targets: ["XGit"]),
        .library(
            name: "GnostrGit",
            targets: ["GnostrGit"]),
    ],
    dependencies: [
        .package(path: "../LibGit2-iOS"),
    ],
    targets: [
        // Targets are the basic building blocks of a package. A target can define a module or a test suite.
        // Targets can depend on other targets in this package, and on products in packages this package depends on.
        .target(
            name: "XGit",
            dependencies: [
                .product(name: "Clibgit2", package: "libgit2-ios"),
            ],
            exclude: ["internal"],
            linkerSettings: [
                .linkedLibrary("z"),
                .linkedLibrary("iconv"),
            ]),
        .target(
            name: "GnostrGit",
            dependencies: ["XGit"],
            linkerSettings: [.linkedLibrary("z"), .linkedLibrary("iconv")]),
        .testTarget(
            name: "GnostrGitTests",
            dependencies: ["GnostrGit"]),
    ],
    cxxLanguageStandard: .cxx14
)
