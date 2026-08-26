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
import NIOConcurrencyHelpers
import NIOHTTP1
import NIOWebSocket

// Install our WS Tranport on the LibP2P Application
public struct WebSocket: Transport {
    public static let key: String = "websockets"

    let application: Application

    public var protocols: [LibP2PProtocol] {
        get { _protocols.withLockedValue { $0 } }
    }
    public let _protocols: NIOLockedValueBox<[LibP2PProtocol]>

    public var proxy: Bool {
        get { _proxy.withLockedValue { $0 } }
    }
    public let _proxy: NIOLockedValueBox<Bool>

    public let uuid: UUID

    init(application: Application, protocols: [LibP2PProtocol], proxy: Bool, uuid: UUID) {
        self.application = application
        self._protocols = .init(protocols)
        self._proxy = .init(proxy)
        self.uuid = uuid
    }

    public var sharedClient: ClientBootstrap {
        let lock = self.application.locks.lock(for: Key.self)
        lock.lock()
        defer { lock.unlock() }
        if let existing = self.application.storage[Key.self] {
            return existing
        }
        let new = ClientBootstrap(group: self.application.eventLoopGroup)
            // Enable SO_REUSEADDR.
            .channelOption(ChannelOptions.socketOption(.so_reuseaddr), value: 1)
            .channelInitializer { channel in
                // Do we install the upgrader here or do we let the Connection install the handlers???
                //channel.pipeline.addHandlers(upgrader.channelHandlers(mode: .initiator)) // The MSS Handler itself needs to have access to the Connection Delegate
                channel.eventLoop.makeSucceededVoidFuture()
            }

        self.application.storage.set(Key.self, to: new)

        return new
    }
    //    public var sharedClient: TCPClient {
    //        let lock = self.application.locks.lock(for: Key.self)
    //        lock.lock()
    //        defer { lock.unlock() }
    //        if let existing = self.application.storage[Key.self] {
    //            return existing
    //        }
    //        let new = TCPClient(
    //
    //        self.application.storage.set(Key.self, to: new)
    //
    //        return new
    //    }

    //    public var configuration: TCPClient.Configuration {
    //        get {
    //            self.application.storage[ConfigurationKey.self] ?? .init()
    //        }
    //        nonmutating set {
    //            if self.application.storage.contains(Key.self) {
    //                self.application.logger.warning("Cannot modify client configuration after client has been used.")
    //            } else {
    //                self.application.storage[ConfigurationKey.self] = newValue
    //            }
    //        }
    //    }

