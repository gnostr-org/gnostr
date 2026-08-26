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

import Foundation
@preconcurrency import RediStack

extension RedisClient {
    /// Gets the provided key as a decodable type.
    public func get<D>(
        _ key: RedisKey,
        asJSON type: D.Type,
        jsonDecoder: JSONDecoder = JSONDecoder()
    ) async throws -> D?
    where D: Decodable {
        let data = try await self.get(key, as: Data.self).get()
        return try data.flatMap { try jsonDecoder.decode(D.self, from: $0) }
    }

    /// Sets key to an encodable item.
    public func set<E>(
        _ key: RedisKey,
        toJSON entity: E,
        jsonEncoder: JSONEncoder = JSONEncoder()
    ) async throws
    where E: Encodable {
        try await self.set(key, to: jsonEncoder.encode(entity)).get()
    }

    /// Sets key to an encodable item with an expiration time.
    public func setex<E>(
        _ key: RedisKey,
        toJSON entity: E,
        expirationInSeconds expiration: Int,
        jsonEncoder: JSONEncoder = JSONEncoder()
    ) async throws
    where E: Encodable {
        try await self.setex(
            key,
            to: jsonEncoder.encode(entity),
            expirationInSeconds: expiration
        ).get()
    }

    /// Removes the specified keys. A key is ignored if it does not exist.
    ///
    /// [https://redis.io/commands/del](https://redis.io/commands/del)
    /// - Parameter keys: A list of keys to delete from the database.
    /// - Returns: The number of keys deleted from the database.
    public func delete(
        _ keys: [RedisKey]
    ) async throws -> Int {
        try await self.delete(keys).get()
    }

    /// Removes the specified keys. A key is ignored if it does not exist.
    ///
    /// [https://redis.io/commands/del](https://redis.io/commands/del)
    /// - Parameter keys: A list of keys to delete from the database.
    /// - Returns: The number of keys deleted from the database.
    public func delete(
        _ keys: RedisKey...
    ) async throws -> Int {
        try await self.delete(keys).get()
    }

    /// Checks the existence of the provided keys in the database.
    ///
    /// [https://redis.io/commands/exists](https://redis.io/commands/exists)
    /// - Parameter keys: A list of keys whose existence will be checked for in the database.
    /// - Returns: The number of provided keys which exist in the database.
    public func exists(_ keys: RedisKey...) async throws -> Int {
        try await self.exists(keys).get()
    }

    /// Checks the existence of the provided keys in the database.
    ///
    /// [https://redis.io/commands/exists](https://redis.io/commands/exists)
    /// - Parameter keys: A list of keys whose existence will be checked for in the database.
    /// - Returns: The number of provided keys which exist in the database.
    public func exists(_ keys: [RedisKey]) async throws -> Int {
        try await self.exists(keys).get()
    }

    /// Sets a timeout on key. After the timeout has expired, the key will automatically be deleted.
    /// - Note: A key with an associated timeout is often said to be "volatile" in Redis terminology.
    ///
    /// [https://redis.io/commands/expire](https://redis.io/commands/expire)
    /// - Parameters:
    ///     - key: The key to set the expiration on.
    ///     - timeout: The time from now the key will expire at.
    /// - Returns: `true` if the expiration was set.
    public func expire(
        _ key: RedisKey,
        after timeout: TimeAmount
    ) async throws
        -> Bool
    {
        try await self.expire(key, after: timeout).get()
    }

    /// Returns the remaining time-to-live (in seconds) of the provided key.
    ///
    /// [https://redis.io/commands/ttl](https://redis.io/commands/ttl)
    /// - Parameter key: The key to check the time-to-live on.
    /// - Returns: The number of seconds before the given key will expire.
    public func ttl(_ key: RedisKey) async throws -> RedisKey.Lifetime {
        try await self.ttl(key).get()
    }

    /// Returns the remaining time-to-live (in milliseconds) of the provided key.
    ///
    /// [https://redis.io/commands/pttl](https://redis.io/commands/pttl)
    /// - Parameter key: The key to check the time-to-live on.
    /// - Returns: The number of milliseconds before the given key will expire.
    public func pttl(_ key: RedisKey) async throws -> RedisKey.Lifetime {
        try await self.pttl(key).get()
    }
}
