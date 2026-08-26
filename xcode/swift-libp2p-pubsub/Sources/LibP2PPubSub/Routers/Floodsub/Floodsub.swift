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

/// FloodSub Implementation
///
/// - Note:
/// When we extend the `BasePubSub` class we get access to...
/// - logger
/// - mainLoop (our main event loop that this pubsub instance lives in)
/// - subscription list
/// - publish(msg:)
/// - Local PeerID
/// - nextMessageSequenceNumber
public final class FloodSub: BasePubSub, PubSubCore, LifecycleHandler, @unchecked Sendable {
    public static let multicodec: String = "/floodsub/1.0.0"

    public init(
        group: EventLoopGroup,
        libp2p: Application,
        debugName: String = "Floodsub",
        emitSelf: Bool = false
    ) throws {
        /// Init our PeerState
        /// - TODO: Use a simple array instead...
        let peerState = BasicPeerState(eventLoop: group.next())  //PeeringState(eventLoop: group.next())

        /// Init our Message Cache
        let messageCache = BasicMessageCache(eventLoop: group.next(), timeToLiveInSeconds: 30)

        /// Register our floodsub route handler
        try registerFloodsubRoute(libp2p)

        /// Init super
        try super.init(
            group: group,
            libp2p: libp2p,
            peerState: peerState,
            messageCache: messageCache,
            debugName: debugName,
            multicodecs: [FloodSub.multicodec],
            globalSignaturePolicy: .strictNoSign,
            canRelayMessages: true,
            emitSelf: emitSelf
        )

        //self.logger[metadataKey: "Floodsub"] = .string("\(UUID().uuidString.prefix(5))")
    }

    public func didBoot(_ application: Application) throws {
        try? self.start()
    }

    public func shutdown(_ application: Application) {
        try? self.stop()
    }

    /// We can override methods if we need to, just make sure to call super...
    public override func start() throws {
        guard self.state == .stopped else { throw Errors.alreadyRunning }
        try super.start()
        // Do whatever we need to do...
    }

    /// We can override methods if we need to, just make sure to call super...
    public override func stop() throws {
        guard self.state == .starting || self.state == .started else { throw Errors.alreadyStopped }
        // Do whatever we need to do...
        try super.stop()
    }

    /// We have to override / implement this method so our BasePubSub implementation isn't constrained to a particular RPC PubSub Message Type
    override func decodeRPC(_ data: Data) throws -> RPCMessageCore {
        try RPC(serializedBytes: data)
    }

    /// We have to override / implement this method so our BasePubSub implementation isn't constrained to a particular RPC PubSub Message Type
    override func encodeRPC(_ rpc: RPCMessageCore) throws -> Data {
        try RPC(rpc).serializedData()
    }

    /// The methods that we must implement will be enforced by the compiler when we conform to PubSub
    override internal func processInboundRPC(
        _ rpc: RPCMessageCore,
        from: PeerID,
        request: Request
    ) -> EventLoopFuture<Void> {
        /// Floodsub doesn't have to do any processing on the main RPC message; therefore we just return.
        self.eventLoop.makeSucceededVoidFuture()
    }

    /// Publish arbitrary data, bundled as an RPC message under the specified topic
    override public func publish(topic: String, data: Data, on: EventLoop?) -> EventLoopFuture<Void> {
        self.logger.info("Attempting to publish data as RPC Message")

        var msg = RPC.Message()
        msg.data = data
        msg.from = Data(self.peerID.id)
        msg.seqno = Data(self.nextMessageSequenceNumber())
        msg.topicIds = [topic]

        return self.publish(msg: msg)
    }

    /// Convenience method for publishing an RPC message as bytes
    override public func publish(topic: String, bytes: [UInt8], on: EventLoop?) -> EventLoopFuture<Void> {
        self.publish(topic: topic, data: Data(bytes), on: on)
    }

    /// Convenience method for publishing an RPC message as a ByteBuffer
    override public func publish(topic: String, buffer: ByteBuffer, on: EventLoop?) -> EventLoopFuture<Void> {
        self.publish(topic: topic, data: Data(buffer.readableBytesView), on: on)
    }

    /// Attempts to subscribe to the specified topic
    func subscribe(topic: Topic) throws -> PubSub.SubscriptionHandler {
        let defaultConfig = LibP2PCore.PubSub.SubscriptionConfig(
            topic: topic,
            signaturePolicy: .strictNoSign,
            validator: .acceptAll,
            messageIDFunc: .concatFromAndSequenceFields
        )
        return try self.subscribe(defaultConfig)
    }

