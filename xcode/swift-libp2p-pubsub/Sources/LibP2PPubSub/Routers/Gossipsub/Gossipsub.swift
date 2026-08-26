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
import LibP2PCore
import NIOConcurrencyHelpers

/// GossipSub Implementation
///
/// When we extend the `BasePubSub` class we get access to...
/// - logger
/// - eventLoop (our main event loop that this pubsub instance lives in)
/// - subscription list
/// - publish(msg:)
/// - Local PeerID
/// - nextMessageSequenceNumber
///
/// - Note: [Gossipsub v1.0.0 Spec](https://github.com/libp2p/specs/blob/master/pubsub/gossipsub/gossipsub-v1.0.md)
/// - Note: The methods that we must implement will be enforced by the compiler when we conform to PubSub
public class GossipSub: BasePubSub, PubSubCore, LifecycleHandler, @unchecked Sendable {
    public static let multicodec: String = "/meshsub/1.0.0"

    public let lowerOutboundDegree = 4
    public let targetOutboundDegree = 6
    public let upperOutboundDegree = 12
    //public let lazyOutboundDegree = 6

    public let heartbeatInterval: TimeAmount = .seconds(1)
    public let mcacheWindowLength = 8
    public let mcacheGossipLength = 5

    public let seenTTL: TimeAmount = .seconds(120)

    private var eventList: [PubSubEvent] = []

    public init(
        group: EventLoopGroup,
        libp2p: Application,
        debugName: String = "Gossipsub",
        emitSelf: Bool = false
    ) throws {
        // Init our PeerState
        let peerState = PeeringState(eventLoop: group.next())

        // Init our Message Cache
        let messageCache = MessageCache(
            eventLoop: group.next(),
            historyWindows: mcacheWindowLength,
            gossipWindows: mcacheGossipLength
        )

        // Regsiter our /meshsub/1.0.0 route
        try registerGossipsubRoute(libp2p)

        // Init super
        try super.init(
            group: group,
            libp2p: libp2p,
            peerState: peerState,
            messageCache: messageCache,
            debugName: debugName,
            multicodecs: [GossipSub.multicodec],
            globalSignaturePolicy: .strictSign,
            canRelayMessages: true,
            emitSelf: emitSelf
        )

        self._eventHandler = { event in
            self.eventList.append(event)
        }
    }

    public func dumpEventList() {
        self.logger.notice("*** Event List ***")
        for event in self.eventList {
            self.logger.notice("\(event.description)")
        }
        self.logger.notice("******************")
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
        // Do whatever else we need to do...
    }

    /// We can override methods if we need to, just make sure to call super...
    public override func stop() throws {
        guard self.state == .starting || self.state == .started else { throw Errors.alreadyStopped }
        // Do whatever we need to do...
        try super.stop()
    }

    /// The current implementation would send a seperate IHave message for each topic a meta peer has in common with us. I'm not sure if this is correct according to the specs.
    /// Ex: if peerA and us and meta peers on topics "news" and "fruit" we would send two iHave messages, one with messageIDs for news and another with messageIDs for fruit
    public override func heartbeat() -> EventLoopFuture<Void> {
        /// Publish iHave messages if necessary
        self.logger.trace("Heartbeat called")
        let tic = Date().timeIntervalSince1970

        var tasks: [EventLoopFuture<Void>] = []

        /// Mesh Maintenance
        tasks.append(
            self.peerState.topicSubscriptions(on: self.eventLoop).flatMap { subscriptions -> EventLoopFuture<Void> in
                guard let ps = self.peerState as? PeeringState else {
                    return self.eventLoop.makeFailedFuture(Errors.invalidPeerStateConformance)
                }

                return subscriptions.map { topic in
                    guard let subs = ps.mesh[topic]?.count else { return self.eventLoop.makeSucceededVoidFuture() }
                    if subs < self.lowerOutboundDegree {
                        self.logger.trace(
                            "\(topic) doesn't have enough subscribers (Has: \(subs), Wants: \(self.lowerOutboundDegree)) attempting to graft some peers"
                        )
                        return self.getMetaPeers().flatMap { metaPeers in
                            guard let metaSubsForTopic = metaPeers[topic] else {
                                self.logger.trace("No peers in fanout to graft")
                                return self.eventLoop.makeSucceededVoidFuture()
                            }
                            return metaSubsForTopic.prefix(self.targetOutboundDegree - subs).map {
                                self.graft(peer: $0, for: topic, andSend: true, includingRecentIHaves: true)
                            }.flatten(on: self.eventLoop)
                        }
                    } else if subs > self.upperOutboundDegree {
                        self.logger.trace(
                            "\(topic) has too many subscribers (Has: \(subs), Wants: \(self.upperOutboundDegree)) attempting to prune some peers"
                        )
                        return self.getPeersSubscribed(to: topic).flatMap { fullPeers in
                            guard fullPeers.count > self.upperOutboundDegree else {
                                return self.eventLoop.makeSucceededVoidFuture()
                            }
                            return fullPeers.prefix(fullPeers.count - self.targetOutboundDegree).map {
                                self.prune(peer: $0.id, for: topic, andSend: true)
                            }.flatten(on: self.eventLoop)
                        }
                    } else {
                        return self.eventLoop.makeSucceededVoidFuture()
                    }
                }.flatten(on: self.eventLoop).transform(to: ())
            }
        )

        /// Fanout Maintenance

        /// Send iHaves to meta peers
        tasks.append(self.sendIHaveMessagesToMetaPeers())

        return EventLoopFuture.whenAllComplete(tasks, on: self.eventLoop).map({ res in
            self.logger.trace("Heartbeat completed in \(Int((Date().timeIntervalSince1970 - tic) * 1_000_000))us")
        })
    }

    /// We have to override / implement this method so our BasePubSub implementation isn't constrained to a particular RPC PubSub Message Type
    override func decodeRPC(_ data: Data) throws -> RPCMessageCore {
        try RPC(serializedBytes: data)
    }

    /// We have to override / implement this method so our BasePubSub implementation isn't constrained to a particular RPC PubSub Message Type
    override func encodeRPC(_ rpc: RPCMessageCore) throws -> Data {
        try RPC(rpc).serializedData()
    }

    //    public func on(_ topic:String, closure:@escaping((PubSub.SubscriptionEvent) -> EventLoopFuture<Void>)) {
    //        self.subscribe(topic: topic, on: closure)
    //    }

    /// Itterates through each meta peer, creates an iHave control message for each topic containing any associated message id's that we may have in our Message Cache, and sends it to them
    func sendIHaveMessagesToMetaPeers() -> EventLoopFuture<Void> {
        /// Itterate through our subscriptions and create a [topic:[ControlIHave]] dictionary
        /// &
        /// Fetch all of our metaPeers (across all topics)
        self.generateIHaveMessages()
            .and(self.getPeerIDs())
            .flatMap { iHaveMsgs, metaPeers -> EventLoopFuture<Void> in
                guard !metaPeers.isEmpty else {
                    self.logger.trace("No peers to send iHave Control messages to")
                    return self.eventLoop.makeSucceededVoidFuture()
                }
                guard !iHaveMsgs.isEmpty else {
                    self.logger.trace("No iHave Control messages to send to peers")
                    return self.eventLoop.makeSucceededVoidFuture()
                }
                guard let ps = self.peerState as? PeeringState else {
                    return self.eventLoop.makeFailedFuture(Errors.invalidPeerStateConformance)
                }
                let messagesSent: NIOLockedValueBox<Int> = .init(0)
                /// For peer in metaPeer
                return metaPeers.compactMap { peer -> EventLoopFuture<Void> in
                    /// Grab the subscription information for this peer id
                    ps.subscriptionForID(peer).map { subscriber, subscriptions -> Void in
                        guard subscriptions.meta.count > 0 else { return }

                        /// Gen RPC Message
                        var rpc = RPC()
                        rpc.control = RPC.ControlMessage()
                        /// For topic in peer.topics
                        rpc.control.ihave = subscriptions.meta.compactMap { topic in
                            /// Append Control Message for topic to RPC
                            iHaveMsgs.first(where: { $0.topicID == topic })
                        }

                        /// If RPC.ControlMessages.count > 0
                        guard rpc.control.ihave.count > 0 else {
                            return
                        }

                        guard var payload = try? rpc.serializedData() else { return }
                        payload = putUVarInt(UInt64(payload.count)) + payload

                        try? subscriber.write(payload.byteArray)
                        self._eventHandler?(
                            .outbound(.iHave(subscriber.id, rpc.control.ihave.compactMap { try? $0.serializedData() }))
                        )

                        messagesSent.withLockedValue { $0 += 1 }

                        return
                    }

                }.flatten(on: self.eventLoop).map {
                    if messagesSent.withLockedValue({ $0 }) > 0 {
                        self.logger.debug("Sent iHave Control messages to \(messagesSent) meta peers")
                    }
                }
            }
    }

