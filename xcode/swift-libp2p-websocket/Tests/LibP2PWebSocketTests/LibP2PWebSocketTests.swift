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
import LibP2PMPLEX
import LibP2PNoise
import Testing

@testable import LibP2PWebSocket

@Suite("Libp2p WebSocket Tests", .serialized)
struct LibP2PWebSocketTests {

    @Test func testInternalWebSocketStartThenStop() throws {
        let host = Application(.testing)
        host.environment.arguments = ["libp2p"]
        host.servers.use(.ws(host: "127.0.0.1", port: 10000))
        host.security.use(.noise)
        host.muxers.use(.mplex)

        #expect(throws: Never.self) { try host.start() }

        print(host.listenAddresses)
        sleep(1)

        #expect(try host.listenAddresses.first == Multiaddr("/ip4/127.0.0.1/tcp/10000/ws"))

        host.shutdown()
    }

    @Test func testInternalWebSocketEcho() async throws {
        let host = try await Application.make(.testing, peerID: .ephemeral())
        host.environment.arguments = ["libp2p"]
        host.servers.use(.ws(host: "127.0.0.1", port: 10000))
        host.security.use(.noise)
        host.muxers.use(.mplex)
        host.logger.logLevel = .trace

        let client = try await Application.make(.testing, peerID: .ephemeral())
        client.environment.arguments = ["libp2p"]
        client.servers.use(.ws(host: "127.0.0.1", port: 10001))
        client.security.use(.noise)
        client.muxers.use(.mplex)
        client.transports.use(.ws)
        client.logger.logLevel = .trace

        host.routes.on("echo", "1.0.0", handlers: [.newLineDelimited]) { req -> Response<ByteBuffer> in
            switch req.event {
            case .ready: return .stayOpen
            case .data(let data): return .respondThenClose(data)
            case .closed, .error: return .close
            }
        }

        try await host.startup()
        try await client.startup()

        let hostAddress = host.listenAddresses.first

        #expect(hostAddress != nil)
        #expect(try hostAddress == Multiaddr("/ip4/127.0.0.1/tcp/10000/ws"))
        #expect(try client.listenAddresses.first == Multiaddr("/ip4/127.0.0.1/tcp/10001/ws"))

        let echoMessage = "Hello from swift libp2p!"
        let response = try await client.newRequest(
            to: hostAddress!,
            forProtocol: "/echo/1.0.0",
            withRequest: Data(echoMessage.utf8),
            withHandlers: .handlers([.newLineDelimited]),
            withTimeout: .seconds(4)
        ).get()

        guard let str = String(data: Data(response), encoding: .utf8) else {
            Issue.record("Failed to decode response data")
            return
        }
        #expect(str == echoMessage)

        try await Task.sleep(for: .milliseconds(50))

        // After 50ms we should have some active connections to between our peers
        print("🔀🔀🔀 Connections Between Peers 🔀🔀🔀")
        let clientConnections = try await client.connections.getConnectionsToPeer(peer: host.peerID, on: nil).get()
        #expect(clientConnections.count > 0)
        for connection in clientConnections {
            print(connection)
        }
        let hostConnections = try await host.connections.getConnectionsToPeer(peer: client.peerID, on: nil).get()
        #expect(hostConnections.count > 0)
        for connection in hostConnections {
            print(connection)
        }
        print("----------------------------------------")

        try await Task.sleep(for: .milliseconds(500))

        // After 500ms of inactivity our connections between our peers should be pruned
        print("🔀🔀🔀 Connections Between Peers 🔀🔀🔀")
        let clientConnections2 = try await client.connections.getConnectionsToPeer(peer: host.peerID, on: nil).get()
        #expect(clientConnections2.count == 0)
        for connection in clientConnections2 {
            print(connection)
        }
        let hostConnections2 = try await host.connections.getConnectionsToPeer(peer: client.peerID, on: nil).get()
        #expect(hostConnections2.count == 0)
        for connection in hostConnections2 {
            print(connection)
        }
        print("----------------------------------------")

        try await Task.sleep(for: .seconds(1))

        try await client.asyncShutdown()
        try await host.asyncShutdown()
    }

