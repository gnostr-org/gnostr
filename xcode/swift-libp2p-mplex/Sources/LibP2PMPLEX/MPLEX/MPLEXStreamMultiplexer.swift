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
//===----------------------------------------------------------------------===//
//
// This source file is part of the SwiftNIO open source project
//
// Copyright (c) 2017-2021 Apple Inc. and the SwiftNIO project authors
// Licensed under Apache License v2.0
//
// See LICENSE.txt for license information
// See CONTRIBUTORS.txt for the list of SwiftNIO project authors
//
// SPDX-License-Identifier: Apache-2.0
//
//===----------------------------------------------------------------------===//

import Foundation
import LibP2P
import NIOCore

/// MPLEX spawns a new channel / pipeline for each stream (similar to HTTP2)
///
/// Each stream having its own pipeline makes adding route specific middleware straightforward and easy (vairous delimiters, object deserailization, length prefixs, etc...)
internal enum MPLEXFlag: UInt64 {
    case NewStream = 0
    case MessageReceiver = 1
    case MessageInitiator = 2
    case CloseReceiver = 3
    case CloseInitiator = 4
    case ResetReceiver = 5
    case ResetInitiator = 6
}

public struct MPLEXStreamID: Hashable, Sendable {
    let id: UInt64
    let initiator: Bool

    init(id: UInt64, flag: MPLEXFlag) {
        self.id = id
        switch flag {
        case .NewStream, .MessageInitiator, .CloseInitiator, .ResetInitiator:
            self.initiator = false
        default:
            self.initiator = true
        }
    }

    init(id: UInt64, mode: LibP2P.Mode) {
        self.id = id
        self.initiator = mode == .initiator ? true : false
    }

    var description: String {
        "[\(id)][\(self.initiator ? "Outbound" : "Inbound")]"
    }
}

/// A channel handler that creates a child channel for each MPLEX stream.
///
/// In general in NIO applications it is helpful to consider each MPLEX stream as an
/// independent stream of MPELX frames. This multiplexer achieves this by creating a
/// number of in-memory `MPLEXStreamChannel` objects, one for each stream. These operate
/// on `MPLEXFrame` objects as their base communication atom, as opposed to the regular
/// NIO `SelectableChannel` objects which use `ByteBuffer` and `IOData`.
public final class MPLEXStreamMultiplexer: ChannelInboundHandler, ChannelOutboundHandler, MessageExtractableHandler {
    public static let protocolCodec: String = "/mplex/6.7.0"

    public typealias InboundIn = MPLEXFrame
    public typealias InboundOut = MPLEXFrame
    public typealias OutboundIn = MPLEXFrame
    public typealias OutboundOut = MPLEXFrame

    /// Muxer Callbacks and Delegates
    public var onStream: ((_Stream) -> Void)? = nil
    public var onStreamEnd: ((LibP2PCore.Stream) -> Void)? = nil
    public var _connection: Connection?

    let localPeerID: PeerID

    // Streams which have a stream ID.
    // - Note: MPLEX supports streams with the same ID to be opened by either party, therefore a simple ID hashmap wont work...
    //private var streams: [MPLEXStreamID: MultiplexerAbstractChannel] = [:]
    private var _streams: [MPLEXStreamID: MultiplexerAbstractChannel] = [:]
    //    var streams: [MultiplexerAbstractChannel] {
    //        _streams.map { $0.value }
    //    }
    private var streamMap: [MPLEXStreamID: MPLEXStream] = [:]

    private var supportedProtocols: [LibP2P.ProtocolRegistration] = []

    // Streams which don't yet have a stream ID assigned to them.
    private var pendingStreams: [ObjectIdentifier: MultiplexerAbstractChannel] = [:]

    struct RegisteredProtocol {
        let protocolString: String
        let initializer: () -> [ChannelHandler]
    }

    // Each supported / registered handler will have it's own initializer, this should be a map of those...
    // Actually this will just be another instance of MSS
    private var inboundStreamStateInitializer: MultiplexerAbstractChannel.InboundStreamStateInitializer!