    private func generateIHaveMessages() -> EventLoopFuture<[RPC.ControlIHave]> {
        guard let mc = self.messageCache as? MessageCache else {
            return self.eventLoop.makeFailedFuture(Errors.invalidMessageStateConformance)
        }
        return self.eventLoop.flatSubmit({  //-> [String:[RPC.ControlIHave]] in
            var iHaves: [RPC.ControlIHave] = []
            return self.subscriptions.keys.compactMap { topic in
                mc.getGossipIDs(topic: topic).map { messageIDs in
                    iHaves.append(
                        RPC.ControlIHave.with({ iHave in
                            iHave.topicID = topic
                            iHave.messageIds = messageIDs
                        })
                    )
                }
            }.flatten(on: self.eventLoop).map {
                iHaves
            }
        })
    }

    /// Get MetaPeer b58String ID's with an array of their subscriptions (instead of the other way around)
    private func getMetaPeers() -> EventLoopFuture<[String: [PeerID]]> {
        guard let ps = self.peerState as? PeeringState else {
            return self.eventLoop.makeFailedFuture(Errors.invalidPeerStateConformance)
        }
        return ps.metaPeerIDs()
    }

    private func getMetaPeers(forTopic topic: String) -> EventLoopFuture<[PeerID]> {
        guard let ps = self.peerState as? PeeringState else {
            return self.eventLoop.makeFailedFuture(Errors.invalidPeerStateConformance)
        }
        return ps.metaPeerIDs().map { $0[topic] ?? [] }
    }

    private func getPeerIDs() -> EventLoopFuture<[String]> {
        guard let ps = self.peerState as? PeeringState else {
            return self.eventLoop.makeFailedFuture(Errors.invalidPeerStateConformance)
        }
        return ps.eventLoop.submit {
            //self.logger.trace("Gossip Peers Count \(ps.peers.count)")
            ps.peers.keys.map { $0 }
        }
    }

    /// FRUIT TOPIC
    let FruitSubscription = PubSub.SubscriptionConfig(
        /// The subscription topic string
        topic: "fruit",
        /// The Message Signature policy (all messages must be signed vs no signatures allowed)
        signaturePolicy: .strictNoSign,
        /// The custom message validation to use for this topic
        validator: .acceptAll,
        /// The custom messageID function to use for this topic
        messageIDFunc: .concatFromAndSequenceFields
    )

    /// Process the inbound messages however you'd like
    ///
    /// This method will get called once the default implementation determines the message is valid (
    ///  - has the correct protobuf structure
    ///  - conforms to our signing policy
    ///  - contains the appropriate messageID
    ///  - and isn't a duplicate
    //    func processRPCMessage(_ message: RPC) -> EventLoopFuture<Void> {
    //        self.logger.info("TODO::ProcessRPCMessage")
    //        self.logger.info("\(message)")
    //        return self.eventLoop.makeSucceededVoidFuture()
    //    }

    /// Publish arbitrary data, bundled as an RPC message under the specified topic
    public override func publish(topic: String, data: Data, on: EventLoop?) -> EventLoopFuture<Void> {
        self.logger.debug("Attempting to publish data as RPC Message")

        var msg = RPC.Message()
        msg.data = data
        msg.from = Data(self.peerID.id)
        msg.seqno = Data(self.nextMessageSequenceNumber())
        msg.topicIds = [topic]

        return self.publish(msg: msg)
    }

    /// Convenience method for publishing an RPC message as bytes
    public override func publish(topic: String, bytes: [UInt8], on: EventLoop?) -> EventLoopFuture<Void> {
        self.publish(topic: topic, data: Data(bytes), on: on)
    }

    /// Convenience method for publishing an RPC message as a ByteBuffer
    public override func publish(topic: String, buffer: ByteBuffer, on: EventLoop?) -> EventLoopFuture<Void> {
        self.publish(topic: topic, data: Data(buffer.readableBytesView), on: on)
    }

    /// Attempts to subscribe to the specified topic
    /// - Warning: This method assumes some defaults such as `stringNoSign` and `acceptAll` message validation.
    /// - Warning: This is probably not what you want! Consider using `subscribe(PubSub.SubscriptionConfig)` instead
    func subscribe(topic: Topic) throws -> PubSub.SubscriptionHandler {
        let defaultConfig = PubSub.SubscriptionConfig(
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
        let _ = self.subscribe(config, on: nil).flatMap {
            self.getMetaPeers(forTopic: config.topic).flatMap { subscribers -> EventLoopFuture<Void> in
                //self.getPeersSubscribed(to: config.topic).flatMap { subscribers -> EventLoopFuture<Void> in
                guard !subscribers.isEmpty else { return self.eventLoop.makeSucceededVoidFuture() }
                let graftSubscribers = subscribers.prefix(self.targetOutboundDegree)
                self.logger.trace("Sending Graft Messages to \(graftSubscribers.count) subscribers")
                return graftSubscribers.map {
                    self.logger.trace("Sending Graft Message to \($0.id)")
                    return self.graft(peer: $0, for: config.topic, andSend: true, includingRecentIHaves: true)
                }.flatten(on: self.eventLoop)
            }
        }

        /// return the subscription handler
        return subHandler
    }

    public func subscribe(
        _ config: PubSub.SubscriptionConfig,
        closure: @escaping ((PubSub.SubscriptionEvent) -> EventLoopFuture<Void>)
    ) throws {
        /// Ensure the Topic we're subscribing to is valid...
        guard !config.topic.isEmpty, config.topic != "" else { throw Errors.invalidTopic }

        self.logger.info("Subscribing to topic: \(config.topic)")

        /// Init Subscription handler
        let subHandler = PubSub.SubscriptionHandler(pubSub: self, topic: config.topic)
        subHandler.on = closure
        self.subscriptions[config.topic] = subHandler

        /// Let the base/parent PubSub implementation know of the subscription...
        let _ = self.subscribe(config, on: nil).flatMap {
            self.getMetaPeers(forTopic: config.topic).flatMap { subscribers -> EventLoopFuture<Void> in
                //self.getPeersSubscribed(to: config.topic).flatMap { subscribers -> EventLoopFuture<Void> in
                guard !subscribers.isEmpty else { return self.eventLoop.makeSucceededVoidFuture() }
                let graftSubscribers = subscribers.prefix(self.targetOutboundDegree)
                self.logger.trace("Sending Graft Messages to \(graftSubscribers.count) subscribers")
                return graftSubscribers.map {
                    self.logger.trace("Sending Graft Message to \($0.id)")
                    return self.graft(peer: $0, for: config.topic, andSend: true, includingRecentIHaves: true)
                }.flatten(on: self.eventLoop)
            }
        }
    }

    override public func unsubscribe(topic: BasePubSub.Topic, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        /// If we have peers that are interested in this topic, let them know that we're unsubscribing...
        self.logger.info("Unsubscribing from topic: \(topic)")
        return self.getPeersSubscribed(to: topic).flatMap { peers -> EventLoopFuture<Void> in
            guard peers.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }

            return peers.map { peer -> EventLoopFuture<Void> in
                self.logger.trace(
                    "Sending \(peer.id) a prune message before unsubscribing, because they were a full peer. ✊"
                )
                return self.prune(peer: peer.id, for: topic, andSend: true)
            }.flatten(on: self.eventLoop).flatMap {
                // Call unsub on our base class...
                super.unsubscribe(topic: topic, on: loop)
            }

            /// Generate an RPC Message containing our unsubscription...
            ///
            /// - Note: We just send the prune message here because our base pub sub handles sending subscription updates
            /// - Note: We should probably try and combine these messages into a single RPC
            /// Combined
            /// - Note: If we unsubscribe and prune at the same time, we stop receiving messages all together
            /// Separate
            /// - Note: If we prune first and then send our updated subscriptions, our peer continues to send IHave messages every heartbeat
            /// - Note: I think this is because our Stream.write future never completes so we never actually send the updated subscriptions... (this is what was happening)
        }
    }

