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

private protocol RPCValidator {
    func validate(message: RPC, from: PeerID) -> Bool
    func validateExtended(message: RPC, from: PeerID) -> BasePubSub.ValidationResult
}

/// The Base PubSub Class all PubSub implementations should build off of
///
/// - TODO: Protocol out messageCache
/// - TODO: Protocol out peeringState
/// - TODO: Further define api and base functionality
///
/// Example:
/// Floodsub would define a time based message cache that just stores ID (not the full message)
/// GossipSub would define a shifting - window based message cache that stores full messages for a period of time, then ID's thereafter
/// Both of these implementations should be able to swap out database drivers using fluent
/// Same for peeringState
///
/// - Note:
/// PubSub seems to opperate by maintaining 2 streams between each peer (one for publishing data, and one for reading data)
/// This is why eveyrthing works as expected when we dial floodsub/1.0.0 but fails when we try and write over the inbound stream.
/// The JS implementation wraps the inbound and outbound stream in a [PeerStreams](https://github.com/libp2p/js-libp2p-interfaces/blob/master/packages/libp2p-pubsub/src/peer-streams.ts) object to simplify reading / writing.
open class BasePubSub: @unchecked Sendable {
    //public static var multicodec: String { "" }

    public typealias Topic = String
    //typealias FancyValidator = (uuid:String, exec:(Pubsub_Pb_Message) -> EventLoopFuture<Bool>)
    //typealias FancyValidatorExtended = (uuid:String, exec:(Pubsub_Pb_Message) -> EventLoopFuture<ValidationResult>)
    typealias Validator = @Sendable (PubSubMessage) -> Bool
    typealias ValidatorExtended = @Sendable (PubSubMessage) -> ValidationResult

    public enum Errors: Error {
        case debugNameMustBeSet
        case invalidMulticodecLength
        case alreadyRunning
        case alreadyStopped
        case invalidTopic
        case invalidSubscription
        case notImplemented

        /// Overrides
        case noRPCDecoder
        case noRPCEncoder

        /// Message Processing
        case duplicateMessage
        case signaturePolicyViolation
        case noIDFunctionForTopic
        case failedMessageValidation

        /// Peerstates & MessageStates
        case invalidPeerStateConformance
        case invalidMessageStateConformance
    }

    public static let MessagePrefix = "libp2p-pubsub:".data(using: .utf8)!

    //public var multicodec: String = ""
    public var peerID: PeerID { self.libp2p!.peerID }

    private weak var libp2p: Application!
    private let elg: EventLoopGroup
    public let eventLoop: EventLoop
    public private(set) var state: ServiceLifecycleState  //State
    private let multicodecs: [SemVerProtocol]
    private let messageSignaturePolicy: PubSub.SignaturePolicy
    private let canRelayMessages: Bool
    private let emitSelf: Bool
    private var messageSequenceNumber: Int = 0

    /// A simple seen cache to store message id's that we've encountered recently (seenTTL)
    //internal var seenCache:[Data:UInt64] = [:]
    internal let seenCache: SeenCache

    internal var messageCache: MessageStateProtocol
    internal var peerState: PeerStateProtocol

    //private var validators:[Topic:[FancyValidator]] = [:]
    //private var validatorsExt:[Topic:[FancyValidatorExtended]] = [:]
    internal var validators: [Topic: [Validator]] = [:]
    internal var validatorsExt: [Topic: [ValidatorExtended]] = [:]
    internal var messageIDFunctions: [Topic: (PubSubMessage) -> Data] = [:]
    internal var topicSignaturePolicy: [Topic: PubSub.SignaturePolicy] = [:]

    private var topology: MulticodecTopology!

    internal var logger: Logger

    private var runLoopTask: RepeatedTask? = nil
    private let runLoopInterval: TimeAmount
    //private var handler:LibP2P.ProtocolHandler!

    internal var subscriptions: [Topic: PubSub.SubscriptionHandler]
    internal var subscriptionTopics: [Topic] {
        get { self.subscriptions.keys.map { $0 } }
    }

    internal enum PubSubEvent {
        case inbound(_PubSubEvent)
        case outbound(_PubSubEvent)

        var description: String {
            switch self {
            case .inbound(let event):
                return "Inbound(\(event.description))"
            case .outbound(let event):
                return "Outbound(\(event.description))"
            }
        }
    }

    internal enum _PubSubEvent {
        case subscriptionChange(PeerID, [String: Bool])
        case graft(PeerID, String)
        case prune(PeerID, String)
        case iHave(PeerID, [Data])
        case iWant(PeerID, [Data])
        case message(PeerID, [PubSubMessage])

        var description: String {
            switch self {
            case .message(let peerID, let messages):
                return
                    "message(\(peerID), [\(messages.map({ "(\($0.topicIds.first ?? "???") -> \(String(data: $0.data, encoding: .utf8) ?? "Not UTF8")"}).joined(separator: ", "))]"
            default:
                return "\(self)"
            }
        }
    }

    internal var _eventHandler: ((PubSubEvent) -> Void)?

    //private var topics:

    internal init(
        group: EventLoopGroup,
        libp2p: Application,
        peerState: PeerStateProtocol,
        messageCache: MessageStateProtocol,
        debugName: String,
        multicodecs: [String],
        globalSignaturePolicy: PubSub.SignaturePolicy = .strictSign,
        canRelayMessages: Bool = false,
        emitSelf: Bool = false
    ) throws {
        guard !debugName.isEmpty && debugName != "" else { throw Errors.debugNameMustBeSet }

        self.multicodecs = multicodecs.compactMap { SemVerProtocol($0) }
        guard self.multicodecs.count > 0 else { throw Errors.invalidMulticodecLength }

        self.logger = Logger(label: "\(debugName)[\(libp2p.peerID.shortDescription)][\(UUID().uuidString.prefix(5))]")
        self.logger.logLevel = .info  //libp2p.logger.logLevel  //LOG_LEVEL

        self.libp2p = libp2p
        self.elg = group
        self.eventLoop = elg.next()
        self.state = .stopped

        self.seenCache = SeenCache(eventLoop: elg.next(), logger: self.logger)
        self.peerState = peerState
        self.messageCache = messageCache

        self.messageSignaturePolicy = globalSignaturePolicy
        self.canRelayMessages = canRelayMessages
        self.emitSelf = emitSelf
        self.subscriptions = [:]

        self.runLoopInterval = .seconds(1)

        /// Set up our Network Topology Filter for our specified multicodecs
        self.topology = nil
    }

    /// Starts our PubSub Service
    /// - Throws: PubSub.Errors when we attempt to start an already started PubSub service
    /// - Note: This kicks off a `scheduleRepeatedAsyncTask`
    public func start() throws {
        guard self.state == .stopped else { throw Errors.alreadyRunning }
        self.state = .starting

        /// Init Message Cache
        //self.messageCache = MCache(eventLoop: self.elg.next(), historyWindows: 5, gossipWindows: 3)
        try self.messageCache.start()

        /// Init Peering State
        //self.peerState = PeeringState(eventLoop: self.elg.next())
        try self.peerState.start()
        for sub in subscriptions {
            // Let our peerstate know of our subscriptions (for mesh / fanout distinction)
            let _ = self.peerState.subscribeSelf(to: sub.key, on: nil)
        }

        /// Set up our Network Topology Filter for our specified multicodecs
        /// - TODO: Have a way to unregister from the topology...
        libp2p.topology.register(
            TopologyRegistration(
                protocol: multicodecs.first!.stringValue,
                handler: TopologyHandler(
                    onConnect: onPeerConnected,
                    onDisconnect: onPeerDisconnected
                )
            )
        )

        /// Set our state to started
        self.state = .started

        /// Register floodsub on our LibP2P node
        /// - TODO: We should probably have a way to control if a hard coded route handler is announced or not...
        // app.registrar.regsiter(multicodecs)

        /// Kick off our run loop
        runLoopTask = self.eventLoop.scheduleRepeatedAsyncTask(initialDelay: .zero, delay: runLoopInterval, runLoop)
    }