    /// The main channel this muxer is installed on
    private let channel: Channel
    private var context: ChannelHandlerContext!

    /// A helper function for gathering the next StreamID to use
    private var nextOutboundStreamID: MPLEXStreamID

    /// Flow control stuff...
    private var flushState: FlushState = .notReading
    private var didReadChannels: MPLEXStreamChannelList = MPLEXStreamChannelList()

    /// The promise to succeed once we're up and running on the pipeline
    private var muxedPromise: EventLoopPromise<Muxer>!

    /// The logger tied to our underlying connection
    private var logger: Logger

    public func handlerAdded(context: ChannelHandlerContext) {
        // We now need to check that we're on the same event loop as the one we were originally given.
        // If we weren't, this is a hard failure, as there is a thread-safety issue here.
        self.channel.eventLoop.preconditionInEventLoop()
        self.context = context
        if context.channel.isActive {
            self.channelActive(context: context)
        }
        // We push the call to `succeed` onto the end of the eventloop stack so we have time to remove upgraders
        // The problem with pushing this succeed onto the stack is that we can start attempting to upgrade a new childChannel before the connection know's it's muxed...
        //context.eventLoop.execute {
        self.muxedPromise.succeed(self)
        //}
    }

    public func handlerRemoved(context: ChannelHandlerContext) {
        logger.trace("MPLEXStreamMultiplexer:Removed...")
        self.context = nil
        self.inboundStreamStateInitializer = nil
        self.muxedPromise = nil
        self.didReadChannels.removeAll()
    }

    // TODO: We should have a frame decoder sitting in front of this handler that parses out individual MPLEX Frames
    public func channelRead(context: ChannelHandlerContext, data: NIOAny) {
        let frame = self.unwrapInboundIn(data)
        // We need to strip the MPLEX Stream ID off the front of the payload
        //let streamID = frame.streamID

        logger.trace(
            "MPLEXStreamMultiplexer:Frame Received -> ID: \(frame.streamID), flag: \(frame.flag), payload: \(frame.payload.bytes.asString(base: .base16))"
        )

        self.flushState.startReading()

        /// If the child channel already exists, forward the message along...
        if let channel = self._streams[frame.streamID] {

            if case .close = frame.payload, let stream = self.streamMap[frame.streamID] {
                logger.trace("Found an existing stream... state == \(stream.streamState)")
                if stream._streamState.withLockedValue({ $0 }) == .writeClosed {
                    logger.trace(
                        "We received a close message on a stream that was already half closed. Shutting down channel."
                    )
                    channel.receiveStreamClosed(nil)
                    channel.channel.close(mode: .all, promise: nil)
                    self.streamMap.removeValue(forKey: frame.streamID)
                    self._streams.removeValue(forKey: frame.streamID)
                    self.onStreamEnd?(stream)
                    return
                } else {
                    //logger.trace("We received a close message on a stream. Responding with close frame")
                    //context.writeAndFlush( self.wrapOutboundOut(MPLEXFrame(streamID: frame.streamID, payload: .close)) , promise: nil)
                    logger.trace("Alerting ChildChannel of close")
                    channel.receiveStreamClosed(nil)
                    stream._streamState.withLockedValue { $0 = .receiveClosed }
                    self.onStreamEnd?(stream)
                    return
                }
            }

            if case .reset = frame.payload {
                logger.trace("Existing stream needs to be reset")
                channel.receiveStreamClosed(.streamClosed)
                channel.channel.close(mode: .all, promise: nil)
                let stream = self.streamMap.removeValue(forKey: frame.streamID)
                self._streams.removeValue(forKey: frame.streamID)
                if let stream = stream { self.onStreamEnd?(stream) }
                return
            }

            channel.receiveInboundFrame(frame)
            if !channel.inList {
                self.didReadChannels.append(channel)
            }
            /// If the frame is requesting a new stream, instantiate a new channel...
        } else if case .newStream = frame.payload {
            logger.trace("Remote requesting NewStream with ID:\(frame.streamID)")

            if self._streams[frame.streamID] != nil {
                logger.warning("Remote Requested New Stream with Existing ID: \(frame.streamID)!!")
            }

            let channel = MultiplexerAbstractChannel(
                allocator: self.channel.allocator,
                parent: self.channel,
                multiplexer: self,
                streamID: frame.streamID,
                // Install MSS with the supported/registered protocols...
                inboundStreamStateInitializer: self.inboundStreamStateInitializer
            )

            self._streams[frame.streamID] = channel
            let stream = MPLEXStream(
                channel: channel.channel,
                mode: .listener,
                id: frame.streamID.id,
                name: "MPLEXStream\(frame.streamID.id)",
                proto: ""
            )
            self.streamMap[frame.streamID] = stream
            channel.configureInboundStream(initializer: self.inboundStreamStateInitializer)
            //channel.receiveInboundFrame(frame)
            if !channel.inList {
                self.didReadChannels.append(channel)
            }
            self.onStream?(stream)
        } else {
            // This frame is for a stream we know nothing about. We can't do much about it, so we
            // are going to fire an error and drop the frame.
            let error = NIOMPLEXErrors.noSuchStream(streamID: frame.streamID)
            logger.error("MPLEXStreamMultiplexer:Error::No Such Stream: \(error)")
            context.fireErrorCaught(error)
        }
    }