    /// For each new Dial, we connect to the desired multiaddr, then install our handlers
    /// 1) Http 1.1 initial request handler (with client upgrader config)
    /// 2) WebSocket Handler
    /// 3) We then let the connection proceed to initialize itself on the channel / pipeline
    public func dial(address: Multiaddr) -> EventLoopFuture<Connection> {
        guard let tcp = address.tcpAddress else {
            self.application.logger.warning("Invalid Mutliaddr. WS can't dial \(address)")
            return self.application.eventLoopGroup.any().makeFailedFuture(Errors.invalidMultiaddr)
        }
        //guard let requestKey = try? LibP2PCrypto.randomBytes(length: 16).asString(base: .base64Pad) else {
        //    return self.application.eventLoopGroup.any().makeFailedFuture(Errors.failedToGenerateWSMaskingKey)
        //}

        return sharedClient.connect(host: tcp.address, port: tcp.port).flatMap {
            channel -> EventLoopFuture<Connection> in

            self.application.logger.trace("Instantiating new Connection")
            let conn = application.connectionManager.generateConnection(
                channel: channel,
                direction: .outbound,
                remoteAddress: address,
                expectedRemotePeer: try? address.getPeerID()
            )

            /// The connection installs the necessary channel handlers here
            //self.application.logger.trace("Asking BasicConnectionLight to instantiate new outbound channel")

            let httpHandler = HTTPInitialRequestHandler(target: address, logger: conn.logger)

            /// - Note: The default requestKey NIO generates seems to work now!
            /// swift-nio recommends 28 char requestKey, Go supports 28 char keys
            /// JS doesn't support 28 char keys, it only seems to support 24 char keys
            /// requestKey: "dGhlIHNhbXBsZSBub25jZQ==",
            /// requestKey: "OfS0wDaT5NoxF2gqm7Zj2YtetzM=",
            let websocketUpgrader = NIOWebSocketClientUpgrader(
                //requestKey: requestKey,
                upgradePipelineHandler: { (channel: Channel, head: HTTPResponseHead) in
                    print(head)
                    let wsh = WebSocketDuplexHandler(mode: .initiator, logger: conn.logger)
                    return channel.pipeline.addHandler(BackPressureHandler(), position: .first).flatMap {
                        channel.pipeline.addHandler(wsh, position: .last).flatMap {
                            /// Add the connection to our connectionManager
                            //return self.application.connections.addConnection(conn, on: nil).flatMap {
                            /// Tell our connection to initialize the channel
                            conn.initializeChannel().map {
                                wsh.fireChannelActiveIfNecessary()
                            }
                            //}
                        }
                    }
                }
            )

            /// Create the Upgrader Configuration
            let config: NIOHTTPClientUpgradeConfiguration = (
                upgraders: [websocketUpgrader],
                completionHandler: { _ in
                    channel.pipeline.removeHandler(httpHandler, promise: nil)
                }
            )

            /// Instantiate the connection with the http handlers and the WS upgrader
            /// /// Add the connection to our connectionManager
            return channel.pipeline.addHTTPClientHandlers(withClientUpgrade: config).flatMap {
                channel.pipeline.addHandler(httpHandler).flatMap {
                    self.application.connections.addConnection(conn, on: nil).flatMap {
                        // We normally call Connection.initializeChannel() here, but we wait to call this until the WebSocket upgrade is completed (above)...
                        channel.eventLoop.makeSucceededFuture(conn)
                    }
                }
            }
        }
    }

    /// Parses the Multiaddr and determines if it's a valid WebSocket endpoint that can be dialed
    public func canDial(address: Multiaddr) -> Bool {
        //address.tcpAddress != nil && !address.protocols().contains(.ws)
        print("WS Can Dial -> \(address)")
        guard let tcp = address.tcpAddress else { return false }
        // Remove once we can dial ipv6 addresses
        guard tcp.ip4 else { return false }
        // We should only dial WS multiaddr // || ma.protocols().contains(.wss))
        guard address.protocols().contains(.ws) else { return false }
        return true
    }

    public func listen(address: Multiaddr) -> EventLoopFuture<Listener> {
        application.eventLoopGroup.any().makeFailedFuture(Errors.notYetImplemeted)
    }

    struct Key: StorageKey, LockKey {
        typealias Value = ClientBootstrap
    }

    //    struct ConfigurationKey: StorageKey {
    //        typealias Value = TCPClient.Configuration
    //    }

    public enum Errors: Error {
        case notYetImplemeted
        case invalidMultiaddr
        case failedToGenerateWSMaskingKey
    }
}

extension Application.Transports.Provider {
    public static var ws: Self {
        .init { app in
            app.transports.use(key: WebSocket.key) {
                WebSocket(application: $0, protocols: [], proxy: false, uuid: UUID())
            }
        }
    }

    //    public static func wss(_ tlsConfig:Any) -> Self {
    //        .init { app in
    //            app.transports.use(key: WebSockets.key) {
    //                WebSockets(application: $0, protocols:[], proxy: false, uuid:UUID())
    //            }
    //        }
    //    }
}

extension WebSocketMaskingKey {
    static var random4ByteKey: Self {
        .init(
            arrayLiteral:
                UInt8.random(in: UInt8.min...UInt8.max),
            UInt8.random(in: UInt8.min...UInt8.max),
            UInt8.random(in: UInt8.min...UInt8.max),
            UInt8.random(in: UInt8.min...UInt8.max)
        )
    }
}