    public func stop() throws {
        guard self.state == .starting || self.state == .started else { throw Errors.alreadyStopped }
        self.state = .stopping

        /// Deinit Message Cache
        //try self.messageCache.stop().wait()
        try self.messageCache.stop()

        /// Deinit PeerState
        //try self.peerState.stop().wait()
        try self.peerState.stop()

        /// Deinit Network Topology Filter for our specified multicodecs
        //self.topology.deinitialize()
        self.topology = nil

        /// Remove our support for the pubsub protocol
        // app.registrar.unregister(multicodecs)

        /// Cancel our run loop
        runLoopTask?.cancel()

        /// Set our state to Stopped
        self.state = .stopped
    }

    /// Called once every X seconds, this method should handle shifting our MessageCache, updating the PeerState, relaying messages and anything else we need to do to stay in sync with the PubSub network
    private func runLoop(_: RepeatedTask) -> EventLoopFuture<Void> {
        guard self.state == .started else { return self.eventLoop.makeSucceededVoidFuture() }
        self.logger.trace("RunLoop executing")

        /// Call Heartbeat on our services...
        return [
            self.seenCache.trim(),
            self.heartbeat(),
            self.messageCache.heartbeat(),
            self.peerState.heartbeat(),
        ].flatten(on: self.eventLoop)
    }

    /// PubSub implementations should override this method in order to handle reccuring/repeated tasks
    public func heartbeat() -> EventLoopFuture<Void> {
        self.eventLoop.makeSucceededVoidFuture()
    }

    //    private func trimSeenCache() -> EventLoopFuture<Void> {
    //        self.eventLoop.submit {
    //            /// Trim the seenCache of expired messages
    //            var expired = DispatchTime.now().uptimeNanoseconds
    //            if expired > 120_000_000_000 { expired -= 120_000_000_000 } else { return }
    //            self.seenCache = self.seenCache.compactMapValues { time in
    //                time < expired ? nil : time
    //            }
    //        }
    //    }

    /// Called when we discover a new peer that supports the PubSub protocol
    ///
    /// We take this opportunity to add the peer to our PeeringState, we then attempt to reach
    /// out to the peer to discover what topics they're subscribed too and the peers they know about
    private func onPeerConnected(peer: PeerID, connection: Connection) {
        /// When running multiple local instances, our global notification center / event bus can propogate events for our own peer. So we make sure we disregard event related to us...
        guard peer != self.peerID else { return }

        guard self.state == .started else {
            self.logger.warning("Ignoring onPeerConnected notification received while in state '\(state)'")
            return
        }

        self.logger.info("Peer Connected: \(peer.b58String)")
        self.peerState.addNewPeer(peer, on: nil).whenComplete { _ in
            self.logger.trace("Peer Stored: \(peer.b58String)")
        }

        guard let codec = self.multicodecs.first?.stringValue else { return }

        // -TODO: Maybe we take this opportunity to open a PubSub stream if one doesn't already exist...
        if let conn = connection as? BasicConnectionLight {
            self.logger.debug(
                "PubSub::Attempting to auto-dial \(connection.remotePeer?.description ?? "nil") for outbound `\(codec)` stream"
            )
            conn.newStream(forProtocol: codec, mode: .ifOutboundDoesntAlreadyExist)
        }
    }

    /// Called when a Peer that support the PubSub protocol disconnects from us (no longer reachable, goes offline, etc)
    ///
    /// We take this opportunity to remove the peer from our peering state.
    /// We also update any metadata in our databases and perform any necessary cleanup
    private func onPeerDisconnected(peer: PeerID) {
        guard self.state == .started else {
            self.logger.warning("Ignoring onPeerDisconnected notification received while in state '\(state)'")
            return
        }
        self.logger.info("Peer Disconnected: \(peer.b58String)")
        let _ = peerState.onPeerDisconnected(peer)
    }

    /// This is the main entry point for all PubSub logic
    /// Both inbound and outbound events get passed though here and routed to their specific handlers...
    public func processRequest(_ req: Request) -> EventLoopFuture<Response<ByteBuffer>> {
        switch req.event {
        case .ready:
            // This should always succeed...
            guard
                let stream = req.connection.hasStream(
                    forProtocol: multicodecs.first!.stringValue,
                    direction: req.streamDirection
                )
            else {
                self.logger.warning("Failed to find stream associated with new inbound request")
                return req.eventLoop.makeSucceededFuture(.stayOpen)
            }
            switch req.streamDirection {
            case .inbound:
                return self.handleNewInboundStream(req, stream: stream)
            case .outbound:
                return self.handleNewOutboundStream(req, stream: stream)
            }

        case .data:
            guard case .inbound = req.streamDirection else {
                self.logger.warning("We dont accept data on outbound PubSub streams")
                return self.eventLoop.makeFailedFuture(Errors.debugNameMustBeSet)
            }
            //self.logger.info("TODO::HandleInboundData \(data)")
            return self.handleInboundData(req)

        case .closed:
            switch req.streamDirection {
            case .inbound:
                self.handleClosedInboundStream(req)
            case .outbound:
                self.handleClosedOutboundStream(req)
            }
            return self.eventLoop.makeSucceededFuture(.close)

        case .error(let error):
            switch req.streamDirection {
            case .inbound:
                self.handleErrorInboundStream(req, error: error)
            case .outbound:
                self.handleErrorOutboundStream(req, error: error)
            }
            return self.eventLoop.makeSucceededFuture(.close)
        }
    }

    func handleNewInboundStream(_ req: Request, stream: LibP2PCore.Stream) -> EventLoopFuture<Response<ByteBuffer>> {
        //self.logger.info("TODO::HandleNewInboundStream")
        guard let remotePeer = req.remotePeer else {
            self.logger.warning(
                "Received new inbound stream without an authenticated remote peer attached! Closing stream!"
            )
            return req.eventLoop.makeSucceededFuture(.close)
        }
        // Alert our peerstore of the new peer and associated stream
        return peerState.attachInboundStream(remotePeer, inboundStream: stream, on: req.eventLoop).flatMap {
            // Check to see if we have an outbound stream to this peer for /floodsub/1.0.0
            //            if req.connection.hasStream(forProtocol: self.multicodecs.first!.stringValue, direction: .outbound) == nil {
            //                // If not, then open one...
            //                req.logger.info("PubSub::Attempting to auto-dial \(req.remotePeer?.description ?? "nil") for outbound \(self.multicodecs.first!.stringValue) stream")
            //                (req.connection as? BasicConnectionLight)?.newStream(forProtocol: self.multicodecs.first!.stringValue, mode: .ifOutboundDoesntAlreadyExist)
            //            }

            req.eventLoop.makeSucceededFuture(.stayOpen)
        }
    }

