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

import CoreFoundation
import LibP2P

/// BasicMessageCache - A simple message cache that uses time based message expiration
///
/// Creates and maintains a dictionary with messages keyed by their ID for quick access
/// Alongside the deictionary it also keeps an ordered (newest -> oldest) list of message ids along with their arivale time for message expiration / deletion
/// Every heartbeat we trim the message cache of all expired messages
final class BasicMessageCache: MessageStateProtocol, @unchecked Sendable {
    typealias MessageID = Data
    typealias Message = (topic: String, data: PubSubMessage)

    /// The eventloop that this Message Cache is constrained to
    internal let eventLoop: EventLoop
    /// The Cache
    var messages: [MessageID: PubSubMessage]
    var expirations: [(MessageID, CFAbsoluteTime)]
    /// Our Logger
    var logger: Logger
    /// Our State
    var state: ServiceLifecycleState

    /// The duration to keep messages for
    private let ttl: Double

    required init(eventLoop: EventLoop, timeToLiveInSeconds ttl: Double = 120) {
        self.eventLoop = eventLoop
        self.ttl = ttl
        self.state = .stopped
        self.messages = [:]
        self.expirations = []

        self.logger = Logger(label: "com.swift.libp2p.pubsub.messagecache[\(UUID().uuidString.prefix(5))]")
        self.logger.logLevel = .trace
        //self.logger[metadataKey: "MessageCache"] = .string("\(UUID().uuidString.prefix(5))")

        self.logger.debug("Instantiated")
    }

    func start() throws {
        guard self.state == .stopped else { throw BasePubSub.Errors.alreadyRunning }
        self.logger.trace("Starting")

        // Do stuff here, maybe re init our caches??

        self.state = .started
    }

    func stop() throws {
        guard self.state == .started || self.state == .starting else { throw BasePubSub.Errors.alreadyStopped }
        if self.state == .stopping {
            self.logger.trace("Force Quiting!")
        }
        self.logger.trace("Stopping")

        // Do stuff here, maybe clear our caches??

        self.state = .stopped
    }

    /// Adds a message to the current window and the cache
    func put(messageID: MessageID, message: Message, on loop: EventLoop? = nil) -> EventLoopFuture<Bool> {
        eventLoop.submit { () -> Bool in
            /// blindly overwrites any existing entries with the specified messageID
            if self.messages[messageID] == nil {
                self.messages[messageID] = message.data
                self.expirations.insert((messageID, CFAbsoluteTimeGetCurrent()), at: 0)
                return true
            } else {
                return false
            }
        }.hop(to: loop ?? eventLoop)
    }

    /// Given a dictionary of messages to store, this method will attempt to add each one and return a dictionary of the added messages.
    func put(messages: [Data: PubSubMessage], on loop: EventLoop? = nil) -> EventLoopFuture<[Data: PubSubMessage]> {
        eventLoop.submit { () -> [Data: PubSubMessage] in
            /// blindly overwrites any existing entries with the specified messageID
            var added: [Data: PubSubMessage] = [:]
            for message in messages {
                //guard let topic = message.value.topicIds.first else { continue }
                if self.messages[message.key] == nil {
                    self.messages[message.key] = message.value
                    added[message.key] = message.value
                }
            }
            return added
        }.hop(to: loop ?? eventLoop)
    }

    /// Retrieves a message from the cache by its ID, if it is still present.
    func get(messageID: MessageID, on loop: EventLoop? = nil) -> EventLoopFuture<Message?> {
        eventLoop.submit { () -> Message? in
            if let msg = self.messages[messageID] {
                return (topic: msg.topicIds.first!, data: msg)
            }
            return nil
        }.hop(to: loop ?? eventLoop)
    }

    func exists(messageID: MessageID, on loop: EventLoop? = nil) -> EventLoopFuture<Bool> {
        eventLoop.submit { () -> Bool in
            self.messages[messageID] != nil
        }.hop(to: loop ?? eventLoop)
    }

    func filter(
        ids: Set<Data>,
        returningOnly filter: PubSub.MessageState.FilterType,
        on loop: EventLoop?
    ) -> EventLoopFuture<[Data]> {
        eventLoop.submit { () -> [Data] in
            switch filter {
            case .known, .full:
                return ids.filter { id in
                    self.messages[id] != nil
                }
            case .unknown:
                return ids.filter { id in
                    self.messages[id] == nil
                }
            }
        }.hop(to: loop ?? eventLoop)
    }

