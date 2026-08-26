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

/// A BasicStream manages a specific protocol Stream over a Connection. Non Muxed Stream. Only one BasicStream per Connection.
public final class MPLEXStream: _Stream {
    //public private(set) var streamState:LibP2PCore.StreamState
    public let _streamState: NIOLockedValueBox<LibP2PCore.StreamState>
    public var streamState: LibP2PCore.StreamState {
        _streamState.withLockedValue { $0 }
    }

    //public private(set) var connection: Connection?
    public let _connection: NIOLockedValueBox<Connection?>
    public var connection: Connection? {
        _connection.withLockedValue { $0 }
    }

    public let channel: Channel
    public let id: UInt64
    public let name: String?
    public let mode: LibP2P.Mode

    private let _protocolCodec: NIOLockedValueBox<String>
    public var protocolCodec: String {
        _protocolCodec.withLockedValue { $0 }
    }

    public var direction: ConnectionStats.Direction {
        self.mode == .listener ? .inbound : .outbound
    }

    private let streamID: MPLEXStreamID
    //public  lazy var eventLoop:EventLoop = {
    //    channel.eventLoop
    //}()

    public required init(
        channel: Channel,
        mode: LibP2P.Mode,
        id: UInt64,
        name: String?,
        proto: String,
        streamState: LibP2PCore.StreamState = .initialized
    ) {
        self.channel = channel
        self.mode = mode
        self.id = id
        self.name = name
        self._streamState = .init(streamState)
        self._protocolCodec = .init(proto)

        self.streamID = MPLEXStreamID(id: id, mode: mode)
        self._connection = .init(nil)
        self._on = .init(nil)
        //print("Stream[\(id)]::Initializing")
    }

    //deinit {
    //print("Stream[\(id)]::Deinitializing")
    //}

    /// Main Delegate/Callback Function
    public typealias EventCallback = (@Sendable (LibP2PCore.StreamEvent) -> EventLoopFuture<Void>)?
    private let _on: NIOLockedValueBox<EventCallback>
    public var on: EventCallback {
        get { _on.withLockedValue { $0 } }
        set { _on.withLockedValue { $0 = newValue } }
    }

    public func write(_ data: Data) -> EventLoopFuture<Void> {
        self.write(channel.allocator.buffer(bytes: data.byteArray))
    }

    public func write(_ bytes: [UInt8]) -> EventLoopFuture<Void> {
        self.write(channel.allocator.buffer(bytes: bytes))
    }

    public func write(_ buffer: ByteBuffer) -> EventLoopFuture<Void> {
        //let promise = self.channel.eventLoop.makePromise(of: Void.self)
        // Prepend our Protocol Header Stream ID / FLAG

        //print("Stream[\(streamID.id)] -> Attempting to write to channel")
        guard self.channel.isActive && self.channel.isWritable else {
            self._streamState.withLockedValue { $0 = .reset }
            return self.channel.eventLoop.makeFailedFuture(Errors.streamNotWritable)
        }
        guard self.streamState == .open else {
            return self.channel.eventLoop.makeFailedFuture(Errors.streamNotWritable)
        }
        // Write it out
        self.channel.writeAndFlush(RawResponse(payload: buffer), promise: nil)
        //self.channel.writeAndFlush(NIOAny(MPLEXFrame(streamID: streamID, payload: .outboundData(buffer))), promise: nil)
        return self.channel.eventLoop.makeSucceededVoidFuture()
        //return promise.futureResult
    }

    /// Sends a close stream message to our remote peer, requesting this Stream be closed.
    /// - Note: Because there can be multiple MPLEXStreams over a single Connection, this will NOT close the underlying Connection.
    public func close(gracefully: Bool) -> EventLoopFuture<Void> {
        switch self._streamState.withLockedValue({ $0 }) {
        case .initialized, .open:
            self._streamState.withLockedValue { $0 = .writeClosed }
        case .receiveClosed:
            self._streamState.withLockedValue { $0 = .closed }
        case .writeClosed, .closed, .reset:
            return self.channel.eventLoop.makeSucceededVoidFuture()
        }
        self.channel.close(mode: .all, promise: nil)
        return self.channel.eventLoop.makeSucceededVoidFuture()
    }

    /// Sends a reset stream message to our remote peer, immediately shutting down the Stream.
    /// - Note: Once an MPLEXStream has been reset, you can no longer write / read to / from it.
    public func reset() -> EventLoopFuture<Void> {
        let promise = self.channel.eventLoop.makePromise(of: Void.self)
        if self.channel.isActive && self.channel.isWritable {
            print("Stream[\(streamID.id)] -> Writing Reset Message")
            self.channel.writeAndFlush(MPLEXFrame(streamID: streamID, payload: .reset), promise: nil)
        } else {
            print("Stream[\(streamID.id)] -> Skipping Reset Message, Channel already closed...")
        }
        //        return promise.futureResult.always { _ in
        //            print("Stream[\(self.streamID.id)] -> Reseting")
        //            self.channel.close(mode: .all, promise: nil)
        //        }
        self._streamState.withLockedValue { $0 = .reset }
        self.channel.close(mode: .all, promise: promise)
        return promise.futureResult
    }

    /// Kicks off the Stream handshake (if we're the initiator)
    public func resume() -> EventLoopFuture<Void> {
        // Ask our connection to establish a channel if it hasn't already (actually dial / bind to remote peer)
        print("TODO: Resume / Kick off our Stream")
        return self.channel.eventLoop.makeSucceededVoidFuture()
    }

    internal func updateStreamState(state: StreamState, protocol: String) {
        // Update our state if it's a valid transition...
        if state.rawValue > self._streamState.withLockedValue({ $0 }).rawValue {
            //print("Stream[\(streamID.id)] -> Updating state from \(self._streamState) -> \(state)")
            self._streamState.withLockedValue { $0 = state }
        }
        //else { print("Stream[\(streamID.id)] -> Skipping invalid state change from \(self._streamState) -> \(state)") }

        // Update our protocol if it hasn't been set yet
        guard self.protocolCodec == "" else {
            //print("Stream[\(streamID.id)] -> Protocol Codec can't be changed once set. Skipping protocol change request from `\(self.protocolCodec)` -> `\(`protocol`)`")
            return
        }
        //print("Stream[\(streamID.id)] -> Updating protocol from \(self.protocolCodec) -> \(`protocol`)")
        self._protocolCodec.withLockedValue { $0 = `protocol` }
    }

    public enum Errors: Error {
        case streamNotWritable
    }
}
