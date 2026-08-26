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
import Testing

@testable import LibP2PTCP

@Suite("Libp2p TCP Tests")
struct LibP2PTCPTests {

    @Test func testTCP() async throws {
        let app = try await Application.make(.detect(), peerID: .ephemeral())

        app.servers.use(.tcp_embedded)
        app.environment.arguments = ["libp2p"]

        try await app.startup()

        #expect(app.environment == Environment.testing)
        #expect(try app.listenAddresses == [Multiaddr("/ip4/127.0.0.1/tcp/10000")])

        try await Task.sleep(for: .milliseconds(100))

        try await app.asyncShutdown()
    }

}