    public func updateStream(channel: Channel, state: LibP2PCore.StreamState, proto: String) -> EventLoopFuture<Void> {
        self.channel.eventLoop.submit {
            if let idx = self.streamMap.first(where: { $1.channel === channel }) {
                self.streamMap[idx.key]?.updateStreamState(state: state, protocol: proto)
            } else {
                self.logger.error("Unknown Child Channel Stream")
            }
        }
    }

    public func channelReadComplete(context: ChannelHandlerContext) {
        // Call channelReadComplete on the children until this has been propagated enough.
        while let channel = self.didReadChannels.removeFirst() {
            channel.receiveParentChannelReadComplete()
        }

        if case .flushPending = self.flushState {
            self.flushState = .notReading
            context.flush()
        } else {
            self.flushState = .notReading
        }

        context.fireChannelReadComplete()
    }

    public func flush(context: ChannelHandlerContext) {
        switch self.flushState {
        case .reading, .flushPending:
            self.flushState = .flushPending
        case .notReading:
            context.flush()
        }
    }

    public func write(context: ChannelHandlerContext, data: NIOAny, promise: EventLoopPromise<Void>?) {
        // for now just forward
        context.write(data, promise: promise)
    }

    public func channelActive(context: ChannelHandlerContext) {
        logger.trace("MPLEX::ChannelActive 🟢")
        // We just got channelActive. Any previously existing channels may be marked active.
        self.activateChannels(self._streams.values, context: context)
        self.activateChannels(self.pendingStreams.values, context: context)

        context.fireChannelActive()
    }

    private func activateChannels<Channels: Sequence>(_ channels: Channels, context: ChannelHandlerContext)
    where Channels.Element == MultiplexerAbstractChannel {
        for channel in channels {
            // We double-check the channel activity here, because it's possible action taken during
            // the activation of one of the child channels will cause the parent to close!
            if context.channel.isActive {
                channel.performActivation()
            }
        }
    }

    public func channelInactive(context: ChannelHandlerContext) {
        logger.trace("MPLEX::ChannelInactive 🔴")
        //for channel in self.streams.values {
        //    channel.receiveStreamClosed(nil)
        //}
        self.inactivateChannels(self._streams.values, context: context)
        //for channel in self.pendingStreams.values {
        //    channel.receiveStreamClosed(nil)
        //}
        self.inactivateChannels(self.pendingStreams.values, context: context)

        context.fireChannelInactive()
    }

    private func inactivateChannels<Channels: Sequence>(_ channels: Channels, context: ChannelHandlerContext)
    where Channels.Element == MultiplexerAbstractChannel {
        for channel in channels {
            channel.receiveStreamClosed(nil)
        }
    }

