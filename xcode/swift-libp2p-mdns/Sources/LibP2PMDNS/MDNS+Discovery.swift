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

/// Discovery Conformance
extension MulticastPeerDiscovery {
    public func advertise(service protocol: String, options: Options? = nil) -> EventLoopFuture<TimeAmount> {
        self.registerService(name: `protocol`).transform(to: .seconds(120))
    }

    public func findPeers(
        supportingService protocol: String,
        options: Options? = nil
    ) -> EventLoopFuture<DiscoverdPeers> {
        self.queryForService(`protocol`).map { socketAddress in
            socketAddress.compactMap { try? $0.toMultiaddr() }.compactMap {
                guard let pid = try? $0.getPeerID() else { return nil }
                return PeerInfo(peer: pid, addresses: [$0])
            }
        }.map { peers in
            DiscoverdPeers(peers: peers, cookie: nil)
        }
    }
}
