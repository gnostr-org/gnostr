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
//===----------------------------------------------------------------------===//
//
// This source file is part of the SwiftNIO open source project
//
// Copyright (c) 2020 Apple Inc. and the SwiftNIO project authors
// Licensed under Apache License v2.0
//
// See LICENSE.txt for license information
// See CONTRIBUTORS.txt for the list of SwiftNIO project authors
//
// SPDX-License-Identifier: Apache-2.0
//
//===----------------------------------------------------------------------===//

import LibP2P
import NIOCore

/// A `ChannelDuplexHandler` and `Muxer` that implements the YAMUX protocol.
public final class YAMUXHandler: Muxer {
    public static let protocolCodec: String = "/yamux/1.0.0"
    public static let initialWindowSize: UInt32 = 1024 * 256

    public var onStream: ((_Stream) -> Void)?
    public var onStreamEnd: ((LibP2PCore.Stream) -> Void)?

    public var _connection: Connection? {
        get { _conn.withLockedValue { $0 } }
        set { _conn.withLockedValue { $0 = newValue } }
    }
    private let _conn: NIOLockedValueBox<Connection?>

    public var streams: [LibP2PCore.Stream] {
        guard let muxer = self.multiplexer else { return [] }
        return muxer.channels.map { (id, channel) in
            channel as any LibP2PCore.Stream
        }
    }

    internal var channel: Channel? {
        self.context.map { $0.channel }
    }

    /// The state machine that drives our session channel (aka the yamux connection)
    private var sessionState: ChildChannelStateMachine

    /// Wether we are on a connection that is dialing or listening
    private var mode: LibP2PCore.Mode

    /// A buffer used to provide scratch space for writing data to the network.
    ///
    /// This is an IUO because it should never be nil, but it is only initialized once the
    /// handler is added to a channel.
    //private var outboundFrameBuffer: CircularBuffer<Frame>

    /// Whether there's a pending unflushed write.
    private var pendingWrite: Bool

    /// The context in which our connection lives
    private var context: ChannelHandlerContext?

    /// Must be optional as we need to pass it a reference to self.
    private var multiplexer: ChannelMultiplexer?

    /// Whether we're expecting a channelReadComplete.
    private var expectingChannelReadComplete: Bool = false

    /// A buffer of pending channel initializations.
    private var pendingChannelInitializations:
        CircularBuffer<
            (
                promise: EventLoopPromise<YAMUXStream>?,
                initializer: ChildChannel.Initializer?
            )
        >

    /// Our Local PeerID
    let localPeerID: PeerID

    /// The promise to succeed once we're up and running on the pipeline
    private var muxedPromise: EventLoopPromise<Muxer>!

    /// The logger tied to our underlying connection
    private var logger: Logger

    /// Set to true, when we're in the process of shutting down (we shouldn't accept new streams if this is true)
    private var isQuiescing = false

    /// Constructs a new YAMUX Multiplexer
    /// - Parameters:
    ///   - connection: The connection that YAMUX should operate on
    ///   - muxedPromise: The promise that will be fulfilled once YAMUX has activated
    ///   - supportedProtocols: A list of supported protocols (deprecated)
    public convenience init(
        connection: Connection,
        muxedPromise: EventLoopPromise<Muxer>,
        supportedProtocols: [LibP2P.ProtocolRegistration]
    ) {
        self.init(
            connection: connection,
            supportedProtocols: supportedProtocols,
            inboundStreamStateInitializer: connection.inboundMuxedChildChannelInitializer,
            muxedPromise: muxedPromise
        )
    }

    private init(
        connection: Connection,
        supportedProtocols: [LibP2P.ProtocolRegistration] = [],
        inboundStreamStateInitializer: ((Channel) -> EventLoopFuture<Void>)?,
        muxedPromise: EventLoopPromise<Muxer>
    ) {
        self.sessionState = .init(localChannelID: 0)
        //self._sessionState = .init(.init(localChannelID: 0))

        self.mode = connection.mode
        //self._mode = .init(connection.mode)
        self.pendingWrite = false

        self.pendingChannelInitializations = CircularBuffer(initialCapacity: 4)

        self._conn = .init(connection)
        self.localPeerID = connection.localPeer
        self.muxedPromise = muxedPromise
        self.logger = connection.logger
        self.logger[metadataKey: "YAMUX"] = .string("Parent")

        self.multiplexer = ChannelMultiplexer(
            delegate: self,
            allocator: connection.channel.allocator,
            mode: mode,
            initialWindowSize: YAMUXHandler.initialWindowSize,
            logger: logger,
            childChannelInitializer: inboundStreamStateInitializer
        )
    }
}

