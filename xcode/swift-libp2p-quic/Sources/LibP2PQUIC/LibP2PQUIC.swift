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
import Multiaddr
import Multicodec
import NIOCore

public struct QUIC: Transport, @unchecked Sendable {
    public static let key: String = "quic"

    public let application: Application
    public var protocols: [LibP2PProtocol]
    public var proxy: Bool

    public init(application: Application, protocols: [LibP2PProtocol] = [], proxy: Bool = false) {
        self.application = application
        self.protocols = protocols
        self.proxy = proxy
    }

    public func dial(address: Multiaddr) -> EventLoopFuture<Connection> {
        self.application.eventLoopGroup.any().makeFailedFuture(Errors.notImplemented)
    }

    public func canDial(address: Multiaddr) -> Bool {
        let protocols = address.protocols()
        guard protocols.contains(.udp) else { return false }
        return protocols.contains(.quic) || protocols.contains(.quic_v1)
    }

    public func listen(address: Multiaddr) -> EventLoopFuture<Listener> {
        self.application.eventLoopGroup.any().makeFailedFuture(Errors.notImplemented)
    }

    public enum Errors: Error {
        case notImplemented
    }
}

extension Application.Transports.Provider {
    public static var quic: Self {
        .init { app in
            app.transports.use(key: QUIC.key) {
                QUIC(application: $0)
            }
        }
    }
}