    /// Retrieves the message IDs for messages in the most recent history windows, scoped to a given topic.
    /// - Note: The number of windows to examine is controlled by the gossipLength parameter
    func getGossipIDs(topic: String, on loop: EventLoop? = nil) -> EventLoopFuture<[MessageID]> {
        eventLoop.submit { () -> [MessageID] in
            self.messages.map { $0.key }
        }.hop(to: loop ?? eventLoop)
    }

    func heartbeat() -> EventLoopFuture<Void> {
        eventLoop.submit {
            var deleted = 0
            let expired = CFAbsoluteTimeGetCurrent() - self.ttl
            while let oldestMessage = self.expirations.last, oldestMessage.1 < expired {
                if let msgToDelete = self.expirations.popLast() {
                    self.messages.removeValue(forKey: msgToDelete.0)
                    deleted += 1
                }
            }
            if deleted > 0 {
                self.logger.trace("Deleted \(deleted) expired messages. Messages in cache \(self.messages.count)")
            }
        }
    }
}

/// SeenCache - A cache of recently seen MessageIDs
///
/// - Note: Used for dropping duplicate messages by the BasePubSub implementation
/// - Note: Every heartbeat we should `trim` the cache to keep it's size under control
final class SeenCache: @unchecked Sendable {
    typealias MessageID = Data
    typealias Timestamp = Double

    /// The eventloop that this MessageID Cache is constrained to
    internal let eventLoop: EventLoop

    /// The Cache
    private var cache: [MessageID: Timestamp]

    /// Our Logger
    var logger: Logger

    /// The duration to keep messages for in seconds
    private let ttl: Double

    required init(eventLoop: EventLoop, timeToLiveInSeconds ttl: Double = 120, logger: Logger) {
        self.eventLoop = eventLoop
        self.ttl = ttl
        self.cache = [:]

        self.logger = logger
        self.logger[metadataKey: "SeenCache"] = .string("[\(UUID().uuidString.prefix(5))][TTL:\(self.ttl)]")

        self.logger.debug("Instantiated")
    }

    /// Adds a message to the current window and the cache
    @discardableResult
    func put(messageID: MessageID, on loop: EventLoop? = nil) -> EventLoopFuture<Bool> {
        eventLoop.submit { () -> Bool in
            if self.cache[messageID] == nil {
                self.cache[messageID] = CFAbsoluteTimeGetCurrent()
                return true
            } else {
                return false
            }
        }.hop(to: loop ?? eventLoop)
    }

    @discardableResult
    func put(messageIDs: [MessageID], on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        eventLoop.submit {
            let time = CFAbsoluteTimeGetCurrent()
            for messageID in messageIDs {
                if self.cache[messageID] == nil {
                    self.cache[messageID] = time
                }
            }
        }.hop(to: loop ?? eventLoop)
    }

    func hasSeen(messageID: MessageID, on loop: EventLoop? = nil) -> EventLoopFuture<Bool> {
        eventLoop.submit { () -> Bool in
            self.cache[messageID] == nil ? false : true
        }.hop(to: loop ?? eventLoop)
    }

    func filter(
        ids: Set<Data>,
        returningOnly filter: PubSub.MessageState.FilterType,
        on loop: EventLoop?
    ) -> EventLoopFuture<[Data]> {
        eventLoop.submit { () -> [Data] in
            switch filter {
            case .known, .full:
                return ids.filter { id in
                    self.cache[id] != nil
                }
            case .unknown:
                return ids.filter { id in
                    self.cache[id] == nil
                }
            }
        }.hop(to: loop ?? eventLoop)
    }

    func trim() -> EventLoopFuture<Void> {
        eventLoop.submit {
            var deleted = 0

            /// Trim the seenCache of expired messages
            let expired = CFAbsoluteTimeGetCurrent() - self.ttl
            self.cache = self.cache.compactMapValues { time in
                if time < expired {
                    deleted += 1
                    return nil
                } else {
                    return time
                }
            }

            if deleted > 0 {
                self.logger.trace("Deleted \(deleted) expired messages. Messages in cache \(self.cache.count)")
            }
        }
    }
}
