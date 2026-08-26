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

import LibP2P

/// The various channel options specific to `MPLEXStreamChannel`s.
///
/// Please note that some of NIO's regular `ChannelOptions` are valid on `MPLEXStreamChannel`s.
public struct MPLEXStreamChannelOptions {
    /// - seealso: `StreamIDOption`.
    public static let streamID: MPLEXStreamChannelOptions.Types.StreamIDOption = .init()
}

extension MPLEXStreamChannelOptions {
    public enum Types {}
}

extension MPLEXStreamChannelOptions.Types {
    /// `StreamIDOption` allows users to query the stream ID for a given `MPLEXStreamChannel`.
    ///
    /// On active `MPLEXStreamChannel`s, it is possible that a channel handler or user may need to know which
    /// stream ID the channel owns. This channel option allows that query. Please note that this channel option
    /// is *get-only*: that is, it cannot be used with `setOption`. The stream ID for a given `MPLEXStreamChannel`
    /// is immutable.
    ///
    /// If a channel is not active, the stream ID will not be present, and attempting to
    /// get this channel option will fail.
    public struct StreamIDOption: ChannelOption {
        public typealias Value = MPLEXStreamID

        public init() {}
    }
}

/// The current state of a stream channel.
private enum StreamChannelState {
    /// The stream has been created, but not configured.
    case idle

    /// The is "active": we haven't sent channelActive yet, but it exists on the network and any shutdown must cause a frame to be emitted.
    case remoteActive

    /// This is also "active", but different to the above: we've sent channelActive, but the NIOMPLEXHandler hasn't seen the frame yet,
    /// and so we can close this channel without action if needed.
    case localActive

    /// This is actually active: channelActive has been fired and the MPLEXHandler believes this stream exists.
    case active

    /// We are closing from a state where channelActive had been fired. In practice this is only ever active, as
    /// in localActive we transition directly to closed.
    case closing

    /// We're closing from a state where we have never fired channel active, but where the channel was on the network.
    /// This means we need to send frames and wait for their side effects.
    case closingNeverActivated

    /// We're fully closed.
    case closed

    mutating func activate() {
        switch self {
        case .idle:
            self = .localActive
        case .remoteActive:
            self = .active
        case .localActive, .active, .closing, .closingNeverActivated, .closed:
            preconditionFailure("Became active from state \(self)")
        }
    }

    mutating func networkActive() {
        switch self {
        case .idle:
            self = .remoteActive
        case .localActive:
            self = .active
        case .closed:
            preconditionFailure("Stream must be reset on network activation when closed")
        case .remoteActive, .active, .closing, .closingNeverActivated:
            preconditionFailure("Cannot become network active twice, in state \(self)")
        }
    }

    mutating func beginClosing() {
        switch self {
        case .active, .closing:
            self = .closing
        case .closingNeverActivated, .remoteActive:
            self = .closingNeverActivated
        case .idle, .localActive:
            preconditionFailure("Idle streams immediately close")
        case .closed:
            preconditionFailure("Cannot begin closing while closed")
        }
    }

    mutating func completeClosing() {
        switch self {
        case .idle, .remoteActive, .closing, .closingNeverActivated, .active, .localActive:
            self = .closed
        case .closed:
            preconditionFailure("Complete closing from \(self)")
        }
    }
}

public struct MPLEXFrame: Equatable, Sendable {
    /// The streams ID
    let streamID: MPLEXStreamID

    /// The payload of this frame
    let payload: FramePayload

    enum FramePayload: Equatable {
        case inboundData(ByteBuffer)
        case outboundData(ByteBuffer)
        case close
        case reset
        case newStream

        var bytes: [UInt8] {
            switch self {
            case .inboundData(let payload):
                return [UInt8](payload.readableBytesView)
            case .outboundData(let payload):
                return [UInt8](payload.readableBytesView)
            default:
                return []
            }
        }

        var buffer: ByteBuffer {
            switch self {
            case .inboundData(let payload):
                return payload
            case .outboundData(let payload):
                return payload
            default:
                return ByteBuffer()
            }
        }
    }

    var flag: MPLEXFlag {
        switch payload {
        case .newStream:
            return .NewStream
        case .inboundData, .outboundData:
            return streamID.initiator ? .MessageInitiator : .MessageReceiver
        case .close:
            return streamID.initiator ? .CloseInitiator : .CloseReceiver
        case .reset:
            return streamID.initiator ? .ResetInitiator : .ResetReceiver
        }
    }
}

extension MPLEXFrame: MessageExtractable {
    func messageBytes() -> ByteBuffer {
        self.payload.buffer
    }
}

