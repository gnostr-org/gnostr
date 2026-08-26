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

extension Application.DiscoveryServices.Provider {
    public static var mdns: Self {
        .init {
            $0.discovery.use { app -> MulticastPeerDiscovery in
                let mdns = MulticastPeerDiscovery(app: app, interfaceAddress: nil)
                app.lifecycle.use(mdns)
                return mdns
            }
        }
    }

    public static func mdns(interfaceAddress: SocketAddress) -> Self {
        .init {
            $0.discovery.use { app -> MulticastPeerDiscovery in
                let mdns = MulticastPeerDiscovery(app: app, interfaceAddress: interfaceAddress)
                app.lifecycle.use(mdns)
                return mdns
            }
        }
    }

    //    private static func defaultInterfaceAddress() throws -> Multiaddr? {
    //        return try System.enumerateDevices().compactMap({ device in
    //            guard device.name == "en0" && device.address != nil else { return nil }
    //            guard let ma = try? device.address?.toMultiaddr() else { return nil }
    //
    //            if let tcp = ma.tcpAddress, tcp.ip4 == true {
    //                return ma
    //            } else {
    //                return nil
    //            }
    //        }).first
    //    }
}
