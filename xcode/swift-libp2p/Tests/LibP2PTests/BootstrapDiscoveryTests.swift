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
import LibP2PTesting
import Testing

extension LibP2PTests {
    @Suite("Bootstrap Discovery Tests")
    struct BootstrapDiscoveryTests {
        @Test func testBootstrapPeersArePublishedOnStartup() async throws {
            let app = try await Application.make(.testing, peerID: .ephemeral)
            app.environment.arguments = ["libp2p"]

            let peerID = try PeerID(.Ed25519)
            let address = try Multiaddr("/ip4/127.0.0.1/tcp/4001/p2p/\(peerID.b58String)")
            let peerInfo = PeerInfo(peer: peerID, addresses: [address])

            app.discovery.use(.bootstrap([peerInfo]))

            final class PeerCapture: NSObject {
                private let lock = NSLock()
                private var stored: [PeerInfo] = []

                func record(_ peer: PeerInfo) {
                    self.lock.lock()
                    self.stored.append(peer)
                    self.lock.unlock()
                }

                var peers: [PeerInfo] {
                    self.lock.lock()
                    defer { self.lock.unlock() }
                    return self.stored
                }
            }

            let capture = PeerCapture()
            app.discovery.onPeerDiscovered(capture) { peer in
                capture.record(peer)
            }

            try await app.startup()
            try await Task.sleep(for: .milliseconds(250))

            let bootstrapService = try #require(app.discovery.service(for: BootstrapPeerDiscovery.self))
            let knownPeers = try await bootstrapService.knownPeers().get()
            #expect(knownPeers.contains(where: { $0.peer == peerID }))
            #expect(knownPeers.contains(where: { $0.addresses.contains(address) }))

            let capturedPeers = capture.peers
            #expect(capturedPeers.contains(where: { $0.peer == peerID }))

            try await app.asyncShutdown()
        }

        @Test func testBootstrapProviderParsesStringMultiaddrs() async throws {
            let app = try await Application.make(.testing, peerID: .ephemeral)
            app.environment.arguments = ["libp2p"]

            let peerID = try PeerID(.Ed25519)
            let address = "/ip4/127.0.0.1/tcp/4002/p2p/\(peerID.b58String)"
            let expectedAddress = try Multiaddr(address)
            app.discovery.use(.bootstrap([address]))

            try await app.startup()
            try await Task.sleep(for: .milliseconds(250))

            let bootstrapService = try #require(app.discovery.service(for: BootstrapPeerDiscovery.self))
            let knownPeers = try await bootstrapService.knownPeers().get()
            #expect(knownPeers.contains(where: { $0.peer == peerID }))
            #expect(knownPeers.contains(where: { $0.addresses.contains(expectedAddress) }))

            try await app.asyncShutdown()
        }
    }
}