extension MPLEXFrame.FramePayload {
    /// A shorthand heuristic for how many bytes we assume a frame consumes on the wire.
    ///
    /// Here we concern ourselves only with per-stream frames: that is, `HEADERS`, `DATA`,
    /// `WINDOW_UDPATE`, `RST_STREAM`, and I guess `PRIORITY`. As a simple heuristic we
    /// hard code fixed lengths for fixed length frames, use a calculated length for
    /// variable length frames, and just ignore encoded headers because it's not worth doing a better
    /// job.
    var estimatedFrameSize: Int {
        let frameHeaderSize = 1

        switch self {
        case .inboundData(let d):
            return d.readableBytes + frameHeaderSize
        default:
            // Unknown or unexpected control frame: say 9 bytes.
            return frameHeaderSize
        }
    }
}

// The type of data read from and written to the channel.
enum MPLEXStreamDataType {
    /// `MPLEXFrame`
    case frame
    /// `MPLEXFrame.FramePayload`
    case framePayload
}

private enum MPLEXStreamData {
    case frame(MPLEXFrame)
    case framePayload(MPLEXFrame.FramePayload)

    var estimatedFrameSize: Int {
        switch self {
        case .frame(let frame):
            return frame.payload.estimatedFrameSize
        case .framePayload(let payload):
            return payload.estimatedFrameSize
        }
    }
}

final class MPLEXStreamChannel: Channel, ChannelCore, @unchecked Sendable {
    /// The stream data type of the channel.
    private let streamDataType: MPLEXStreamDataType

    weak var channel: Channel! {
        self
    }

    internal init(
        allocator: ByteBufferAllocator,
        parent: Channel,
        multiplexer: MPLEXStreamMultiplexer,
        streamID: MPLEXStreamID?,
        streamDataType: MPLEXStreamDataType
    ) {
        self.allocator = allocator
        self.closePromise = parent.eventLoop.makePromise()
        self.parent = parent
        self.eventLoop = parent.eventLoop
        self.streamID = streamID
        self.multiplexer = multiplexer
        self._isActiveAtomic = .makeAtomic(value: false)
        self._isWritable = .makeAtomic(value: true)
        self.state = .idle
        self.streamDataType = streamDataType

        // To begin with we initialize autoRead to false, but we are going to fetch it from our parent before we
        // go much further.
        self.autoRead = false
        self._pipeline = ChannelPipeline(channel: self)

        //print("MPLEXStreamChannel::Initialized")
        //print("MPLEXStreamChannel::\(remoteAddress?.description ?? "NIL")")
    }

    //deinit {
    //    print("MPLEXStreamChannel::Deinitialized")
    //}

    internal func configure(
        initializer: ((Channel, MPLEXStreamID) -> EventLoopFuture<Void>)?,
        userPromise promise: EventLoopPromise<Channel>?
    ) {
        assert(self.streamDataType == .frame)
        // We need to configure this channel. This involves doing four things:
        // 1. Setting our autoRead state from the parent
        // 2. Calling the initializer, if provided.
        // 3. Activating when complete.
        // 4. Catching errors if they occur.
        self.getAutoReadFromParent { autoReadResult in
            //guard let self = self else { fatalError("MPLEXStreamChannel::Lost reference to self") }
            switch autoReadResult {
            case .success(let autoRead):
                self.autoRead = autoRead
                if let initializer = initializer {
                    // This initializer callback can only be invoked if we already have a stream ID.
                    // So we force-unwrap here.
                    initializer(self, self.streamID!).whenComplete { result in
                        switch result {
                        case .success:
                            self.postInitializerActivate(promise: promise)
                        case .failure(let error):
                            self.configurationFailed(withError: error, promise: promise)
                        }
                    }
                } else {
                    self.postInitializerActivate(promise: promise)
                }
            case .failure(let error):
                self.configurationFailed(withError: error, promise: promise)
            }
        }
    }

    internal func configure(
        initializer: ((Channel) -> EventLoopFuture<Void>)?,
        userPromise promise: EventLoopPromise<Channel>?
    ) {
        assert(self.streamDataType == .framePayload)
        // We need to configure this channel. This involves doing four things:
        // 1. Setting our autoRead state from the parent
        // 2. Calling the initializer, if provided.
        // 3. Activating when complete.
        // 4. Catching errors if they occur.
        self.getAutoReadFromParent { autoReadResult in
            //guard let self = self else { fatalError("MPLEXStreamChannel::Lost reference to self") }
            switch autoReadResult {
            case .success(let autoRead):
                self.autoRead = autoRead
                if let initializer = initializer {
                    initializer(self).whenComplete { result in
                        switch result {
                        case .success:
                            self.postInitializerActivate(promise: promise)
                        case .failure(let error):
                            self.configurationFailed(withError: error, promise: promise)
                        }
                    }
                } else {
                    self.postInitializerActivate(promise: promise)
                }
            case .failure(let error):
                self.configurationFailed(withError: error, promise: promise)
            }
        }
    }

