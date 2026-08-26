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

/// An object that controls multiplexing messages to multiple child channels.
final class ChannelMultiplexer {

    internal var channels: [UInt32: YAMUXStream]

    private var erroredChannels: [UInt32]

    /// The main delegate (parent channel) that we write to and read from
    ///
    /// - Warning:This object can cause a reference cycle, so we require it to be optional so that we can break the cycle manually.
    private var delegate: MultiplexerDelegate?

    /// The next local channel ID to use.
    private var nextChannelID: UInt32

    private let allocator: ByteBufferAllocator

    private var childChannelInitializer: ChildChannel.Initializer?

    /// Whether new channels are allowed. Set to `false` once the parent channel is shut down at the TCP level.
    private var canCreateNewChannels: Bool

    /// Whether we're opperating as the client or server in this connection
    private var mode: LibP2PCore.Mode

    /// The Initial window size for each child channel
    private var initialWindowSize: UInt32

    /// The logger we'll pass into each child channel
    private var logger: Logger

    init(
        delegate: MultiplexerDelegate,
        allocator: ByteBufferAllocator,
        mode: LibP2PCore.Mode,
        initialWindowSize: UInt32,
        logger: Logger,
        childChannelInitializer: ChildChannel.Initializer?
    ) {
        self.channels = [:]
        self.channels.reserveCapacity(8)
        self.erroredChannels = []
        self.delegate = delegate
        self.nextChannelID = mode == .initiator ? 1 : 2
        self.allocator = allocator
        self.mode = mode
        self.initialWindowSize = initialWindowSize
        self.childChannelInitializer = childChannelInitializer
        self.canCreateNewChannels = true
        self.logger = logger
        self.logger[metadataKey: "YAMUX"] = .string("Muxer")
    }

    // Time to clean up. We drop references to things that may be keeping us alive.
    // Note that we don't drop the child channels because we expect that they'll be cleaning themselves up.
    func parentHandlerRemoved() {
        self.delegate = nil
        self.childChannelInitializer = nil
        self.canCreateNewChannels = false
    }
}

// MARK: Calls from child channels

extension ChannelMultiplexer {
    /// A `ChildChannel` has issued a write.
    func writeFromChannel(_ message: Frame, _ promise: EventLoopPromise<Void>?) {
        guard let delegate = self.delegate else {
            promise?.fail(ChannelError.ioOnClosedChannel)
            return
        }

        delegate.writeFromChildChannel(message, promise)
    }

    /// A `ChildChannel` has issued a flush.
    func childChannelFlush() {
        // Nothing to do.
        guard let delegate = self.delegate else {
            return
        }

        delegate.flushFromChildChannel()
    }

    func childChannelClosed(channelID: UInt32) {
        // This should never return `nil`, but we don't want to assert on it because
        // even if the object was never in the map, nothing bad will happen: it's gone!
        if let s = self.channels.removeValue(forKey: channelID) {
            self.delegate?.childChannelRemoved(stream: s)
        } else {
            self.logger.warning("Removed unregistered child channel")
        }
    }

    func childChannelErrored(channelID: UInt32, expectClose: Bool) {
        // This should never return `nil`, but we don't want to assert on it because
        // even if the object was never in the map, nothing bad will happen: it's gone!
        self.channels.removeValue(forKey: channelID)

        if expectClose {
            // We keep track of the errored channel because we will tolerate receiving a close for it.
            self.erroredChannels.append(channelID)
        }
    }
}

// MARK: Calls from YAMUX handlers.