    /// Attempts to subscribe to the specified topic
    public func subscribe(_ config: PubSub.SubscriptionConfig) throws -> PubSub.SubscriptionHandler {
        /// Ensure the Topic we're subscribing to is valid...
        guard !config.topic.isEmpty, config.topic != "" else { throw Errors.invalidTopic }

        self.logger.info("Subscribing to topic: \(config.topic)")

        /// Init Subscription handler
        let subHandler = PubSub.SubscriptionHandler(pubSub: self, topic: config.topic)
        self.subscriptions[config.topic] = subHandler

        /// Let the base/parent PubSub implementation know of the subscription...
        let _ = self.subscribe(config, on: nil)

        /// return the subscription handler
        return subHandler
    }

    //    public override func unsubscribe(topic:Topic, on loop:EventLoop? = nil) -> EventLoopFuture<Void> {
    //
    //        return super.unsubscribe(topic: topic, on: loop)
    //    }

    private func publish(message: RPC.Message, to: Topic) -> EventLoopFuture<Void> {
        self.logger.info("TODO::Publish RPC PubSub Message")
        return self.eventLoop.makeSucceededVoidFuture()
    }

    /// This will only be called for new (unseen) messages that have passed signature policies and the installed validators
    // func processInboundMessage(PubSubMessage) { }
    /// This will be called whenever we receive a new subscription message from a remote peer
    // func processNewSubscriptions(subs, fromPeer:PeerID) { }

    override internal func processInboundMessages(
        _ messages: [PubSubMessage],
        from: PeerID,
        request: Request
    ) -> EventLoopFuture<Void> {
        messages.map {
            self.processInboundMessage($0, from: from, request: request)
        }.flatten(on: request.eventLoop)
    }