    /// Gets the 'autoRead' option from the parent channel and invokes the `body` closure with the
    /// result. This may be done synchronously if the parent `Channel` supports synchronous options.
    private func getAutoReadFromParent(_ body: @escaping (Result<Bool, Error>) -> Void) {
        // This force unwrap is safe as parent is assigned in the initializer, and never unassigned.
        // Note we also don't set the value here: the additional `map` causes an extra allocation
        // when using a Swift 5.0 compiler.
        if let syncOptions = self.parent!.syncOptions {
            let autoRead = Result(catching: { try syncOptions.getOption(ChannelOptions.autoRead) })
            body(autoRead)
        } else {
            self.parent!.getOption(ChannelOptions.autoRead).whenComplete { autoRead in
                body(autoRead)
            }
        }
    }

    /// Activates the channel if the parent channel is active and succeeds the given `promise`.
    private func postInitializerActivate(promise: EventLoopPromise<Channel>?) {
        // This force unwrap is safe as parent is assigned in the initializer, and never unassigned.
        // If parent is not active, we expect to receive a channelActive later.
        if self.parent!.isActive {
            self.performActivation()
        }

        // We aren't using cascade here to avoid the allocations it causes.
        promise?.succeed(self)
    }

    /// Handle any error that occurred during configuration.
    private func configurationFailed(withError error: Error, promise: EventLoopPromise<Channel>?) {
        switch self.state {
        case .idle, .localActive, .closed:
            // The stream isn't open on the network, nothing to close.
            self.errorEncountered(error: error)
        case .remoteActive, .active, .closing, .closingNeverActivated:
            // In all of these states the stream is still on the network and we may need to take action.
            self.closedWhileOpen()
        }

        promise?.fail(error)
    }

    /// Activates this channel.
    internal func performActivation() {
        precondition(self.parent?.isActive ?? false, "Parent must be active to activate the child")
        //print("MPLEXFrame[\(streamID?.id ?? 111)]:performActivation()")
        if self.state == .closed || self.state == .closingNeverActivated {
            // We're already closed, or we've been asked to close and are waiting for the network.
            // No need to activate in either case
            return
        }

        self.modifyingState { $0.activate() }
        self.pipeline.fireChannelActive()
        self.tryToAutoRead()
        self.deliverPendingWrites()
    }

    internal func networkActivationReceived() {
        //print("MPLEXFrame[\(streamID?.id ?? 111)]:networkActivationReceived")
        if self.state == .closed {
            // Uh-oh: we got an activation but we think we're closed! We need to send a RST_STREAM frame. We'll only do it if we have a stream ID.
            if let streamID = self.streamID {
                let resetFrame = MPLEXFrame(streamID: streamID, payload: .reset)
                self.parent?.writeAndFlush(resetFrame, promise: nil)
            }
            return
        }
        self.modifyingState { $0.networkActive() }

        //if self.writabilityManager.isWritable != self._isWritable.load() {
        //    // We have probably delayed telling the user that this channel isn't writable, but we should do
        //    // it now.
        //    self._isWritable.store(self.writabilityManager.isWritable)
        //    self.pipeline.fireChannelWritabilityChanged()
        //}

        // If we got here, we may need to flush some pending reads. Notably we don't call read0 here as
        // we don't actually want to start reading before activation, which tryToRead will refuse to do.
        if self.pendingReads.count > 0 {
            self.tryToRead()
        }
    }

    private var _pipeline: ChannelPipeline!

    public let allocator: ByteBufferAllocator

    private let closePromise: EventLoopPromise<()>

    private let multiplexer: MPLEXStreamMultiplexer

    public var closeFuture: EventLoopFuture<Void> {
        self.closePromise.futureResult
    }

    public var pipeline: ChannelPipeline {
        self._pipeline
    }

    public var localAddress: SocketAddress? {
        self.parent?.localAddress
    }

    public var remoteAddress: SocketAddress? {
        self.parent?.remoteAddress
    }

    public let parent: Channel?

    func localAddress0() throws -> SocketAddress {
        self.parent!.localAddress!
    }

    func remoteAddress0() throws -> SocketAddress {
        self.parent!.remoteAddress!
    }