    func handleNewOutboundStream(_ req: Request, stream: LibP2PCore.Stream) -> EventLoopFuture<Response<ByteBuffer>> {
        self.logger.debug("TODO::HandleNewOutboundStream")
        guard let remotePeer = req.remotePeer else {
            self.logger.warning(
                "Received new outbound stream without an authenticated remote peer attached! Closing stream!"
            )
            return req.eventLoop.makeSucceededFuture(.close)
        }
        // Store the new outbound (write) stream in our peerstate
        return peerState.attachOutboundStream(remotePeer, outboundStream: stream, on: req.eventLoop).map {
            // Tell the remote peer about our current subscriptions...
            guard let subs = try? self.generateSubscriptionPayload() else {
                self.logger.warning("Failed to generate subscription payload for new \(remotePeer)")
                return .stayOpen
            }  // Or should we close here??
            self.logger.trace("Sharing our subscriptions with our new \(remotePeer)")
            return .respond(req.allocator.buffer(bytes: subs))
        }
    }

    func handleClosedInboundStream(_ req: Request) {
        // Tell our peerstate that the inbound (read) stream for this peer has been closed...
        // Maybe we follow suit and close our outbound (write) stream as well...
        self.logger.debug("TODO::HandleClosedInboundStream")
        guard let remotePeer = req.remotePeer else { return }
        peerState.detachInboundStream(remotePeer, on: req.eventLoop).whenComplete { _ in
            self.logger.trace("Detached inbound stream for \(remotePeer)")
        }
    }

    func handleClosedOutboundStream(_ req: Request) {
        // Tell our peerstate that our outbound (write) stream for this peer has been closed...
        // Maybe we attempt to reopen? Or we request the inbound (read) stream to close as well...
        self.logger.debug("TODO::HandleClosedOutboundStream")
        guard let remotePeer = req.remotePeer else { return }
        peerState.detachOutboundStream(remotePeer, on: req.eventLoop).whenComplete { _ in
            self.logger.trace("Detached outbound stream for \(remotePeer)")
        }
    }

    func handleErrorInboundStream(_ req: Request, error: Error) {
        self.logger.debug("TODO::HandleErrorInboundStream -> \(error)")
    }

    func handleErrorOutboundStream(_ req: Request, error: Error) {
        self.logger.debug("TODO::HandleErrorOutboundStream -> \(error)")
    }

    /// As the base pub sub implementation there are a few things we can do to offload some boiler plate code from our specific Router implementations...
    ///
    /// 1) We ensure the inbound data can be decoded into an RPCMessageCore compliant object
    /// 2) Handles subscription messages
    /// 3) For each message in the RPCMessageCore we...
    ///     1) Validate the messages signature policy
    ///     2) Calculate the messages ID
    ///     3) Discard duplicate / seen messages
    ///     4) Validate the message by running it though the appropriate validators
    ///     5) Store the message in our Message Cache
    ///     6) Alert the Router of the message
    ///     7) Pass the message along to any subscribers
    func handleInboundData(_ request: Request) -> EventLoopFuture<Response<ByteBuffer>> {
        /// Record the time for metrics purposes
        let tic = DispatchTime.now().uptimeNanoseconds

        /// Ensure the request has a remotePeer installed on it
        guard let remotePeer = request.remotePeer else {
            self.logger.warning("Failed to determine message originator (RemotePeer)")
            return request.eventLoop.makeSucceededFuture(.close)
        }

        /// Ask our router to decode the inbound data as an RPCMessageCore compliant object
        guard let rpc = try? self.decodeRPC(Data(request.payload.readableBytesView)) else {
            self.logger.warning("Failed to decode RPC PubSub Message")
            self.logger.warning(
                "UTF8: \(String(data: Data(request.payload.readableBytesView), encoding: .utf8) ?? "Not UTF8")"
            )
            self.logger.warning("Hex: \(Data(request.payload.readableBytesView).asString(base: .base16))")
            /// Do we close the stream? Or keep it open and give them another shot...
            return request.eventLoop.makeSucceededFuture(.close)
        }

        var tasks: [EventLoopFuture<Void>] = []

        /// If message contains subscriptions, process them...
        tasks.append(self.processSubscriptions(rpc, peer: remotePeer))

        /// Give the router a chance to process the entire RPC
        /// - Note: Floodsub / Randomsub might choose to disregard this by not overriding the method, but more complex routers like Gossipsub will need to override this in order to process extra data like control messages
        tasks.append(self.processInboundRPC(rpc, from: remotePeer, request: request))

        /// Handle the published messages (one at a time)
        //        tasks.append(contentsOf: rpc.messages.compactMap { message -> EventLoopFuture<Void> in
        //            /// Process this message
        //            self.processPubSubMessage(message).flatMap { message -> EventLoopFuture<Void> in
        //                /// If we get to this point, it means that the message is new and valid
        //
        //                /// Pass each message onto our specific implementations
        //                return self.processInboundMessage(message, from: remotePeer, request: request).flatMap {
        //
        //                    /// Let our eventhandler know of the message...
        //                    self._eventHandler?(.message(remotePeer, [message]))
        //
        //                    /// Alert the SubscriptionHandlers interested in this message
        //                    if let handler = self.subscriptions[message.topicIds.first!] {
        //                        self.logger.trace("Forwarding new valid message to handler")
        //                        let _ = handler.on?(.data(message))
        //                    } else {
        //                        self.logger.warning("No Subscription Handler for topic:`\(message.topicIds.first!)`")
        //                    }
        //
        //
        //                    /// We're finally done processing this message...
        //                    return self.eventLoop.makeSucceededVoidFuture()
        //                }
        //            }
        //        })

        /// Handle the published messages (all at once)
        tasks.append(
            self.batchProcessPubSubMessages(rpc.messages).flatMap { newMessages -> EventLoopFuture<Void> in
                guard !newMessages.isEmpty else { return request.eventLoop.makeSucceededVoidFuture() }

                /// Give our router implementation a chance to process the new messages...
                return self.processInboundMessages(newMessages, from: remotePeer, request: request).flatMap {

                    /// - TODO: Event sub, possibly remove later...
                    self._eventHandler?(.inbound(.message(remotePeer, newMessages)))

                    /// Sort the messages based on topic (if a message contains multiple topic ids, this will duplicate the message for each topic)
                    /// Example message "🍍" has topicIds "food" and "fruit", the message "🍍" will appear twice in the dictionary below. Allowing us to notify both the Food and Fruit Subscription handlers seperately
                    let messagesPerTopic = self.sortMessagesByTopic(newMessages)

                    /// Pass the messages onto any SubscriptionHandlers at this point
                    for (topic, msgs) in messagesPerTopic {
                        if let handler = self.subscriptions[topic] {
                            for message in msgs {
                                //self.logger.trace("Alerting `\(topic)` handler of message \(message)")
                                let _ = handler.on?(.data(message))
                            }
                        } else {
                            self.logger.warning("No Subscription Handler for topic:`\(topic)`")
                        }
                    }

                    /// We're finally done processing this message...
                    return request.eventLoop.makeSucceededVoidFuture()

                }
            }
        )

        return EventLoopFuture.whenAllComplete(tasks, on: request.eventLoop).map { results in
            self.logger.debug("Processed Inbound RPC Message in \(DispatchTime.now().uptimeNanoseconds - tic)ns")
            var successes: Int = 0
            for result in results { if case .success = result { successes += 1 } }
            if successes != results.count {
                self.logger.trace("Performed \(results.count) tasks")
                self.logger.trace("\(successes) successes")
                self.logger.trace("\(results.count - successes) failures")
            }
            return .stayOpen
        }
    }
    //    func handleInboundData2(_ request:Request) -> EventLoopFuture<ResponseType<ByteBuffer>> {
    //        /// Record the time for metrics purposes
    //        let tic = DispatchTime.now().uptimeNanoseconds
    //
    //        /// Ensure the request has a remotePeer installed on it
    //        guard let remotePeer = request.remotePeer else {
    //            self.logger.warning("Failed to determine message originator (RemotePeer)")
    //            return request.eventLoop.makeSucceededFuture(.close)
    //        }
    //
    //        /// Ask our router to decode the inbound data as an RPCMessageCore compliant object
    //        guard let rpc = try? self.decodeRPC(Data(request.payload.readableBytesView)) else {
    //            self.logger.warning("Failed to decode RPC PubSub Message")
    //            self.logger.info("UTF8: \(String(data: Data(request.payload.readableBytesView), encoding: .utf8) ?? "Not UTF8")")
    //            self.logger.info("Hex: \(Data(request.payload.readableBytesView).asString(base: .base16))")
    //            /// Do we close the stream? Or keep it open and give them another shot...
    //            return request.eventLoop.makeSucceededFuture(.close)
    //        }
    //
    //
    //
    //        /// If message contains subscriptions, process them...
    //        return self.processSubscriptions(rpc, peer: remotePeer).flatMap {
    //            /// Give the router a chance to process the entire RPC
    //            /// - Note: Floodsub / Randomsub might choose to disregard this by not overriding the method, but more complex routers like Gossipsub will need to override this in order to process extra data like control messages
    //            self.processInboundRPC(rpc, from: remotePeer, request: request).flatMap {
    //                /// Handle the published messages (all at once)
    //                self.batchProcessPubSubMessages(rpc.messages).flatMap { newMessages -> EventLoopFuture<Void> in
    //                    guard !newMessages.isEmpty else { return request.eventLoop.makeSucceededVoidFuture() }
    //
    //                    /// - TODO: Event sub, possibly remove later...
    //                    self._eventHandler?(.message(remotePeer, newMessages))
    //
    //                    /// Sort the messages based on topic (if a message contains multiple topic ids, this will duplicate the message for each topic)
    //                    /// Example message "🍍" has topicIds "food" and "fruit", the message "🍍" will appear twice in the dictionary below. Allowing us to notify both the Food and Fruit Subscription handlers seperately
    //                    let messagesPerTopic = self.sortMessagesByTopic(newMessages)
    //
    //                    /// Pass the messages onto any SubscriptionHandlers at this point
    //                    for (topic, msgs) in messagesPerTopic {
    //                        if let handler = self.subscriptions[topic] {
    //                            for message in msgs {
    //                                let _ = handler.on?(.data(message))
    //                            }
    //                        } else {
    //                            self.logger.warning("No Subscription Handler for topic:`\(topic)`")
    //                        }
    //                    }
    //
    //                    /// We're finally done processing this message...
    //                    return request.eventLoop.makeSucceededVoidFuture()
    //                }.map {
    //                    self.logger.debug("Processed Inbound RPC Message in \(DispatchTime.now().uptimeNanoseconds - tic)ns")
    //                    return .stayOpen
    //                }
    //            }
    //        }
    //    }

