//===----------------------------------------------------------------------===//
//
// This source file is part of the swift-libp2p open source project
//
// Copyright (c) 2022-2025 swift-libp2p project authors
// Licensed under MIT
//
// See LICENSE for license information
// See CONTRIBUTORS for the list of swift-libp2p project authors
//
// SPDX-License-Identifier: MIT
//
//===----------------------------------------------------------------------===//

import Foundation
import LibP2P
import LibP2PNoise
import LibP2PYAMUX
import Testing

@testable import LibP2PPubSub

@Suite("Libp2p PubSub Tests", .serialized)
struct LibP2PPubSubTests {

    @Test func testAppConfiguration_Floodsub() async throws {
        let app = try await Application.make(.testing, peerID: PeerID(.Ed25519))
        app.logger.logLevel = .trace
        app.environment.arguments = [app.environment.arguments.first ?? "xctest"]

        /// Configure our networking stack!
        app.servers.use(.tcp(host: "127.0.0.1", port: 10000))
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.pubsub.use(.floodsub)

        #expect(app.pubsub.available.map({ $0.description }) == ["/floodsub/1.0.0"])
        #expect(app.pubsub.service(for: FloodSub.self) != nil)
        #expect(app.pubsub.service(forKey: FloodSub.multicodec) != nil)

        try await app.startup()

        try await Task.sleep(for: .milliseconds(10))

        try await app.asyncShutdown()
    }

    @Test func testAppConfiguration_Gossipsub() async throws {
        let app = try await Application.make(.testing, peerID: PeerID(.Ed25519))
        app.logger.logLevel = .trace
        app.environment.arguments = [app.environment.arguments.first ?? "xctest"]

        /// Configure our networking stack!
        app.servers.use(.tcp(host: "127.0.0.1", port: 10000))
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.pubsub.use(.gossipsub)

        #expect(app.pubsub.available.map({ $0.description }) == ["/meshsub/1.0.0"])
        #expect(app.pubsub.service(for: GossipSub.self) != nil)
        #expect(app.pubsub.service(forKey: GossipSub.multicodec) != nil)

        try await app.startup()

        try await Task.sleep(for: .milliseconds(10))

        try await app.asyncShutdown()
    }

}

struct TestHelper {
    static var externalIntegrationTestsEnabled: Bool {
        guard ProcessInfo.processInfo.environment["PerformExternalIntegrationTests"] == "true" else {
            return false
        }
        return hasCommand("swift-libp2p-pubsub")
    }

    private static func hasCommand(_ name: String) -> Bool {
        let process = Process()
        process.executableURL = URL(fileURLWithPath: "/usr/bin/env")
        process.arguments = ["bash", "-lc", "command -v \(name) >/dev/null 2>&1"]
        process.standardOutput = FileHandle.nullDevice
        process.standardError = FileHandle.nullDevice

        do {
            try process.run()
            process.waitUntilExit()
            return process.terminationStatus == 0
        } catch {
            return false
        }
    }
}

extension Trait where Self == ConditionTrait {
    /// This test is only available when the `PerformExternalIntegrationTests` environment variable is set to `true`
    public static var externalIntegrationTestsEnabled: Self {
        enabled(
            if: TestHelper.externalIntegrationTestsEnabled,
            "This test is only available when the `PerformExternalIntegrationTests` environment variable is set to `true`"
        )
    }
}
