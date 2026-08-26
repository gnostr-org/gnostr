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

import NIOCore

struct Header: Equatable {

    /// The version field is used for future backward compatibility.
    /// At the current time, the field is always set to 0, to indicate the initial version.
    internal enum Version: UInt8 {
        case v0 = 0x00
    }

    /// The type field is used to switch the frame message type. The following message types are supported:
    ///
    /// - 0x0 Data - Used to transmit data. May transmit zero length payloads depending on the flags.
    /// - 0x1 Window Update - Used to updated the senders receive window size. This is used to implement per-stream flow control.
    /// - 0x2 Ping - Used to measure RTT. It can also be used to heart-beat and do keep-alives over TCP.
    /// - 0x3 Go Away - Used to close a session.
    internal enum MessageType: UInt8, CaseIterable {
        case data = 0x00
        case windowUpdate = 0x01
        case ping = 0x02
        case goAway = 0x03
    }

    /// The flags field is used to provide additional information related to the message type.
    ///
    /// - 0x1 SYN - Signals the start of a new stream. May be sent with a data or window update message. Also sent with a ping to indicate outbound.
    /// - 0x2 ACK - Acknowledges the start of a new stream. May be sent with a data or window update message. Also sent with a ping to indicate response.
    /// - 0x4 FIN - Performs a half-close of a stream. May be sent with a data message or window update.
    /// - 0x8 RST - Reset a stream immediately. May be sent with a data or window update message.
    internal enum Flag: UInt16, CaseIterable {
        case syn = 0x01
        case ack = 0x02
        case fin = 0x04
        case reset = 0x08
    }

    /// The version field is used for future backward compatibility.
    /// At the current time, the field is always set to 0, to indicate the initial version.
    let version: Header.Version

    /// The type field is used to switch the frame message type.
    let messageType: Header.MessageType

    /// The flags field is used to provide additional information related to the message type.
    private(set) var flags: Set<Header.Flag>

    /// The StreamID field is used to identify the logical stream the frame is addressing.
    ///
    /// - The client side should use odd ID's, and the server even. This prevents any collisions.
    /// - Additionally, the 0 ID is reserved to represent the session.
    /// - Both Ping and Go Away messages should always use the 0 StreamID.
    let streamID: UInt32

    /// The meaning of the length field depends on the message type
    ///
    /// - Data - provides the length of bytes following the header
    /// - Window update - provides a delta update to the window size
    /// - Ping - Contains an opaque value, echoed back
    /// - Go Away - Contains an error code
    private(set) var length: UInt32

    init(
        version: Header.Version = .v0,
        messageType: Header.MessageType,
        flags: Set<Header.Flag> = [],
        streamID: UInt32,
        length: UInt32
    ) {
        self.version = version
        self.messageType = messageType
        self.flags = Set(flags)
        self.streamID = streamID
        self.length = length
    }

    /// Attempts to read and parse the 12 bytes that make up a YAMUX Header.
    /// - Note: this method is non destructive, it only consumes bytes from the ByteBuffer when a Header is returned.
    init(buffer: inout ByteBuffer) throws {
        self = try Header.decode(&buffer)
    }

    func encode(into buffer: inout ByteBuffer) {
        buffer.writeInteger(version.rawValue)
        buffer.writeInteger(messageType.rawValue)
        var flagFeild: UInt16 = 0
        for flag in flags {
            flagFeild |= flag.rawValue
        }
        buffer.writeInteger(flagFeild)
        buffer.writeInteger(streamID)
        buffer.writeInteger(length)
    }

