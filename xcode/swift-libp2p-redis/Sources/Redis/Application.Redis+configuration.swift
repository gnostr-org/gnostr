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
//
//  Created by Vapor
//  Modified by swift-libp2p
//

import LibP2P

extension Application.Redis {
    /// The Redis configuration to use to communicate with a Redis instance.
    ///
    /// See `Application.Redis.id`
    public var configuration: RedisConfiguration? {
        get {
            self.application.redisStorage.configuration(for: self.id)
        }
        nonmutating set {
            guard let newConfig = newValue else {
                fatalError("Modifying configuration is not supported")
            }
            self.application.redisStorage.use(newConfig, as: self.id)
        }
    }
}