    private func processSubscriptions(_ rpc: RPCMessageCore, peer remotePeer: PeerID) -> EventLoopFuture<Void> {
        /// Make sure we have subscription messages to work on, otherwise just return...
        guard rpc.subs.count > 0 else { return self.eventLoop.makeSucceededVoidFuture() }

        /// Create a [Topic:Subscribed] dictionary from the subscription options messages
        var subs: [String: Bool] = [:]
        for sub in rpc.subs {
            subs[sub.topicID] = sub.subscribe
        }

        /// Log the subscription changes...
        self.logger.debug("\(remotePeer)::Subscriptions: \(subs)")

        /// - TODO: Event sub, possibly remove later...
        _eventHandler?(.inbound(.subscriptionChange(remotePeer, subs)))

        /// Update our PeeringState with the new subscription changes
        return self.peerState.update(subscriptions: subs, for: remotePeer, on: nil).map {
            /// Notify our handlers of the subscription changes now that they're reflected in our PeeringState
            self.notifyHandlers(for: subs, peer: remotePeer)
        }
    }

    public func generateSubscriptionPayload() throws -> [UInt8] {
        /// Generate an RPC Message containing all of our subscriptions...
        var rpc = RPC()
        rpc.subscriptions = self.subscriptions.map {
            var subOpt = RPC.SubOpts()
            subOpt.topicID = $0.key
            subOpt.subscribe = true
            return subOpt
        }

        let payload = try rpc.serializedData()
        return putUVarInt(UInt64(payload.count)) + payload
    }

    private func notifyHandlers(for subs: [String: Bool], peer remotePeer: PeerID) {
        for sub in subs {
            guard let handler = self.subscriptions[sub.key] else {
                self.logger.warning("No subscription handler for topic:`\(sub.key)`")
                continue
            }
            /// - TODO: Should we alert our subscription handler when a peer unsubscribes from a topic?
            if sub.1 == true {
                self.logger.debug("Notifying `\(sub.key)` subscription handler of new subscriber/peer")
                let _ = handler.on?(.newPeer(remotePeer))
            }
        }
    }

    /// Processes a single PubSubMessage at a time
    /// - Returns: The message if it's new (unseen) and valid
    ///
    /// Processing / Validation Includes
    /// 1) Validating Signature Policy conformance
    /// 2) Calculating Message ID
    /// 3) Ensuring we haven't already seen the message
    /// 4) Validating the Message (by running it through the appropriate installed validators)
    /// 5) Storing the message
    private func processPubSubMessage(_ message: PubSubMessage) -> EventLoopFuture<PubSubMessage> {
        /// Ensure the message conforms to our MessageSignaturePolicy
        guard passesMessageSignaturePolicy(message) else {
            self.logger.warning("Failed signature policy, discarding message")
            return self.eventLoop.makeFailedFuture(Errors.signaturePolicyViolation)
        }

        /// Derive the message id using the overidable messageID function
        guard let messageIDFunc = self.messageIDFunctions[message.topicIds.first!] else {
            self.logger.warning(
                "No MessageIDFunction defined for topic '\(message.topicIds.first!)'. Dropping Message."
            )
            return self.eventLoop.makeFailedFuture(Errors.noIDFunctionForTopic)
        }

        /// Get the messages ID
        let id = messageIDFunc(message)

        self.logger.trace("Message ID `\(id.asString(base: .base16))`")
        self.logger.trace("\(message.description)")

        /// Check to ensure we haven't seen this message already...
        return self.messageCache.exists(messageID: id, on: nil).flatMap { exists -> EventLoopFuture<PubSubMessage> in
            guard exists == false else {
                self.logger.trace("Dropping Duplicate Message")
                return self.eventLoop.makeFailedFuture(Errors.duplicateMessage)
            }

            /// Validate the unseen message before storing it in our message cache...
            return self.validate(message: message).flatMap { valid -> EventLoopFuture<PubSubMessage> in
                guard valid else {
                    self.logger.warning("Dropping Invalid Message: \(message)")
                    return self.eventLoop.makeFailedFuture(Errors.failedMessageValidation)
                }

                /// Store the message in our message cache
                self.logger.trace("Storing Message: \(id.asString(base: .base16))")
                /// - Note: We can run into issues where we end up saving duplicate messages cause when we check for existance they haven't been saved yet, and by the time we get around to saving them, theirs multiple copies ready to be stored.
                /// We temporarily added the `valid` flag to the `put` method to double check existance of a message before forwarding it and alerting our handler.
                return self.messageCache.put(
                    messageID: id,
                    message: (topic: message.topicIds.first!, data: message),
                    on: nil
                ).flatMap { valid -> EventLoopFuture<PubSubMessage> in
                    guard valid else {
                        self.logger.warning("Encountered Duplicate Message While Attempting To Store In Message Cache")
                        return self.eventLoop.makeFailedFuture(Errors.duplicateMessage)
                    }

                    /// Pass each message onto our specific implementations
                    return self.eventLoop.makeSucceededFuture(message)
                }
            }
        }
    }

