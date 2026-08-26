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

@Suite("Coder Tests")
struct CoderTests {

    // TODO: Test entire sequences (init session, open stream, exchange data, close stream, close session)

    @Test func testFrameDecoderNewStreams() throws {
        let inboundData: [UInt8] = [
            0, 0, 0, 5, 0, 0, 0, 1, 0, 0, 0, 12, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33,
            0, 0, 0, 5, 0, 0, 0, 3, 0, 0, 0, 12, 72, 101, 108, 108, 111, 32, 87, 111, 114, 108, 100, 33,
        ]

        let channel = EmbeddedChannel()
        let inbound = channel.allocator.buffer(bytes: inboundData)

        var payload = ByteBuffer()
        payload.writeString("Hello World!")

        let exepectedInOuts = [
            (
                inbound,
                [
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [.syn, .fin],
                            streamID: 1,
                            length: UInt32(payload.readableBytes)
                        ),
                        payload: payload
                    ),
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [.syn, .fin],
                            streamID: 3,
                            length: UInt32(payload.readableBytes)
                        ),
                        payload: payload
                    ),
                ]
            )
        ]

        #expect(throws: Never.self) {
            try ByteToMessageDecoderVerifier.verifyDecoder(
                inputOutputPairs: exepectedInOuts,
                decoderFactory: { FrameDecoder() }
            )
        }
    }

    @Test func testFrameDecoderSessionOpen_Ping() throws {
        let inboundData: [UInt8] = [
            0, 2, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0,
        ]

        let channel = EmbeddedChannel()
        let inbound = channel.allocator.buffer(bytes: inboundData)

        let exepectedInOuts = [
            (
                inbound,
                [
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .ping,
                            flags: [.syn],
                            streamID: 0,
                            length: 0
                        )
                    )
                ]
            )
        ]

        #expect(throws: Never.self) {
            try ByteToMessageDecoderVerifier.verifyDecoder(
                inputOutputPairs: exepectedInOuts,
                decoderFactory: { FrameDecoder() }
            )
        }
    }

    /// This is a capture / replay of a `swift-libp2p` Client dialing a `go-libp2p` Echo Server (./examples/echo) and issuing 2 echos requests
    /// - Note: This capture takes place AFTER the connection has been upgraded (both sec and muxer)
    @Test func testFrameDecoder_EchoCapture() throws {
        let inboundHexData: [String] = [
            // Peer Starting Session
            "000200010000000000000000",
            // Peer Agreed to open Channel 2
            "000100010000000200000000",
            // Peer upgrade request /multistream/1.0.0 - /ipfs/id/1.0.0
            "000000000000000200000024132f6d756c746973747265616d2f312e302e300a0f2f697066732f69642f312e302e300a",
            // Peer Agreed to open Channel 1
            "000100020000000100000000",
            // Peer speaks multistream on Channel 1
            "000000000000000100000014132f6d756c746973747265616d2f312e302e300a",
            // Peer Agreed to open Channel 3
            "000100020000000300000000",
            // Peer speaks multistream on Channel 3
            "000000000000000300000014132f6d756c746973747265616d2f312e302e300a",
            // Peer Confirmed Channel 2 closed
            "000100040000000200000000",
            // Peer agrees to upgrades Channel 1 to /echo/1.0.0
            "00000000000000010000000d0c2f6563686f2f312e302e300a",
            // Peer agrees to upgrades Channel 3 to /ipfs/id/1.0.0
            "0000000000000003000000100f2f697066732f69642f312e302e300a",
            // Peer begins sending ID on Channel 3
            "000000000000000300000002af08",
            // Peer finished sending ID on Channel 3
            "00000000000000030000042f0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100a8871d16208d36d56e5f53e8284e43ee1b11b8aa05d4b0ace9bf1d7457e3311664e82e4b82f30e53a42134722483fde245175766d9ccd3a0395dac375bb4946374c7237bc1e4cde42f183588902ebf5504734d2d557bb519059f2a0659dcaa4ba9e2bcfcbbf72141d7c277dfeb5d3f9ee2334e72f5edf09c783c52f765c2e8d7290dcf06ad59ff3e7e95df552a3780ce7e18d51652bd11a988924932ce4ceb1f0cd1d0ae9fbe00fec43fb12cf044e8abd22a2f7ab59f39cd7be53c391440f5e77c2205ce4cc901d8f901416a15ddc87a341c81852065bdcf14771f8196ba34df4ac72ca2f073caf24150cc6d677a9c63c1e68729985eb2546548188fdcfa36f30203010001120804ac1100020627101a0b2f6563686f2f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e30220804c0a8410106425a2a0032256769746875622e636f6d2f6c69627032702f676f2d6c69627032702f6578616d706c65734042fd040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100a8871d16208d36d56e5f53e8284e43ee1b11b8aa05d4b0ace9bf1d7457e3311664e82e4b82f30e53a42134722483fde245175766d9ccd3a0395dac375bb4946374c7237bc1e4cde42f183588902ebf5504734d2d557bb519059f2a0659dcaa4ba9e2bcfcbbf72141d7c277dfeb5d3f9ee2334e72f5edf09c783c52f765c2e8d7290dcf06ad59ff3e7e95df552a3780ce7e18d51652bd11a988924932ce4ceb1f0cd1d0ae9fbe00fec43fb12cf044e8abd22a2f7ab59f39cd7be53c391440f5e77c2205ce4cc901d8f901416a15ddc87a341c81852065bdcf14771f8196ba34df4ac72ca2f073caf24150cc6d677a9c63c1e68729985eb2546548188fdcfa36f30203010001120203011a460a22122007a83d8b64f660d12e2d5c712ced377fe3f72454aaeec7d4b95c31996fadd49610829bcbd490fcad9c181a0a0a08047f0000010627101a0a0a0804ac1100020627102a80027b1faebecbcb26848711986fc0379a817d6660d2a63d557dfabe9a210c90bd2d28f08b9d26e07174a93ecba23a19a9babf489d193d9f4849d19d8cd4fbb92c2c1163f235c727a138165ffbceb733f1088fc209f63ec839a2caf185d8cf71887f78b9f9f40898937fd05ef9bed29387198ce8a9165028019039ba90b55feb48a1415fc0b7764e32ab5e12de32e14c0df488fa00ee16bef318b37b44e215f41f23d4b6e813c9b3f65e75f3f5b7b27a45fe11ae3696467aa41219fc2e9311a3c2393b60370f492df04197fee7b10b37cdd35d53eb645c7b03a0b59bb6340a529d77eeac96accc4070c917b6adbe5d90d02fbac5936c953eac4e8dbb5dc992a85e99",
            // Peer Closes Channel 3
            "000100040000000300000000",
            // Receive our Echo message back on Channel 1
            "00000000000000010000001948656c6c6f2066726f6d207377696674206c6962703270210a",
            // Peer closing Channel 1
            "000100040000000100000000",
            // Peer agrees to open Channel 5
            "000100020000000500000000",
            // Peer speaks multistream on Channel 5
            "000000000000000500000014132f6d756c746973747265616d2f312e302e300a",
            // Peer agrees to upgrade to /echo/1.0.0 on Channel 5
            "00000000000000050000000d0c2f6563686f2f312e302e300a",
            // Receive our Echo message back on Channel 5
            "00000000000000050000001948656c6c6f2066726f6d207377696674206c6962703270210a",
            // Peer closing Channel 5
            "000100040000000500000000",
        ]

        let channel = EmbeddedChannel()
        let inbound = try channel.allocator.buffer(plainHexEncodedBytes: inboundHexData.joined())

        // Inbound payloads
        let payloads: [ByteBuffer] = try [
            ByteBuffer(
                plainHexEncodedBytes: "132f6d756c746973747265616d2f312e302e300a0f2f697066732f69642f312e302e300a"
            ),
            ByteBuffer(plainHexEncodedBytes: "132f6d756c746973747265616d2f312e302e300a"),
            ByteBuffer(plainHexEncodedBytes: "132f6d756c746973747265616d2f312e302e300a"),
            ByteBuffer(plainHexEncodedBytes: "0c2f6563686f2f312e302e300a"),
            ByteBuffer(plainHexEncodedBytes: "0f2f697066732f69642f312e302e300a"),
            ByteBuffer(plainHexEncodedBytes: "af08"),
            ByteBuffer(
                plainHexEncodedBytes:
                    "0aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100a8871d16208d36d56e5f53e8284e43ee1b11b8aa05d4b0ace9bf1d7457e3311664e82e4b82f30e53a42134722483fde245175766d9ccd3a0395dac375bb4946374c7237bc1e4cde42f183588902ebf5504734d2d557bb519059f2a0659dcaa4ba9e2bcfcbbf72141d7c277dfeb5d3f9ee2334e72f5edf09c783c52f765c2e8d7290dcf06ad59ff3e7e95df552a3780ce7e18d51652bd11a988924932ce4ceb1f0cd1d0ae9fbe00fec43fb12cf044e8abd22a2f7ab59f39cd7be53c391440f5e77c2205ce4cc901d8f901416a15ddc87a341c81852065bdcf14771f8196ba34df4ac72ca2f073caf24150cc6d677a9c63c1e68729985eb2546548188fdcfa36f30203010001120804ac1100020627101a0b2f6563686f2f312e302e301a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e30220804c0a8410106425a2a0032256769746875622e636f6d2f6c69627032702f676f2d6c69627032702f6578616d706c65734042fd040aab02080012a60230820122300d06092a864886f70d01010105000382010f003082010a0282010100a8871d16208d36d56e5f53e8284e43ee1b11b8aa05d4b0ace9bf1d7457e3311664e82e4b82f30e53a42134722483fde245175766d9ccd3a0395dac375bb4946374c7237bc1e4cde42f183588902ebf5504734d2d557bb519059f2a0659dcaa4ba9e2bcfcbbf72141d7c277dfeb5d3f9ee2334e72f5edf09c783c52f765c2e8d7290dcf06ad59ff3e7e95df552a3780ce7e18d51652bd11a988924932ce4ceb1f0cd1d0ae9fbe00fec43fb12cf044e8abd22a2f7ab59f39cd7be53c391440f5e77c2205ce4cc901d8f901416a15ddc87a341c81852065bdcf14771f8196ba34df4ac72ca2f073caf24150cc6d677a9c63c1e68729985eb2546548188fdcfa36f30203010001120203011a460a22122007a83d8b64f660d12e2d5c712ced377fe3f72454aaeec7d4b95c31996fadd49610829bcbd490fcad9c181a0a0a08047f0000010627101a0a0a0804ac1100020627102a80027b1faebecbcb26848711986fc0379a817d6660d2a63d557dfabe9a210c90bd2d28f08b9d26e07174a93ecba23a19a9babf489d193d9f4849d19d8cd4fbb92c2c1163f235c727a138165ffbceb733f1088fc209f63ec839a2caf185d8cf71887f78b9f9f40898937fd05ef9bed29387198ce8a9165028019039ba90b55feb48a1415fc0b7764e32ab5e12de32e14c0df488fa00ee16bef318b37b44e215f41f23d4b6e813c9b3f65e75f3f5b7b27a45fe11ae3696467aa41219fc2e9311a3c2393b60370f492df04197fee7b10b37cdd35d53eb645c7b03a0b59bb6340a529d77eeac96accc4070c917b6adbe5d90d02fbac5936c953eac4e8dbb5dc992a85e99"
            ),
            ByteBuffer(plainHexEncodedBytes: "48656c6c6f2066726f6d207377696674206c6962703270210a"),
            ByteBuffer(plainHexEncodedBytes: "132f6d756c746973747265616d2f312e302e300a"),
            ByteBuffer(plainHexEncodedBytes: "0c2f6563686f2f312e302e300a"),
            ByteBuffer(plainHexEncodedBytes: "48656c6c6f2066726f6d207377696674206c6962703270210a"),
        ]

        let exepectedInOuts = [
            (
                inbound,
                [
                    // Peer Starting Session
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .ping,
                            flags: [.syn],
                            streamID: 0,
                            length: 0
                        )
                    ),

                    // Agreeing to Session Start
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.ping, flags: Set([LibP2PYAMUX.Header.Flag.ack]), streamID: 0, length: 0), payload: nil)
                    // Outbound Data: '000200020000000000000000' --

                    // Peer Requesting to open Channel 2
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .windowUpdate,
                            flags: [.syn],
                            streamID: 2,
                            length: 0
                        )
                    ),

                    // Agree to open Channel 2
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.windowUpdate, flags: Set([LibP2PYAMUX.Header.Flag.ack]), streamID: 2, length: 0), payload: nil)
                    // Outbound Data: '000100020000000200000000' --

                    // Peer upgrade request /multistream/1.0.0 - /ipfs/id/1.0.0
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 2,
                            length: UInt32(payloads[0].readableBytes)
                        ),
                        payload: payloads[0]
                    ),

                    // Agree to upgrade request /multistream/1.0.0 - /ipfs/id/1.0.0
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.data, flags: Set([]), streamID: 2, length: 36), payload: Optional([132f6d756c746973747265616d2f312e302e300a0f2f697066732f69642f312e302e300a](36 bytes)))
                    // Outbound Data: '000000000000000200000024132f6d756c746973747265616d2f312e302e300a0f2f697066732f69642f312e302e300a'

                    // Ask to open Channel 1
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.windowUpdate, flags: Set([LibP2PYAMUX.Header.Flag.syn]), streamID: 1, length: 0), payload: nil)
                    // Outbound Data: '000100010000000100000000'

                    // Ask to open Channel 3
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.windowUpdate, flags: Set([LibP2PYAMUX.Header.Flag.syn]), streamID: 3, length: 0), payload: nil)
                    // Outbound Data: '000100010000000300000000'

                    // Publish our id request on Channel 2
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.data, flags: Set([]), streamID: 2, length: 379), payload: Optional([f9020a24080112205174aa8ed342a375ed9bf58d77a2eb8f585d826cd47ee7b6...b4b65cd2545323cf2dc1b3b4fd6d7e91c2a1728e30cca535ebc5323f4eadc10e](379 bytes)))
                    // Outbound Data: '00000000000000020000017bf9020a24080112205174aa8ed342a375ed9bf58d77a2eb8f585d826cd47ee7b6c4d2938ef15827681231047f000001062711a503260024080112205174aa8ed342a375ed9bf58d77a2eb8f585d826cd47ee7b6c4d2938ef15827681a0e2f697066732f69642f312e302e301a132f697066732f69642f707573682f312e302e301a102f697066732f70696e672f312e302e301a132f7032702f69642f64656c74612f312e302e302208047f0000010627102a0a697066732f302e312e30321073776966742d697066732f302e312e3042a9010a24080112205174aa8ed342a375ed9bf58d77a2eb8f585d826cd47ee7b6c4d2938ef1582768120203011a3b0a260024080112205174aa8ed342a375ed9bf58d77a2eb8f585d826cd47ee7b6c4d2938ef158276810c490aaf6e5321a0a0a08047f0000010627112a404865f6352cc02213513390fada3422ef39534fa65e2114a2606933c69c33e3a1b4b65cd2545323cf2dc1b3b4fd6d7e91c2a1728e30cca535ebc5323f4eadc10e'

                    // Close Channel 2
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.windowUpdate, flags: Set([LibP2PYAMUX.Header.Flag.fin]), streamID: 2, length: 0), payload: nil)
                    // Outbound Data: '000100040000000200000000'

                    // Peer Agreed to open Channel 1
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .windowUpdate,
                            flags: [.ack],
                            streamID: 1,
                            length: 0
                        )
                    ),

                    // Attempt to upgrade Channel 1 to /multistream/1.0.0 /echo/1.0.0
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.data, flags: Set([]), streamID: 1, length: 33), payload: Optional([132f6d756c746973747265616d2f312e302e300a0c2f6563686f2f312e302e300a](33 bytes)))
                    // Outbound Data: '000000000000000100000021132f6d756c746973747265616d2f312e302e300a0c2f6563686f2f312e302e300a'

                    // Peer speaks multistream on Channel 1
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 1,
                            length: UInt32(payloads[1].readableBytes)
                        ),
                        payload: payloads[1]
                    ),

                    // Peer Agreed to open Channel 3
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .windowUpdate,
                            flags: [.ack],
                            streamID: 3,
                            length: 0
                        )
                    ),

                    // Attempt to upgrade Channel 3 to /multistream/1.0.0 /ipfs/id/1.0.0
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.data, flags: Set([]), streamID: 3, length: 36), payload: Optional([132f6d756c746973747265616d2f312e302e300a0f2f697066732f69642f312e302e300a](36 bytes)))
                    // Outbound Data: '000000000000000300000024132f6d756c746973747265616d2f312e302e300a0f2f697066732f69642f312e302e300a'

                    // Peer speaks multistream on Channel 3
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 3,
                            length: UInt32(payloads[2].readableBytes)
                        ),
                        payload: payloads[2]
                    ),

                    // Peer Confirmed Channel 2 closed
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .windowUpdate,
                            flags: [.fin],
                            streamID: 2,
                            length: 0
                        )
                    ),

                    // Peer agrees to upgrades Channel 1 to /echo/1.0.0
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 1,
                            length: UInt32(payloads[3].readableBytes)
                        ),
                        payload: payloads[3]
                    ),

                    // Peer agrees to upgrades Channel 3 to /ipfs/id/1.0.0
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 3,
                            length: UInt32(payloads[4].readableBytes)
                        ),
                        payload: payloads[4]
                    ),

                    // Peer begins sending ID on Channel 3
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 3,
                            length: UInt32(payloads[5].readableBytes)
                        ),
                        payload: payloads[5]
                    ),

                    // Peer finished sending ID on Channel 3
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 3,
                            length: UInt32(payloads[6].readableBytes)
                        ),
                        payload: payloads[6]
                    ),

                    // Peer Closes Channel 3
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .windowUpdate,
                            flags: [.fin],
                            streamID: 3,
                            length: 0
                        )
                    ),

                    // We agree to close Channel 3
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.windowUpdate, flags: Set([LibP2PYAMUX.Header.Flag.fin]), streamID: 3, length: 0), payload: nil)
                    // Outbound Data: '000100040000000300000000'

                    // Send our Echo message 'Hello from swift libp2p!' on upgraded Channel 1
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.data, flags: Set([]), streamID: 1, length: 25), payload: Optional([48656c6c6f2066726f6d207377696674206c6962703270210a](25 bytes)))
                    // Outbound Data: '00000000000000010000001948656c6c6f2066726f6d207377696674206c6962703270210a'

                    // Receive our Echo message back on Channel 1
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 1,
                            length: UInt32(payloads[7].readableBytes)
                        ),
                        payload: payloads[7]
                    ),

                    // Peer closing Channel 1
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .windowUpdate,
                            flags: [.fin],
                            streamID: 1,
                            length: 0
                        )
                    ),

                    // Agree to close Channel 1
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.windowUpdate, flags: Set([LibP2PYAMUX.Header.Flag.fin]), streamID: 1, length: 0), payload: nil)
                    // Outbound Data: '000100040000000100000000'

                    // Ask to open Channel 5
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.windowUpdate, flags: Set([LibP2PYAMUX.Header.Flag.syn]), streamID: 5, length: 0), payload: nil)
                    // Outbound Data: '000100010000000500000000'

                    // Peer Agreed to open Channel 5
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .windowUpdate,
                            flags: [.ack],
                            streamID: 5,
                            length: 0
                        )
                    ),

                    // Ask to upgrade Channel 5 to /multistream/1.0.0 /echo/1.0.0
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.data, flags: Set([]), streamID: 5, length: 33), payload: Optional([132f6d756c746973747265616d2f312e302e300a0c2f6563686f2f312e302e300a](33 bytes)))
                    // Outbound Data: '000000000000000500000021132f6d756c746973747265616d2f312e302e300a0c2f6563686f2f312e302e300a'

                    // Peer speaks multistream on Channel 5
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 5,
                            length: UInt32(payloads[8].readableBytes)
                        ),
                        payload: payloads[8]
                    ),

                    // Peer agrees to upgrade to /echo/1.0.0 on Channel 5
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 5,
                            length: UInt32(payloads[9].readableBytes)
                        ),
                        payload: payloads[9]
                    ),

                    // Send our Echo message 'Hello from swift libp2p!' on upgraded Channel 5
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.data, flags: Set([]), streamID: 5, length: 25), payload: Optional([48656c6c6f2066726f6d207377696674206c6962703270210a](25 bytes)))
                    // Outbound Data: '00000000000000050000001948656c6c6f2066726f6d207377696674206c6962703270210a'

                    // Receive our Echo message back on Channel 5
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .data,
                            flags: [],
                            streamID: 5,
                            length: UInt32(payloads[10].readableBytes)
                        ),
                        payload: payloads[10]
                    ),

                    // Peer closing Channel 5
                    Frame(
                        header: Header(
                            version: .v0,
                            messageType: .windowUpdate,
                            flags: [.fin],
                            streamID: 5,
                            length: 0
                        )
                    ),

                    // Agree to close Channel 5
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.windowUpdate, flags: Set([LibP2PYAMUX.Header.Flag.fin]), streamID: 5, length: 0), payload: nil)
                    // Outbound Data: '000100040000000500000000'

                    // Terminate the session without error
                    // Outbound Frame: Frame(header: LibP2PYAMUX.Header(version: LibP2PYAMUX.Header.Version.v0, messageType: LibP2PYAMUX.Header.MessageType.goAway, flags: Set([]), streamID: 0, length: 0), payload: nil)
                    // Outbound Data: '000300000000000000000000'
                ]
            )
        ]

        #expect(throws: Never.self) {
            try ByteToMessageDecoderVerifier.verifyDecoder(
                inputOutputPairs: exepectedInOuts,
                decoderFactory: { FrameDecoder() }
            )
        }
    }
}