    @Test(.externalIntegrationTestsEnabled)
    func testExternalSwiftHostSameLAN() async throws {
        guard let b = ProcessInfo.processInfo.environment["PerformIntegrationTests"], b == "true" else {
            print("Skipping Integration Test")
            return
        }
        let client = try await Application.make(.testing, peerID: .ephemeral())
        client.environment.arguments = ["libp2p"]
        client.servers.use(.ws(host: "0.0.0.0", port: 10000))
        client.security.use(.noise)
        client.muxers.use(.mplex)
        client.transports.use(.ws)
        client.logger.logLevel = .trace

        try await client.startup()

        let hostAddress = try Multiaddr(
            "/ip4/192.168.1.44/tcp/10000/p2p/12D3KooWQekqjrkfVMP8aC2rExb3VQiuGJ4YGtq4gt5SsRMsmrdw"
        )

        #expect(try client.listenAddresses.first == Multiaddr("/ip4/0.0.0.0/tcp/10000"))

        let echoMessage = "Hello from swift libp2p!"
        let response = try await client.newRequest(
            to: hostAddress,
            forProtocol: "/echo/1.0.0",
            withRequest: Data(echoMessage.utf8),
            withHandlers: .handlers([.newLineDelimited]),
            withTimeout: .seconds(2)
        ).get()

        let str = try #require(String(data: Data(response), encoding: .utf8))
        #expect(str == echoMessage)

        try await client.asyncShutdown()
    }

    /// **************************************
    ///    Testing Go Host WS Interoperability
    /// **************************************
    /// In order to run this example, use the js-LibP2P Examples/Echo example in listening mode on port 10000
    /// - Note: Using a shell / terminal window execute the following command to get it echoed back to you
    /// ```
    /// git clone https://github.com/libp2p/js-libp2p.git //if you dont have the js-libp2p repo yet
    /// cd js-libp2p
    /// npm install
    /// cd examples/echo/src
    /// // Run
    /// node listener.js
    /// // Copy one of the listening addresses into `hostAddress`
    /// // Run this test
    /// ```
    /// - Note: Outbound Go WebSockets don't work. Not sure why.
    /// - Note: I think it's either a timing issue (it has worked a couple times in the past)
    @Test(.externalIntegrationTestsEnabled)
    func testWebSocketSwiftClientGoHost() async throws {
        let client = try await Application.make(.testing, peerID: .ephemeral())
        client.servers.use(.ws(host: "0.0.0.0", port: 10000))
        client.security.use(.noise)
        client.muxers.use(.mplex)
        client.transports.use(.ws)
        client.logger.logLevel = .trace

        try await client.startup()

        let hostAddress = try Multiaddr(
            "/ip4/127.0.0.1/tcp/10001/ws/p2p/QmVdiowCX42i1PeFpo2eadzHHMwa1Dn1VBCr4egQrj2XDm"
        )

        #expect(try client.listenAddresses.first == Multiaddr("/ip4/0.0.0.0/tcp/10000/ws"))

        let echoMessage = "Hello from swift libp2p!"
        let response = try await client.newRequest(
            to: hostAddress,
            forProtocol: "/echo/1.0.0",
            withRequest: Data(echoMessage.utf8),
            withHandlers: .handlers([.newLineDelimited]),
            withTimeout: .seconds(2)
        ).get()

        let str = try #require(String(data: Data(response), encoding: .utf8))
        #expect(str == echoMessage)

        try await client.asyncShutdown()
    }