    /// If we want to intercept MPLEX message flags and handle closing children here we could, otherwise we have a
    /// system where the child channel informs the muxer that it needs to close
    public func userInboundEventTriggered(context: ChannelHandlerContext, event: Any) {
        switch event {
        case let evt as StreamClosedEvent:
            if let channel = self._streams[evt.streamID] {
                channel.receiveStreamClosed(evt.reason)
            }
        case let evt as NIOMPLEXStreamCreatedEvent:
            if let channel = self._streams[evt.streamID] {
                channel.networkActivationReceived()
            }
        default:
            break
        }

        context.fireUserInboundEventTriggered(event)
    }

    /// Create a new `MPLEXStreamMultiplexer`.
    public convenience init(
        connection: Connection,
        muxedPromise: EventLoopPromise<Muxer>,
        supportedProtocols: [LibP2P.ProtocolRegistration]
    ) {
        self.init(
            connection: connection,
            supportedProtocols: supportedProtocols,
            inboundStreamStateInitializer: .excludesStreamID(connection.inboundMuxedChildChannelInitializer),
            muxedPromise: muxedPromise
        )
    }

    /// Create a new `MPLEXStreamMultiplexer`.
    public convenience init(
        connection: Connection,
        inboundStreamInitializer: ((Channel) -> EventLoopFuture<Void>)?,
        muxedPromise: EventLoopPromise<Muxer>
    ) {
        self.init(
            connection: connection,
            inboundStreamStateInitializer: .excludesStreamID(inboundStreamInitializer),
            muxedPromise: muxedPromise
        )
    }

    private init(
        connection: Connection,
        supportedProtocols: [LibP2P.ProtocolRegistration] = [],
        inboundStreamStateInitializer: MultiplexerAbstractChannel.InboundStreamStateInitializer,
        muxedPromise: EventLoopPromise<Muxer>
    ) {
        self.inboundStreamStateInitializer = inboundStreamStateInitializer
        self._connection = connection
        self.channel = connection.channel
        self.nextOutboundStreamID = MPLEXStreamID(id: 0, mode: .initiator)
        self.supportedProtocols = supportedProtocols
        self.localPeerID = connection.localPeer
        self.muxedPromise = muxedPromise
        self.logger = connection.logger
        self.logger[metadataKey: "MPLEX"] = .string("Parent")
    }
}

// A `StreamClosedEvent` is fired whenever a stream is closed.
///
/// This event is fired whether the stream is closed normally, or via RST_STREAM,
/// or via GOAWAY. Normal closure is indicated by having `reason` be `nil`. In the
/// case of closure by GOAWAY the `reason` is always `.refusedStream`, indicating that
/// the remote peer has not processed this stream. In the case of RST_STREAM,
/// the `reason` contains the error code sent by the peer in the RST_STREAM frame.
public struct StreamClosedEvent {
    /// The stream ID of the stream that is closed.
    public let streamID: MPLEXStreamID

    /// The reason for the stream closure. `nil` if the stream was closed without
    /// error. Otherwise, the error code indicating why the stream was closed.
    public let reason: MPLEXErrorCode?

    public init(streamID: MPLEXStreamID, reason: MPLEXErrorCode?) {
        self.streamID = streamID
        self.reason = reason
    }
}

extension StreamClosedEvent: Hashable {}

/// A `NIOMPLEXStreamCreatedEvent` is fired whenever an MPLEX stream is created.
public struct NIOMPLEXStreamCreatedEvent {
    public let streamID: MPLEXStreamID

    public init(streamID: MPLEXStreamID) {
        self.streamID = streamID
    }
}

extension NIOMPLEXStreamCreatedEvent: Hashable {}

//extension MPLEXStreamMultiplexer {
extension MPLEXStreamMultiplexer {
    /// The state of the multiplexer for flush coalescing.
    ///
    /// The stream multiplexer aims to perform limited flush coalescing on the read side by delaying flushes from the child and
    /// parent channels until channelReadComplete is received. To do this we need to track what state we're in.
    enum FlushState {
        /// No channelReads have been fired since the last channelReadComplete, so we probably aren't reading. Let any
        /// flushes through.
        case notReading

        /// We've started reading, but don't have any pending flushes.
        case reading

        /// We're in the read loop, and have received a flush.
        case flushPending

