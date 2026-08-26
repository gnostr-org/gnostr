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

extension Application.Resolvers.Provider {
    public static var dnsaddr: Self {
        .init { app in
            app.resolvers.use {
                let dnsAddr = DNSAddr(application: $0)
                app.lifecycle.use(dnsAddr)
                return dnsAddr
            }
        }
    }

    public static func dnsaddr(host: SocketAddress) -> Self {
        .init { app in
            app.resolvers.use {
                let dnsAddr = DNSAddr(application: $0, host: host)
                app.lifecycle.use(dnsAddr)
                return dnsAddr
            }
        }
    }
}
