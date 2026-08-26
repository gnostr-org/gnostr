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
import VarInt

@testable import Multicodec

@Suite("Multicodec Tests")
struct MulticodecTests {

    @Test func testCodecRawValueDirect() throws {
        //Access a Codecs int64 value via the 'rawValue' property
        #expect(Codecs.eth_block.rawValue == 144)
        #expect(Codecs.dag_pb.rawValue == 112)
        #expect(Codecs.blake2b_8.rawValue == 0xb201)

        //Or the 'code' property
        #expect(Codecs.eth_block.code == 144)
        #expect(Codecs.dag_pb.code == 112)
        #expect(Codecs.blake2b_8.code == 0xb201)

        //Or compare the Codecs code to an int directly
        #expect(Codecs.eth_block == 144)
        #expect(Codecs.dag_pb == 112)
        #expect(Codecs.blake2b_8 == 0xb201)
    }

    func testCodecNamesDirect() throws {
        //Access a Codecs name via the name property
        #expect(Codecs.eth_block.name == "eth-block")
        #expect(Codecs.dag_pb.name == "dag-pb")
        #expect(Codecs.udp.name == "udp")
        #expect(Codecs.blake2b_8.name == "blake2b-8")

        //Or compare the Codecs name to a string directly...
        #expect(Codecs.eth_block == "eth-block")
        #expect(Codecs.dag_pb == "dag-pb")
        #expect(Codecs.udp == "udp")
        #expect(Codecs.blake2b_8 == "blake2b-8")
    }

    @Test func testCodecIntInstantiation() throws {
        #expect(try Codecs(144).name == "eth-block")
        #expect(try Codecs(112).name == "dag-pb")
        #expect(try Codecs(0x0111).name == "udp")
        #expect(try Codecs(0xb201).name == "blake2b-8")
    }

    @Test func testCodecsStringInstantiation() throws {
        let code = try Codecs("keccak-256")
        #expect(code.rawValue == 0x1b)
    }

    @Test func testEncodeDecodeBuffer() throws {
        let buf: [UInt8] = Array("hey".utf8)
        let prefixedBuf = try addPrefix(multiCodec: "protobuf", bytes: buf)
        #expect(try getCodec(bytes: prefixedBuf) == "protobuf")
        #expect(try removePrefix(bytes: prefixedBuf) == buf)
    }

    @Test func testEncodeDecodeBuffer1() throws {
        let buf: [UInt8] = Array("hey".utf8)
        let prefixedBuf = try addPrefix(code: 0x70, bytes: buf)
        #expect(try getCodec(bytes: prefixedBuf) == "dag-pb")
        #expect(try removePrefix(bytes: prefixedBuf) == buf)
    }

    @Test func testEncodeDecodeBuffer2() throws {
        let prefixedBuf = "hey".encodeUTF8(as: .dag_cbor)
        #expect(try getCodec(bytes: prefixedBuf) == "dag-cbor")
        #expect(String(bytes: try removePrefix(bytes: prefixedBuf), encoding: .utf8) == "hey")
    }

    @Test func testEncodeDecodeBuffer3() throws {
        let prefixedBuf = "hey".encodeUTF8(as: .eth_block)
        #expect(try getCodec(bytes: prefixedBuf) == "eth-block")
        let decoded = try prefixedBuf.decodeMulticodec(using: .utf8)
        #expect(decoded.codec == "eth-block")
        #expect(decoded.codec == Codecs.eth_block)
        #expect(decoded.contents == "hey")
    }

    @Test func testEncodeDecodeBufferAllCasesViaString() throws {
        let buf: [UInt8] = Array("hey".utf8)
        for codec in Codecs.allCases {
            let prefixedBuf = try addPrefix(multiCodec: codec.name, bytes: buf)
            #expect(try getCodec(bytes: prefixedBuf) == codec.name)
            #expect(try removePrefix(bytes: prefixedBuf) == buf)
        }
    }

    @Test func testEncodeDecodeBufferAllCasesViaCodec() throws {
        let buf: [UInt8] = Array("hey".utf8)
        for codec in Codecs.allCases {
            let prefixedBuf = addPrefix(codec: codec, bytes: buf)
            #expect(try getCodec(bytes: prefixedBuf) == codec.name)
            #expect(try removePrefix(bytes: prefixedBuf) == buf)
        }
    }