    /// Attempts to read and parse the 12 bytes that make up a YAMUX Header.
    /// - Note: this method is non destructive, it only consumes bytes from the ByteBuffer when a Header is returned.
    static func decode(_ buffer: inout ByteBuffer) throws -> Header {
        // Ensure we have at least 12 readable bytes
        guard buffer.readableBytes >= 12 else { throw YAMUX.Error.invalidPacketFormat }
        let readerIndex = buffer.readerIndex
        // Read the version
        guard let rawV = buffer.getInteger(at: readerIndex, as: UInt8.self) else {
            throw YAMUX.Error.invalidPacketFormat
        }
        guard let version = Version(rawValue: rawV) else { throw YAMUX.Error.invalidPacketFormat }
        // Read the header type
        guard let rawT = buffer.getInteger(at: readerIndex + 1, as: UInt8.self) else {
            throw YAMUX.Error.invalidPacketFormat
        }
        guard let hType = MessageType(rawValue: rawT) else { throw YAMUX.Error.invalidPacketFormat }
        // Read the flags
        guard let rawFlags = buffer.getInteger(at: readerIndex + 2, as: UInt16.self) else {
            throw YAMUX.Error.invalidPacketFormat
        }
        var flagFeild: Set<Flag> = []
        for f in Flag.allCases {
            if (f.rawValue & rawFlags) == f.rawValue {
                flagFeild.insert(f)
            }
        }
        // Read the stream id
        guard let streamID = buffer.getInteger(at: readerIndex + 4, as: UInt32.self) else {
            throw YAMUX.Error.invalidPacketFormat
        }
        // Read the length
        guard let length = buffer.getInteger(at: readerIndex + 8, as: UInt32.self) else {
            throw YAMUX.Error.invalidPacketFormat
        }

        // Now that we've read the entire header, consume the bytes
        buffer.moveReaderIndex(to: readerIndex + 12)

        return Header(
            version: version,
            messageType: hType,
            flags: flagFeild,
            streamID: streamID,
            length: length
        )
    }

    /// Checks whether a Header is valid, throws when it's not
    func validate() throws {
        switch self.messageType {
        case .data:
            guard self.streamID != 0 else { throw YAMUX.Error.invalidPacketFormat }
            if length == 0 {
                // If the payload is empty, we must have a flag set...
                guard !self.flags.isEmpty else { throw YAMUX.Error.invalidPacketFormat }
            }
        case .windowUpdate:
            guard self.streamID != 0 else { throw YAMUX.Error.invalidPacketFormat }
            return
        case .ping:
            guard self.streamID == 0 else { throw YAMUX.Error.invalidPacketFormat }
            guard self.flags.isEmpty || self.flags.isSubset(of: [.syn, .ack]) else {
                throw YAMUX.Error.invalidPacketFormat
            }
        case .goAway:
            guard self.streamID == 0 else { throw YAMUX.Error.invalidPacketFormat }
            guard self.flags.isEmpty else { throw YAMUX.Error.invalidPacketFormat }
        }
    }
}

extension Header {

    mutating func addFlag(_ flag: Flag) {
        self.flags.insert(flag)
    }

    mutating func setLength(_ length: UInt32) {
        self.length = length
    }
}

extension Header {
    internal enum Message {
        case data(length: UInt32)
        case windowUpdate(delta: UInt32)
        case ping(payload: UInt32)
        case goAway(errorCode: YAMUX.NetworkError)

        var type: Header.MessageType {
            switch self {
            case .data: return .data
            case .windowUpdate: return .windowUpdate
            case .ping: return .ping
            case .goAway: return .goAway
            }
        }

        var rawValue: UInt8 {
            switch self {
            case .data: return MessageType.data.rawValue
            case .windowUpdate: return MessageType.windowUpdate.rawValue
            case .ping: return MessageType.ping.rawValue
            case .goAway: return MessageType.goAway.rawValue
            }
        }

        var length: UInt32 {
            switch self {
            case .data(let length):
                return length
            case .windowUpdate(let delta):
                return delta
            case .ping(let payload):
                return payload
            case .goAway(let errorCode):
                return errorCode.code
            }
        }
    }

    init(
        version: Header.Version = .v0,
        message: Header.Message,
        flags: [Header.Flag] = [],
        streamID: UInt32
    ) {
        self.version = version
        self.messageType = message.type
        self.flags = Set(flags)
        self.streamID = streamID
        self.length = message.length
    }
}

extension ByteBuffer {
    /// Attempts to read and parse the 12 bytes that make up a YAMUX Header.
    /// - Note: this method is non destructive, it only consumes bytes from the ByteBuffer when a Header is returned.
    mutating func readHeader() -> Header? {
        try? Header.decode(&self)
    }

    /// Encodes the header and appends the data to this buffer
    mutating func write(header: Header) {
        header.encode(into: &self)
    }
}