        mutating func startReading() {
            if case .notReading = self {
                self = .reading
            }
        }
    }
}

//extension MPLEXStreamMultiplexer {
extension MPLEXStreamMultiplexer {
    /// Create a new `Channel` for a new stream initiated by this peer.
    ///
    /// This method is intended for situations where the NIO application is initiating the stream. For clients,
    /// this is for all request streams. For servers, this is for pushed streams.
    ///
    /// - note:
    /// Resources for the stream will be freed after it has been closed.
    ///
    /// - parameters:
    ///     - promise: An `EventLoopPromise` that will be succeeded with the new activated channel, or
    ///         failed if an error occurs.
    ///     - streamStateInitializer: A callback that will be invoked to allow you to configure the
    ///         `ChannelPipeline` for the newly created channel.
    public func createStreamChannel(
        promise: EventLoopPromise<Channel>?,
        streamID: MPLEXStreamID,
        _ streamStateInitializer: @escaping (Channel, MPLEXStreamID) -> EventLoopFuture<Void>
    ) {
        self.channel.eventLoop.execute {
            let channel = MultiplexerAbstractChannel(
                allocator: self.channel.allocator,
                parent: self.channel,
                multiplexer: self,
                streamID: streamID,
                inboundStreamStateInitializer: .includesStreamID(nil)  //.excludesStreamID(nil)
            )
            self._streams[streamID] = channel
            //self.pendingStreams[channel.channelID] = channel
            channel.configure(initializer: streamStateInitializer, userPromise: promise)
        }
    }

    private func nextStreamID() -> MPLEXStreamID {
        let streamID = self.nextOutboundStreamID
        self.nextOutboundStreamID = MPLEXStreamID(id: streamID.id + 1, mode: .initiator)
        return streamID
    }
}

// MARK:- Child to parent calls
//extension MPLEXStreamMultiplexer {
extension MPLEXStreamMultiplexer {
    internal func childChannelClosed(streamID: MPLEXStreamID) {
        //print("childChannelClosed() _Streams[\(self._streams.count)] StreamMap[\(self.streamMap.count)]")
        self._streams.removeValue(forKey: streamID)
        self.streamMap.removeValue(forKey: streamID)
    }

    internal func childChannelClosed(channelID: ObjectIdentifier) {
        self.pendingStreams.removeValue(forKey: channelID)
    }

    internal func childChannelWrite(_ frame: MPLEXFrame, promise: EventLoopPromise<Void>?) {
        //logger.trace("Child Channel Writing: Stream[\(frame.streamID.id)] -> \(Array<UInt8>(frame.messageBytes().readableBytesView).asString(base: .base16))")
        self.context.write(self.wrapOutboundOut(frame), promise: promise)
    }

    internal func childChannelWriteClosed(_ id: MPLEXStreamID) {
        guard let str = self.streamMap[id] else { return }
        switch str._streamState.withLockedValue({ $0 }) {
        case .initialized, .open, .receiveClosed:
            str._streamState.withLockedValue { $0 = .writeClosed }
        case .writeClosed, .closed, .reset:
            print(
                "MPLEXStreamMultiplexer::ERROR:Invalid child channel stream state transition \(str.streamState) -> .writeClosed"
            )
            return
        }
    }

    internal func childChannelFlush() {
        self.flush(context: self.context)
    }

    /// Requests a `MPLEXStreamID` for the given `Channel`.
    ///
    /// - Precondition: The `channel` must not already have a `streamID`.
    internal func requestStreamID(forChannel channel: Channel) -> MPLEXStreamID {
        let channelID = ObjectIdentifier(channel)

        // This unwrap shouldn't fail: the multiplexer owns the stream and the stream only requests
        // a streamID once.
        guard let abstractChannel = self.pendingStreams.removeValue(forKey: channelID) else {
            preconditionFailure("No pending streams have channelID \(channelID)")
        }
        assert(abstractChannel.channelID == channelID)

        let streamID = self.nextStreamID()
        self._streams[streamID] = abstractChannel
        return streamID
    }
}