    /// **************************************
    ///    Testing Go Client WS Interoperability
    /// **************************************
    /// In order to run this example, use the go-LibP2P Examples/Echo example in listening mode on port 10000
    /// - Note: Using a shell / terminal window execute the following command to get it echoed back to you
    /// ```
    /// git clone https://github.com/libp2p/go-libp2p.git //if you dont have the go-libp2p repo yet
    /// cd go-libp2p/examples/echo
    /// go build
    /// // Run this test, then...
    /// ./echo -l 10000 -d <your listening address>
    /// ```
    /// - Note: Inbound Go WebSockets work
    @Test(.externalIntegrationTestsEnabled)
    func testWebSocketSwiftHostGoClient() async throws {
        let host = try await Application.make(.testing, peerID: .ephemeral())
        host.servers.use(.ws(host: "192.168.1.3", port: 10000))
        host.security.use(.noise)
        host.muxers.use(.mplex)
        host.logger.logLevel = .trace

        print(
            "Swift Libp2p host listening on: \(host.listenAddresses.map { try! $0.encapsulate(proto: .p2p, address: host.peerID.b58String) })"
        )

        let expectedMessages: Int = 1
        var echoedMessages: [String] = []

        try await confirmation(expectedCount: expectedMessages) { confirm in
            let suspend = Task { try await Task.sleep(for: .seconds(60)) }

            host.routes.on("echo", "1.0.0", handlers: [.newLineDelimited]) { req -> Response<ByteBuffer> in
                print("/echo/1.0.0 request -> \(req)")
                switch req.event {
                case .ready:
                    return .stayOpen
                case .data(let data):
                    if let str = String(data: Data(data.readableBytesView), encoding: .utf8) {
                        echoedMessages.append(str)
                    } else {
                        print("Non UTF8 Message Encountered")
                    }
                    return .respondThenClose(data)
                case .closed:
                    if echoedMessages.count == expectedMessages {
                        confirm()
                        suspend.cancel()
                    }
                    return .close
                case .error:
                    return .close
                }
            }

            try await host.startup()
            #expect(try host.listenAddresses.first == Multiaddr("/ip4/192.168.1.3/tcp/10000/ws"))

            try await suspend.value
        }

        #expect(echoedMessages.count == expectedMessages)
        #expect(echoedMessages.first == "Hello, world!")

        try await host.asyncShutdown()
    }

    /// **************************************
    ///    Testing JS Host WS Interoperability
    /// **************************************
    /// In order to run this example, use the js-LibP2P Examples/Echo example in listening mode on port 10000
    /// - Note: Using a shell / terminal window execute the following command to get it echoed back to you
    /// ```
    /// git clone https://github.com/libp2p/js-libp2p.git //if you dont have the js-libp2p repo yet
    /// cd js-libp2p
    /// npm install
    /// cd examples/echo/src
    /// // Run
    /// node listener.js
    /// // Copy one of the listening addresses into `hostAddress`
    /// // Run this test
    /// ```
    /// - Note: Unlike GO, JS does not delimit their messages with a newLine char
    @Test(.externalIntegrationTestsEnabled)
    func testWebSocketSwiftClientJSHost() async throws {
        let client = try await Application.make(.testing, peerID: .ephemeral())
        client.servers.use(.ws(host: "127.0.0.1", port: 10000))
        client.security.use(.noise)
        client.muxers.use(.mplex)
        client.transports.use(.ws)
        client.logger.logLevel = .trace

        try await client.startup()

        let hostAddress = try Multiaddr(
            "/ip4/192.168.1.22/tcp/10334/ws/p2p/QmcrQZ6RJdpYuGvZqD5QEHAv6qX4BrQLJLQPQUrTrzdcgm"
        )

        #expect(try client.listenAddresses.first == Multiaddr("/ip4/127.0.0.1/tcp/10000/ws"))

        let echoMessage = "Hello from swift libp2p!"

        let response = try await client.newRequest(
            to: hostAddress,
            forProtocol: "/echo/1.0.0",
            withRequest: Data(echoMessage.utf8)
        ).get()

        let str = try #require(String(data: Data(response), encoding: .utf8))
        #expect(str == echoMessage)

        try await client.asyncShutdown()
    }