extension YAMUXHandler: ChannelDuplexHandler {
    public typealias InboundIn = Frame
    public typealias OutboundOut = Frame
    public typealias InboundOut = Never
    public typealias OutboundIn = Never

    public func handlerAdded(context: ChannelHandlerContext) {
        self.context = context
        if context.channel.isActive {
            self.initialize(context: context)
        }
        self.muxedPromise.succeed(self)
    }

    public func handlerRemoved(context: ChannelHandlerContext) {
        self.context = nil

        // We don't actually need to nil out the multiplexer here (it will nil its reference to us)
        // but we _can_, and it doesn't hurt.
        self.multiplexer?.parentHandlerRemoved()
        self.multiplexer = nil

        while let next = self.pendingChannelInitializations.popFirst() {
            next.promise?.fail(ChannelError.eof)
        }
    }

    public func channelActive(context: ChannelHandlerContext) {
        self.initialize(context: context)
    }

    private func initialize(context: ChannelHandlerContext) {
        // The connection is active.
        self.logger.trace("Initialized")
        // Init our session state (channel 0)
        if let frame = self.sessionState.initialize(mode: self.mode) {
            self.logger.trace("Listener opening session control stream 0")
            do {
                try self.writeMessage(frame, context: context, promise: nil)
            } catch {
                self.logger.error("Failed to send session init frame \(error)")
                context.fireErrorCaught(error)
                return
            }
        }
        context.fireChannelActive()
    }

    public func channelInactive(context: ChannelHandlerContext) {
        self.multiplexer?.parentChannelInactive()
    }

    public func channelRead(context: ChannelHandlerContext, data: NIOAny) {
        self.expectingChannelReadComplete = true

        let frame = self.unwrapInboundIn(data)

        do {
            for message in frame.messages {
                try self.processInboundMessage(message, context: context)
            }
        } catch {
            context.fireErrorCaught(error)
        }
    }

    public func channelReadComplete(context: ChannelHandlerContext) {
        logger.trace("ChannelReadComplete Fired")
        self.multiplexer?.parentChannelReadComplete()
        self.expectingChannelReadComplete = false

        if self.pendingWrite {
            self.pendingWrite = false
            context.flush()
        }

        self.createPendingChannelsIfPossible()
    }

    private func writeMessage(
        _ frame: Frame,
        context: ChannelHandlerContext,
        promise: EventLoopPromise<Void>? = nil
    ) throws {
        //self.outboundFrameBuffer.clear()
        self.logger.trace("WriteMessage::\(frame)")
        context.write(self.wrapOutboundOut(frame), promise: nil)
        context.flush()
        promise?.succeed()
    }

    private func processInboundMessage(
        _ message: Message,
        context: ChannelHandlerContext
    ) throws {
        self.logger.trace("REC -> \(message)")

        // We need to switch on the message and pull out any Session messages
        // before we forward them to the multiplexer
        switch message {

        // Session Message
        case .sessionOpen:
            self.logger.trace("Acking our Control Stream")
            let frame = try self.sessionState.receivedSessionOpen(mode: self.mode)
            self.writeFromChildChannel(frame, nil)
            // The session will be opened, lets create any pending channels we might have
            self.createPendingChannelsIfPossible()

        // Session Message
        case .sessionOpenConfirmation:
            self.logger.trace("Got our Session Open Confirmation")
            try self.sessionState.receivedSessionOpenConfirmation()
            // The session is open, lets create any pending channels we might have
            self.createPendingChannelsIfPossible()

        // Session Message
        case .ping(let ping):
            // Reply to the ping
            self.logger.trace("Responding to ping")
            let frame = Frame(
                header: .init(version: .v0, messageType: .ping, flags: [.ack], streamID: 0, length: ping.payload)
            )
            try self.writeMessage(frame, context: context)

        // Session Message
        case .disconnect(let goAway):
            // Welp, we immediately have to close.
            self.logger.trace("Received Disconnect: \(goAway)")
            // Inform our session state
            try self.sessionState.receivedDisconnect(error: YAMUX.NetworkError(goAway.reason))
            // FIXME: Do we respond to disconnects?
            // FIXME: Should we flush the channel first?
            // Close the channel
            context.close(promise: nil)

        // Channel Messages
        default:
            forwardToMultiplexer(message: message)
        }
    }

    private func forwardToMultiplexer(message: Message) {
        self.logger.trace("Forwarding Message to Multiplexer")
        self.logger.trace("\(message)")
        do {
            try self.multiplexer?.receiveMessage(message)
        } catch {
            self.logger.error("YAMUX::ProcessInboundMessage::Error -> \(error)")
        }
    }
}