    private func publish(message: RPC.Message, to: Topic) -> EventLoopFuture<Void> {
        self.logger.debug("TODO::Publish RPC PubSub Message")
        return self.eventLoop.makeSucceededVoidFuture()
    }

    /// Our chance to process the inbound RPC
    ///
    /// - Note: Our BasePubSub handles subscriptions all we need to do here is handle what's unique to GossipSub, which is the Control Messages...
    override internal func processInboundRPC(
        _ rpc: RPCMessageCore,
        from: PeerID,
        request: Request
    ) -> EventLoopFuture<Void> {
        guard let rpc = rpc as? RPC else {
            self.logger.error("Gossipsub was passed a RPCMessageCore that wasn't an RPC")
            return self.eventLoop.makeSucceededVoidFuture()
        }

        /// Process Control Messages
        return self.processControlMessages(rpc, peer: request.remotePeer!).flatMap { res -> EventLoopFuture<Void> in
            /// If the inbound control messages warrent a response, we'll send an RPC message back to the remotePeer now
            self.replyToControlIfNecessary(res, request: request)
        }

        //return self.processMessages(rpc, peer: from)
    }

    /// Our chance to process individual PubSub Message
    ///
    /// This method will get called once the default implementation determines the message is valid
    ///  - has the correct protobuf structure
    ///  - conforms to our signing policy
    ///  - contains the appropriate messageID
    ///  - and isn't a duplicate
    internal override func processInboundMessage(
        _ msg: PubSubMessage,
        from: PeerID,
        request: Request
    ) -> EventLoopFuture<Void> {
        //self.processInboundMessageFlatMap(msg, from: from, request: request)
        self.logger.warning("TODO::Process Inbound Message")
        return self.eventLoop.makeSucceededVoidFuture()
    }

    /// Our chance to process a batch of PubSub Messages
    ///
    /// This method will get called once the default implementation determines the message is valid
    ///  - has the correct protobuf structure
    ///  - conforms to our signing policy
    ///  - contains the appropriate messageID
    ///  - and isn't a duplicate
    internal override func processInboundMessages(
        _ messages: [PubSubMessage],
        from: PeerID,
        request: Request
    ) -> EventLoopFuture<Void> {
        self.logger.trace("Batch Processing Inbound Messages")

        /// Sort the messages based on topic (if a message contains multiple topic ids, this will duplicate the message for each topic)
        /// Example message "🍍" has topicIds "food" and "fruit", the message "🍍" will appear twice in the dictionary below. Allowing us to notify both the Food and Fruit Subscription handlers seperately
        let messagesPerTopic = self.sortMessagesByTopic(messages)

        /// Build a list of messages to send
        //var messagesToSend

        /// Forward the messages onto any other subscribers to this topic (excluding the sender)
        return messagesPerTopic.compactMap { (topic, msgs) -> EventLoopFuture<Void> in
            /// Note: This method will result in multiple messages being sent to a peer with multiple common subscriptions to us
            /// Example: PeerA and us are both subscribed to topics Food and Fruit, we'll send an RPC message forwarding all Food messages and another RPC message forwarding all Fruit messages
            /// We should try and bundle those messages together into a single RPC message
            self.peerState.peersSubscribedTo(topic: topic, on: nil).flatMap { subscribers -> EventLoopFuture<Void> in

                /// Ensure there's subscribers to send the messages to, otherwise bail
                guard subscribers.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }

                /// Prepare the message
                var forwardedRPC = RPC()
                forwardedRPC.msgs = msgs.compactMap { $0 as? RPC.Message }
                var payload = try! forwardedRPC.serializedData()
                payload = putUVarInt(UInt64(payload.count)) + payload

                /// Send the message to each peer subscribed to this topic
                return subscribers.compactMap { peerStreams -> EventLoopFuture<Void> in
                    guard peerStreams.id != from else { return self.eventLoop.makeSucceededVoidFuture() }
                    self.logger.debug("Forwarding message to mesh subscriber \(peerStreams.id)")
                    try? peerStreams.write(payload.byteArray)
                    return self.eventLoop.makeSucceededVoidFuture()
                }.flatten(on: self.eventLoop)
            }

        }.flatten(on: self.eventLoop)

        /// Send all of the above messages...
    }

}

/// RPC Control Message Logic
extension GossipSub {

    private func processControlMessages(
        _ rpc: RPC,
        peer: PeerID
    ) -> EventLoopFuture<(graftRejections: [RPC.ControlPrune], iWantResponses: [RPC.Message], iWant: RPC.ControlIWant?)>
    {
        guard rpc.hasControl else { return self.eventLoop.makeSucceededFuture(([], [], nil)) }
        return self.processGrafts(rpc.control.graft, peer: peer)
            .and(self.processPrunes(rpc.control.prune, peer: peer))
            .and(self.processIWants(rpc.control.iwant, peer: peer))
            .and(self.processIHavesUsingSeenCache(rpc.control.ihave, peer: peer)).map { res in
                (res.0.0.0, res.0.1, res.1)
            }
    }

    /// Processes Inbound Control Graft Messages
    ///
    /// [Process Graft Spec](https://github.com/libp2p/specs/blob/master/pubsub/gossipsub/gossipsub-v1.0.md#graft)
    /// - On receiving a `GRAFT(topic)` message, the router will check to see if it is indeed subscribed to the topic identified in the message.
    /// - If so, the router will add the sender to `mesh[topic]`.
    /// - If the router is no longer subscribed to the topic, it will respond with a `PRUNE(topic)` message to inform the sender that it should remove its mesh link
    private func processGrafts(
        _ grafts: [RPC.ControlGraft],
        peer remotePeer: PeerID
    ) -> EventLoopFuture<[RPC.ControlPrune]> {
        guard grafts.count > 0 else { return self.eventLoop.makeSucceededFuture([]) }

        return grafts.compactMap { graft -> EventLoopFuture<RPC.ControlPrune?> in
            self.logger.trace("RPC::Control - Handling Graft(\(graft.topicID)) Message")

            if self.subscriptions[graft.topicID] != nil {
                /// Check to see if this peer is already a full peer, or in our fanout...
                self.logger.trace(
                    "Received a Graft Message for a topic `\(graft.topicID)` we're subscribed to! Checking \(remotePeer)'s current peer status"
                )

                /// This peice of code will reject an inbound graft request immediately if we're above our `UpperOutboundDegree` limit for this topic.
                /// Pros: It's nice to be in control of wether or not a remote peer can force a grafting
                /// Cons: It seems to prevent some peers from ever joining our mesh
                //                guard let ps = self.peerState as? PeeringState else { return self.eventLoop.makeFailedFuture(Errors.invalidPeerStateConformance) }
                //                // - TODO: Bad Access Code Error when attempting to read dictionary value count
                //                guard (ps.mesh[graft.topicID]?.count ?? 0) < self.upperOutboundDegree else {
                //                    self.logger.trace("Adding \(remotePeer) to our mesh would exceed our upper network degree bounds")
                //                    return ps.newMetaPeer(remotePeer, for: graft.topicID).flatMap {
                //                        self.logger.trace("Rejecting Graft request for \(remotePeer)")
                //                        return self.eventLoop.makeSucceededFuture(
                //                            RPC.ControlPrune.with { pruneMsg in
                //                                pruneMsg.topicID = graft.topicID
                //                            }
                //                        )
                //                    }
                //                }

                // - TODO: Event sub, possibly remove later...
                self._eventHandler?(.inbound(.graft(remotePeer, graft.topicID)))

                // - TODO: Do we need to respond with a graft message to confirm? or does no response == confirmation??
                // - No Response = Confirmation
                // - Prune Response = Rejection
                return self.graft(peer: remotePeer, for: graft.topicID, andSend: true, includingRecentIHaves: true)
                    .transform(to: nil)

            } else {
                // We're not subscribed to the topic, reject the graft message by sending a prune message
                self.logger.debug(
                    "Received a Graft Message for a topic `\(graft.topicID)` that we're not subscribed to. Responding with a Prune Message..."
                )

                return self.eventLoop.makeSucceededFuture(
                    RPC.ControlPrune.with { pruneMsg in
                        pruneMsg.topicID = graft.topicID
                    }
                )

            }
        }.flatten(on: self.eventLoop).map { optionalPrunes -> [RPC.ControlPrune] in
            optionalPrunes.compactMap { $0 }
        }
    }