    /// Processes a batch of PubSubMessage, in an attempt to reduce eventLoop hops...
    /// - Returns: All of the new (unseen) valid messages
    ///
    /// Processing / Validation Includes
    /// 1) Validating Signature Policy conformance
    /// 2) Calculating Message ID
    /// 3) Ensuring we haven't already seen the message
    /// 4) Validating the Message (by running it through the appropriate installed validators)
    /// 5) Storing the message
    private func batchProcessPubSubMessages(_ messages: [PubSubMessage]) -> EventLoopFuture<[PubSubMessage]> {
        self.ensureSignaturePolicyConformance(messages).flatMap { signedMessages -> EventLoopFuture<[PubSubMessage]> in
            guard !signedMessages.isEmpty else { return self.eventLoop.makeSucceededFuture([]) }

            /// Compute the message ID for each message
            return self.computeMessageIds(signedMessages).flatMap {
                identifiedMessages -> EventLoopFuture<[PubSubMessage]> in
                guard !identifiedMessages.isEmpty else { return self.eventLoop.makeSucceededFuture([]) }

                /// Using the computed ID's, discard any messages that we've already seen / encountered
                return self.discardKnownMessagesUsingSeenCache(identifiedMessages).flatMap {
                    newMessages -> EventLoopFuture<[PubSubMessage]> in
                    guard !newMessages.isEmpty else { return self.eventLoop.makeSucceededFuture([]) }

                    /// Store the new / unique messages in our Seen & MessaheCache
                    return self.storeMessages(newMessages).flatMap {
                        storedMessages -> EventLoopFuture<[PubSubMessage]> in

                        /// Return the new / unique messages for further processing
                        self.eventLoop.makeSucceededFuture(storedMessages.map { $0.value })
                    }
                }
            }
        }
    }

    /// This should really just call our implementers
    //    private func onMessage(_ stream:LibP2P.Stream, request:Request) -> EventLoopFuture<Data?> {
    //        let payload = Data(request.payload.readableBytesView)
    //        self.logger.info("Raw Msg:\(payload.asString(base: .base16))")
    //
    //        var pl = payload
    //
    //        /// The inbound message is uvarint length prefixed
    //        let lengthPrefix = uVarInt(pl.bytes)
    //        if lengthPrefix.bytesRead > 0 {
    //            pl = pl.dropFirst(lengthPrefix.bytesRead)
    //        }
    //
    //        return self.processInboundMessage(pl, from: stream, request: request)
    //
    //        /// Return
    //        //return request.eventLoop.makeSucceededFuture(nil)
    //    }

    /// Can be overridden by our custom router implementations
    internal func processInboundRPC(_ rpc: RPCMessageCore, from: PeerID, request: Request) -> EventLoopFuture<Void> {
        request.eventLoop.makeSucceededVoidFuture()
    }

    /// Has to be overriden by our custom router implementations
    internal func processInboundMessage(_ msg: PubSubMessage, from: PeerID, request: Request) -> EventLoopFuture<Void> {
        assertionFailure(
            "Your PubSub implementation must override the PubSub::ProcessInboundMessage(:Data, :Stream, :LibP2P.ProtocolRequest) method"
        )
        return request.eventLoop.makeSucceededVoidFuture()
    }

    /// Has to be overriden by our custom router implementations
    internal func processInboundMessages(
        _ messages: [PubSubMessage],
        from: PeerID,
        request: Request
    ) -> EventLoopFuture<Void> {
        assertionFailure(
            "Your PubSub implementation must override the PubSub::ProcessInboundMessage(:Data, :Stream, :LibP2P.ProtocolRequest) method"
        )
        return request.eventLoop.makeSucceededVoidFuture()
    }

    /// Has to be overriden by our custom router implementations
    internal func decodeRPC(_ data: Data) throws -> RPCMessageCore {
        assertionFailure("Your PubSub implementation must override the PubSub::decodeRPC(_ data:Data) method")
        throw Errors.noRPCDecoder
    }

    internal func encodeRPC(_ rpc: RPCMessageCore) throws -> Data {
        assertionFailure("Your PubSub implementation must override the PubSub::encodeRPC(_ rpc:RPCMessageCore) method")
        throw Errors.noRPCEncoder
    }

    /// Checks to make sure the message conforms to our Message Signing Policy
    /// - Parameter message: The RPC Message to check signature validity for
    /// - Returns: `True` if the message conforms to our policy and is safe to process further.
    /// `False` if the message doesn't conform to our policy, or if the signature is invalid, we should discrad the message immediately in this case.
    ///
    /// - Warning: We should discard any message that fails this check (results in a false return value)
    func passesMessageSignaturePolicy(_ message: PubSubMessage) -> Bool {
        guard let topic = message.topicIds.first, let policy = self.topicSignaturePolicy[topic] else {
            self.logger.warning("No Signature Policy for `\(message.topicIds.first ?? "")` Dropping Message.")
            return false
        }
        switch policy {
        case .strictNoSign:
            guard message.signature.isEmpty && message.key.isEmpty else {
                //guard !message.hasSignature && !message.hasKey else {
                self.logger.warning(
                    "Message Signature Policy Mismatch. Current Policy == Strict No Sign and the Message is signed. Dropping Message."
                )
                return false
            }
            return true
        case .strictSign:
            guard !message.signature.isEmpty && !message.key.isEmpty else {
                //guard message.hasSignature && message.hasKey else {
                self.logger.warning(
                    "Message Signature Policy Mismatch. Current Policy == Strict Sign and the Message isn't signed. Dropping Message."
                )
                return false
            }
            /// Validate Message Signature
            return (try? self.verifyMessageSignature(message)) == true
        }
    }

    func publish(topics: [Topic], messages: [RPC.Message], on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        print("TODO::Publishing Topics:\(topics) -> \(messages)")
        return self.eventLoop.makeSucceededVoidFuture().hop(to: loop ?? eventLoop)
    }

    public func publish(topic: String, data: Data, on: EventLoop?) -> EventLoopFuture<Void> {
        self.eventLoop.makeFailedFuture(Errors.notImplemented)
    }

    public func publish(topic: String, bytes: [UInt8], on: EventLoop?) -> EventLoopFuture<Void> {
        self.eventLoop.makeFailedFuture(Errors.notImplemented)
    }

    public func publish(topic: String, buffer: ByteBuffer, on: EventLoop?) -> EventLoopFuture<Void> {
        self.eventLoop.makeFailedFuture(Errors.notImplemented)
    }

