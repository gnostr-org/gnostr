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
@preconcurrency import RediStack

extension Application.Redis {
    private struct PubSubKey: StorageKey, LockKey {
        typealias Value = [RedisID: any RedisClient & Sendable]
    }

    var pubsubClient: any RedisClient {
        if let existing = self.application.storage[PubSubKey.self]?[self.id] {
            return existing
        } else {
            let lock = self.application.locks.lock(for: PubSubKey.self)
            lock.lock()
            defer { lock.unlock() }

            let pool = self.pool(for: self.eventLoop.next())

            if let existingStorage = self.application.storage[PubSubKey.self] {
                var copy = existingStorage
                copy[self.id] = pool
                self.application.storage.set(PubSubKey.self, to: copy)
            } else {
                self.application.storage.set(
                    PubSubKey.self,
                    to: [self.id: pool]
                )
            }
            return pool
        }
    }
}