extension ChannelMultiplexer {
    func receiveMessage(_ message: Message) throws {
        let channel: ChildChannel?

        switch message {
        case .channelOpen(let message):
            self.logger.trace("receiveMessage::channelOpen -> New Channel Requested with ID:\(message.senderChannel)")
            // Ensure the proposed ChannelID is valid
            try isValidInboundChannelID(message.senderChannel)
            self.logger.trace(
                "receiveMessage::channelOpen -> Attempting to open new channel ID:\(message.senderChannel)"
            )
            // Create / Open the new Channel
            channel = try self.openNewChannel(
                channelID: message.senderChannel,
                initializer: self.childChannelInitializer
            )

        case .channelOpenConfirmation(let message):
            channel = try self.existingChannel(localID: message.recipientChannel)

        case .channelOpenFailure(let message):
            channel = try self.existingChannel(localID: message.recipientChannel)

        case .channelClose(let message):
            channel = try self.existingChannel(localID: message.recipientChannel)
            if channel == nil, let errorIndex = self.erroredChannels.firstIndex(of: message.recipientChannel) {
                // This is the end of our need to keep track of the channel.
                self.erroredChannels.remove(at: errorIndex)
            }

        case .channelReset(let message):
            channel = try self.existingChannel(localID: message.recipientChannel)
            if channel == nil, let errorIndex = self.erroredChannels.firstIndex(of: message.recipientChannel) {
                // This is the end of our need to keep track of the channel.
                self.erroredChannels.remove(at: errorIndex)
            }

        case .channelWindowAdjust(let message):
            channel = try self.existingChannel(localID: message.recipientChannel)

        case .channelData(let message):
            channel = try self.existingChannel(localID: message.recipientChannel)

        default:
            // Not a channel message, we don't do anything more with this.
            self.logger.warning("Warning - Unsupported message type")
            self.logger.trace("\(message)")
            self.logger.trace("----")
            return
        }

        if let channel = channel {
            self.logger.trace("Sending message to channel")
            channel.receiveInboundMessage(message)
        } else {
            self.logger.warning("Warning - Channel not found!")
            self.logger.trace("\(message)")
            self.logger.trace("----")
        }
    }

    func createInboundChildChannel(
        channelID: UInt32,
        _ promise: EventLoopPromise<Channel>? = nil,
        _ channelInitializer: ChildChannel.Initializer?
    ) {
        do {
            guard let el = self.delegate?.channel?.eventLoop else {
                throw YAMUX.Error.channelSetupRejected(
                    reasonCode: 0,
                    reason: "Multiplexer lost reference to parent/delegate"
                )
            }
            // Ensure the proposed ChannelID is valid
            try isValidInboundChannelID(channelID)
            // Open the Channel
            let channel = try self.openNewChannel(
                channelID: channelID,
                initializer: channelInitializer ?? childChannelInitializer
            )

            let channelConfigPromise = el.makePromise(of: Channel.self)

            channel.configure(userPromise: channelConfigPromise)

            promise?.completeWith(
                channelConfigPromise.futureResult.map { _ in
                    let s = self.channels[channelID]!
                    // inform our delegate
                    self.delegate?.childChannelCreated(stream: s)
                    // Return the stream
                    return s._channel
                }
            )
        } catch {
            promise?.fail(error)
        }
    }

    func createOutboundChildChannel(
        _ promise: EventLoopPromise<YAMUXStream>? = nil,
        _ channelInitializer: ChildChannel.Initializer?
    ) {
        do {
            guard let el = self.delegate?.channel?.eventLoop else {
                throw YAMUX.Error.channelSetupRejected(
                    reasonCode: 0,
                    reason: "Multiplexer lost reference to parent/delegate"
                )
            }

            let channelID = self.nextChannelID
            self.nextChannelID &+= 2

            if self.nextChannelID >= UInt32.max - 1 {
                throw YAMUX.Error.channelSetupRejected(reasonCode: 0, reason: "Stream Count Exhaustion")
            }

            let channel = try self.openNewChannel(channelID: channelID, initializer: channelInitializer)

            let channelConfigPromise = el.makePromise(of: Channel.self)

            channel.configure(userPromise: channelConfigPromise)

            promise?.completeWith(
                channelConfigPromise.futureResult.map { _ in
                    let s = self.channels[channelID]!
                    // inform our delegate
                    self.delegate?.childChannelCreated(stream: s)
                    // Return the stream
                    return s
                }
            )
        } catch {
            promise?.fail(error)
        }
    }

    func parentChannelReadComplete() {
        for channel in self.channels.values {
            channel._channel.receiveParentChannelReadComplete()
        }
    }

    func parentChannelInactive() {
        self.canCreateNewChannels = false
        for channel in self.channels.values {
            channel._channel.parentChannelInactive()
        }
    }

