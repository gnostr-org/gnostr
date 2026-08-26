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
import NIOConcurrencyHelpers

/// YAMUXStream is a high level wrapper for our underlying child channel stream, it's used by Libp2p to send and receive data
public final class YAMUXStream: _Stream {
    //public private(set) var streamState:LibP2PCore.StreamState
    public let _streamState: NIOLockedValueBox<LibP2PCore.StreamState>
    public var streamState: LibP2PCore.StreamState {
        _streamState.withLockedValue { $0 }
    }

    //public private(set) var connection: Connection?
    public let _connection: NIOLockedValueBox<(any LibP2PCore.Connection)?>
    public var connection: Connection? {
        _connection.withLockedValue { $0 }
    }

    public var channel: Channel { self._channel }
    internal let _channel: ChildChannel

    public let id: UInt64
    public let name: String?
    public let mode: LibP2PCore.Mode
    public var protocolCodec: String {
        get { _protocolCodec.withLockedValue { $0 } }
    }
    private let _protocolCodec: NIOLockedValueBox<String>

    public var direction: ConnectionStats.Direction {
        self.mode == .listener ? .inbound : .outbound
    }

    private let streamID: ChannelIdentifier

    public required init(
        channel: Channel,
        mode: LibP2PCore.Mode,
        id: UInt64,
        name: String?,
        proto: String,
        streamState: LibP2PCore.StreamState = .initialized
    ) {
        guard let cc = channel as? ChildChannel else {
            fatalError("YamuxStream initialized with wrong channel type.")
        }
        self.mode = mode
        self.id = id
        self.name = name
        self.streamID = ChannelIdentifier(channelID: UInt32(id))

        self._channel = cc
        self._connection = .init(nil)
        self._streamState = .init(streamState)
        self._protocolCodec = .init(proto)
        self._on = .init(nil)
        //print("Stream[\(id)]::Initializing")
    }

    //deinit {
    //    print("Stream[\(id)]::Deinitializing")
    //}

    /// Main Delegate/Callback Function
    public var on: (@Sendable (LibP2PCore.StreamEvent) -> EventLoopFuture<Void>)? {
        get {
            _on.withLockedValue { $0 }
        }
        set {
            _on.withLockedValue { $0 = newValue }
        }
    }
    private let _on: NIOLockedValueBox<(@Sendable (LibP2PCore.StreamEvent) -> EventLoopFuture<Void>)?>

    public func write(_ data: Data) -> EventLoopFuture<Void> {
        self.write(channel.allocator.buffer(bytes: data.byteArray))
    }

    public func write(_ bytes: [UInt8]) -> EventLoopFuture<Void> {
        self.write(channel.allocator.buffer(bytes: bytes))
    }

    public func write(_ buffer: ByteBuffer) -> EventLoopFuture<Void> {
        let promise = self.channel.eventLoop.makePromise(of: Void.self)

        //print("Stream[\(streamID.channelID)] -> Attempting to write to channel")
        guard self.channel.isActive && self.channel.isWritable else {
            self._streamState.withLockedValue { $0 = .reset }
            return self.channel.eventLoop.makeFailedFuture(Errors.streamNotWritable)
        }
        guard self.streamState == .open else {
            return self.channel.eventLoop.makeFailedFuture(Errors.streamNotWritable)
        }
        // Write it out (as a RawResponse)
        self._channel.write(RawResponse(payload: buffer), promise: promise)
        self._channel.flush()

        return promise.futureResult
    }

    /// Sends a close stream message to our remote peer, requesting this Stream be closed.
    /// - Note: Because there can be multiple YAMUXStreams over a single Connection, this will NOT close the underlying Connection.
    public func close(gracefully: Bool) -> EventLoopFuture<Void> {
        //print("Stream[\(streamID.channelID)] -> close(gracefully:\(gracefully)) called with state: \(self._streamState)")
        switch self._streamState.withLockedValue({ $0 }) {
        case .initialized, .open:
            _streamState.withLockedValue { $0 = .writeClosed }

            let _ = self.on?(.closed)
            return self._channel.closeChannel()

        case .receiveClosed:
            _streamState.withLockedValue { $0 = .closed }

            let _ = self.on?(.closed)
            return self._channel.closeChannel()

        case .writeClosed, .closed, .reset:
            break
        }

        return self.channel.eventLoop.makeSucceededVoidFuture()
    }

    /// Sends a reset stream message to our remote peer, immediately shutting down the Stream.
    /// - Note: Once an YAMUXStream has been reset, you can no longer write / read to / from it.
    public func reset() -> EventLoopFuture<Void> {
        let promise = self.channel.eventLoop.makePromise(of: Void.self)
        //if self.channel.isActive && self.channel.isWritable {
        //print("Stream[\(streamID.channelID)] -> Writing Reset Message")

        self._channel.processOutboundMessage(
            .channelReset(.init(recipientChannel: streamID.channelID, reasonCode: 0, description: "")),
            promise: promise
        )

        self._streamState.withLockedValue { $0 = .reset }
        let _ = self.on?(.reset)
        return promise.futureResult
    }

    /// Kicks off the Stream handshake (if we're the initiator)
    public func resume() -> EventLoopFuture<Void> {
        // Ask our connection to establish a channel if it hasn't already (actually dial / bind to remote peer)
        print("Stream[\(streamID.channelID)]::TODO: Resume / Kick off our Stream")
        return self.channel.eventLoop.makeSucceededVoidFuture()
    }

    internal func updateStreamState(state: StreamState, protocol: String) {
        // Update our state if it's a valid transition...
        if state.rawValue > self._streamState.withLockedValue({ $0 }).rawValue {
            //print("Stream[\(streamID.channelID)] -> Updating state from \(self._streamState) -> \(state)")
            self._streamState.withLockedValue { $0 = state }
        }
        //else { print("Stream[\(streamID.channelID)] -> Skipping invalid state change from \(self._streamState) -> \(state)") }

        // Update our protocol if it hasn't been set yet
        guard self.protocolCodec == "" else {
            //print("Stream[\(streamID.channelID)] -> Protocol Codec can't be changed once set. Skipping protocol change request from `\(self.protocolCodec)` -> `\(`protocol`)`")
            return
        }
        //print("Stream[\(streamID.channelID)] -> Updating protocol from \(self.protocolCodec) -> \(`protocol`)")
        self._protocolCodec.withLockedValue { $0 = `protocol` }
    }

    public enum Errors: Error {
        case streamNotWritable
    }
}
