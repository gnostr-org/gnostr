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

/// MessageCache creates a sliding window cache that remembers messages for as
/// long as `history` slots.
///
/// When queried for messages to advertise, the cache only returns messages in
/// the last `gossip` slots.
///
/// The `gossip` parameter must be smaller or equal to `history`, or this
/// function will throw.
///
/// The slack between `gossip` and `history` accounts for the reaction time
/// between when a message is advertised via IHAVE gossip, and the peer pulls it
/// via an IWANT command.
final class MessageCache: MessageStateProtocol, @unchecked Sendable {
    typealias MessageID = Data
    typealias Message = (topic: String, data: PubSubMessage)
    typealias HistoryWindow = [MessageID: Message]

    /// The eventloop that this Message Cache is constrained to
    internal let eventLoop: EventLoop
    /// The Cache
    var windows: [HistoryWindow]
    /// Our Logger
    var logger: Logger
    /// Our State
    var state: ServiceLifecycleState

    /// The number of history windows to keep
    /// - Alias: mcache_len
    private let cacheLength: Int

    /// The number of windows to examine when sending gossip
    /// - Alias: mcache_gossip
    private let gossipLength: Int

    required init(eventLoop: EventLoop, historyWindows: Int = 3, gossipWindows: Int = 2) {
        precondition(
            historyWindows > gossipWindows,
            "Invalid parameters for message cache. GossipWindows [\(gossipWindows)] cannot be larger than historyWindows [\(historyWindows)]"
        )
        print("PubSub::MessageChache Instantiated...")
        self.windows = []
        self.eventLoop = eventLoop
        self.cacheLength = historyWindows
        self.gossipLength = gossipWindows
        self.logger = Logger(label: "com.swift.libp2p.pubsub.mcache[\(UUID().uuidString.prefix(5))]")
        self.logger.logLevel = .info  //LOG_LEVEL
        self.state = .stopped

        /// Initialize our cache windows
        let _ = self.shift()
    }

    func start() throws {
        guard self.state == .stopped else { throw BasePubSub.Errors.alreadyRunning }
        self.logger.info("Starting")

        // Do stuff here, maybe re init our caches??

        self.state = .started
    }

    func stop() throws {
        guard self.state == .started || self.state == .starting else { throw BasePubSub.Errors.alreadyStopped }
        if self.state == .stopping {
            self.logger.info("Force Quiting!")
        }
        self.logger.info("Stopping")

        // Do stuff here, maybe clear our caches??

        self.state = .stopped
    }

    /// Adds a message to the current window and the cache
    func put(messageID: MessageID, message: Message, on loop: EventLoop? = nil) -> EventLoopFuture<Bool> {
        eventLoop.submit { () -> Bool in
            /// blindly overwrites any existing entries with the specified messageID
            if self.windows.isEmpty { self.windows[0] = HistoryWindow() }
            if self.windows[0][messageID] == nil {
                self.windows[0][messageID] = message
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
            if self.windows.isEmpty { self.windows[0] = HistoryWindow() }
            var added: [Data: PubSubMessage] = [:]
            for message in messages {
                guard let topic = message.value.topicIds.first else { continue }
                if self.windows[0][message.key] == nil {
                    self.windows[0][message.key] = (topic, message.value)
                    added[message.key] = message.value
                }
            }
            return added
        }.hop(to: loop ?? eventLoop)
    }

    private func _get(messageID: MessageID) -> Message? {
        var msg: Message? = nil
        for window in self.windows {
            if let message = window[messageID] {
                msg = message
                break
            }
        }
        return msg
    }

    private func _exists(messageID: MessageID, fullOnly: Bool = false) -> Bool {
        var exists: Bool = false

        for window in self.windows.prefix(fullOnly ? cacheLength : windows.count) {
            if window[messageID] != nil {
                exists = true
                break
            }
        }

        return exists
    }

    /// Retrieves a message from the cache by its ID, if it is still present.
    func get(messageID: MessageID, on loop: EventLoop? = nil) -> EventLoopFuture<Message?> {
        eventLoop.submit { () -> Message? in
            self._get(messageID: messageID)
        }.hop(to: loop ?? eventLoop)
    }

    /// Retrieves a message from the cache by its ID, if it is still present.
    func get(messageIDs: Set<MessageID>, on loop: EventLoop? = nil) -> EventLoopFuture<[Message]> {
        eventLoop.submit { () -> [Message] in
            messageIDs.compactMap { self._get(messageID: $0) }
        }.hop(to: loop ?? eventLoop)
    }

    func exists(messageID: MessageID, on loop: EventLoop? = nil) -> EventLoopFuture<Bool> {
        eventLoop.submit { () -> Bool in
            self._exists(messageID: messageID)
        }.hop(to: loop ?? eventLoop)
    }

    /// Retrieves the message IDs for messages in the most recent history windows, scoped to a given topic.
    /// - Note: The number of windows to examine is controlled by the gossipLength parameter
    func getGossipIDs(topic: String, on loop: EventLoop? = nil) -> EventLoopFuture<[MessageID]> {
        eventLoop.submit { () -> [MessageID] in
            var ids: [MessageID] = []
            for (idx, window) in self.windows.enumerated() {
                guard idx < self.gossipLength else { break }

                ids.append(
                    contentsOf: window.filter({ message in
                        message.value.topic == topic
                    }).map { $0.key }
                )
            }

            return ids
        }.hop(to: loop ?? eventLoop)
    }

    /// BasePubSub Calls this method every X (usually 1) seconds, we take the opportunity to shift our Message Cache
    var runningHeartbeatCounter: UInt64 = 0
    func heartbeat() -> EventLoopFuture<Void> {
        self.eventLoop.submit {
            /// Every 30 seconds we shift our message store
            if self.runningHeartbeatCounter >= 2 {
                self.runningHeartbeatCounter = 0
                self.logger.trace("Shifting Message Cache Window")
                self.shift()
            }

            /// Increment our heartbeat counter...
            self.runningHeartbeatCounter += 1
        }
    }

    /// Shifts the current window, discarding messages older than the history length of the cache (mcache_len)
    /// - Warning: Ensure that this method is only called once per heartbeat interval (otherwise we'll drop message before they expire)
    @discardableResult
    func shift(on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        eventLoop.submit { () -> Void in
            /// Remove all windows that are older than our cacheLength
            while self.windows.count >= self.cacheLength {
                self.windows.removeLast()
            }
            /// Insert a new window at index 0
            self.windows.insert(HistoryWindow(), at: 0)
        }.hop(to: loop ?? eventLoop)
    }

    /// Given an array of message ids, this method will filter them using the specified filter and return the ID's that satisfy the filter...
    /// Example: .known -> returns only those message id's that we have in our cache
    /// Example: .unknown -> returns only those message id's that we haven't seen / encountered lately
    /// Example: .full -> returns only those message id's for which we have the full message contents
    func filter(
        ids: Set<Data>,
        returningOnly filter: PubSub.MessageState.FilterType,
        on loop: EventLoop? = nil
    ) -> EventLoopFuture<[Data]> {
        eventLoop.submit { () -> [Data] in
            switch filter {
            case .known:
                return ids.filter { id in
                    self._exists(messageID: id)
                }
            case .unknown:
                return ids.filter { id in
                    !self._exists(messageID: id)
                }
            case .full:
                return ids.filter { id in
                    self._exists(messageID: id, fullOnly: true)
                }
            }
        }.hop(to: loop ?? eventLoop)
    }
}
