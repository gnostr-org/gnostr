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

extension Application {
    public var tcp: TCP_Embedded {
        .init(application: self)
    }

    public struct TCP_Embedded {
        public let application: Application
    }
}

extension Application.Transports.Provider {
    public static var tcp: Self {
        .init { app in
            app.transports.use(key: TCP_Embedded.key) {
                TCP_Embedded(application: $0, protocols: [], proxy: false, uuid: UUID())
            }
        }
    }
}