extension MPLEXStreamMultiplexer: Muxer {
    var muxer: Muxer {
        self
    }

    public var streams: [LibP2PCore.Stream] {
        self.streamMap.map { $0.value }
    }

    enum Errors: Error {
        case unsupportedProtocol
    }

    //    public func newStream(channel: Channel, proto: LibP2P.ProtocolRegistration) throws -> EventLoopFuture<_Stream> {
    //
    //    }

    public func newStream(channel: Channel, proto: String) throws -> EventLoopFuture<_Stream> {
        //        guard let reg = self.supportedProtocols.first(where: { $0.protocolString() == proto } ) else {
        //            logger.error("Error: Asked to open stream for unsupported protocol '\(proto)'")
        //            logger.error(self.supportedProtocols.map { $0.protocolString() }.joined(separator: ", "))
        //            throw Errors.unsupportedProtocol
        //        }

        let streamPromise = channel.eventLoop.makePromise(of: _Stream.self)
        let channelPromise = channel.eventLoop.makePromise(of: Channel.self)
        let streamID = nextStreamID()

        channelPromise.futureResult.whenComplete { result in
            self.logger.trace("ChannelPromise stream[\(streamID.id)] Finished")
            switch result {
            case .failure(let error):
                self.logger.error("Error opening newStream ID:\(streamID.id)")
                streamPromise.fail(error)
            case .success(let ch):
                self.logger.trace("Opened new stream[\(streamID.id)] proceeding with MSS negotiation...")
                let stream = MPLEXStream(
                    channel: ch,
                    mode: .initiator,
                    id: streamID.id,
                    name: "MPLEXStream\(streamID.id)",
                    proto: proto
                )
                self.streamMap[streamID] = stream
                self.onStream?(stream)
                streamPromise.succeed(stream)
            }
        }

        logger.trace("Attempting to open new stream with ID:\(streamID.id)")
        /// Send the NewStream frame
        self.channel.writeAndFlush(
            self.wrapOutboundOut(MPLEXFrame(streamID: streamID, payload: .newStream)),
            promise: nil
        )

        self.createStreamChannel(promise: channelPromise, streamID: streamID) { chan, _ in
            self._connection!.outboundMuxedChildChannelInitializer(chan, protocol: proto)
        }

        return streamPromise.futureResult
    }

    public func openStream(_ stream: inout LibP2PCore.Stream) throws -> EventLoopFuture<Void> {
        throw NSError(domain: "Not Yet Implemented", code: 0, userInfo: nil)
    }

    public func getStream(id: UInt64, mode: LibP2P.Mode) -> EventLoopFuture<LibP2PCore.Stream?> {
        self.channel.eventLoop.submit {
            let streamID = MPLEXStreamID(id: id, mode: mode)
            return self.streamMap[streamID]
        }
    }

    public func removeStream(channel: Channel) {
        if let str = self.streamMap.first(where: { $0.value.channel === channel }) {
            self.logger.info(
                "Attempting to remove stream Stream[\(str.value.id)][\(str.value.protocolCodec)][\(str.value.name ?? "``")][\(str.value.mode)]"
            )
            str.value.channel.close(mode: .all, promise: nil)
            self.streamMap.removeValue(forKey: str.key)
            // Instantiate our StreamID
            let streamID = MPLEXStreamID(id: str.value.id, mode: str.value.mode)
            // Ensure the stream actually existed...
            if (self._streams.removeValue(forKey: streamID)) != nil {
                // Send a reset frame...
                //self.channel.writeAndFlush(self.wrapOutboundOut( MPLEXFrame(streamID: streamID, payload: .reset) ), promise: nil)
                self.logger.info(
                    "Removed Stream[\(str.value.id)][\(str.value.protocolCodec)][\(str.value.name ?? "``")][\(str.value.mode)]"
                )
            } else {
                self.logger.warning(
                    "Failed to remove Stream[\(str.value.id)][\(str.value.protocolCodec)][\(str.value.name ?? "``")][\(str.value.mode)]"
                )
            }
        } else {
            self.logger.warning("Failed to find requested stream to remove")
        }
    }

}
