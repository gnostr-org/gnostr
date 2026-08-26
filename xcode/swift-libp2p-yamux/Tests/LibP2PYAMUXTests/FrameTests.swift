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

@Suite("Frame Tests")
struct FrameTests {

    let defaultWindowSize = UInt32(1024 * 256)

    /// Header frame encoding / decoding
    @Test func testFrameToMessages() throws {
        // Create a basic header frame
        var payload = ByteBuffer()
        payload.writeString("Hello World!")
        let header = Header(
            version: .v0,
            messageType: .data,
            flags: [.syn, .fin],
            streamID: 1,
            length: UInt32(payload.readableBytes)
        )
        let frame = Frame(header: header, payload: payload)

        // To messages
        #expect(
            frame.messages == [
                .channelOpen(.init(senderChannel: 1, initialWindowSize: 0, maximumPacketSize: defaultWindowSize)),
                .channelData(.init(recipientChannel: 1, data: payload)),
                .channelClose(.init(recipientChannel: 1)),
            ]
        )

        // Serialize
        var wireBuffer = ByteBuffer()
        wireBuffer.write(frame: frame)

        // Ensure the header encoded within 12 bytes
        #expect(wireBuffer.readableBytes == 12 + Int(header.length))
        // Ensure all bytes are 0 for this particular header
        #expect(
            wireBuffer.readableBytesView.withUnsafeBytes { Array($0) } == [0, 0, 0, 5, 0, 0, 0, 1, 0, 0, 0, 12] + [
                72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33,
            ]
        )

        let recoveredFrame = try wireBuffer.readFrame()

        #expect(recoveredFrame.header == header)
        #expect(recoveredFrame.payload == payload)
        #expect(
            recoveredFrame.messages == [
                .channelOpen(.init(senderChannel: 1, initialWindowSize: 0, maximumPacketSize: defaultWindowSize)),
                .channelData(.init(recipientChannel: 1, data: payload)),
                .channelClose(.init(recipientChannel: 1)),
            ]
        )
        #expect(wireBuffer.readableBytes == 0)
    }

    /// Header frame encoding / decoding
    @Test func testFrameToMessages_AddFlags() throws {
        // Create a basic header frame
        let payload = ByteBuffer(string: "Hello World!")
        //var frame = try Frame(streamID: 1, message: .data(payload: payload))
        var frame = Frame(
            header: Header(
                version: .v0,
                messageType: .data,
                flags: [],
                streamID: 1,
                length: UInt32(payload.readableBytes)
            ),
            payload: payload
        )
        frame.addFlag(.syn)
        frame.addFlag(.fin)

        // To messages
        #expect(
            frame.messages == [
                .channelOpen(.init(senderChannel: 1, initialWindowSize: 0, maximumPacketSize: defaultWindowSize)),
                .channelData(.init(recipientChannel: 1, data: payload)),
                .channelClose(.init(recipientChannel: 1)),
            ]
        )

        // Attempt to add a duplicate flag
        frame.addFlag(.syn)

        // Ensure we can't add duplicate flags
        #expect(
            frame.messages == [
                .channelOpen(.init(senderChannel: 1, initialWindowSize: 0, maximumPacketSize: defaultWindowSize)),
                .channelData(.init(recipientChannel: 1, data: payload)),
                .channelClose(.init(recipientChannel: 1)),
            ]
        )

        // Add another flag
        frame.addFlag(.ack)

        // Ensure we can't add duplicate flags
        #expect(
            frame.messages == [
                .channelOpen(.init(senderChannel: 1, initialWindowSize: 0, maximumPacketSize: defaultWindowSize)),
                .channelOpenConfirmation(
                    .init(
                        recipientChannel: 1,
                        senderChannel: 1,
                        initialWindowSize: 0,
                        maximumPacketSize: defaultWindowSize
                    )
                ),
                .channelData(.init(recipientChannel: 1, data: payload)),
                .channelClose(.init(recipientChannel: 1)),
            ]
        )

        // Add another flag
        frame.addFlag(.reset)

        // Ensure we can't add duplicate flags
        #expect(
            frame.messages == [
                .channelOpen(.init(senderChannel: 1, initialWindowSize: 0, maximumPacketSize: defaultWindowSize)),
                .channelOpenConfirmation(
                    .init(
                        recipientChannel: 1,
                        senderChannel: 1,
                        initialWindowSize: 0,
                        maximumPacketSize: defaultWindowSize
                    )
                ),
                .channelData(.init(recipientChannel: 1, data: payload)),
                .channelClose(.init(recipientChannel: 1)),
                .channelReset(.init(recipientChannel: 1, reasonCode: 0, description: "Stream Reset")),
            ]
        )
    }
}