    func publish(msg: RPC.Message, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        /// Get all streams subscribed to the topic...
        guard let topic = msg.topicIds.first else { return self.eventLoop.makeFailedFuture(Errors.invalidTopic) }

        return self.getPeersSubscribed(to: topic).flatMap { subscribers -> EventLoopFuture<Void> in
            if subscribers.isEmpty {
                self.logger.trace(
                    "No known peers subscribed to topic: \(topic). Proceeding with message signing and local storage."
                )
            }

            var msgToSend = RPC.Message()

            /// Sign the message if necessary...
            do {
                if let policy = self.topicSignaturePolicy[topic] {
                    switch policy {
                    case .strictSign:
                        self.logger.trace("Attempting to sign message")
                        let bytes = try BasePubSub.MessagePrefix + msg.serializedData()
                        msgToSend = msg
                        msgToSend.signature = try self.peerID.signature(for: bytes)
                        // pubkey.data and marshalPublicKey have an extra 0801 prepended
                        msgToSend.key = try Data(self.peerID.marshalPublicKey())
                        self.logger.trace("Signed Message: \(msgToSend)")
                    case .strictNoSign:
                        // Nothing to do...
                        msgToSend = msg
                    }
                }

                /// Construct the RPC message
                var rpc = RPC()
                rpc.msgs = [msgToSend]
                /// Serialize it
                var payload = try rpc.serializedData()
                /// prepend a varint length prefix
                payload = Data(putUVarInt(UInt64(payload.count)) + payload.byteArray)

                self.logger.trace("\(payload.asString(base: .base16))")

                /// Store the message in our message cache...
                if let msgID = self.messageIDFunctions[topic]?(msgToSend) {
                    let _ = self.messageCache.put(messageID: msgID, message: (topic, msgToSend), on: nil)
                    /// Do we also add it to our seenCache??
                    self.seenCache.put(messageID: msgID)
                }

                if self.emitSelf, let handler = self.subscriptions[topic] {
                    self._eventHandler?(.outbound(.message(self.peerID, [msgToSend])))
                    let _ = handler.on?(.data(msgToSend))
                }

                /// For each peer subscribed to the topic, send the message their way...
                for subscriber in subscribers {
                    self.logger.debug("Attempting to send message to \(subscriber.id)")
                    self._eventHandler?(.outbound(.message(subscriber.id, [msg])))
                    try? subscriber.write(payload.byteArray)
                }

                /// This is gossipsub related... we should move this into the gsub logic
                if subscribers.isEmpty {
                    self.logger.warning(
                        "This message was signed and stored locally. If your attached router supports publishing past messages then the message might be published to the network eventually."
                    )
                }

                return self.eventLoop.makeSucceededVoidFuture()
            } catch {
                return self.eventLoop.makeFailedFuture(error)
            }
        }.hop(to: loop ?? eventLoop)
    }

    public func subscribe(topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        self.logger.debug("TODO::Subscribe to topic: \(topic)")
        return self.eventLoop.makeSucceededVoidFuture().hop(to: loop ?? eventLoop)
    }

    public func subscribe(_ config: PubSub.SubscriptionConfig, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        /// Ensure the Topic we're subscribing to is valid...
        guard !config.topic.isEmpty, config.topic != "" else {
            return self.eventLoop.makeFailedFuture(Errors.invalidTopic)
        }
        self.logger.debug("Subscribing to topic `\(config.topic)`")
        return self.eventLoop.submit {
            /// Ensure that the topic and subscription handler has been set in our Subscription list by the specific PubSub implementation
            guard self.subscriptions[config.topic] != nil else { throw Errors.invalidSubscription }

            /// Assign our Topic specific MessageSignaturePolicy
            self.assignSignaturePolicy(for: config.topic, policy: config.signaturePolicy)

            /// Allow all messages on this topic
            let _ = self.addValidator(for: config.topic, validator: config.validator.validationFunction)

            /// Assign our MessageID function for the specified topic (different topics can use differnt message ID functions)
            self.logger.trace("Using the \(config.messageIDFunc) as our MessageID function for topic:'\(config.topic)'")
            self.assignMessageIDFunction(for: config.topic, config.messageIDFunc.messageIDFunction)

            /// Let our peerstate know of our subscriptions (for mesh / fanout distinction)
            let _ = self.peerState.subscribeSelf(to: config.topic, on: nil)

            /// Alert everyone we know of to our new subscription
            let _ = self.peerState.getAllPeers(on: loop).map { subs in
                //let _ = self.peerState.peersSubscribedTo(topic: config.topic, on: loop).map { subs in
                guard subs.count > 0 else {
                    self.logger.warning("No subscribers to share subscription with")
                    return
                }
                guard let subPayload = try? self.generateSubPayload(forTopics: [config.topic]) else {
                    self.logger.warning(
                        "Failed to serialize subscription payload. Unable to alert peers of subscription"
                    )
                    return
                }

                for sub in subs {
                    try? sub.write(subPayload)
                    self._eventHandler?(.outbound(.subscriptionChange(sub.id, [config.topic: true])))
                }
            }
        }.hop(to: loop ?? eventLoop)
    }

    public func getPeersSubscribed(to topic: String, on: EventLoop?) -> EventLoopFuture<[PeerID]> {
        self.peerState.peersSubscribedTo(topic: topic, on: on).map { subscribers -> [PeerID] in
            subscribers.map { $0.id }
        }
    }

    public func generateUnsubPayload(forTopics topics: [String]) throws -> [UInt8] {
        /// Generate an RPC Message containing all of our subscriptions...
        var rpc = RPC()
        rpc.subscriptions = topics.map {
            var subOpt = RPC.SubOpts()
            subOpt.topicID = $0
            subOpt.subscribe = false
            return subOpt
        }

        let payload = try rpc.serializedData()
        return putUVarInt(UInt64(payload.count)) + payload
    }

    public func generateSubPayload(forTopics topics: [String]) throws -> [UInt8] {
        /// Generate an RPC Message containing all of our subscriptions...
        var rpc = RPC()
        rpc.subscriptions = topics.map {
            var subOpt = RPC.SubOpts()
            subOpt.topicID = $0
            subOpt.subscribe = true
            return subOpt
        }

        let payload = try rpc.serializedData()
        return putUVarInt(UInt64(payload.count)) + payload
    }

    /// This method simply removes any references to the subscribed topic.
    /// It does NOT handle gracefully unsubscribing from a topic on a network.
    /// For example, letting peers know of the unsubscription by sending RPC messages.
    /// Gracefully unsubscribing is left to the specific PubSub implemtation
    public func unsubscribe(topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        self.logger.debug("Unsubscribing from topic: \(topic)")

        /// Alert all of the peers we know of to our unsubscription
        return self.peerState.getAllPeers(on: loop).flatMap { subs -> EventLoopFuture<Void> in
            //return self.peerState.peersSubscribedTo(topic: topic, on: loop).flatMap { subs -> EventLoopFuture<Void> in
            do {
                let rpc = try self.generateUnsubPayload(forTopics: [topic])
                for sub in subs {
                    if (try? sub.write(rpc)) != nil {
                        self.logger.trace("Sent our `\(topic)` unsub to \(sub.id)")
                        self._eventHandler?(.outbound(.subscriptionChange(sub.id, [topic: false])))
                    } else {
                        self.logger.trace("Failed to send our `\(topic)` unsub to \(sub.id)")
                    }
                }

                return self.eventLoop.flatSubmit {
                    self.validators.removeValue(forKey: topic)
                    self.messageIDFunctions.removeValue(forKey: topic)
                    self.topicSignaturePolicy.removeValue(forKey: topic)
                    self.subscriptions.removeValue(forKey: topic)

                    // Let our peerstate know of our unsubscription
                    return self.peerState.unsubscribeSelf(from: topic, on: nil).transform(to: ())
                }
            } catch {
                self.logger.warning("Failed to unsubscribe from topic `\(topic)` -> \(error)")
                return self.eventLoop.makeFailedFuture(error)
            }
        }.hop(to: loop ?? eventLoop)
    }

}

/// Batch message functions
extension BasePubSub {
    /// Given an array of `RPC.Message`s, this method will ensure each message conforms to our SignaturePolicy, dropping/discarding the messages that don't
    internal func ensureSignaturePolicyConformance(_ messages: [PubSubMessage]) -> EventLoopFuture<[PubSubMessage]> {
        self.eventLoop.submit {
            messages.filter { self.passesMessageSignaturePolicy($0) }
        }
    }

