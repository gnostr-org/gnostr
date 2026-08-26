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
// Copyright (c) 2020-2021 Apple Inc. and the SwiftNIO project authors
// Licensed under Apache License v2.0
//
// See LICENSE.txt for license information
// See CONTRIBUTORS.txt for the list of SwiftNIO project authors
//
// SPDX-License-Identifier: Apache-2.0
//
//===----------------------------------------------------------------------===//

import NIOCore

/// This structure defines an abstraction over `MPLEXStreamChannel` that is used
/// by the `MPLEXStreamMultiplexer`.
///
/// The goal of this type is to help reduce the coupling between
/// `MPLEXStreamMultiplexer` and `MPLEXChannel`. By providing this abstraction layer,
/// it makes it easier for us to change the potential backing implementation without
/// having to modify `MPLEXStreamMultiplexer`.
///
/// Note that while this is a `struct`, this `struct` has _reference semantics_.
/// The implementation of `Equatable` & `Hashable` on this type reinforces that requirement.
struct MultiplexerAbstractChannel {
    private var baseChannel: MPLEXStreamChannel

    init(
        allocator: ByteBufferAllocator,
        parent: Channel,
        multiplexer: MPLEXStreamMultiplexer,
        streamID: MPLEXStreamID?,
        inboundStreamStateInitializer: InboundStreamStateInitializer
    ) {
        switch inboundStreamStateInitializer {
        case .includesStreamID:
            assert(streamID != nil)
            self.baseChannel = .init(
                allocator: allocator,
                parent: parent,
                multiplexer: multiplexer,
                streamID: streamID,
                streamDataType: .frame
            )

        case .excludesStreamID:
            self.baseChannel = .init(
                allocator: allocator,
                parent: parent,
                multiplexer: multiplexer,
                streamID: streamID,
                streamDataType: .framePayload
            )
        }
    }
}

extension MultiplexerAbstractChannel {
    enum InboundStreamStateInitializer {
        case includesStreamID(((Channel, MPLEXStreamID) -> EventLoopFuture<Void>)?)
        case excludesStreamID(((Channel) -> EventLoopFuture<Void>)?)
    }
}

// MARK: API for MPLEXStreamMultiplexer
extension MultiplexerAbstractChannel {
    var channel: Channel {
        self.baseChannel.channel
    }

    var streamID: MPLEXStreamID? {
        self.baseChannel.streamID
    }

    var channelID: ObjectIdentifier {
        ObjectIdentifier(self.baseChannel)
    }

    var inList: Bool {
        self.baseChannel.inList
    }

    var streamChannelListNode: MPLEXStreamChannelListNode {
        get {
            self.baseChannel.streamChannelListNode
        }
        nonmutating set {
            self.baseChannel.streamChannelListNode = newValue
        }
    }

    func configureInboundStream(initializer: InboundStreamStateInitializer) {
        switch initializer {
        case .includesStreamID(let initializer):
            self.baseChannel.configure(initializer: initializer, userPromise: nil)
        case .excludesStreamID(let initializer):
            self.baseChannel.configure(initializer: initializer, userPromise: nil)
        }
    }

    func configure(
        initializer: ((Channel, MPLEXStreamID) -> EventLoopFuture<Void>)?,
        userPromise promise: EventLoopPromise<Channel>?
    ) {
        self.baseChannel.configure(initializer: initializer, userPromise: promise)
    }

    func configure(initializer: ((Channel) -> EventLoopFuture<Void>)?, userPromise promise: EventLoopPromise<Channel>?)
    {
        self.baseChannel.configure(initializer: initializer, userPromise: promise)
    }

    func performActivation() {
        self.baseChannel.performActivation()
    }

    func networkActivationReceived() {
        self.baseChannel.networkActivationReceived()
    }

    func receiveInboundFrame(_ frame: MPLEXFrame) {
        self.baseChannel.receiveInboundFrame(frame)
    }

    func receiveParentChannelReadComplete() {
        self.baseChannel.receiveParentChannelReadComplete()
    }

    func receiveStreamClosed(_ reason: MPLEXErrorCode?) {
        self.baseChannel.receiveStreamClosed(reason)
    }

    func receiveStreamError(_ error: NIOMPLEXErrors.StreamError) {
        self.baseChannel.receiveStreamError(error)
    }
}

extension MultiplexerAbstractChannel: Equatable {
    static func == (lhs: MultiplexerAbstractChannel, rhs: MultiplexerAbstractChannel) -> Bool {
        lhs.baseChannel === rhs.baseChannel
    }
}

extension MultiplexerAbstractChannel: Hashable {
    func hash(into hasher: inout Hasher) {
        hasher.combine(ObjectIdentifier(self.baseChannel))
    }
}