    /// **************************************
    ///    Testing JS Client WS Interoperability
    /// **************************************
    /// In order to run this example, use the js-LibP2P Examples/Echo example in listening mode on port 10000
    /// - Note: Using a shell / terminal window execute the following command to get it echoed back to you
    /// ```
    /// git clone https://github.com/libp2p/js-libp2p.git //if you dont have the js-libp2p repo yet
    /// cd js-libp2p
    /// npm install
    /// cd examples/echo/src
    /// // Run this test, then...
    /// // Edit the dialer.js to support WS and to dial this computers address...
    /// node dailer-ws.js
    /// ```
    /// - Note: This test requires the custom PeerID due to JS echo example using static keypairs and expecting them in the handshake
    /// - Note: Unlike GO, JS does not delimit their messages with a newLine char
    @Test(.externalIntegrationTestsEnabled)
    func testWebSocketSwiftHostJSClient() async throws {
        let str = """
            {
              "id": "QmcrQZ6RJdpYuGvZqD5QEHAv6qX4BrQLJLQPQUrTrzdcgm",
              "privKey": "CAASqAkwggSkAgEAAoIBAQDLZZcGcbe4urMBVlcHgN0fpBymY+xcr14ewvamG70QZODJ1h9sljlExZ7byLiqRB3SjGbfpZ1FweznwNxWtWpjHkQjTVXeoM4EEgDSNO/Cg7KNlU0EJvgPJXeEPycAZX9qASbVJ6EECQ40VR/7+SuSqsdL1hrmG1phpIju+D64gLyWpw9WEALfzMpH5I/KvdYDW3N4g6zOD2mZNp5y1gHeXINHWzMF596O72/6cxwyiXV1eJ000k1NVnUyrPjXtqWdVLRk5IU1LFpoQoXZU5X1hKj1a2qt/lZfH5eOrF/ramHcwhrYYw1txf8JHXWO/bbNnyemTHAvutZpTNrsWATfAgMBAAECggEAQj0obPnVyjxLFZFnsFLgMHDCv9Fk5V5bOYtmxfvcm50us6ye+T8HEYWGUa9RrGmYiLweuJD34gLgwyzE1RwptHPj3tdNsr4NubefOtXwixlWqdNIjKSgPlaGULQ8YF2tm/kaC2rnfifwz0w1qVqhPReO5fypL+0ShyANVD3WN0Fo2ugzrniCXHUpR2sHXSg6K+2+qWdveyjNWog34b7CgpV73Ln96BWae6ElU8PR5AWdMnRaA9ucA+/HWWJIWB3Fb4+6uwlxhu2L50Ckq1gwYZCtGw63q5L4CglmXMfIKnQAuEzazq9T4YxEkp+XDnVZAOgnQGUBYpetlgMmkkh9qQKBgQDvsEs0ThzFLgnhtC2Jy//ZOrOvIAKAZZf/mS08AqWH3L0/Rjm8ZYbLsRcoWU78sl8UFFwAQhMRDBP9G+RPojWVahBL/B7emdKKnFR1NfwKjFdDVaoX5uNvZEKSl9UubbC4WZJ65u/cd5jEnj+w3ir9G8n+P1gp/0yBz02nZXFgSwKBgQDZPQr4HBxZL7Kx7D49ormIlB7CCn2i7mT11Cppn5ifUTrp7DbFJ2t9e8UNk6tgvbENgCKXvXWsmflSo9gmMxeEOD40AgAkO8Pn2R4OYhrwd89dECiKM34HrVNBzGoB5+YsAno6zGvOzLKbNwMG++2iuNXqXTk4uV9GcI8OnU5ZPQKBgCZUGrKSiyc85XeiSGXwqUkjifhHNh8yH8xPwlwGUFIZimnD4RevZI7OEtXw8iCWpX2gg9XGuyXOuKORAkF5vvfVriV4e7c9Ad4Igbj8mQFWz92EpV6NHXGCpuKqRPzXrZrNOA9PPqwSs+s9IxI1dMpk1zhBCOguWx2m+NP79NVhAoGBAI6WSoTfrpu7ewbdkVzTWgQTdLzYNe6jmxDf2ZbKclrf7lNr/+cYIK2Ud5qZunsdBwFdgVcnu/02czeS42TvVBgs8mcgiQc/Uy7yi4/VROlhOnJTEMjlU2umkGc3zLzDgYiRd7jwRDLQmMrYKNyEr02HFKFn3w8kXSzW5I8rISnhAoGBANhchHVtJd3VMYvxNcQb909FiwTnT9kl9pkjhwivx+f8/K8pDfYCjYSBYCfPTM5Pskv5dXzOdnNuCj6Y2H/9m2SsObukBwF0z5Qijgu1DsxvADVIKZ4rzrGb4uSEmM6200qjJ/9U98fVM7rvOraakrhcf9gRwuspguJQnSO9cLj6",
              "pubKey": "CAASpgIwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQDLZZcGcbe4urMBVlcHgN0fpBymY+xcr14ewvamG70QZODJ1h9sljlExZ7byLiqRB3SjGbfpZ1FweznwNxWtWpjHkQjTVXeoM4EEgDSNO/Cg7KNlU0EJvgPJXeEPycAZX9qASbVJ6EECQ40VR/7+SuSqsdL1hrmG1phpIju+D64gLyWpw9WEALfzMpH5I/KvdYDW3N4g6zOD2mZNp5y1gHeXINHWzMF596O72/6cxwyiXV1eJ000k1NVnUyrPjXtqWdVLRk5IU1LFpoQoXZU5X1hKj1a2qt/lZfH5eOrF/ramHcwhrYYw1txf8JHXWO/bbNnyemTHAvutZpTNrsWATfAgMBAAE="
            }
            """
        let peerID = try PeerID(fromJSON: str.data(using: .utf8)!)

        let host = try await Application.make(.testing, peerID: peerID)
        host.servers.use(.ws(host: "192.168.1.3", port: 10000))
        host.security.use(.noise)
        host.muxers.use(.mplex)
        host.logger.logLevel = .trace

        print(
            "Swift Libp2p host listening on: \(host.listenAddresses.map { try! $0.encapsulate(proto: .p2p, address: host.peerID.b58String) })"
        )

        let echoedMessages: NIOLockedValueBox<[String]> = .init([])

        try await confirmation(expectedCount: 1) { confirm in
            let suspend = Task { try await Task.sleep(for: .seconds(60)) }

            host.routes.on("echo", "1.0.0") { req -> Response<ByteBuffer> in
                print("/echo/1.0.0 request -> \(req)")
                switch req.event {
                case .ready:
                    return .stayOpen
                case .data(let data):
                    if let str = String(data: Data(data.readableBytesView), encoding: .utf8) {
                        echoedMessages.withLockedValue { $0.append(str) }
                    } else {
                        print("Non UTF8 Message Encountered")
                    }
                    return .respondThenClose(data)
                case .closed:
                    confirm()
                    suspend.cancel()
                    return .close
                case .error:
                    return .close
                }
            }

            try await host.startup()
            #expect(try host.listenAddresses.first == Multiaddr("/ip4/192.168.1.3/tcp/10000/ws"))

            try await suspend.value
        }

        #expect(echoedMessages.withLockedValue({ $0 }).count == 1)
        #expect(echoedMessages.withLockedValue({ $0 }).first == "hey")

        try await host.asyncShutdown()
    }
}

struct TestHelper {
    static var integrationTestsEnabled: Bool {
        if let b = ProcessInfo.processInfo.environment["PerformIntegrationTests"], b == "true" {
            return true
        }
        return false
    }
}

extension Trait where Self == ConditionTrait {
    /// This test is only available when the `PerformIntegrationTests` environment variable is set to `true`
    public static var externalIntegrationTestsEnabled: Self {
        enabled(
            if: TestHelper.integrationTestsEnabled,
            "This test is only available when the `PerformIntegrationTests` environment variable is set to `true`"
        )
    }
}