    /// Given an array of `RPC.Message`s, this method will compute the Message ID for each message (or drop the message if it's invalid) and returns an `[ID:RPC.Message]` dictionary
    internal func computeMessageIds(_ messages: [PubSubMessage]) -> EventLoopFuture<[Data: PubSubMessage]> {
        self.eventLoop.submit {
            var msgs: [Data: PubSubMessage] = [:]
            for message in messages {
                /// Ensure the message has a topic and that we have a messageIDFunc registered for that topic
                guard let firstTopic = message.topicIds.first, let messageIDFunc = self.messageIDFunctions[firstTopic]
                else {
                    self.logger.warning(
                        "No MessageIDFunction defined for topic '\(message.topicIds.first ?? "")'. Dropping Message."
                    )
                    continue
                }
                /// Compute the message id and insert it into our dictionary
                msgs[Data(messageIDFunc(message))] = message
            }
            return msgs
        }
    }

    /// Given a dictionary of Messages and their IDs, this method will discard any messages that are already present in our message cache, returning a dictionary of new and unique messages
    /// I think this should sort on our seenCache instead of the messageCache
    //    internal func discardKnownMessagesUsingMessageCache(_ messages:[Data:PubSubMessage]) -> EventLoopFuture<[Data:PubSubMessage]> {
    //        let ids = messages.keys.map { $0 }
    //        return self.messageCache.filter(ids: Set(ids), returningOnly: .unknown, on: self.eventLoop).map { unknownIDs -> [Data:PubSubMessage] in
    //            var newMessages:[Data:PubSubMessage] = [:]
    //            unknownIDs.forEach { newMessages[$0] = messages[$0] }
    //            return newMessages
    //        }
    //    }

    internal func discardKnownMessagesUsingSeenCache(
        _ messages: [Data: PubSubMessage]
    ) -> EventLoopFuture<[Data: PubSubMessage]> {
        let ids = messages.keys.map { $0 }
        return self.seenCache.filter(ids: Set(ids), returningOnly: .unknown, on: self.eventLoop).map {
            unknownIDs -> [Data: PubSubMessage] in
            var newMessages: [Data: PubSubMessage] = [:]
            for unknownID in unknownIDs { newMessages[unknownID] = messages[unknownID] }
            return newMessages
        }
    }

    /// Given a dictionary of Messages, this method will validate each message using the appropriate validation function, and silently discard any messages that fail to validate for any reason. Returns a dictionary of Valid RPC.Messages indexed by their ID
    internal func validateMessages(_ messages: [Data: PubSubMessage]) -> EventLoopFuture<[Data: PubSubMessage]> {
        var validMessages: [Data: PubSubMessage] = [:]
        return messages.map { message in
            self.validate(message: message.value, on: self.eventLoop).map { valid in
                validMessages[message.key] = message.value
            }
        }.flatten(on: self.eventLoop).map {
            validMessages
        }
    }

    /// Given a dictionary of MessageIDs and their Message, this function will
    /// - Store the messageIDs in our SeenCache (for duplicate message dropping)
    /// - Store the messages in our MessageCache for further propogation / forwarding...
    internal func storeMessages(_ messages: [Data: PubSubMessage]) -> EventLoopFuture<[Data: PubSubMessage]> {
        /// Store the message IDs in our SeenCache
        self.seenCache.put(messageIDs: messages.keys.map { $0 }).flatMap {
            /// Then store the complete message in our MessageCache
            self.messageCache.put(messages: messages, on: self.eventLoop)
        }
    }

    internal func sortMessagesByTopic(
        _ messages: [Data: PubSubMessage]
    ) -> [String: [(id: Data, message: PubSubMessage)]] {
        var messagesByTopic: [String: [(Data, PubSubMessage)]] = [:]
        for message in messages {
            for topic in message.value.topicIds {
                if messagesByTopic[topic] == nil { messagesByTopic[topic] = [] }
                messagesByTopic[topic]?.append((message.key, message.value))
            }
        }
        return messagesByTopic
    }

    internal func sortMessagesByTopic(_ messages: [PubSubMessage]) -> [String: [PubSubMessage]] {
        var messagesByTopic: [String: [PubSubMessage]] = [:]
        for message in messages {
            for topic in message.topicIds {
                if messagesByTopic[topic] == nil { messagesByTopic[topic] = [] }
                messagesByTopic[topic]?.append(message)
            }
        }
        return messagesByTopic
    }
}

extension BasePubSub {

    public func getTopics(on loop: EventLoop? = nil) -> EventLoopFuture<[Topic]> {
        self.peerState.topicSubscriptions(on: loop)
    }

    //    public func getPeersSubscribed(to topic: Topic, on loop:EventLoop? = nil) -> EventLoopFuture<[PeerID]> {
    //        return self.peerState.peersSubscribedTo(topic: topic, on: loop)
    //    }