    func setOption<Option: ChannelOption>(_ option: Option, value: Option.Value) -> EventLoopFuture<Void> {
        if self.eventLoop.inEventLoop {
            do {
                return self.eventLoop.makeSucceededFuture(try self.setOption0(option, value: value))
            } catch {
                return self.eventLoop.makeFailedFuture(error)
            }
        } else {
            return self.eventLoop.submit { try self.setOption0(option, value: value) }
        }
    }

    public func getOption<Option: ChannelOption>(_ option: Option) -> EventLoopFuture<Option.Value> {
        if self.eventLoop.inEventLoop {
            do {
                return self.eventLoop.makeSucceededFuture(try self.getOption0(option))
            } catch {
                return self.eventLoop.makeFailedFuture(error)
            }
        } else {
            return self.eventLoop.submit { try self.getOption0(option) }
        }
    }

    private func setOption0<Option: ChannelOption>(_ option: Option, value: Option.Value) throws {
        self.eventLoop.preconditionInEventLoop()

        switch option {
        case _ as ChannelOptions.Types.AutoReadOption:
            self.autoRead = value as! Bool
        default:
            fatalError("setting option \(option) on MPLEXStreamChannel not supported")
        }
    }

    private func getOption0<Option: ChannelOption>(_ option: Option) throws -> Option.Value {
        self.eventLoop.preconditionInEventLoop()

        switch option {
        case _ as MPLEXStreamChannelOptions.Types.StreamIDOption:
            if let streamID = self.streamID {
                return streamID as! Option.Value
            } else {
                throw NIOMPLEXErrors.noStreamIDAvailable()
            }
        case _ as ChannelOptions.Types.AutoReadOption:
            return self.autoRead as! Option.Value
        default:
            fatalError("option \(option) not supported on MPLEXStreamChannel")
        }
    }

    public var isWritable: Bool {
        self._isWritable.load()
    }

    private let _isWritable: NIOAtomic<Bool>

    private var _isActive: Bool {
        self.state == .active || self.state == .closing || self.state == .localActive
    }

    public var isActive: Bool {
        self._isActiveAtomic.load()
    }

    private let _isActiveAtomic: NIOAtomic<Bool>

    public var _channelCore: ChannelCore {
        self
    }

    public let eventLoop: EventLoop

    internal var streamID: MPLEXStreamID?

    private var state: StreamChannelState

    //private var windowManager: InboundWindowManager

    /// If close0 was called but the stream could not synchronously close (because it's currently
    /// active), the promise is stored here until it can be fulfilled.
    private var pendingClosePromise: EventLoopPromise<Void>?

    /// A buffer of pending inbound reads delivered from the parent channel.
    ///
    /// In the future this buffer will be used to manage interactions with read() and even, one day,
    /// with flow control. For now, though, all this does is hold frames until we have set the
    /// channel up.
    private var pendingReads: CircularBuffer<MPLEXFrame>! = CircularBuffer(initialCapacity: 8)

    /// Whether `autoRead` is enabled. By default, all `MPLEXStreamChannel` objects inherit their `autoRead`
    /// state from their parent.
    private var autoRead: Bool

    /// Whether a call to `read` has happened without any frames available to read (that is, whether newly
    /// received frames should be immediately delivered to the pipeline).
    private var unsatisfiedRead: Bool = false

    /// A buffer of pending outbound writes to deliver to the parent channel.
    ///
    /// To correctly respect flushes, we deliberately withold frames from the parent channel until this
    /// stream is flushed, at which time we deliver them all. This buffer holds the pending ones.
    private var pendingWrites: MarkedCircularBuffer<MPLEXStreamData> = MarkedCircularBuffer(initialCapacity: 8)
    //private var pendingWrites: MarkedCircularBuffer<(MPLEXStreamData, EventLoopPromise<Void>?)> = MarkedCircularBuffer(initialCapacity: 8)

    /// A list node used to hold stream channels.
    internal var streamChannelListNode: MPLEXStreamChannelListNode = MPLEXStreamChannelListNode()

    /// An object that controls whether this channel should be writable.
    //private var writabilityManager: MPLEXStreamChannelFlowController

    public func register0(promise: EventLoopPromise<Void>?) {
        fatalError("not implemented \(#function)")
    }

    public func bind0(to: SocketAddress, promise: EventLoopPromise<Void>?) {
        fatalError("not implemented \(#function)")
    }

    public func connect0(to: SocketAddress, promise: EventLoopPromise<Void>?) {
        fatalError("not implemented \(#function)")
    }