// MARK: Create a child channel

extension YAMUXHandler {
    /// Creates a YAMUX channel.
    ///
    /// This function is **not** thread-safe: it may only be called from on the channel.
    ///
    /// - parameters:
    ///     - promise: An `EventLoopPromise` that will be fulfilled with the channel when it becomes active.
    ///     - channelInitializer: A callback that will be invoked to initialize the channel.
    public func createChannel(
        _ promise: EventLoopPromise<YAMUXStream>? = nil,
        _ channelInitializer: ((Channel) -> EventLoopFuture<Void>)?
    ) {
        self.pendingChannelInitializations.append(
            (promise: promise, initializer: channelInitializer)
        )
        self.createPendingChannelsIfPossible()
    }

    private func createPendingChannelsIfPossible() {
        guard self.sessionState.isActiveOnNetwork, self.pendingChannelInitializations.count > 0 else {
            // No work to do
            return
        }

        if let multiplexer = self.multiplexer {
            while let next = self.pendingChannelInitializations.popFirst() {
                multiplexer.createOutboundChildChannel(next.promise, next.initializer)
            }
        } else {
            while let next = self.pendingChannelInitializations.popFirst() {
                next.promise?.fail(YAMUX.Error.creatingChannelAfterClosure)
            }
        }
    }
}

// MARK: Disconnect

extension YAMUXHandler {
    // This function is for testing purposes only.
    internal func _disconnect() throws {
        // As this is test-only there are a bunch of preconditions in here, we don't really mind if we hit them in testing.
        if let frame = try? self.sessionState.shouldClose() {
            try self.writeMessage(frame, context: self.context!)
            self.context!.flush()
        }
    }
}

// MARK: Functions called from the multiplexer

extension YAMUXHandler: MultiplexerDelegate {
    func writeFromChildChannel(_ message: Frame, _ promise: EventLoopPromise<Void>?) {
        guard let context = self.context else {
            promise?.fail(ChannelError.ioOnClosedChannel)
            return
        }

        do {
            try self.writeMessage(message, context: context, promise: promise)
        } catch {
            promise?.fail(error)
        }
    }

    func flushFromChildChannel() {
        // If a child channel flushes and we aren't in a channelReadComplete loop, we need to flush. Otherwise
        // we can just wait.
        if !self.expectingChannelReadComplete {
            self.context?.flush()
        }
    }

    func childChannelCreated(stream: any LibP2PCore._Stream) {
        self.onStream?(stream)
    }

    func childChannelRemoved(stream: any LibP2PCore._Stream) {
        self.onStreamEnd?(stream)
    }
}

// MARK: MUXER Conformance

extension YAMUXHandler {

    public func newStream(channel: Channel, proto: String) throws -> EventLoopFuture<any LibP2PCore._Stream> {
        self.logger.trace("NewStream::Initializing outbound child channel for protocol `\(proto)`")
        let promise = self.context!.eventLoop.makePromise(of: YAMUXStream.self)

        // Ask our multiplexer to open a new channel
        //self.multiplexer!.createOutboundChildChannel(promise) { channel -> EventLoopFuture<Void> in
        self.createChannel(promise) { channel -> EventLoopFuture<Void> in
            self.logger.trace("NewStream::Initializing outbound child channel")
            // Pass the new channel into libp2p to be configured
            return self._connection!.outboundMuxedChildChannelInitializer(channel, protocol: proto)
        }

        return promise.futureResult.map { stream in
            self.logger.trace("NewStream::Outbound child channel initialized")
            // Update our stream state
            stream.updateStreamState(state: .open, protocol: proto)
            // Return the configured stream
            return stream as _Stream
        }
    }

    public func openStream(_ stream: inout LibP2PCore.Stream) throws -> EventLoopFuture<Void> {
        self.logger.trace("YAMUX::TODO: Open Stream")
        return self.channel!.eventLoop.makeFailedFuture(YAMUX.Error.unsupportedVersion(""))
    }

    public func getStream(id: UInt64, mode: LibP2PCore.Mode) -> EventLoopFuture<LibP2PCore.Stream?> {
        self.channel!.eventLoop.submit {
            self.multiplexer?.channels[UInt32(id)]
        }
    }

