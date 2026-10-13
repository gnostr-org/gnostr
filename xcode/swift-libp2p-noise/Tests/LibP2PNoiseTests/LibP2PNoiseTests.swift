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
import LibP2PYAMUX
import Testing

@testable import LibP2PNoise

@Suite("Libp2p Noise Tests")
struct LibP2PNoiseTests {
    @Test func testAppConfiguration() throws {
        let app = try Application(.detect())
        app.security.use(.noise)
        #expect(app.security.available.map { $0.description } == ["/noise"])
        let _ = try #require(app.security.upgrader(for: NoiseUpgrader.self))
        let _ = try #require(app.security.upgrader(forKey: NoiseUpgrader.key))
    }

    @Test(.timeLimit(.minutes(2)))
    func testNoiseInterop() async throws {
        let host = try makeLocalEchoHost(port: 10002)
        let client = try makeLocalClient(port: 10003)

        try await host.startup()
        try await client.startup()

        let response = try await client.newRequest(
            to: host.listenAddresses.first!.encapsulate(proto: .p2p, address: host.peerID.b58String),
            forProtocol: "/echo/1.0.0",
            withRequest: "Hello Noise".data(using: .utf8)!,
            withHandlers: .handlers([.newLineDelimited])
        ).get()

        #expect(response == "Hello Noise".data(using: .utf8)!)

        try await Task.sleep(for: .milliseconds(100))

        let connections = try await host.connectionManager.getTotalConnectionCount().get()
        #expect(connections == 1)

        try await host.asyncShutdown()
        try await client.asyncShutdown()
    }
}

private func makeLocalEchoHost(port: Int) throws -> Application {
    let lib = try Application(.testing, peerID: PeerID(.Ed25519))
    lib.environment.arguments = ["libp2p"]
    lib.security.use(.noise)
    lib.muxers.use(.yamux)
    lib.servers.use(.tcp(host: "127.0.0.1", port: port))

    lib.logger.logLevel = .notice

    lib.routes.group("echo", handlers: [.newLineDelimited]) { echo in
        echo.on("1.0.0") { req -> Response<ByteBuffer> in
            switch req.event {
            case .ready: return .stayOpen
            case .data(let data): return .respondThenClose(data)
            case .closed: return .close
            case .error(let error):
                req.logger.error("\(error)")
                return .close
            }
        }
    }

    return lib
}

private func makeLocalClient(port: Int) throws -> Application {
    let lib = try Application(.testing, peerID: PeerID(.Ed25519))
    lib.environment.arguments = ["libp2p"]
    lib.security.use(.noise)
    lib.muxers.use(.yamux)
    lib.servers.use(.tcp(host: "127.0.0.1", port: port))

    lib.logger.logLevel = .notice

    return lib
}