    private func processPrunes(_ prunes: [RPC.ControlPrune], peer remotePeer: PeerID) -> EventLoopFuture<Void> {
        guard prunes.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }

        return prunes.map { prune in
            self.logger.debug("RPC::Control - Handling Prune(\(prune.topicID)) Message")

            // - TODO: Event sub, possibly remove later...
            self._eventHandler?(.inbound(.prune(remotePeer, prune.topicID)))

            return self.prune(peer: remotePeer, for: prune.topicID, andSend: false)

        }.flatten(on: self.eventLoop)
    }

    /// Processes an array of iWant Control messages in an inbound RPC message. If we contain any of the desired messages we may have in our message cache
    private func processIWants(_ iWants: [RPC.ControlIWant], peer remotePeer: PeerID) -> EventLoopFuture<[RPC.Message]>
    {
        guard iWants.count > 0 else { return self.eventLoop.makeSucceededFuture([]) }
        /// Pull the messageIds out from each iWant message
        /// Reduce the array of array of message ids into a single array
        /// Instantiate a Set<Data> with the flattened array which results in...
        /// A unique set of requested Message IDs.
        let ids = Set(iWants.map { iWant in iWant.messageIds }.reduce([], +))

        /// Ask our message cache for the message associated with each id
        return (self.messageCache as! MessageCache).get(messageIDs: ids, on: self.eventLoop).map {
            $0.compactMap { msg in msg.data as? RPC.Message }
        }.always { _ in
            /// - TODO: Event sub, possibly remove later...
            self._eventHandler?(.inbound(.iWant(remotePeer, ids.map { $0 })))
        }
    }

    /// Given a set of iHave messages, this method checks our Message Cache for ID's that we haven't seen and bundles them into a single RPC.ControlIWant message
    /// This version uses the MCache as our source of seen messages (not what the spec says)
    private func processIHavesUsingMCache(
        _ iHaves: [RPC.ControlIHave],
        peer remotePeer: PeerID
    ) -> EventLoopFuture<RPC.ControlIWant?> {
        guard iHaves.count > 0 else { return self.eventLoop.makeSucceededFuture(nil) }

        /// Get a unique set of message IDs (across all topics) to check our message cache for...
        let ids: Set<Data> = Set(
            iHaves.compactMap {
                /// Ensure we're subscribed to the topic
                guard self.subscriptions[$0.topicID] != nil else { return nil }
                return $0.messageIds
            }.reduce([], +)
        )

        /// - TODO: Event sub, possibly remove later...
        self._eventHandler?(.inbound(.iHave(remotePeer, ids.map { $0 })))

        /// For each messageID
        return self.messageCache.filter(ids: ids, returningOnly: .unknown, on: self.eventLoop).map { needed in
            /// If we don't have any needed messageID's return nil
            guard !needed.isEmpty else { return nil }
            /// Otherwise, we now have an array of desired, unseen, messageIDs.
            /// Let's generate and return a single iWant control message from the ids
            return RPC.ControlIWant.with { iWant in
                iWant.messageIds = needed.compactMap { $0 }
            }
        }
    }

    /// This version uses the base pub subs `SeenCache` as our source of seen messages (what the spec says)
    private func processIHavesUsingSeenCache(
        _ iHaves: [RPC.ControlIHave],
        peer remotePeer: PeerID
    ) -> EventLoopFuture<RPC.ControlIWant?> {
        guard iHaves.count > 0 else { return self.eventLoop.makeSucceededFuture(nil) }

        /// Get a unique set of message IDs (across all topics) to check our message cache for...
        let ids: Set<Data> = Set(
            iHaves.compactMap {
                /// Ensure we're subscribed to the topic
                guard self.subscriptions[$0.topicID] != nil else { return nil }
                return $0.messageIds
            }.reduce([], +)
        )

        guard !ids.isEmpty else {
            self.logger.warning("We're discarding received iHave messages...")
            self.logger.warning("\(iHaves.map { "\($0.topicID) - \($0.messageIds.count)" }.joined(separator: "\n"))")
            return self.eventLoop.makeSucceededFuture(nil)
        }

        /// - TODO: Event sub, possibly remove later...
        self._eventHandler?(.inbound(.iHave(remotePeer, ids.map { $0 })))

        /// For each messageID
        /// The spec says we should check our seen cache instead of mcache
        return self.seenCache.filter(ids: ids, returningOnly: .unknown, on: self.eventLoop).map { needed in
            /// If we don't have any needed messageID's return nil
            guard !needed.isEmpty else { return nil }
            /// Otherwise, we now have an array of desired, unseen, messageIDs.
            /// Let's generate and return a single iWant control message from the ids
            return RPC.ControlIWant.with { iWant in
                iWant.messageIds = needed.compactMap { $0 }
            }
        }
    }

    private func replyToControlIfNecessary(
        _ res: (graftRejections: [RPC.ControlPrune], iWantResponses: [RPC.Message], iWant: RPC.ControlIWant?),
        request: Request
    ) -> EventLoopFuture<Void> {
        if !res.graftRejections.isEmpty || !res.iWantResponses.isEmpty || res.iWant != nil {
            guard let stream = request.connection.hasStream(forProtocol: GossipSub.multicodec, direction: .outbound)
            else {
                self.logger.warning("Failed to find outbound gossipsub stream to peer \(request.remotePeer!)")
                self.logger.warning("Skipping Control Message Response")
                return self.eventLoop.makeSucceededVoidFuture()
            }
            self.logger.trace("We have \(res.graftRejections.count) graft rejection messages")
            self.logger.trace("We have \(res.iWantResponses.count) messages to send in response to iWant requests")
            self.logger.trace(
                "We have \(res.iWant?.messageIds.count ?? 0) iWants in response to the iHaves we received"
            )

            for reject in res.graftRejections {
                self._eventHandler?(.outbound(.prune(request.remotePeer!, reject.topicID)))
            }
            if !res.iWantResponses.isEmpty {
                self._eventHandler?(.outbound(.message(request.remotePeer!, res.iWantResponses)))
            }
            if let want = res.iWant {
                self._eventHandler?(.outbound(.iWant(request.remotePeer!, want.messageIds)))
            }

            /// We need to respond to the sender with an RPC message
            var rpc = RPC()
            rpc.msgs = res.iWantResponses
            rpc.control = RPC.ControlMessage.with { ctrl in
                ctrl.iwant = res.iWant == nil ? [] : [res.iWant!]
                ctrl.prune = res.graftRejections
            }

            var payload = try! rpc.serializedData()
            payload = putUVarInt(UInt64(payload.count)) + payload

            /// Respond to the remote peer
            self.logger.debug("Responding to Control Message")
            let _ = stream.write(payload.byteArray)

        } else {
            self.logger.trace("No Control Response Necessary")
        }
        return self.eventLoop.makeSucceededVoidFuture()
    }
}

/// These should probably be moved to Gossipsub...
extension GossipSub {
    /// Constructs and sends an RPC Prune Control message to the specified peer and topic
    private func _sendPrune(peer: PubSub.Subscriber, for topic: Topic) -> EventLoopFuture<Void> {
        /// Construct the Prune RPC Message
        let rpcPrune = RPC.with { rpc in
            rpc.control = RPC.ControlMessage.with { ctrlMsg in
                ctrlMsg.prune = [
                    RPC.ControlPrune.with { pruneMsg in
                        pruneMsg.topicID = topic
                    }
                ]
            }
        }

        /// Serialize it and format it (with uVarInt length prefix)
        var prunePayload = try! rpcPrune.serializedData()
        prunePayload = putUVarInt(UInt64(prunePayload.count)) + prunePayload
        self.logger.trace("Prune Raw Message: \(prunePayload.asString(base: .base16))")

        /// Send it
        try? peer.write(prunePayload.byteArray)
        self._eventHandler?(.outbound(.prune(peer.id, topic)))

        /// Complete the future...
        return self.eventLoop.makeSucceededVoidFuture()
    }
    func prune(peer: PeerID, for topic: Topic, andSend sendPruneMessage: Bool) -> EventLoopFuture<Void> {
        guard let ps = self.peerState as? PeeringState else {
            return self.eventLoop.makeFailedFuture(Errors.invalidPeerStateConformance)
        }
        /// Ensure that the peer is in fact a full peer in our PeerState
        return ps.isFullPeer(peer).flatMap { isFullPeer in
            guard isFullPeer else {
                self.logger.debug("\(peer) isn't a full peer, no pruning necessary")
                return self.eventLoop.makeSucceededVoidFuture()
            }
            self.logger.trace("Pruning full peer \(peer)")
            return ps.makeMetaPeer(peer, for: topic).flatMap {
                guard sendPruneMessage else { return self.eventLoop.makeSucceededVoidFuture() }
                return ps.streamsFor(peer).flatMap { peerStreams in
                    self.logger.trace("Sending prune message to \(peer)")
                    return self._sendPrune(peer: peerStreams, for: topic)
                }
            }
        }
    }