    //    func shouldQuiesce(on el: EventLoop) -> EventLoopFuture<Void> {
    //        self.canCreateNewChannels = false
    //
    //        var tasks:[EventLoopFuture<Void>] = []
    //        for channel in self.channels.values {
    //            if channel.id == 0 { continue }
    //            let _ = channel.close(gracefully: true)
    //            tasks.append(channel._channel.closeFuture)
    //        }
    //
    //        return tasks.flatten(on: el).flatMapAlways { res in
    //            if let session = self.channels[0] {
    //                let _ = session.close(gracefully: true)
    //                return session._channel.closeFuture
    //            } else {
    //                self.logger.warning("Lost Session Reference")
    //                return el.makeSucceededVoidFuture()
    //            }
    //        }
    //    }

    func shouldQuiesce(on el: EventLoop) -> EventLoopFuture<Void> {
        // Stop accepting new channels
        self.canCreateNewChannels = false

        // Loop through our current chilc channels and issue closes on them
        var tasks: [EventLoopFuture<Void>] = []
        for channel in self.channels.values {
            let _ = channel.close(gracefully: true)
            tasks.append(channel._channel.closeFuture)
        }

        // return the future result of the close calls
        return tasks.flatten(on: el)
    }

    private func isValidInboundChannelID(_ id: UInt32) throws {
        // Ensure the ChannelID has the correct polarity
        guard self.mode == .initiator ? id.isEven : id.isOdd else {
            throw YAMUX.Error.protocolViolation(protocolName: "ChannelID", violation: "Incorrect channel ID parity")
        }
        // Ensure the ChannelID isn't already used...
        guard self.channels[id] == nil else {
            throw YAMUX.Error.channelSetupRejected(reasonCode: 0, reason: "Stream ID already in use")
        }
        guard !self.erroredChannels.contains(id) else {
            throw YAMUX.Error.channelSetupRejected(reasonCode: 0, reason: "Stream ID already in use")
        }
    }

    /// Opens a new channel and adds it to the multiplexer.
    private func openNewChannel(channelID: UInt32, initializer: ChildChannel.Initializer?) throws -> ChildChannel {
        guard let parentChannel = self.delegate?.channel else {
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Opening new channel after channel shutdown"
            )
        }

        guard self.canCreateNewChannels else {
            throw YAMUX.Error.tcpShutdown
        }

        // Determine this streams direction / mode
        let direction: LibP2P.Mode
        switch mode {
        case .listener:
            direction = (channelID % 2 == 0) ? .initiator : .listener
        case .initiator:
            direction = (channelID % 2 == 0) ? .listener : .initiator
        }

        // Create the ChildChannel
        let channel = ChildChannel(
            allocator: self.allocator,
            parent: parentChannel,
            multiplexer: self,
            initializer: initializer,
            localChannelID: channelID,
            targetWindowSize: Int32(self.initialWindowSize),
            initialOutboundWindowSize: self.initialWindowSize,
            direction: direction,
            logger: logger
        )

        // Init our Libp2p Stream
        let stream = YAMUXStream(
            channel: channel,
            mode: direction,
            id: UInt64(channelID),
            name: nil,
            proto: "",
            streamState: .initialized
        )

        // Store / register it
        self.channels[channelID] = stream

        return channel
    }

    private func existingChannel(localID: UInt32) throws -> ChildChannel? {
        if let channel = self.channels[localID] {
            return channel._channel
        } else if self.erroredChannels.contains(localID) {
            return nil
        } else {
            throw YAMUX.Error.protocolViolation(
                protocolName: "channel",
                violation: "Unexpected request with local channel id \(localID)"
            )
        }
    }
}

extension UInt32 {
    var isEven: Bool {
        self % 2 == 0
    }

    var isOdd: Bool {
        !self.isEven
    }
}

/// An internal protocol to encapsulate the object that owns the multiplexer.
protocol MultiplexerDelegate {
    var channel: Channel? { get }

    func writeFromChildChannel(_: Frame, _: EventLoopPromise<Void>?)

    func flushFromChildChannel()

    func childChannelCreated(stream: any LibP2PCore._Stream)

    func childChannelRemoved(stream: any LibP2PCore._Stream)
}
