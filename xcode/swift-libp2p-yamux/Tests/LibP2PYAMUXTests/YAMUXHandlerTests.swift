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

import NIOCore
import NIOEmbedded
import Testing

@testable import LibP2P
@testable import LibP2PYAMUX

@Suite("Handler Tests")
struct YAMUXHandlerTests {
    @Test func testHandlerInitializationOnAdd_WhenListener() async throws {
        let peerID = try PeerID(.Ed25519)
        let connection = LibP2P.DummyConnection(peer: peerID, direction: .inbound)
        let channel = connection.channel as! EmbeddedChannel
        let promise = channel.eventLoop.makePromise(of: Muxer.self)
        let handler = YAMUXHandler(connection: connection, muxedPromise: promise, supportedProtocols: [])

        // Activate the channel
        _ = try await channel.connect(to: .init(unixDomainSocketPath: "/foo"))

        // Add our handler to the already activated channel
        #expect(throws: Never.self) { try channel.pipeline.syncOperations.addHandler(handler) }
        // Ensure we emit the Session Open message
        #expect(
            try channel.readOutbound(as: Frame.self)
                == .init(header: .init(version: .v0, messageType: .ping, flags: [.syn], streamID: 0, length: 0))
        )

        try await channel.close()
    }

    @Test func testHandlerInitializationOnAdd_WhenInitiator() async throws {
        let peerID = try PeerID(.Ed25519)
        let connection = LibP2P.DummyConnection(peer: peerID, direction: .outbound)
        let channel = connection.channel as! EmbeddedChannel
        let promise = channel.eventLoop.makePromise(of: Muxer.self)
        let handler = YAMUXHandler(connection: connection, muxedPromise: promise, supportedProtocols: [])

        // Activate the channel
        _ = try await channel.connect(to: .init(unixDomainSocketPath: "/foo"))

        // Add our handler to the already activated channel
        #expect(throws: Never.self) { try channel.pipeline.syncOperations.addHandler(handler) }
        #expect(
            try channel.readOutbound(as: Frame.self) == nil
        )

        try await channel.close()
    }

    @Test(.disabled())
    func testHandlerInitializationActive_WhenListener() async throws {
        let peerID = try PeerID(.Ed25519)
        let connection = LibP2P.DummyConnection(peer: peerID, direction: .inbound)
        let channel = connection.channel as! EmbeddedChannel
        let promise = channel.eventLoop.makePromise(of: Muxer.self)
        let handler = YAMUXHandler(connection: connection, muxedPromise: promise, supportedProtocols: [])

        // Add our handler to the inactive channel
        #expect(throws: Never.self) { try channel.pipeline.syncOperations.addHandler(handler) }
        // Ensure we can't read
        #expect(try channel.readOutbound() == nil)

        // Activate the channel
        _ = try await channel.connect(to: .init(unixDomainSocketPath: "/foo"))
        // Ensure we emit the Session Open message
        #expect(
            try channel.readOutbound(as: Frame.self)
                == .init(header: .init(version: .v0, messageType: .ping, flags: [.syn], streamID: 0, length: 0))
        )

        try await channel.close()
    }

    @Test(.disabled())
    func testHandlerInitializationActive_WhenInitiator() async throws {
        let peerID = try PeerID(.Ed25519)
        let connection = LibP2P.DummyConnection(peer: peerID, direction: .outbound)
        let channel = connection.channel as! EmbeddedChannel
        let promise = channel.eventLoop.makePromise(of: Muxer.self)
        let handler = YAMUXHandler(connection: connection, muxedPromise: promise, supportedProtocols: [])

        // Add our handler to the inactive channel
        #expect(throws: Never.self) { try channel.pipeline.syncOperations.addHandler(handler) }
        // Ensure we can't read
        #expect(try channel.readOutbound() == nil)

        // Activate the channel
        _ = try await channel.connect(to: .init(unixDomainSocketPath: "/foo"))
        // Ensure we emit the Session Open message
        #expect(
            try channel.readOutbound(as: Frame.self) == nil
        )

        try await channel.close()
    }
}
