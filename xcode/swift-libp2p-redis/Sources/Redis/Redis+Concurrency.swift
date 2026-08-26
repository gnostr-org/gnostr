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
import LibP2P
import NIOCore
@preconcurrency import RediStack

extension Application.Redis {
    public func send(
        command: String,
        with arguments: [RESPValue] = []
    ) async throws
        -> RESPValue
    {
        try await self.application.redis(self.id)
            .pool(for: self.eventLoop)
            .logging(to: self.application.logger)
            .send(command: command, with: arguments).get()
    }

    public func subscribe(
        to channels: [RedisChannelName],
        messageReceiver receiver: @escaping RedisSubscriptionMessageReceiver,
        onSubscribe subscribeHandler: RedisSubscriptionChangeHandler?,
        onUnsubscribe unsubscribeHandler: RedisSubscriptionChangeHandler?
    ) async throws {
        try await self.application.redis(self.id)
            .pubsubClient
            .logging(to: self.application.logger)
            .subscribe(
                to: channels,
                messageReceiver: receiver,
                onSubscribe: subscribeHandler,
                onUnsubscribe: unsubscribeHandler
            )
            .get()
    }

    public func unsubscribe(from channels: [RedisChannelName]) async throws {
        try await self.application.redis(self.id)
            .pubsubClient
            .logging(to: self.application.logger)
            .unsubscribe(from: channels)
            .get()
    }

    public func psubscribe(
        to patterns: [String],
        messageReceiver receiver: @escaping RedisSubscriptionMessageReceiver,
        onSubscribe subscribeHandler: RedisSubscriptionChangeHandler?,
        onUnsubscribe unsubscribeHandler: RedisSubscriptionChangeHandler?
    ) async throws {
        try await self.application.redis(self.id)
            .pubsubClient
            .logging(to: self.application.logger)
            .psubscribe(
                to: patterns,
                messageReceiver: receiver,
                onSubscribe: subscribeHandler,
                onUnsubscribe: unsubscribeHandler
            )
            .get()
    }

    public func punsubscribe(from patterns: [String]) async throws {
        try await self.application.redis(self.id)
            .pubsubClient
            .logging(to: self.application.logger)
            .punsubscribe(from: patterns)
            .get()
    }
}

extension Request.Redis {
    public func send(
        command: String,
        with arguments: [RESPValue] = []
    ) async throws
        -> RESPValue
    {
        try await self.request.application.redis(self.id)
            .pool(for: self.eventLoop)
            .logging(to: self.request.logger)
            .send(command: command, with: arguments)
            .get()
    }

    public func subscribe(
        to channels: [RedisChannelName],
        messageReceiver receiver: @escaping RedisSubscriptionMessageReceiver,
        onSubscribe subscribeHandler: RedisSubscriptionChangeHandler?,
        onUnsubscribe unsubscribeHandler: RedisSubscriptionChangeHandler?
    ) async throws {
        try await self.request.application.redis(self.id)
            .pubsubClient
            .logging(to: self.request.logger)
            .subscribe(
                to: channels,
                messageReceiver: receiver,
                onSubscribe: subscribeHandler,
                onUnsubscribe: unsubscribeHandler
            )
            .get()
    }

    public func unsubscribe(from channels: [RedisChannelName]) async throws {
        try await self.request.application.redis(self.id)
            .pubsubClient
            .logging(to: self.request.logger)
            .unsubscribe(from: channels)
            .get()
    }

    public func psubscribe(
        to patterns: [String],
        messageReceiver receiver: @escaping RedisSubscriptionMessageReceiver,
        onSubscribe subscribeHandler: RedisSubscriptionChangeHandler?,
        onUnsubscribe unsubscribeHandler: RedisSubscriptionChangeHandler?
    ) async throws {
        try await self.request.application.redis(self.id)
            .pubsubClient
            .logging(to: self.request.logger)
            .psubscribe(
                to: patterns,
                messageReceiver: receiver,
                onSubscribe: subscribeHandler,
                onUnsubscribe: unsubscribeHandler
            )
            .get()
    }

    public func punsubscribe(from patterns: [String]) async throws {
        try await self.request.application.redis(self.id)
            .pubsubClient
            .logging(to: self.request.logger)
            .punsubscribe(from: patterns)
            .get()
    }
}