    public func write0(_ data: NIOAny, promise: EventLoopPromise<Void>?) {
        guard self.state != .closed else {
            promise?.fail(ChannelError.ioOnClosedChannel)
            return
        }

        let streamData: MPLEXStreamData
        switch self.streamDataType {
        case .frame:
            if let frame = self.tryUnwrapData(data, as: MPLEXFrame.self) {
                streamData = .frame(frame)
            } else if let bytes = self.tryUnwrapData(data, as: ByteBuffer.self) {
                streamData = .frame(MPLEXFrame(streamID: self.streamID!, payload: .outboundData(bytes)))
            } else {
                fatalError("Unknown data written to MPLEXStreamChannel -> \(data)")
            }
        case .framePayload:
            if let framePayload = self.tryUnwrapData(data, as: MPLEXFrame.FramePayload.self) {
                streamData = .framePayload(framePayload)
            } else if let bytes = self.tryUnwrapData(data, as: ByteBuffer.self) {
                streamData = .frame(MPLEXFrame(streamID: self.streamID!, payload: .outboundData(bytes)))
            } else {
                fatalError("Unknown data written to MPLEXStreamChannel -> \(data)")
            }
        }

        // We need a promise to attach our flow control callback to.
        // Regardless of whether the write succeeded or failed, we don't count
        // the bytes any longer.
        //let promise = promise ?? self.eventLoop.makePromise()
        //let writeSize = streamData.estimatedFrameSize

        // Right now we deal with this math by just attaching a callback to all promises. This is going
        // to be annoyingly expensive, but for now it's the most straightforward approach.
        //promise.futureResult.hop(to: self.eventLoop).whenComplete { (_: Result<Void, Error>) in
        //    if case .changed(newValue: let value) = self.writabilityManager.wroteBytes(writeSize) {
        //        self.changeWritability(to: value)
        //    }
        //}
        //self.pendingWrites.append((streamData, promise))
        self.pendingWrites.append(streamData)

        // Ok, we can make an outcall now, which means we can safely deal with the flow control.
        //if case .changed(newValue: let value) = self.writabilityManager.bufferedBytes(writeSize) {
        //    self.changeWritability(to: value)
        //}
    }

    public func flush0() {
        self.pendingWrites.mark()

        if self._isActive {
            self.deliverPendingWrites()
        }
    }

    public func read0() {
        if self.unsatisfiedRead {
            // We already have an unsatisfied read, let's do nothing.
            return
        }

        // At this stage, we have an unsatisfied read. If there is no pending data to read,
        // we're going to call read() on the parent channel. Otherwise, we're going to
        // succeed the read out of our pending data.
        self.unsatisfiedRead = true
        if self.pendingReads.count > 0 {
            self.tryToRead()
        } else {
            self.parent?.read()
        }
    }

    public func close0(error: Error, mode: CloseMode, promise: EventLoopPromise<Void>?) {
        // If the stream is already closed, we can fail this early and abort processing. If it's not, we need to emit a
        // RST_STREAM frame.
        //print("MPLEXFrame[\(streamID?.id ?? 111)]:Close0")
        guard self.state != .closed else {
            promise?.fail(ChannelError.alreadyClosed)
            return
        }

        // Store the pending close promise: it'll be succeeded later.
        if let promise = promise {
            if let pendingPromise = self.pendingClosePromise {
                pendingPromise.futureResult.cascade(to: promise)
            } else {
                self.pendingClosePromise = promise
            }
        }

        switch self.state {
        case .idle, .localActive, .closed:
            // The stream isn't open on the network, just go straight to closed cleanly.
            self.closedCleanly()
        case .remoteActive, .active, .closing, .closingNeverActivated:
            // In all of these states the stream is still on the network and we need to wait.
            self.closedWhileOpen()
        }
    }

    public func triggerUserOutboundEvent0(_ event: Any, promise: EventLoopPromise<Void>?) {
        // do nothing
    }

    public func channelRead0(_ data: NIOAny) {
        // do nothing
    }

    public func errorCaught0(error: Error) {
        // do nothing
    }

    /// Called when the channel was closed from the pipeline while the stream is still open.
    ///
    /// Will emit a RST_STREAM frame in order to close the stream. Note that this function does not
    /// directly close the stream: it waits until the stream closed notification is fired.
    private func closedWhileOpen() {
        precondition(self.state != .closed)
        guard self.state != .closing else {
            // If we're already closing, nothing to do here.
            return
        }

        self.modifyingState { $0.beginClosing() }

        // We should have a stream ID here, force-unwrap is safe.
        let closeFrame = MPLEXFrame(streamID: self.streamID!, payload: .close)
        self.receiveOutboundFrame(closeFrame, promise: nil)
        self.multiplexer.childChannelFlush()
        self.multiplexer.childChannelWriteClosed(self.streamID!)
    }

