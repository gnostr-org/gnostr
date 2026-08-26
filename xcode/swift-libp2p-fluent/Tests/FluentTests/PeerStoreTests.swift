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
//
//  Created by Vapor
//  Modified by swift-libp2p
//

import LibP2P
import LibP2PCore
import LibP2PCrypto
import LibP2PTesting
import Testing
import XCTFluent

@testable import Fluent

@Suite("PeerStore Tests")
struct PeerStoreTests {

    let test = ArrayTestDatabase()

    private func configure(_ app: Application) async throws {
        // Set our log level
        app.logger.logLevel = .info
        // Setup test db.
        app.databases.use(test.configuration, as: .test)
        app.peerstore.use(.fluent)
        app.peerstore.prepareMigrations()
    }

    @Test func peerStoreMigrationName() {
        #expect(PeerStoreEntry.migration.name == "Fluent.PeerStoreEntry.Create")
    }

    @Test func testPeerStoreStoreAndFetchPeerID() async throws {
        try await withApp(configure: configure) { app in
            let peer = try PeerID(.Ed25519)
            test.append([TestOutput(PeerStoreEntry(id: UUID(), peerID: peer))])

            do {
                let peers1 = try await app.peers.all().get()
                #expect(peers1.count == 1)
                let recoveredPeer = try #require(peers1.first)
                #expect(recoveredPeer.id == peer)
            } catch {
                Issue.record(error)
            }
        }
    }
}
