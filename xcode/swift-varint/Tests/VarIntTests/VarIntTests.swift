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

import Testing

@testable import VarInt

@Suite("VarInt Tests")
struct VarIntTests {
    @Test func testVarInt() {
        /// 1     => 00000001
        /// 127   => 01111111
        /// 128   => 10000000 00000001
        /// 255   => 11111111 00000001
        /// 300   => 10101100 00000010
        /// 16384 => 10000000 10000000 00000001

        #expect(putUVarInt(1) == [1])
        #expect(putUVarInt(127) == [127])
        #expect(putUVarInt(128) == [128, 1])
        #expect(putUVarInt(255) == [255, 1])
        #expect(putUVarInt(300) == [172, 2])
        #expect(putUVarInt(16384) == [128, 128, 1])

        #expect(uVarInt(putUVarInt(1)).0 == 1)
        #expect(uVarInt(putUVarInt(127)).0 == 127)
        #expect(uVarInt(putUVarInt(128)).0 == 128)
        #expect(uVarInt(putUVarInt(255)).0 == 255)
        #expect(uVarInt(putUVarInt(300)).0 == 300)
        #expect(uVarInt(putUVarInt(16384)).0 == 16384)

        #expect(uVarInt([1]).0 == 1)
        #expect(uVarInt([127]).0 == 127)
        #expect(uVarInt([128, 1]).0 == 128)
        #expect(uVarInt([255, 1]).0 == 255)
        #expect(uVarInt([172, 2]).0 == 300)
        #expect(uVarInt([128, 128, 1]).0 == 16384)

        #expect(putUVarInt(1).asBinaryChunks() == "00000001")
        #expect(putUVarInt(127).asBinaryChunks() == "01111111")
        #expect(putUVarInt(128).asBinaryChunks() == "10000000 00000001")
        #expect(putUVarInt(255).asBinaryChunks() == "11111111 00000001")
        #expect(putUVarInt(300).asBinaryChunks() == "10101100 00000010")
        #expect(putUVarInt(16384).asBinaryChunks() == "10000000 10000000 00000001")
    }

    @Test func testUVarIntLengthPrefix() throws {
        /// Create some arbitrary data
        let bytes = [UInt8]("Hello World".data(using: .utf8)!)

        /// Prefix the bytes with their length so we can recover the data later
        let uVarIntLengthPrefixedBytes = putUVarInt(UInt64(bytes.count)) + bytes

        /// ... send the data across a network or something ...

        /// Read the length prefixed data to determine the length of the payload
        let lengthPrefix = uVarInt(uVarIntLengthPrefixedBytes)
        #expect(lengthPrefix.value == 11)  // 11 -> Hello World == 11 bytes
        #expect(lengthPrefix.bytesRead == 1)  // 1  -> The value `11` fits into 1 byte

        /// So dropping the first byte will result in our original data again...
        let recBytes = [UInt8](uVarIntLengthPrefixedBytes.dropFirst(lengthPrefix.bytesRead))

        /// Assert the original bytes and the recovered bytes are equal
        #expect(bytes == recBytes)
    }
}