    private func closedCleanly() {
        guard self.state != .closed else {
            return
        }
        self.modifyingState { $0.completeClosing() }
        self.dropPendingReads()
        self.failPendingWrites(error: ChannelError.eof)
        if let promise = self.pendingClosePromise {
            self.pendingClosePromise = nil
            promise.succeed(())
        }
        self.pipeline.fireChannelInactive()

        self.eventLoop.execute {
            self.removeHandlers(pipeline: self.pipeline)
            self.closePromise.succeed(())
            if let streamID = self.streamID {
                self.multiplexer.childChannelClosed(streamID: streamID)
            } else {
                self.multiplexer.childChannelClosed(channelID: ObjectIdentifier(self))
            }
            /// Should I do this???
            //print("MPLEXFrame::Dereferencing")
            self._pipeline = nil
            self.pendingReads.removeAll()
            self.pendingReads = nil
            //            print("Unsatisfied Read: \(self.unsatisfiedRead)")
            //            print("Pending Reads: \(self.pendingReads.count)")
        }
    }

    fileprivate func errorEncountered(error: Error) {
        guard self.state != .closed else {
            return
        }

        if self.state == .active {
            //print("MPLEXFrame[\(self.streamID!.id)]::ErrorEncountered -> Sending Reset Frame")
            // We should have a stream ID here, force-unwrap is safe.
            let resetFrame = MPLEXFrame(streamID: self.streamID!, payload: .close)
            self.receiveOutboundFrame(resetFrame, promise: nil)
            self.multiplexer.childChannelFlush()
        }

        self.modifyingState { $0.completeClosing() }
        self.dropPendingReads()
        self.failPendingWrites(error: error)
        if let promise = self.pendingClosePromise {
            self.pendingClosePromise = nil
            promise.fail(error)
        }
        self.pipeline.fireErrorCaught(error)
        self.pipeline.fireChannelInactive()

        self.eventLoop.execute {
            self.removeHandlers(pipeline: self.pipeline)
            self.closePromise.fail(error)
            if let streamID = self.streamID {
                self.multiplexer.childChannelClosed(streamID: streamID)
            } else {
                self.multiplexer.childChannelClosed(channelID: ObjectIdentifier(self))
            }
        }
    }

    private func tryToRead() {
        // If there's no read to satisfy, no worries about it.
        guard self.unsatisfiedRead else {
            return
        }

        // If we're not active, we will hold on to these reads.
        guard self._isActive else {
            return
        }

        // If there are no pending reads, do nothing.
        guard self.pendingReads.count > 0 else {
            return
        }

        // Ok, we're satisfying a read here.
        self.unsatisfiedRead = false
        self.deliverPendingReads()
        self.tryToAutoRead()
    }

    private func changeWritability(to newWritability: Bool) {
        self._isWritable.store(newWritability)
        self.pipeline.fireChannelWritabilityChanged()
    }

    private func tryToAutoRead() {
        if self.autoRead {
            // If auto-read is turned on, recurse into channelPipeline.read().
            // This cannot recurse indefinitely unless frames are being delivered
            // by the read stacks, which is generally fairly unlikely to continue unbounded.
            self.pipeline.read()
        }
    }
}

// MARK:- Functions used to manage pending reads and writes.
extension MPLEXStreamChannel {
    /// Drop all pending reads.
    private func dropPendingReads() {
        /// We don't need to report the dropped reads, just remove them all.
        self.pendingReads.removeAll()
    }

    /// Deliver all pending reads to the channel.
    private func deliverPendingReads() {
        assert(self._isActive)
        while self.pendingReads.count > 0 {
            let frame = self.pendingReads.removeFirst()

            let anyStreamData: ByteBuffer

            switch self.streamDataType {
            case .frame:
                if frame.payload.buffer.readableBytes > 0 {
                    //print("MPLEXStreamChannel::DeliverPendingReads -> frame to buffer")
                    anyStreamData = frame.payload.buffer
                } else {
                    print(
                        "MPLEXFrame[\(streamID?.id ?? 111)]::DeliverPendingReads -> Warning: Dropping frame with empty buffer"
                    )
                    continue
                }

            case .framePayload:
                //print("MPLEXStreamChannel::DeliverPendingReads -> framePayload to buffer")
                switch frame.payload {
                case .close, .reset:
                    print("MPLEXFrame[\(streamID?.id ?? 111)]::Warning: Dropping \(frame.payload)")
                    continue
                case .inboundData(let buffer):
                    anyStreamData = buffer
                default:
                    continue
                }
            }

            //switch frame.payload {
            //case .data(let data):
            //    dataLength = data.payload.readableBytes
            //default:
            //    dataLength = nil
            //}

            self.pipeline.fireChannelRead(anyStreamData)

            //if let size = dataLength, let increment = self.windowManager.bufferedFrameEmitted(size: size) {
            //    // To have a pending read, we must have a stream ID.
            //    let frame = MPLEXFrame(streamID: self.streamID!, payload: .windowUpdate(windowSizeIncrement: increment))
            //    self.receiveOutboundFrame(frame, promise: nil)
            //    // This flush should really go away, but we need it for now until we sort out window management.
            //    self.multiplexer.childChannelFlush()
            //}
        }
        self.pipeline.fireChannelReadComplete()
    }

