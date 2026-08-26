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

import LibP2P
import NIOTestUtils
import Testing

@testable import LibP2PYAMUX

@Suite("Yamux Tests")
struct LibP2PYAMUXTests {

    @Test func testAppConfiguration() throws {
        let app = try Application(.detect())
        app.muxers.use(.yamux)
        #expect(app.muxers.available.map { $0.description } == ["/yamux/1.0.0"])
        let _ = try #require(app.muxers.upgrader(for: YAMUX.self))
        let _ = try #require(app.muxers.upgrader(forKey: YAMUX.key))
    }

}

struct TestHelper {
    static var internalIntegrationTestsEnabled: Bool {
        if let b = ProcessInfo.processInfo.environment["PerformInternalIntegrationTests"], b == "true" {
            return true
        }
        return false
    }

    static var externalIntegrationTestsEnabled: Bool {
        if let b = ProcessInfo.processInfo.environment["PerformExternalIntegrationTests"], b == "true" {
            return true
        }
        return false
    }
}

extension Trait where Self == ConditionTrait {
    /// This test is only available when the `PerformInternalIntegrationTests` environment variable is set to `true`
    public static var internalIntegrationTestsEnabled: Self {
        enabled(
            if: TestHelper.internalIntegrationTestsEnabled,
            "This test is only available when the `PerformInternalIntegrationTests` environment variable is set to `true`"
        )
    }

    /// This test is only available when the `PerformExternalIntegrationTests` environment variable is set to `true`
    public static var externalIntegrationTestsEnabled: Self {
        enabled(
            if: TestHelper.externalIntegrationTestsEnabled,
            "This test is only available when the `PerformExternalIntegrationTests` environment variable is set to `true`"
        )
    }
}