    @Test func testEncodeDecodeBufferAllCasesViaInt() throws {
        let buf: [UInt8] = Array("hey".utf8)
        for codec in Codecs.allCases {
            let prefixedBuf = try addPrefix(code: codec.rawValue, bytes: buf)
            #expect(try getCodec(bytes: prefixedBuf) == codec.name)
            #expect(try removePrefix(bytes: prefixedBuf) == buf)
        }
    }

    @Test func testVarIntRoundTrip() throws {
        #expect(uVarInt(putUVarInt(Codecs.keccak_256.rawValue)).0 == 0x1b)
    }

    /// Int instantiation time is roughly equal between the enum and dictionary (0.00025s) ...
    //    func testEnumCodecsIntInstantiationPerformance() throws {
    //        measure {
    //            XCTAssertTrue(try! Codecs(144) == "eth-block")
    //            XCTAssertTrue(try! Codecs(112) == "dag-pb")
    //            XCTAssertTrue(try! Codecs(0xb201) == "blake2b-8")
    //        }
    //    }
    //    func testDictionaryCodecsIntInstantiationPerformance() throws {
    //        measure {
    //            XCTAssertTrue(codecs.first(where: {$1 ==   0x90})?.key == "eth-block")
    //            XCTAssertTrue(codecs.first(where: {$1 ==   0x70})?.key ==    "dag-pb")
    //            XCTAssertTrue(codecs.first(where: {$1 == 0xb201})?.key == "blake2b-8")
    //        }
    //    }

    /// 0.000958s ( the Enum is 20 times slower than the dictionary when instantiating from string )
    //    func testEnumCodecsStringInstantiationPerformance() throws {
    //        measure {
    //            XCTAssertTrue(try! Codecs("eth-block") == 144)
    //            XCTAssertTrue(try! Codecs("dag-pb") == 112)
    //            XCTAssertTrue(try! Codecs("blake2b-8") == 0xb201)
    //        }
    //    }

    /// 0.0000527
    //    func testDictionaryCodecsStringInstantiationPerformance() throws {
    //        measure {
    //            XCTAssertTrue(codecs["eth-block"] ==    144)
    //            XCTAssertTrue(codecs["dag-pb"]    ==    112)
    //            XCTAssertTrue(codecs["blake2b-8"] == 0xb201)
    //        }
    //    }

    @Test func testP2PCodecClassification() throws {
        //Create our buffer with an unknown codec
        let hexInt = UInt64("01a5", radix: 16)!  //p2p int64 code
        let code: [UInt8] = putUVarInt(hexInt)  //p2p code as UInt8 array
        let buf: [UInt8] = Array("hey".utf8)  //Test buffer string
        let prefixedBuf = code + buf  //A p2p buffer

        #expect(try getCodec(bytes: prefixedBuf) == Codecs.p2p.name)
    }

    @Test func testP2P() throws {
        #expect(try Codecs(0x01a5).name == "p2p")
    }

    @Test func testIPFS() throws {
        #expect(try Codecs(0xe3).name == "ipfs")
    }

    @Test func testP2PIPFSInEquality() throws {
        #expect(Codecs.ipfs != Codecs.p2p)
    }

    // MARK: - Error Handling

    /// throws error on unknown codec name when getting the code
    @Test func testStringInstantiationWithUnknownCodecName() throws {
        #expect(throws: MulticodecError.UnknownCodecString) {
            try Codecs("this-codec-doesnt-exist")
        }
    }

    @Test func testGetCodecFromBufferWithUnknownCodec() throws {
        //Create our buffer with an unknown codec
        let hexInt = UInt64("ffee", radix: 16)!  //65518
        let code: [UInt8] = putUVarInt(hexInt)
        let buf: [UInt8] = Array("hey".utf8)
        let prefixedBuf = code + buf

        //Ensure it throws the UnknownCodecId Error...
        #expect(throws: MulticodecError.UnknownCodecId) {
            try getCodec(bytes: prefixedBuf)
        }
    }

    @Test func testPrefixBufferWithUnknownCodec() throws {
        let buf: [UInt8] = Array("hey".utf8)
        #expect(throws: MulticodecError.UnknownCodecId) {
            try addPrefix(code: 0xffee, bytes: buf)
        }
    }
}