    /// Constructs and sends an RPC Graft Control message to the specified peer and topic
    private func _sendGraft(
        peer: PubSub.Subscriber,
        for topic: Topic,
        withRecentIHaves: [Data]? = nil
    ) -> EventLoopFuture<Void> {
        /// Construct the Graft RPC Message
        let rpcGraft = RPC.with { rpc in
            rpc.control = RPC.ControlMessage.with { ctrlMsg in
                ctrlMsg.graft = [
                    RPC.ControlGraft.with { graftMsg in
                        graftMsg.topicID = topic
                    }
                ]
                if let ids = withRecentIHaves {
                    ctrlMsg.ihave = [
                        RPC.ControlIHave.with { haveMsg in
                            haveMsg.messageIds = ids
                            haveMsg.topicID = topic
                        }
                    ]
                }
            }
        }

        /// Serialize it and format it (with uVarInt length prefix)
        var graftPayload = try! rpcGraft.serializedData()
        graftPayload = putUVarInt(UInt64(graftPayload.count)) + graftPayload
        self.logger.trace("Graft Raw Message: \(graftPayload.asString(base: .base16))")

        /// Send it
        try? peer.write(graftPayload.byteArray)
        self._eventHandler?(.outbound(.graft(peer.id, topic)))
        if let ids = withRecentIHaves {
            self._eventHandler?(.outbound(.iHave(peer.id, ids)))
        }

        /// We optimisticaly update this peer locally to be a full message (grafted peer).
        /// If they deny this grafting request then they'll send back a prune request, at which point we update our peering state to reflect the failed grafting...
        /// - TODO: Update Peering State

        /// Complete the future...
        return self.eventLoop.makeSucceededVoidFuture()
    }
    func graft(
        peer: PeerID,
        for topic: Topic,
        andSend sendGraftMessage: Bool,
        includingRecentIHaves: Bool = false
    ) -> EventLoopFuture<Void> {
        guard let ps = self.peerState as? PeeringState else {
            return self.eventLoop.makeFailedFuture(Errors.invalidPeerStateConformance)
        }
        return ps.isFullPeer(peer).flatMap { isFullPeer -> EventLoopFuture<Void> in
            guard !isFullPeer else {
                self.logger.debug("\(peer) is already a full peer, no grafting necessary")
                return self.eventLoop.makeSucceededVoidFuture()
            }
            self.logger.trace("Grafting meta peer \(peer)")
            return ps.makeFullPeer(peer, for: topic).flatMap {
                guard sendGraftMessage else { return self.eventLoop.makeSucceededVoidFuture() }
                return ps.streamsFor(peer).flatMap { peerStreams in
                    self.logger.trace("Sending Graft Message to \(peer)")
                    if includingRecentIHaves {
                        return (self.messageCache as! MessageCache).getGossipIDs(topic: topic).flatMap { ids in
                            guard !ids.isEmpty else {
                                return self._sendGraft(peer: peerStreams, for: topic, withRecentIHaves: nil)
                            }
                            return self._sendGraft(peer: peerStreams, for: topic, withRecentIHaves: ids)
                        }
                    } else {
                        return self._sendGraft(peer: peerStreams, for: topic, withRecentIHaves: nil)
                    }
                }
            }
        }
    }

    /// This method constructs and publishes an IWant message containign the specified Message IDs.
    /// If the peer has these messages, we can expect an RPC message containing them to arrive on our handler.
    func requestMessages(_ ids: [Data], from peer: PeerID) -> EventLoopFuture<Void> {
        guard ids.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }
        return self.peerState.streamsFor(peer, on: nil).flatMap { stream in
            let rpcWant = RPC.with { rpc in
                rpc.control = RPC.ControlMessage.with { ctrlMsg in
                    ctrlMsg.iwant = [
                        RPC.ControlIWant.with { wantMsg in
                            wantMsg.messageIds = ids
                        }
                    ]
                }
            }

            /// Serialize it and format it (with uVarInt length prefix)
            var wantPayload = try! rpcWant.serializedData()
            wantPayload = putUVarInt(UInt64(wantPayload.count)) + wantPayload
            self.logger.debug("IWant Raw Message: \(wantPayload.asString(base: .base16))")

            /// - FIXME: stream.write never completes, it should...
            //return stream.write(wantPayload)

            /// Send it
            try? stream.write(wantPayload.byteArray)
            self._eventHandler?(.outbound(.iWant(peer, ids)))
            /// Complete the future...
            return self.eventLoop.makeSucceededVoidFuture()
        }
    }

    func publishIHave(_ ids: [String], for topic: Topic) -> EventLoopFuture<Void> {
        self.getPeersSubscribed(to: topic).flatMap { subscribers in
            guard subscribers.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }

            let rpcHave = RPC.with { rpc in
                rpc.control = RPC.ControlMessage.with { ctrlMsg in
                    ctrlMsg.ihave = [
                        RPC.ControlIHave.with { haveMsg in
                            haveMsg.messageIds = ids.compactMap { $0.data(using: .utf8) }
                            haveMsg.topicID = topic
                        }
                    ]
                }
            }

            /// Serialize it and format it (with uVarInt length prefix)
            var havePayload = try! rpcHave.serializedData()
            havePayload = putUVarInt(UInt64(havePayload.count)) + havePayload
            self.logger.debug("IHave Raw Message: \(havePayload.asString(base: .base16))")

            /// - FIXME: This future never completes, because our write future never completes...
            //return subscribers.map {
            //    $0.1.write(havePayload)
            //}.flatten(on: self.mainLoop)

            for peer in subscribers {
                try? peer.write(havePayload.byteArray)
                self._eventHandler?(.outbound(.iHave(peer.id, ids.compactMap { $0.data(using: .utf8) })))
            }

            return self.eventLoop.makeSucceededVoidFuture()
        }
    }
}