    /// Delivers all pending flushed writes to the parent channel.
    private func deliverPendingWrites() {
        // If there are no pending writes, don't bother with the processing.
        guard self.pendingWrites.hasMark else {
            return
        }

        // Get a streamID from the multiplexer if we haven't got one already.
        if self.streamID == nil {
            self.streamID = self.multiplexer.requestStreamID(forChannel: self)
        }

        while self.pendingWrites.hasMark {
            //let (streamData, promise) = self.pendingWrites.removeFirst()
            let streamData = self.pendingWrites.removeFirst()
            let frame: MPLEXFrame

            switch streamData {
            case .frame(let f):
                frame = f
            case .framePayload(let payload):
                // This unwrap is okay: we just ensured that `self.streamID` was set above.
                frame = MPLEXFrame(streamID: self.streamID!, payload: payload)
            }

            //self.receiveOutboundFrame(frame, promise: promise)
            self.receiveOutboundFrame(frame, promise: nil)
        }
        self.multiplexer.childChannelFlush()
    }

    /// Fails all pending writes with the given error.
    private func failPendingWrites(error: Error) {
        assert(self.state == .closed)
        while self.pendingWrites.count > 0 {
            //self.pendingWrites.removeFirst().1?.fail(error)
            let _ = self.pendingWrites.removeFirst()
        }
    }
}

// MARK:- Functions used to communicate between the `MPLEXStreamMultiplexer` and the `MPLEXStreamChannel`.
extension MPLEXStreamChannel {
    /// Called when a frame is received from the network.
    ///
    /// - parameters:
    ///     - frame: The `MPLEXFrame` received from the network.
    func receiveInboundFrame(_ frame: MPLEXFrame) {
        guard self.state != .closed else {
            // Do nothing
            return
        }

        // Record the size of the frame so that when we receive a window update event our
        // calculation on whether we emit a WINDOW_UPDATE frame is based on the bytes we have
        // actually delivered into the pipeline.
        //if case .data(let dataPayload) = frame.payload {
        //self.windowManager.bufferedFrameReceived(size: dataPayload.data.readableBytes)

        // No further window update frames should be sent.
        //if dataPayload.endStream {
        //    self.windowManager.closed = true
        //}
        //}
        //else if case .headers(let headersPayload) = frame.payload, headersPayload.endStream {
        // No further window update frames should be sent.
        //    self.windowManager.closed = true
        //}

        self.pendingReads.append(frame)

        if self.state == .localActive {
            self.networkActivationReceived()
        }
    }

    /// Called when a frame is sent to the network.
    ///
    /// - parameters:
    ///     - frame: The `MPLEXFrame` to send to the network.
    ///     - promise: The promise associated with the frame write.
    private func receiveOutboundFrame(_ frame: MPLEXFrame, promise: EventLoopPromise<Void>?) {
        guard self.state != .closed else {
            let error = ChannelError.alreadyClosed
            promise?.fail(error)
            self.errorEncountered(error: error)
            return
        }
        self.multiplexer.childChannelWrite(frame, promise: promise)
    }

    /// Called when a stream closure is received from the network.
    ///
    /// - parameters:
    ///     - reason: The reason received from the network, if any.
    func receiveStreamClosed(_ reason: MPLEXErrorCode?) {
        //print("MPLEXFrame[\(streamID?.id ?? 111)]:receiveStreamClosed()")
        // Avoid emitting any WINDOW_UPDATE frames now that we're closed.
        //self.windowManager.closed = true

        // The stream is closed, we should force forward all pending frames, even without
        // unsatisfied read, to ensure the handlers can see all frames before receiving
        // channelInactive.
        if self.pendingReads.count > 0 && self._isActive {
            self.unsatisfiedRead = false
            self.deliverPendingReads()
        }

        if let reason = reason {
            // To receive from the network, it must be safe to force-unwrap here.
            let err = NIOMPLEXErrors.streamClosed(streamID: self.streamID!, errorCode: reason)
            self.errorEncountered(error: err)
        } else {
            //print("MPLEXFrame[\(self.streamID!.id)]::RecieveStreamClosed while in state \(self.state)")
            if self.state == .closed || self.state == .closing || self.state == .idle || self.state == .localActive {
                //print("MPLEXFrame[\(self.streamID!.id)]::Closing cleanly")
                self.closedCleanly()
            } else {
                //print("MPLEXFrame[\(self.streamID!.id)]::Closing while open")
                self.closedWhileOpen()
                self.closedCleanly()
            }
        }
    }

