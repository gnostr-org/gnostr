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
import NIOTestUtils
import Testing

@testable import LibP2PYAMUX

@Suite("Header Tests")
struct HeaderTests {

    /// Header frame encoding / decoding
    @Test func testHeaderEncoding_Zero() throws {
        // Create a basic header frame
        let header = Header(version: .v0, messageType: .data, flags: [], streamID: 0, length: 0)
        var buffer = ByteBuffer()
        // Encode the header into our bytebuffer
        header.encode(into: &buffer)

        // Ensure the header encoded within 12 bytes
        #expect(buffer.readableBytes == 12)
        // Ensure all bytes are 0 for this particular header
        #expect(
            buffer.readableBytesView.withUnsafeBytes { Array($0) } == [UInt8](repeating: 0, count: 12)
        )

        // Decode the header
        let decoded = try Header.decode(&buffer)

        // Ensure the decoded header is equal to the encoded header
        #expect(decoded == header)
        // Ensure we consumed all 12 bytes while decoding
        #expect(buffer.readableBytes == 0)
    }

    @Test func testHeaderEncoding_MessageType_GoAway() throws {
        for (index, errorCode) in YAMUX.NetworkError.allCases.enumerated() {
            // Create a basic header frame
            let header = Header(version: .v0, message: .goAway(errorCode: errorCode), flags: [], streamID: 0)
            var buffer = ByteBuffer()
            // Encode the header into our bytebuffer
            header.encode(into: &buffer)

            // Ensure the header encoded within 12 bytes
            #expect(buffer.readableBytes == 12)
            // Ensure all bytes are 0 for this particular header
            #expect(
                buffer.readableBytesView.withUnsafeBytes { Array($0) } == [
                    0, 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, UInt8(index),
                ]
            )

            // Decode the header
            let decoded = try Header.decode(&buffer)

            // Ensure the decoded header is equal to the encoded header
            #expect(decoded == header)
            // Ensure the decoded error code is equal to the encoded header error code
            #expect(decoded.messageType == .goAway)
            #expect(decoded.length == errorCode.code)
            // Ensure we consumed all 12 bytes while decoding
            #expect(buffer.readableBytes == 0)
        }
    }

    /// Header frame encoding / decoding
    @Test func testHeaderEncoding_Types() throws {
        for (index, mType) in Header.MessageType.allCases.enumerated() {
            // Create a basic header frame
            let header = Header(version: .v0, messageType: mType, flags: [], streamID: 0, length: 0)
            var buffer = ByteBuffer()
            // Encode the header into our bytebuffer
            header.encode(into: &buffer)

            // Ensure the header encoded within 12 bytes
            #expect(buffer.readableBytes == 12)
            // Ensure all bytes are 0 for this particular header
            #expect(
                buffer.readableBytesView.withUnsafeBytes { Array($0) } == [
                    0, UInt8(index), 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                ]
            )

            // Decode the header
            let decoded = try Header.decode(&buffer)

            // Ensure the decoded header is equal to the encoded header
            #expect(decoded == header)
            // Double check the header type is correct
            #expect(decoded.messageType == mType)
            // Ensure we consumed all 12 bytes while decoding
            #expect(buffer.readableBytes == 0)
        }
    }

    @Test func testHeaderEncoding_Flags() throws {
        // All permutations of our flags in order
        let testVectors: [Set<Header.Flag>] = [
            [],
            [.syn],
            [.ack],
            [.syn, .ack],
            [.fin],
            [.syn, .fin],
            [.ack, .fin],
            [.syn, .ack, .fin],
            [.reset],
            [.syn, .reset],
            [.ack, .reset],
            [.syn, .ack, .reset],
            [.fin, .reset],
            [.syn, .fin, .reset],
            [.ack, .fin, .reset],
            [.syn, .ack, .fin, .reset],
        ]

        // For each flag, ensure we can encode / decode it
        for (index, flags) in testVectors.enumerated() {
            // Create a basic header frame
            let header = Header(version: .v0, messageType: .data, flags: flags, streamID: 0, length: 0)
            var buffer = ByteBuffer()
            // Encode the header into our bytebuffer
            header.encode(into: &buffer)

            // Ensure the header encoded within 12 bytes
            #expect(buffer.readableBytes == 12)
            // Ensure all bytes are 0 for this particular header
            #expect(
                buffer.readableBytesView.withUnsafeBytes { Array($0) } == [
                    0, 0, 0, UInt8(index), 0, 0, 0, 0, 0, 0, 0, 0,
                ]
            )

            // Decode the header
            let decoded = try Header.decode(&buffer)

            // Ensure the decoded header is equal to the encoded header
            #expect(decoded == header)
            // Double Check the Flags are the same
            #expect(decoded.flags == flags)
            // Ensure we consumed all 12 bytes while decoding
            #expect(buffer.readableBytes == 0)
        }
    }

    @Test func testHeaderEncoding_All_Separate() throws {
        // All permutations of our flags in order
        let testVectors: [Set<Header.Flag>] = [
            [],
            [.syn],
            [.ack],
            [.syn, .ack],
            [.fin],
            [.syn, .fin],
            [.ack, .fin],
            [.syn, .ack, .fin],
            [.reset],
            [.syn, .reset],
            [.ack, .reset],
            [.syn, .ack, .reset],
            [.fin, .reset],
            [.syn, .fin, .reset],
            [.ack, .fin, .reset],
            [.syn, .ack, .fin, .reset],
        ]

        // For each header type
        for (typeIndex, messageType) in Header.MessageType.allCases.enumerated() {
            // For each flag, ensure we can encode / decode it
            for (flagIndex, flags) in testVectors.enumerated() {
                // Create a basic header frame
                let header = Header(version: .v0, messageType: messageType, flags: flags, streamID: 0, length: 0)
                var buffer = ByteBuffer()
                // Encode the header into our bytebuffer
                header.encode(into: &buffer)

                // Ensure the header encoded within 12 bytes
                #expect(buffer.readableBytes == 12)
                // Ensure all bytes are 0 for this particular header
                #expect(
                    buffer.readableBytesView.withUnsafeBytes { Array($0) } == [
                        0, UInt8(typeIndex), 0, UInt8(flagIndex), 0, 0, 0, 0, 0, 0, 0, 0,
                    ]
                )

                // Decode the header
                let decoded = try Header.decode(&buffer)

                // Ensure the decoded header is equal to the encoded header
                #expect(decoded == header)
                // Double Check the type is the same
                #expect(decoded.messageType == messageType)
                // Double Check the Flags are the same
                #expect(decoded.flags == flags)
                // Ensure we consumed all 12 bytes while decoding
                #expect(buffer.readableBytes == 0)
            }
        }
    }

    @Test func testHeaderEncoding_All_Conjoined() throws {
        // All permutations of our flags in order
        let testVectors: [Set<Header.Flag>] = [
            [],
            [.syn],
            [.ack],
            [.syn, .ack],
            [.fin],
            [.syn, .fin],
            [.ack, .fin],
            [.syn, .ack, .fin],
            [.reset],
            [.syn, .reset],
            [.ack, .reset],
            [.syn, .ack, .reset],
            [.fin, .reset],
            [.syn, .fin, .reset],
            [.ack, .fin, .reset],
            [.syn, .ack, .fin, .reset],
        ]

        var buffer = ByteBuffer()
        var buffer2 = ByteBuffer()

        // For each header type
        for (typeIndex, messageType) in Header.MessageType.allCases.enumerated() {
            // For each flag, ensure we can encode / decode it
            for (flagIndex, flags) in testVectors.enumerated() {
                // Create a basic header frame
                let header = Header(version: .v0, messageType: messageType, flags: flags, streamID: 0, length: 0)
                // Encode the header into our bytebuffer
                header.encode(into: &buffer)
                // Use the convenience method on ByteBuffer
                buffer2.write(header: header)

                // Ensure the bytebuffer contains all of the headers so far
                #expect(buffer.readableBytes == ((typeIndex * 16) + (flagIndex + 1)) * 12)
                #expect(buffer2.readableBytes == ((typeIndex * 16) + (flagIndex + 1)) * 12)
            }
        }

        #expect(buffer.readableBytes == Header.MessageType.allCases.count * testVectors.count * 12)
        #expect(buffer2.readableBytes == Header.MessageType.allCases.count * testVectors.count * 12)

        // For each header type
        for (_, messageType) in Header.MessageType.allCases.enumerated() {
            // For each flag, ensure we can encode / decode it
            for (_, flags) in testVectors.enumerated() {
                // Decode the header
                let decoded = try Header.decode(&buffer)
                // Also decode the header using the ByteBuffer convenience method
                let decoded2 = buffer2.readHeader()

                // Ensure the convenience method decodes the same result
                #expect(decoded == decoded2)

                // Ensure the version is v0
                #expect(decoded.version == .v0)
                // Double Check the type is the same
                #expect(decoded.messageType == messageType)
                // Double Check the Flags are the same
                #expect(decoded.flags == flags)
                // Ensure the streamID and length are the same
                #expect(decoded.streamID == 0)
                #expect(decoded.length == 0)
            }
        }

        // Ensure we consumed all bytes while decoding
        #expect(buffer.readableBytes == 0)
        #expect(buffer2.readableBytes == 0)
    }

    @Test func testHeaderValidity() throws {

        // Invalid data message on StreamID 0
        #expect(throws: YAMUX.Error.invalidPacketFormat) {
            try Header(version: .v0, message: .data(length: 1), flags: [], streamID: 0).validate()
        }

        // Invalid data message on StreamID 0
        #expect(throws: YAMUX.Error.invalidPacketFormat) {
            try Header(version: .v0, message: .data(length: 0), flags: [], streamID: 1).validate()
        }

        // Valid data message, non zero length payload
        #expect(throws: Never.self) {
            try Header(version: .v0, message: .data(length: 1), flags: [], streamID: 1).validate()
        }

        // Valid data message, empty payload but contains one or more flags
        #expect(throws: Never.self) {
            try Header(version: .v0, message: .data(length: 0), flags: [.syn], streamID: 1).validate()
        }

        // Invalid window message on StreamID 0
        #expect(throws: YAMUX.Error.invalidPacketFormat) {
            try Header(version: .v0, message: .windowUpdate(delta: 1), flags: [], streamID: 0).validate()
        }

        // Valid window message on StreamID 1
        #expect(throws: Never.self) {
            try Header(version: .v0, message: .windowUpdate(delta: 1), flags: [], streamID: 1).validate()
        }

        // Invalid ping message on non zero Stream
        #expect(throws: YAMUX.Error.invalidPacketFormat) {
            try Header(version: .v0, message: .ping(payload: 1), flags: [], streamID: 1).validate()
        }

        // Valid ping message on StreamID 0
        #expect(throws: Never.self) {
            try Header(version: .v0, message: .ping(payload: 1), flags: [], streamID: 0).validate()
        }

        // Invalid goAway message on non zero Stream
        #expect(throws: YAMUX.Error.invalidPacketFormat) {
            try Header(version: .v0, message: .goAway(errorCode: .noError), flags: [], streamID: 1).validate()
        }

        // Valid goAway message on StreamID 0
        #expect(throws: Never.self) {
            try Header(version: .v0, message: .goAway(errorCode: .noError), flags: [], streamID: 0).validate()
        }
    }
}