/// Old code
extension GossipSub {
    //    internal func processInboundMessageMap(_ msg:Data, from stream: Stream, request: LibP2P.ProtocolRequest) {
    //        self.logger.info("Processing Inbound Message Using our Map Method")
    //
    //        /// This should only ever be an RPC message.
    //        guard let rpc = try? RPC(serializedData: msg) else {
    //            self.logger.warning("Failed to decode RPC PubSub Message")
    //            self.logger.info("UTF8: \(String(data: request.payload, encoding: .utf8) ?? "Not UTF8")")
    //            self.logger.info("Hex: \(request.payload.asString(base: .base16))")
    //            return //request.eventLoop.makeSucceededFuture(nil)
    //        }
    //
    //        guard let remotePeer = stream.connection?.remotePeer else {
    //            self.logger.warning("Failed to determine message originator (RemotePeer)")
    //            return //request.eventLoop.makeSucceededFuture(nil)
    //        }
    //
    //        let tic = Date().timeIntervalSince1970
    //
    //        /// Forward Messages to Full Peers
    //        let _ = self.processSubscriptions(rpc, peer: remotePeer).map {
    //            /// Process Control Messages
    //            self.processControlMessages(rpc, peer: remotePeer).map { res in
    //                /// If the inbound control messages warrent a response, we'll send an RPC message back to the remotePeer now
    //                self.replyToControlIfNecessary(res, stream: stream).map {
    //                    /// Process Messages
    //                    /// Ensure each message conforms to our signature policy, discard any that don't
    //                    self.ensureSignaturePolicyConformance(rpc.msgs).map { signedMessages in
    //                        /// Compute the message ID for each message
    //                        self.computeMessageIds(signedMessages).map { identifiedMessages in
    //                            /// Using the computed ID's, discard any messages that we've already seen / encountered
    //                            self.discardKnownMessages(identifiedMessages).map { newMessages in
    //                                /// Store the new / unique messages in our MessaheCache
    //                                self.storeMessages(newMessages).map { storedMessages in
    //
    //                                    /// - TODO: Event sub, possibly remove later...
    //                                    self._eventHandler?(.message(remotePeer, storedMessages.map { $0.value }))
    //
    //                                    /// Sort the messages based on topic (if a message contains multiple topic ids, this will duplicate the message for each topic)
    //                                    /// Example message "🍍" has topicIds "food" and "fruit", the message "🍍" will appear twice in the dictionary below. Allowing us to notify both the Food and Fruit Subscription handlers seperately
    //                                    let messagesPerTopic = self.sortMessagesByTopic(storedMessages)
    //
    //                                    /// Pass the messages onto any SubscriptionHandlers at this point
    //                                    for (topic, msgs) in messagesPerTopic {
    //                                        if let handler = self.subscriptions[topic] {
    //                                            for (_, message) in msgs {
    //                                                let _ = handler.on?(.data(message))
    //                                            }
    //                                        } else {
    //                                            self.logger.warning("No Subscription Handler for topic:`\(topic)`")
    //                                        }
    //                                    }
    //
    //                                    /// Forward the messages onto any other subscribers to this topic (excluding the sender)
    //                                    let _ = messagesPerTopic.map { (topic, msgs) in
    //                                        /// Note: This method will result in multiple messages being sent to a peer with multiple common subscriptions to us
    //                                        /// Example: PeerA and us are both subscribed to topics Food and Fruit, we'll send an RPC message forwarding all Food messages and another RPC message forwarding all Fruit messages
    //                                        /// We should try and bundle those messages together into a single RPC message
    //                                        self.peerState.peersSubscribedTo2(topic: topic, on: nil).map { subscribers in
    //
    //                                            /// Ensure there's subscribers to send the messages to, otherwise bail
    //                                            guard subscribers.count > 0 else { return }
    //
    //                                            /// Prepare the message
    //                                            var forwardedRPC = RPC()
    //                                            forwardedRPC.msgs = msgs.map { $0.message }
    //                                            var payload = try! forwardedRPC.serializedData()
    //                                            payload = putUVarInt(UInt64(payload.count)) + payload
    //
    //                                            /// Send the message to each peer subscribed to this topic
    //                                            let _ = subscribers.map { (peerID, stream) in
    //                                                guard peerID != remotePeer else { return }
    //                                                self.logger.info("Forwarding message to subscriber \(peerID)")
    //
    //                                                let _ = stream.write(payload)
    //                                            }
    //                                        }
    //                                    }
    //                                }
    //                            }
    //                        }
    //                    }
    //                }
    //            }
    //        }.always { result in
    //            let toc = Int((Date().timeIntervalSince1970 - tic) * 1_000_000)
    //            self.logger.info("Processed \(rpc.msgs.count) Inbound Messages from Peer \(remotePeer) in \(toc)us")
    //        }
    //    }

    // This should only handle processing messages...
    //    internal func processInboundMessageFlatMap2(_ msg:Data, from stream: LibP2P.Stream, request: Request) {
    //        self.logger.info("Processing Inbound Message Using our FlatMap Method")
    //        /// This should only ever be an RPC message.
    //        guard let rpc = try? RPC(serializedData: msg) else {
    //            self.logger.warning("Failed to decode RPC PubSub Message")
    //            self.logger.info("UTF8: \(String(data: Data(request.payload.readableBytesView), encoding: .utf8) ?? "Not UTF8")")
    //            self.logger.info("Hex: \(Data(request.payload.readableBytesView).asString(base: .base16))")
    //            return //request.eventLoop.makeSucceededFuture(nil)
    //        }
    //
    //        guard let remotePeer = stream.connection?.remotePeer else {
    //            self.logger.warning("Failed to determine message originator (RemotePeer)")
    //            return //request.eventLoop.makeSucceededFuture(nil)
    //        }
    //
    //        self.logger.info("Subscription: \(rpc.subscriptions)")
    //        self.logger.info("Controls: \(rpc.control)")
    //        self.logger.info("Messages: \(rpc.msgs)")
    //
    //        let tic = Date().timeIntervalSince1970
    //
    //        /// Forward Messages to Full Peers
    //        let _ = self.processSubscriptions(rpc, peer: remotePeer).flatMap { _ -> EventLoopFuture<Void> in
    //            /// Process Control Messages
    //            self.processControlMessages(rpc, peer: remotePeer).flatMap { res -> EventLoopFuture<Void> in
    //                /// If the inbound control messages warrent a response, we'll send an RPC message back to the remotePeer now
    //                self.replyToControlIfNecessary(res, stream: stream).flatMap { _ -> EventLoopFuture<Void> in
    //                    /// Process Messages
    //                    guard !rpc.msgs.isEmpty else { return self.eventLoop.makeSucceededVoidFuture() }
    //                    /// Ensure each message conforms to our signature policy, discard any that don't
    //                    return self.ensureSignaturePolicyConformance(rpc.msgs).flatMap { signedMessages -> EventLoopFuture<Void> in
    //                        guard !signedMessages.isEmpty else { return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                        /// Compute the message ID for each message
    //                        return self.computeMessageIds(signedMessages).flatMap { identifiedMessages -> EventLoopFuture<Void> in
    //                            guard !identifiedMessages.isEmpty else { return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                            /// Using the computed ID's, discard any messages that we've already seen / encountered
    //                            return self.discardKnownMessages(identifiedMessages).flatMap { newMessages -> EventLoopFuture<Void> in
    //                                guard !newMessages.isEmpty else { return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                                /// Store the new / unique messages in our MessaheCache
    //                                return self.storeMessages(newMessages).flatMap { storedMessages -> EventLoopFuture<Void> in
    //                                    guard !storedMessages.isEmpty else { return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                                    /// - TODO: Event sub, possibly remove later...
    //                                    self._eventHandler?(.message(remotePeer, storedMessages.map { $0.value }))
    //
    //                                    /// Sort the messages based on topic (if a message contains multiple topic ids, this will duplicate the message for each topic)
    //                                    /// Example message "🍍" has topicIds "food" and "fruit", the message "🍍" will appear twice in the dictionary below. Allowing us to notify both the Food and Fruit Subscription handlers seperately
    //                                    let messagesPerTopic = self.sortMessagesByTopic(storedMessages)
    //
    //                                    /// Pass the messages onto any SubscriptionHandlers at this point
    //                                    for (topic, msgs) in messagesPerTopic {
    //                                        if let handler = self.subscriptions[topic] {
    //                                            for (_, message) in msgs {
    //                                                let _ = handler.on?(.data(message))
    //                                            }
    //                                        } else {
    //                                            self.logger.warning("No Subscription Handler for topic:`\(topic)`")
    //                                        }
    //                                    }
    //
    //                                    /// Forward the messages onto any other subscribers to this topic (excluding the sender)
    //                                    return messagesPerTopic.compactMap { (topic, msgs) -> EventLoopFuture<Void> in
    //                                        /// Note: This method will result in multiple messages being sent to a peer with multiple common subscriptions to us
    //                                        /// Example: PeerA and us are both subscribed to topics Food and Fruit, we'll send an RPC message forwarding all Food messages and another RPC message forwarding all Fruit messages
    //                                        /// We should try and bundle those messages together into a single RPC message
    //                                        self.peerState.peersSubscribedTo2(topic: topic, on: nil).flatMap { subscribers -> EventLoopFuture<Void> in
    //
    //                                            /// Ensure there's subscribers to send the messages to, otherwise bail
    //                                            guard subscribers.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }
    //
    //                                            /// Prepare the message
    //                                            var forwardedRPC = RPC()
    //                                            forwardedRPC.msgs = msgs.compactMap { $0.message as? RPC.Message }
    //                                            var payload = try! forwardedRPC.serializedData()
    //                                            payload = putUVarInt(UInt64(payload.count)) + payload
    //
    //                                            /// Send the message to each peer subscribed to this topic
    //                                            return subscribers.compactMap { (peerID, stream) -> EventLoopFuture<Void> in
    //                                                guard peerID != remotePeer else { return self.eventLoop.makeSucceededVoidFuture() }
    //                                                self.logger.info("Forwarding message to subscriber \(peerID)")
    //
    //                                                return stream.write(payload.bytes)
    //                                            }.flatten(on: self.eventLoop)
    //                                        }
    //
    //                                    }.flatten(on: self.eventLoop)
    //                                }
    //                            }
    //                        }
    //                    }
    //                }
    //            }
    //        }.always { result in
    //            let toc = Int((Date().timeIntervalSince1970 - tic) * 1_000_000)
    //            self.logger.info("Processed \(rpc.msgs.count) Inbound Messages from Peer \(remotePeer) in \(toc)us")
    //        }
    //    }