    //    func receiveWindowUpdatedEvent(_ windowSize: Int) {
    //        if let increment = self.windowManager.newWindowSize(windowSize) {
    //            // To receive from the network, it must be safe to force-unwrap here.
    //            let frame = MPLEXFrame(streamID: self.streamID!, payload: .windowUpdate(windowSizeIncrement: increment))
    //            self.receiveOutboundFrame(frame, promise: nil)
    //            // This flush should really go away, but we need it for now until we sort out window management.
    //            self.multiplexer.childChannelFlush()
    //        }
    //    }
    //
    //    func initialWindowSizeChanged(delta: Int) {
    //        if let increment = self.windowManager.initialWindowSizeChanged(delta: delta) {
    //            // To receive from the network, it must be safe to force-unwrap here.
    //            let frame = MPLEXFrame(streamID: self.streamID!, payload: .windowUpdate(windowSizeIncrement: increment))
    //            self.receiveOutboundFrame(frame, promise: nil)
    //            // This flush should really go away, but we need it for now until we sort out window management.
    //            self.multiplexer.childChannelFlush()
    //        }
    //    }

    func receiveParentChannelReadComplete() {
        self.tryToRead()
    }

    //    func parentChannelWritabilityChanged(newValue: Bool) {
    //        // There's a trick here that's worth noting: if the child channel hasn't either sent a frame
    //        // or been activated on the network, we don't actually want to change the observable writability.
    //        // This is because in this case we really want user code to send a frame as soon as possible to avoid
    //        // issues with their stream ID becoming out of date. Once the state transitions we can update
    //        // the writability if needed.
    //        guard case .changed(newValue: let localValue) = self.writabilityManager.parentWritabilityChanged(newValue) else {
    //            return
    //        }
    //
    //        // Ok, the writability changed.
    //        switch self.state {
    //        case .idle, .localActive:
    //            // Do nothing here.
    //            return
    //        case .remoteActive, .active, .closing, .closingNeverActivated, .closed:
    //            self._isWritable.store(localValue)
    //            self.pipeline.fireChannelWritabilityChanged()
    //        }
    //    }

    func receiveStreamError(_ error: NIOMPLEXErrors.StreamError) {
        assert(error.streamID == self.streamID)
        self.pipeline.fireErrorCaught(error.baseError)
    }
}

extension MPLEXStreamChannel {
    // A helper function used to ensure that state modification leads to changes in the channel active atomic.
    private func modifyingState<ReturnType>(
        _ closure: (inout StreamChannelState) throws -> ReturnType
    ) rethrows -> ReturnType {
        defer {
            self._isActiveAtomic.store(self._isActive)
        }
        return try closure(&self.state)
    }
}

// MARK: Custom String Convertible
extension MPLEXStreamChannel {
    public var description: String {
        "MPLEXStreamChannel(streamID: \(String(describing: self.streamID)), isActive: \(self.isActive), isWritable: \(self.isWritable))"
    }
}

extension MPLEXStreamChannel {
    internal struct SynchronousOptions: NIOSynchronousChannelOptions {
        private let channel: MPLEXStreamChannel

        fileprivate init(channel: MPLEXStreamChannel) {
            self.channel = channel
        }

        /// Set `option` to `value` on this `Channel`.
        ///
        /// - Important: Must be called on the `EventLoop` the `Channel` is running on.
        public func setOption<Option: ChannelOption>(_ option: Option, value: Option.Value) throws {
            try self.channel.setOption0(option, value: value)
        }

        /// Get the value of `option` for this `Channel`.
        ///
        /// - Important: Must be called on the `EventLoop` the `Channel` is running on.
        public func getOption<Option: ChannelOption>(_ option: Option) throws -> Option.Value {
            try self.channel.getOption0(option)
        }
    }

    /// Returns a view of the `Channel` exposing synchronous versions of `setOption` and `getOption`.
    public var syncOptions: NIOSynchronousChannelOptions? {
        SynchronousOptions(channel: self)
    }
}