    public func updateStream(channel: Channel, state: LibP2PCore.StreamState, proto: String) -> EventLoopFuture<Void> {
        self.logger.trace("YAMUX::Stream[\(proto)]::Update Stream State -> \(state)")

        return self.channel!.eventLoop.submit {
            if let idx = self.multiplexer?.channels.first(where: { $1.channel === channel }) {
                self.multiplexer?.channels[idx.key]?.updateStreamState(state: state, protocol: proto)
            } else {
                self.logger.error("Unknown Child Channel / Stream")
            }
        }
    }

    public func removeStream(channel childChannel: Channel) {
        guard let c = self.channel else { return }
        self.logger.trace("Attempting to Remove Stream")
        let promise = c.eventLoop.makePromise(of: Void.self)

        promise.futureResult.whenComplete { result in
            self.logger.trace("Removed Stream -> \(result)")
        }

        c.eventLoop.execute {
            if let (_, stream) = self.multiplexer?.channels.first(where: { $0.value._channel === childChannel }) {
                promise.completeWith(stream.close(gracefully: true))
            } else {
                promise.fail(YAMUX.Error.unknownChildChannel)
            }
        }

        return
    }

    public func closeAllStreams(promise: EventLoopPromise<Void>) {
        guard let channel = self.channel, let multiplexer else {
            promise.fail(YAMUX.Error.tcpShutdown)
            return
        }

        let close = multiplexer.shouldQuiesce(on: channel.eventLoop).always { res in
            if let frame = try? self.sessionState.shouldClose() {
                self.writeFromChildChannel(frame, nil)
            }
        }

        promise.completeWith(close)
    }
}

// MARK: Helper methods for managing Session state (channel 0)

extension ChildChannelStateMachine {
    fileprivate mutating func initialize(mode: LibP2P.Mode) -> Frame? {
        switch mode {
        case .initiator:
            // wait for ChannelOpen ID == 0 message
            return nil
        case .listener:
            // Spin the state machine
            self.sendChannelOpen(.init(senderChannel: 0, initialWindowSize: 0, maximumPacketSize: 0))
            // return the frame to send
            return .init(header: .init(version: .v0, messageType: .ping, flags: [.syn], streamID: 0, length: 0))
        }
    }

    fileprivate mutating func receivedSessionOpen(mode: LibP2P.Mode) throws -> Frame {
        switch mode {
        case .initiator:
            // Receive the channel open message
            self.receiveChannelOpen(.init(senderChannel: 0, initialWindowSize: 0, maximumPacketSize: 0))
            // Send the channel open confirmation message
            self.sendChannelOpenConfirmation(
                .init(recipientChannel: 0, senderChannel: 0, initialWindowSize: 0, maximumPacketSize: 0)
            )
            // Return the ack frame
            return .init(header: .init(version: .v0, messageType: .ping, flags: [.ack], streamID: 0, length: 0))
        case .listener:
            // 'receive' the channel open confirmation message
            try self.receiveChannelOpenConfirmation(
                .init(recipientChannel: 0, senderChannel: 0, initialWindowSize: 0, maximumPacketSize: 0)
            )
            // Return the ack frame
            return .init(header: .init(version: .v0, messageType: .ping, flags: [.ack], streamID: 0, length: 0))
        }
    }

    fileprivate mutating func receivedSessionOpenConfirmation() throws {
        try self.receiveChannelOpenConfirmation(
            .init(recipientChannel: 0, senderChannel: 0, initialWindowSize: 0, maximumPacketSize: 0)
        )
    }

    fileprivate mutating func shouldClose() throws -> Frame? {
        // Update our state machine
        try self.sendChannelClose(.init(recipientChannel: 0))
        try self.receiveChannelClose(.init(recipientChannel: 0))
        // Return the goAway frame with noError code
        return .init(
            header: .init(
                version: .v0,
                messageType: .goAway,
                flags: [],
                streamID: 0,
                length: YAMUX.NetworkError.noError.code
            )
        )
    }

    fileprivate mutating func shouldReset(error: YAMUX.NetworkError) throws -> Frame {
        // Update our state machine
        try self.sendChannelReset(.init(recipientChannel: 0, reasonCode: error.code, description: "\(error)"))
        // Return the goAway frame with the error that cause the reset
        return .init(header: .init(version: .v0, messageType: .goAway, flags: [], streamID: 0, length: error.code))
    }

    fileprivate mutating func receivedDisconnect(error: YAMUX.NetworkError) throws {
        // Update our state machine
        try self.receiveChannelReset(.init(recipientChannel: 0, reasonCode: error.code, description: "\(error)"))
        // Return the reset frame
        //return .init(header: .init(version: .v0, messageType: .goAway, flags: [], streamID: 0, length: error.code))
    }
}
