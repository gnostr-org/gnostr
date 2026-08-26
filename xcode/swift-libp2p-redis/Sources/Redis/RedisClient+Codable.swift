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

import AsyncKit
import Foundation
import NIOCore
import RediStack

extension RedisClient {
    /// Gets the provided key as a decodable type.
    public func get<D>(
        _ key: RedisKey,
        asJSON type: D.Type,
        jsonDecoder: JSONDecoder = JSONDecoder()
    ) -> EventLoopFuture<D?>
    where D: Decodable {
        self.get(key, as: Data.self).flatMapThrowing { data in
            try data.flatMap { try jsonDecoder.decode(D.self, from: $0) }
        }
    }

    /// Sets key to an encodable item.
    public func set<E>(
        _ key: RedisKey,
        toJSON entity: E,
        jsonEncoder: JSONEncoder = JSONEncoder()
    ) -> EventLoopFuture<Void>
    where E: Encodable {
        do {
            return try self.set(key, to: jsonEncoder.encode(entity))
        } catch {
            return self.eventLoop.makeFailedFuture(error)
        }
    }

    /// Sets key to an encodable item with an expiration time.
    public func setex<E>(
        _ key: RedisKey,
        toJSON entity: E,
        expirationInSeconds expiration: Int,
        jsonEncoder: JSONEncoder = JSONEncoder()
    ) -> EventLoopFuture<Void>
    where E: Encodable {
        do {
            return try self.setex(
                key,
                to: jsonEncoder.encode(entity),
                expirationInSeconds: expiration
            )
        } catch {
            return self.eventLoop.makeFailedFuture(error)
        }
    }
}