    override internal func processInboundMessage(
        _ message: PubSubMessage,
        from: PeerID,
        request: Request
    ) -> EventLoopFuture<Void> {

        guard let message = message as? RPC.Message else {
            self.logger.error("Floodsub was passed a PubSubMessage that wasn't an RPC.Message")
            return self.eventLoop.makeSucceededVoidFuture()
        }

        /// The message has already been vetted
        /// Forward the message onto any other subscribers to this topic (excluding the sender)
        return self.peerState.peersSubscribedTo(topic: message.topicIds.first!, on: nil).flatMap {
            subscribers -> EventLoopFuture<Void> in

            guard subscribers.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }

            self.logger.info(
                "Checking \(subscribers.count) `\(message.topicIds.first!)` subscribers for message propogation"
            )

            var forwardedRPC = RPC()
            forwardedRPC.msgs = [message]
            let payload = try! forwardedRPC.serializedData()

            return subscribers.map { peerStreams in
                guard peerStreams.id != from else {
                    self.logger.info("Skipping OP")
                    return self.eventLoop.makeSucceededVoidFuture()
                }
                self.logger.info("Forwarding message to subscriber \(peerStreams.id)")

                try? peerStreams.write(putUVarInt(UInt64(payload.count)) + payload)
                return self.eventLoop.makeSucceededVoidFuture()
            }.flatten(on: self.eventLoop)
        }
    }

    //    internal func processInboundMessage(_ msg:Data, from stream: LibP2P.Stream, request: Request) -> EventLoopFuture<Data?> {
    //        /// This should only ever be an RPC message.
    //        guard let rpc = try? RPC(serializedData: msg) else {
    //            self.logger.warning("Failed to decode RPC PubSub Message")
    //            self.logger.info("UTF8: \(String(data: Data(request.payload.readableBytesView), encoding: .utf8) ?? "Not UTF8")")
    //            self.logger.info("Hex: \(Data(request.payload.readableBytesView).asString(base: .base16))")
    //            return request.eventLoop.makeSucceededFuture(nil)
    //        }
    //
    //        guard let remotePeer = request.remotePeer else {
    //        //guard let remotePeer = stream.connection?.remotePeer else {
    //            self.logger.warning("Failed to determine message originator (RemotePeer)")
    //            return request.eventLoop.makeSucceededFuture(nil)
    //        }
    //
    //        /// Handle the RPC Control Messages (for Floodsub this is only just a list of subscription changes)
    //        if rpc.subscriptions.count > 0 {
    //            var subs:[String:Bool] = [:]
    //            rpc.subscriptions.forEach {
    //                subs[$0.topicID] = $0.subscribe
    //            }
    //            self.logger.info("\(remotePeer)::Subscriptions: \(subs)")
    //
    //            /// - TODO: Event sub, possibly remove later...
    //            _eventHandler?(.subscriptionChange(remotePeer, subs))
    //
    //            let _ = self.peerState.update(subscriptions: subs, for: remotePeer)
    //
    //            /// Notify our subscription handlers of any relevant peers
    //            for sub in subs {
    //                if sub.1 == true, let handler = self.subscriptions[sub.key] {
    //                    self.logger.info("Notifying `\(sub.key)` subscription handler of new subscriber/peer")
    //                    let _ = handler.on?(.newPeer(remotePeer))
    //                }
    //                /// - TODO: Should we alert our subscription handler when a peer unsubscribes from a topic?
    //            }
    //        }
    //
    //        /// Handle the published messages
    //        let _ = rpc.msgs.compactMap { message -> EventLoopFuture<Void> in
    //
    //            /// Ensure the message conforms to our MessageSignaturePolicy
    //            guard passesMessageSignaturePolicy(message) else {
    //                self.logger.warning("Failed signature policy, discarding message")
    //                return self.eventLoop.makeSucceededVoidFuture()
    //            }
    //
    //            /// Derive the message id using the overidable messageID function
    //            guard let messageIDFunc = self.messageIDFunctions[message.topicIds.first!] else {
    //                self.logger.warning("No MessageIDFunction defined for topic '\(message.topicIds.first!)'. Dropping Message.")
    //                return self.eventLoop.makeSucceededVoidFuture()
    //            }
    //
    //            let id = messageIDFunc(message)
    //
    //            self.logger.info("Message ID `\(id.asString(base: .base16))`")
    //            self.logger.info("\(message.description)")
    //
    //            /// Check to ensure we haven't seen this message already...
    //            return self.messageCache.exists(messageID: id, on: nil).flatMap { exists -> EventLoopFuture<Void> in
    //                guard exists == false else { self.logger.trace("Dropping Duplicate Message"); return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                /// Validate the unseen message before storing it in our message cache...
    //                return self.validate(message: message).flatMap { valid -> EventLoopFuture<Void> in
    //                    guard valid else { self.logger.warning("Dropping Invalid Message: \(message)"); return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                    /// Store the message in our message cache
    //                    self.logger.info("Storing Message: \(id.asString(base: .base16))");
    //                    /// - Note: We can run into issues where we end up saving duplicate messages cause when we check for existance they haven't been saved yet, and by the time we get around to saving them, theirs multiple copies ready to be stored.
    //                    /// We temporarily added the `valid` flag to the `put` method to double check existance of a message before forwarding it and alerting our handler.
    //                    return self.messageCache.put(messageID: id, message: (topic: message.topicIds.first!, data: message), on: nil).flatMap { valid in
    //                        guard valid else { self.logger.warning("Encountered Duplicate Message While Attempting To Store In Message Cache"); return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                        /// Should we pass the message onto any SubscriptionHandlers at this point?
    //                        if let handler = self.subscriptions[message.topicIds.first!] {
    //                            self.logger.trace("Forwarding message to handler: ID:\(id.asString(base: .base16))")
    //                            let _ = handler.on?(.data(message))
    //                        } else {
    //                            self.logger.warning("No Subscription Handler for topic:`\(message.topicIds.first!)`")
    //                        }
    //
    //                        /// - TODO: Event sub, possibly remove later...
    //                        self._eventHandler?(.message(remotePeer, [message]))
    //
    //                        /// Forward the message onto any other subscribers to this topic (excluding the sender)
    //                        return self.peerState.peersSubscribedTo2(topic: message.topicIds.first!, on: nil).flatMap { subscribers -> EventLoopFuture<Void> in
    //
    //                            guard subscribers.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                            var forwardedRPC = RPC()
    //                            forwardedRPC.msgs = [message]
    //                            let payload = try! forwardedRPC.serializedData()
    //
    //                            return subscribers.map { (peerID, stream) in
    //                                guard peerID != remotePeer else { return self.eventLoop.makeSucceededVoidFuture() }
    //                                self.logger.trace("Forwarding message to subscriber \(peerID)")
    //
    //                                return stream.write(putUVarInt(UInt64(payload.count)) + payload)
    //                            }.flatten(on: self.eventLoop)
    //                        }
    //                    }
    //                }
    //            }
    //        }
    //
    //        /// Return the response if any...
    //        return self.eventLoop.makeSucceededFuture(nil)
    //    }

}