    func getPeersSubscribed(to topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<[PubSub.Subscriber]> {
        self.peerState.peersSubscribedTo(topic: topic, on: loop)
    }

    func validate(message: PubSubMessage, on loop: EventLoop? = nil) -> EventLoopFuture<Bool> {
        self.eventLoop.submit { () -> Bool in
            guard let topic = message.topicIds.first else {
                self.logger.warning("No message topic")
                return false
            }
            guard let validators = self.validators[topic] else {
                print("Warning! No Validators found for Topic: '\(topic)'. Failing message validation by default.")
                return false
            }
            return validators.allSatisfy { $0(message) }  //TODO: Fail this if it takes to long to return
        }.hop(to: loop ?? eventLoop)
    }

    /// Warning! We currently only validate the message using the first validation function in the tpoics array
    /// - TODO: Make this more efficient by failing quickly after the first failed / rejected validation function...
    func validateExtended(message: PubSubMessage, on loop: EventLoop? = nil) -> EventLoopFuture<ValidationResult> {
        self.eventLoop.submit { () -> ValidationResult in
            guard let topic = message.topicIds.first else {
                self.logger.warning("No message topic")
                return .reject
            }
            guard let validators = self.validatorsExt[topic] else {
                print("Warning! No ExtValidators found for Topic: '\(topic)'. Failing message validation by default.")
                return .reject
            }
            /// Run the message through all of our validation functions and store the results...
            let validationResults = validators.map { $0(message) }
            /// If any validation functions failed, reject the message
            if validationResults.contains(.reject) { return .reject }
            /// If any validations functions requested a throttle, throttle the message
            if validationResults.contains(.throttle) { return .throttle }
            /// If any validation functions requested the message to be ignored, ignore the message
            if validationResults.contains(.ignore) { return .ignore }
            /// Otherwise, all validation functions accepted the message
            return .accept
        }.hop(to: loop ?? eventLoop)
    }

    /// Adds a validation function (validator) to the specified topic
    func addValidator(
        for topic: Topic,
        validator: @escaping Validator,
        on loop: EventLoop? = nil
    ) -> EventLoopFuture<Void> {
        self.eventLoop.submit { () -> Void in
            if var existingValidators = self.validators[topic] {
                existingValidators.append(validator)
                self.validators[topic] = existingValidators
            } else {
                self.validators[topic] = [validator]
            }
        }.hop(to: loop ?? eventLoop)
    }

    /// Adds an extended validation function (validatorExt) to the specified topic
    func addValidatorExtended(
        for topic: Topic,
        validator: @escaping ValidatorExtended,
        on loop: EventLoop? = nil
    ) -> EventLoopFuture<Void> {
        self.eventLoop.submit { () -> Void in
            if var existingValidators = self.validatorsExt[topic] {
                existingValidators.append(validator)
                self.validatorsExt[topic] = existingValidators
            } else {
                self.validatorsExt[topic] = [validator]
            }
        }.hop(to: loop ?? eventLoop)
    }

    /// Removes all validators bound to the specified topic
    func removeValidators(for topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        self.eventLoop.submit { () -> Void in
            self.validators.removeValue(forKey: topic)
        }.hop(to: loop ?? eventLoop)
    }

    /// Removes all validators bound to the specified topic
    func removeValidatorsExtended(for topic: Topic, on loop: EventLoop? = nil) -> EventLoopFuture<Void> {
        self.eventLoop.submit { () -> Void in
            self.validatorsExt.removeValue(forKey: topic)
        }.hop(to: loop ?? eventLoop)
    }

    func assignMessageIDFunction(for topic: Topic, _ idFunc: @escaping (PubSubMessage) -> Data) {
        self.messageIDFunctions[topic] = idFunc
    }

    func assignSignaturePolicy(for topic: Topic, policy: PubSub.SignaturePolicy) {
        self.topicSignaturePolicy[topic] = policy
    }

    /// Do we actually increment this for every message we send / per topic?
    /// Or do we just return a random UInt64 (what JS appears to do)
    func nextMessageSequenceNumber() -> [UInt8] {
        Array(withUnsafeBytes(of: UInt64.random(in: 0...UInt64.max).bigEndian) { $0 })
        //self.messageSequenceNumber += 1
        //return self.messageSequenceNumber
    }

    internal func verifyMessageSignature(_ message: PubSubMessage) throws -> Bool {
        //self.mainLoop.submit { () -> Bool in
        guard let key = try? PeerID(marshaledPublicKey: message.key) else {
            self.logger.warning("Failed to recover public key from message data")
            return false
        }

        if key.b58String != message.from.asString(base: .base58btc) {
            self.logger.warning("Message Key does NOT belong to sender")
            return false
        }

        let messageWithoutSignature = try RPC.Message.with { msg in
            msg.from = message.from
            msg.data = message.data
            msg.seqno = message.seqno
            msg.topicIds = message.topicIds
        }.serializedData()

        let verified = try key.isValidSignature(
            message.signature,
            for: BasePubSub.MessagePrefix + messageWithoutSignature
        )

        return verified == true
        //}
    }

    /// ValidationResult represents the decision of an extended validator
    enum ValidationResult {
        /// Accept is a validation decision that indicates a valid message that should be accepted and delivered to the application and forwarded to the network.
        case accept
        /// Reject is a validation decision that indicates an invalid message that should not be delivered to the application or forwarded to the application. Furthermore the peer that forwarded the message should be penalized by peer scoring routers.
        case reject
        /// Ignore is a validation decision that indicates a message that should be ignored: it will be neither delivered to the application nor forwarded to the network. However, in contrast to `Reject`, the peer that forwarded the message must not be penalized by peer scoring routers.
        case ignore
        /// Used Internally to throttle messages from a particular peer
        case throttle
    }

    enum SubscriptionEvent {
        case newPeer(PeerID)
        case data(PubSubMessage)
        case error(Error)

        internal var rawValue: String {
            switch self {
            case .newPeer: return "newPeer"
            case .data: return "data"
            case .error: return "error"
            }
        }
    }

    /// Code for multiple async validators per topic...

    /// return (try? validators.allSatisfy({ validator in
    ///     try validator.exec(message).wait()
    /// })) ?? false
    //    func validate(message: Pubsub_Pb_Message) -> EventLoopFuture<Bool> {
    //        self.mainLoop.flatSubmit { () -> EventLoopFuture<Bool> in
    //            guard let validators = self.validators[message.topic] else { print("Warnin! No Validators found for Topic: '\(message.topic)'. Failing message validation by default."); return self.mainLoop.makeSucceededFuture(false) }
    //
    //            let promise = self.mainLoop.makePromise(of: Bool.self)
    //            let _ = validators.map { $0.exec(message).cascade(to: promise) }
    //
    //            return promise.futureResult
    //        }
    //    }
    //
    //    /// Warning! We currently only validate the message using the first validation function in the tpoics array
    //    func validateExtended(message:Pubsub_Pb_Message) -> EventLoopFuture<ValidationResult> {
    //        self.mainLoop.flatSubmit { () -> EventLoopFuture<ValidationResult> in
    //            guard let validator = self.validatorsExt[message.topic]?.first else { print("Warnin! No Validators found for Topic: '\(message.topic)'. Failing message validation by default."); return self.mainLoop.makeSucceededFuture(.reject) }
    //            return validator.exec(message)
    //        }
    //    }
    //
    //    func addValidator(for topic:Topic, validator:FancyValidator, on loop:EventLoop? = nil) -> EventLoopFuture<Void> {
    //        self.mainLoop.submit { () -> Void in
    //            if self.validators[topic] != nil {
    //                /// Ensure the validator doesn't already exist in our array
    //                guard !self.validators[topic]!.contains(where: { $0.uuid == validator.uuid }) else { return }
    //                /// Add the new validator to our array
    //                self.validators[topic]!.append(validator)
    //            } else {
    //                /// Create the topic and add the first validator
    //                self.validators[topic] = [validator]
    //            }
    //        }
    //    }
    //
    //    func addValidatorExtended(for topic:Topic, validator:FancyValidatorExtended, on loop:EventLoop? = nil) -> EventLoopFuture<Void> {
    //        self.mainLoop.submit { () -> Void in
    //            if self.validatorsExt[topic] != nil {
    //                /// Ensure the validator doesn't already exist in our array
    //                guard !self.validatorsExt[topic]!.contains(where: { $0.uuid == validator.uuid }) else { return }
    //                /// Add the new validator to our array
    //                self.validatorsExt[topic]!.append(validator)
    //            } else {
    //                self.validatorsExt[topic] = [validator]
    //            }
    //        }
    //    }

}

protocol LibP2P_PubSub_MessageCacheProtocol {
    func put(message: PubSubMessage, with id: String, on loop: EventLoop) -> EventLoopFuture<Void>
    func get(messageID: String, on loop: EventLoop) -> EventLoopFuture<PubSubMessage?>
    func exists(messageID: String, on loop: EventLoop) -> EventLoopFuture<Bool>
    func heartbeat() -> EventLoopFuture<Void>
}

/// Defines the basic operations for a PubSub Peering State
/// - Add, Update, Delete Peer
/// - Topic <-> Peer relationships
protocol PubSub_PeeringState {
    func put(_ peer: PeerID) -> EventLoopFuture<Void>
}

/// Defines the basic operations for a PubSub Router Subscription State
/// - Subscribe / Unsubscribe
protocol PubSub_Subscriptions {

}

extension Array where Element == String {
    mutating func appendIfNotPresent(_ item: String) {
        if !self.contains(item) { self.append(item) }
    }
}