    //    internal override func processInboundMessage(_ msg: Data, from stream: Stream, request: LibP2P.ProtocolRequest) -> EventLoopFuture<Data?> {
    //        /// This should only ever be an RPC message.
    //        guard let rpc = try? RPC(serializedData: msg) else {
    //            self.logger.warning("Failed to decode RPC PubSub Message")
    //            self.logger.info("UTF8: \(String(data: request.payload, encoding: .utf8) ?? "Not UTF8")")
    //            self.logger.info("Hex: \(request.payload.asString(base: .base16))")
    //            return request.eventLoop.makeSucceededFuture(nil)
    //        }
    //
    //        guard let remotePeer = stream.connection?.remotePeer else {
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
    //        /// - TODO: As we itterate over the control messages, we should bundle all the repsonses we need and then send them all out in a single message at the end of this loop.
    //        /// iWant responses
    //        /// iHave responses
    //        /// prune / graft responses
    //        /// All packaged up in one RPC response...
    //        ///
    //        /// We should also make sure we're still subscribed to this topic (cause we can still receive messages for a period of time after unsubscribing)
    //        if rpc.hasControl {
    //            let ctrl = rpc.control
    //
    //            /// Itterate over all graft messages and handle/process them accordingly
    //            for graft in ctrl.graft {
    //                self.logger.info("RPC::Control - Handling Graft(\(graft.topicID)) Message")
    //                /// At the momment we only check to see if we're subscribed to the topic
    //                /// But we should also consider the peers relative distance, our high water mark, the peers metadata (measured latency, etc)
    //                /// before agreeing to the Graft request
    //                if self.subscriptions[graft.topicID] != nil {
    //                    /// Check to see if this peer is already a full peer, or in our fanout...
    //                    self.logger.info("Received a Graft Message for a topic `\(graft.topicID)` we're subscribed to! Checking \(remotePeer)'s current peer status")
    //                    let _ = self.peerState.isFullPeer(remotePeer).flatMap { isFullPeer -> EventLoopFuture<Void> in
    //                        guard isFullPeer == false else { self.logger.info("Remote peer confirmed our Graft Request"); return self.mainLoop.makeSucceededVoidFuture() }
    //                        self.logger.info("Attempting to upgrade Metadata Peer to Full Peer")
    //                        return self.peerState.makeFullPeer(remotePeer, for: graft.topicID).flatMap { _ -> EventLoopFuture<Void> in
    //                            // Lets accept the graft by responding with a graft
    //                            self.logger.info("Accepting the Graft by sending back a Graft Message...")
    //
    //                            guard let rpc = try? (RPC.with { rpcMsg in
    //                                rpcMsg.control = RPC.ControlMessage.with { ctrlMsg in
    //                                    ctrlMsg.graft = [
    //                                        RPC.ControlGraft.with { grftMsg in
    //                                            grftMsg.topicID = graft.topicID
    //                                        }
    //                                    ]
    //                                }
    //                            }).serializedData() else { self.logger.warning("Failed to construct Graft Response Message"); return self.mainLoop.makeSucceededVoidFuture() }
    //                            return stream.write( putUVarInt(UInt64(rpc.count)) + rpc )
    //                        }
    //                    }
    //
    //                } else {
    //                    // We're not subscribed to the topic, reject the graft message by sending a prune message
    //                    self.logger.info("Received a Graft Message for a topic `\(graft.topicID)` that we're not subscribed to. Responding with a Prune Message...")
    //                    guard let rpc = try? (RPC.with { rpcMsg in
    //                        rpcMsg.control = RPC.ControlMessage.with { ctrlMsg in
    //                            ctrlMsg.prune = [
    //                                RPC.ControlPrune.with { pruneMsg in
    //                                    pruneMsg.topicID = graft.topicID
    //                                }
    //                            ]
    //                        }
    //                    }).serializedData() else { continue }
    //                    let _ = stream.write( putUVarInt(UInt64(rpc.count)) + rpc )
    //                }
    //
    //                /// - TODO: Event sub, possibly remove later...
    //                _eventHandler?(.graft(remotePeer, graft.topicID))
    //            }
    //
    //            /// Itterate over all prune messages and handle/process them accordingly
    //            for prune in ctrl.prune {
    //                self.logger.info("RPC::Control - Handling Prune(\(prune.topicID)) Message")
    //
    //                let _ = self.peerState.isFullPeer(remotePeer).flatMap { isFullPeer -> EventLoopFuture<Void> in
    //                    guard isFullPeer else { return self.mainLoop.makeSucceededVoidFuture() }
    //                    self.logger.info("Pruning Full Peer: \(remotePeer) at their request")
    //                    return self.peerState.makeMetaPeer(remotePeer, for: prune.topicID)
    //                }
    //
    //                /// - TODO: Event sub, possibly remove later...
    //                _eventHandler?(.prune(remotePeer, prune.topicID))
    //            }
    //
    //            for iHave in ctrl.ihave {
    //                self.logger.info("RPC::Control - Handle IHave(\(iHave.topicID)) Message")
    //                self.logger.info("\(iHave.messageIds.compactMap { $0.asString(base: .base16) }.joined(separator: ", "))")
    //
    //                let _ = iHave.messageIds.map { msgId in
    //                    self.messageCache.exists(messageID: msgId, on: nil).map { exists -> Data? in
    //                        guard !exists else { return nil }
    //                        return msgId
    //                    }
    //                }.flatten(on: self.mainLoop).map { msgWeNeed in
    //                    guard msgWeNeed.compactMap({ $0 }).count > 0 else { self.logger.info("All caught up with the gossip!"); return }
    //                    self.logger.info("Messages we need \(msgWeNeed.compactMap { $0?.asString(base: .base16) }.joined(separator: ","))")
    //
    //                    //Go ahead an request those messages...
    //                    let rpc = RPC.with { r in
    //                        r.control = RPC.ControlMessage.with { ctrl in
    //                            ctrl.iwant = [RPC.ControlIWant.with { iWant in
    //                                iWant.messageIds = msgWeNeed.compactMap { $0 }
    //                            }]
    //                        }
    //                    }
    //
    //                    var payload = try! rpc.serializedData()
    //                    payload = putUVarInt(UInt64(payload.count)) + payload
    //
    //                    let _ = stream.write(payload)
    //                }
    //
    //                /// - TODO: Event sub, possibly remove later...
    //                _eventHandler?(.iHave(remotePeer, iHave.messageIds))
    //            }
    //
    //            for iWant in ctrl.iwant {
    //                self.logger.info("RPC::Control - Handle IWant Message")
    //                self.logger.info("\(iWant.messageIds.map { "\($0.asString(base: .base16))" }.joined(separator: " ,"))")
    //
    //                let _ = iWant.messageIds.compactMap { msgId in
    //                    self.messageCache.get(messageID: msgId, on: nil)
    //                }.flatten(on: self.mainLoop).map { messages in
    //                    var rpc = RPC()
    //                    rpc.msgs = messages.compactMap { $0?.data }
    //
    //                    var payload = try! rpc.serializedData()
    //                    payload = putUVarInt(UInt64(payload.count)) + payload
    //
    //                    self.logger.info("Responding to iWant control message by sending \(rpc.msgs.count)/\(iWant.messageIds.count) of the requested messages to peer \(remotePeer)")
    //
    //                    let _ = stream.write(payload)
    //                }
    //
    //                /// - TODO: Event sub, possibly remove later...
    //                _eventHandler?(.iWant(remotePeer, iWant.messageIds))
    //            }
    //        }
    //
    //        /// Handle the published messages
    //        let _ = rpc.msgs.flatMap { message -> EventLoopFuture<Void> in
    //
    //            /// Ensure the message conforms to our MessageSignaturePolicy
    //            guard passesMessageSignaturePolicy(message) else {
    //                self.logger.warning("Failed signature policy, discarding message")
    //                return self.mainLoop.makeSucceededVoidFuture()
    //            }
    //
    //            /// Derive the message id using the overidable messageID function
    //            guard let messageIDFunc = self.messageIDFunctions[message.topicIds.first!] else {
    //                self.logger.warning("No MessageIDFunction defined for topic '\(message.topicIds.first!)'. Dropping Message.")
    //                return self.mainLoop.makeSucceededVoidFuture()
    //            }
    //
    //            let id = messageIDFunc(message)
    //
    //            self.logger.info("Message ID `\(id.asString(base: .base16))`")
    //            self.logger.info("\(message.description)")
    //
    //            /// Check to ensure we haven't seen this message already...
    //            return self.messageCache.exists(messageID: id, on: nil).flatMap { exists -> EventLoopFuture<Void> in
    //                guard exists == false else { self.logger.warning("Dropping Duplicate Message"); return self.mainLoop.makeSucceededVoidFuture() }
    //
    //                /// Validate the unseen message before storing it in our message cache...
    //                return self.validate(message: message).flatMap { valid -> EventLoopFuture<Void> in
    //                    guard valid else { self.logger.warning("Dropping Invalid Message: \(message)"); return self.mainLoop.makeSucceededVoidFuture() }
    //
    //                    /// Store the message in our message cache
    //                    self.logger.info("Storing Message: \(id.asString(base: .base16))");
    //                    /// - Note: We can run into issues where we end up saving duplicate messages cause when we check for existance they haven't been saved yet, and by the time we get around to saving them, theirs multiple copies ready to be stored.
    //                    /// We temporarily added the `valid` flag to the `put` method to double check existance of a message before forwarding it and alerting our handler.
    //                    return self.messageCache.put(messageID: id, message: (topic: message.topicIds.first!, data: message), on: nil).flatMap { valid in
    //                        guard valid else { self.logger.warning("Encountered Duplicate Message While Attempting To Store In Message Cache"); return self.mainLoop.makeSucceededVoidFuture() }
    //
    //                        /// Should we pass the message onto any SubscriptionHandlers at this point?
    //                        if let handler = self.subscriptions[message.topicIds.first!] {
    //                            self.logger.info("Forwarding message to handler: ID:\(id.asString(base: .base16))")
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
    //                            guard subscribers.count > 0 else { return self.mainLoop.makeSucceededVoidFuture() }
    //
    //                            var forwardedRPC = RPC()
    //                            forwardedRPC.msgs = [message]
    //                            let payload = try! forwardedRPC.serializedData()
    //
    //                            return subscribers.map { (peerID, stream) in
    //                                guard peerID != remotePeer else { return self.mainLoop.makeSucceededVoidFuture() }
    //                                self.logger.info("Forwarding message to subscriber \(peerID)")
    //
    //                                return stream.write(putUVarInt(UInt64(payload.count)) + payload)
    //                            }.flatten(on: self.mainLoop)
    //                        }
    //                    }
    //                }
    //            }
    //        }
    //
    //        /// Return our response if we have one...
    //        return self.mainLoop.makeSucceededFuture(nil)
    //    }

    //    private func replyToControlIfNecessary2(_ res:(graftRejections:[RPC.ControlPrune], iWantResponses:[RPC.Message], iWant:RPC.ControlIWant?), stream:LibP2P.Stream) -> EventLoopFuture<Void> {
    //        if !res.graftRejections.isEmpty || !res.iWantResponses.isEmpty || res.iWant != nil {
    //            self.logger.info("We have \(res.graftRejections.count) graft rejection messages")
    //            self.logger.info("We have \(res.iWantResponses.count) messages to send in response to iWant requests")
    //            self.logger.info("We have \(res.iWant?.messageIds.count ?? 0) iWants in response to the iHaves we received")
    //
    //            /// We need to respond to the sender with an RPC message
    //            var rpc = RPC()
    //            rpc.msgs = res.iWantResponses
    //            rpc.control = RPC.ControlMessage.with { ctrl in
    //                ctrl.iwant = res.iWant == nil ? [] : [res.iWant!]
    //                ctrl.prune = res.graftRejections
    //            }
    //
    //            var payload = try! rpc.serializedData()
    //            payload = putUVarInt(UInt64(payload.count)) + payload
    //
    //            /// Respond to the remote peer
    //            self.logger.info("Responding to Control Message")
    //            let _ = stream.write(payload.bytes)
    //
    //        } else {
    //            self.logger.info("No Control Response Necessary")
    //        }
    //        return self.eventLoop.makeSucceededVoidFuture()
    //    }

    /// Given an array of `RPC.Message`s, this method will ensure each message conforms to our SignaturePolicy, dropping/discarding the messages that don't
    //    private func ensureSignaturePolicyConformance(_ messages:[RPC.Message]) -> EventLoopFuture<[RPC.Message]> {
    //        self.eventLoop.submit {
    //            messages.filter { self.passesMessageSignaturePolicy($0) }
    //        }
    //    }

    /// Given an array of `RPC.Message`s, this method will compute the Message ID for each message (or drop the message if it's invalid) and returns an `[ID:RPC.Message]` dictionary
    //    private func computeMessageIds(_ messages:[RPC.Message]) -> EventLoopFuture<[Data:RPC.Message]> {
    //        self.eventLoop.submit {
    //            var msgs:[Data:RPC.Message] = [:]
    //            messages.forEach { message in
    //                /// Ensure the message has a topic and that we have a messageIDFunc registered for that topic
    //                guard let firstTopic = message.topicIds.first, let messageIDFunc = self.messageIDFunctions[firstTopic] else {
    //                    self.logger.warning("No MessageIDFunction defined for topic '\(message.topicIds.first!)'. Dropping Message.")
    //                    return
    //                }
    //                /// Compute the message id and insert it into our dictionary
    //                msgs[Data(messageIDFunc(message))] = message
    //            }
    //            return msgs
    //        }
    //    }
    //
    /// Given a dictionary of Messages and their IDs, this method will discard any messages that are already present in our message cache, returning a dictionary of new and unique messages
    //    private func discardKnownMessages(_ messages:[Data:RPC.Message]) -> EventLoopFuture<[Data:RPC.Message]> {
    //        let ids = messages.keys.map { $0 }
    //        return (self.messageCache as! MCache).filter(ids: Set(ids), returningOnly: .unknown, on: self.eventLoop).map { unknownIds -> [Data:RPC.Message] in
    //            var newMessages:[Data:RPC.Message] = [:]
    //            unknownIds.forEach { newMessages[$0] = messages[$0] }
    //            return newMessages
    //        }
    //    }

    /// Given a dictionary of Messages, this method will validate each message using the appropriate validation function, and silently discard any messages that fail to validate for any reason. Returns a dictionary of Valid RPC.Messages indexed by their ID
    //    private func validateMessages(_ messages:[Data:RPC.Message]) -> EventLoopFuture<[Data:RPC.Message]> {
    //        var validMessages:[Data:RPC.Message] = [:]
    //        return messages.map { message in
    //            self.validate(message: message.value, on: self.eventLoop).map { valid in
    //                validMessages[message.key] = message.value
    //            }
    //        }.flatten(on: self.eventLoop).map {
    //            return validMessages
    //        }
    //    }

    //    private func storeMessages(_ messages:[Data:PubSubMessage]) -> EventLoopFuture<[Data:PubSubMessage]> {
    //        self.messageCache.put(messages: messages, on: self.eventLoop)
    //    }

    //    private func sortMessagesByTopic(_ messages:[Data:RPC.Message]) -> [String:[(id:Data, message:RPC.Message)]] {
    //        var messagesByTopic:[String:[(Data, RPC.Message)]] = [:]
    //        for message in messages {
    //            for topic in message.value.topicIds {
    //                if messagesByTopic[topic] == nil { messagesByTopic[topic] = [] }
    //                messagesByTopic[topic]?.append( (message.key, message.value) )
    //            }
    //        }
    //        return messagesByTopic
    //    }
}
