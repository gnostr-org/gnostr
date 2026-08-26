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

import CID
import Foundation
import Multibase
import Multihash
import Testing

@testable import Multiaddr

@Suite("Multiaddr Tests")
struct MultiaddrTests {
    @Test(.disabled())
    func testDump() throws {
        let addr = try Multiaddr("/dns6/foo.com/tcp/443/https")
        dump(addr)
    }

    @Test("Invalid Data Initialization", arguments: [Data()])
    func testDoesntInstantiateFromEmptyData(_ input: Data) {
        #expect(throws: MultiaddrError.invalidFormat) {
            try Multiaddr(input)
        }
    }

    @Test("Invalid String Initialization", arguments: ["", "/", "?", "😀"])
    func testDoesntInstantiateFromEmptyString(_ input: String) {
        #expect(throws: MultiaddrError.invalidFormat) {
            try Multiaddr(input)
        }
    }

    @Test func testCreateMultiaddrFromString() throws {
        let m = try! Multiaddr("/ip4/127.0.0.1/udp/1234")
        let expectedAddress1 = try Address(addrProtocol: .ip4, address: "127.0.0.1")
        let expectedAddress2 = try Address(addrProtocol: .udp, address: "1234")

        #expect(m.addresses.first == expectedAddress1)
        #expect(m.addresses.last == expectedAddress2)
    }

    @Test func testCreateMultiaddrFromString_LeadingSlashRequired() {
        #expect(throws: MultiaddrError.invalidFormat) {
            try Multiaddr("ip4/127.0.0.1/udp/1234")
        }
    }

    @Test func testSwapMultiaddrFromString() throws {
        let ma = try! Multiaddr("/ip4/127.0.0.1/udp/1234")
        // Assert Invalid Address Throws Error
        #expect(throws: MultiaddrError.parseIPv4AddressFail) { try ma.swap(address: "192.168.1.644", forCodec: .ip4) }
        // Assert Valid Address works
        #expect(try ma.swap(address: "192.168.1.44", forCodec: .ip4).description == "/ip4/192.168.1.44/udp/1234")
        // Assert Invalid UDP Port Throws Error
        #expect(throws: MultiaddrError.invalidPortValue) { try ma.swap(address: "1235.1", forCodec: .udp) }
        #expect(throws: MultiaddrError.invalidPortValue) { try ma.swap(address: "123511", forCodec: .udp) }
        // Assert Valid Port works
        #expect(try ma.swap(address: "1235", forCodec: .udp).description == "/ip4/127.0.0.1/udp/1235")
        // Assert our original ma is unchanged
        #expect(ma.description == "/ip4/127.0.0.1/udp/1234")
    }

    @Test func testSwapMultiaddrFromStringMutating() throws {
        var ma = try! Multiaddr("/ip4/127.0.0.1/udp/1234")
        // Assert Invalid Address Throws Error
        #expect(throws: MultiaddrError.parseIPv4AddressFail) {
            try ma.mutatingSwap(address: "192.168.1.644", forCodec: .ip4)
        }
        // Assert Valid Address works
        try ma.mutatingSwap(address: "192.168.1.44", forCodec: .ip4)
        #expect(ma.description == "/ip4/192.168.1.44/udp/1234")

        // Assert Invalid UDP Port Throws Error
        #expect(throws: MultiaddrError.invalidPortValue) { try ma.mutatingSwap(address: "1235.1", forCodec: .udp) }
        #expect(throws: MultiaddrError.invalidPortValue) { try ma.mutatingSwap(address: "123511", forCodec: .udp) }
        // Assert Valid Port works
        try ma.mutatingSwap(address: "1235", forCodec: .udp)
        #expect(ma.description == "/ip4/192.168.1.44/udp/1235")
    }

    @Test func testHashable() throws {
        let addresses = [
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt"),
            try Multiaddr("/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"),

            /// Duplicate Entries
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt"),
            try Multiaddr("/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"),
        ]

        #expect(addresses.count == 10)

        /// Convert to Set and ensure we only have 5
        #expect(Set(addresses).count == 5)
    }

    @Test func testContainsEquatable() throws {
        var addresses = [
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt"),
            try Multiaddr("/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"),
        ]

        /// Mostly Duplicate Entries
        let duplicates = [
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmQCU2EcMqAqQPR2i9bChDtGNJchTbq5TbXJJ16u19uLTa"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmbLHAnMoJPWSCR5Zhtx6BHJX9KiKNN6tpvbUcqanj75Nb"),
            try Multiaddr("/dnsaddr/bootstrap.libp2p.io/p2p/QmcZf59bWwK5XFi76CZX8cbJ4BhTzzA3gU1ZjYZcYW3dwt"),
            try Multiaddr("/ip4/104.131.131.82/tcp/4001/p2p/QmaCpDMGvV2BGHeYERUEnRQAwe3N8SzbUtfsmvsqQLuvuJ"),
        ]

        for duplicate in duplicates {
            if !addresses.contains(duplicate) {
                addresses.append(duplicate)
            }
        }

        #expect(addresses.count == 5)
    }

    @Test func testCreateMultiaddrFromBytes_IPv4() throws {
        let bytes = [0x04, 0xc0, 0x00, 0x02, 0x2a] as [UInt8]  // 04c000022a
        let data = Data(bytes: bytes, count: bytes.count)
        let m = try Multiaddr(data)

        #expect(m.description == "/ip4/192.0.2.42")
    }

    @Test func testCreateMultiaddrFromBytes_TcpAddress() throws {
        let bytes = [0x06, 0x10, 0xe1] as [UInt8]
        let data = Data(bytes: bytes, count: bytes.count)

        let m_fromString = try Multiaddr("/tcp/4321")
        let m_fromData = try Multiaddr(data)

        #expect(m_fromData.description == m_fromString.description)
        #expect(try m_fromData.binaryPacked() == m_fromString.binaryPacked())
    }

    @Test func testDnsSerialization() throws {
        let addr = try Multiaddr("/dns6/foo.com")
        let serialized = try addr.binaryPacked()

        let deserialized = try Multiaddr(serialized)
        #expect(addr == deserialized)
    }

    @Test func testCreateMultiaddrFromBytes_Onion() throws {
        let bytes = [0xBC, 0x03, 0x9a, 0x18, 0x08, 0x73, 0x06, 0x36, 0x90, 0x43, 0x09, 0x1f, 0x00, 0x50] as [UInt8]
        let data = Data(bytes: bytes, count: bytes.count)

        let m_fromString = try Multiaddr("/onion/timaq4ygg2iegci7:80")
        let m_fromData = try Multiaddr(data)

        #expect(m_fromData.description == m_fromString.description)
        #expect(try m_fromData.binaryPacked() == m_fromString.binaryPacked())
    }

    /// IPFS Overload no longer exists (these should no longer be equal)
    /// - Note: https://github.com/multiformats/multicodec/pull/283
    @Test func testCreateMultiaddrFromBytes_IpfsAddress() throws {
        let bytes =
            [
                0xa5, 0x03, 0x22, 0x12, 0x20, 0xd5, 0x2e, 0xbb, 0x89, 0xd8, 0x5b, 0x02, 0xa2, 0x84, 0x94, 0x82, 0x03,
                0xa6, 0x2f, 0xf2, 0x83, 0x89, 0xc5, 0x7c, 0x9f, 0x42, 0xbe, 0xec, 0x4e, 0xc2, 0x0d, 0xb7, 0x6a, 0x68,
                0x91, 0x1c, 0x0b,
            ] as [UInt8]
        let data = Data(bytes: bytes, count: bytes.count)

        let m_fromData = try Multiaddr(data)
        let m_fromStringIPFS = try Multiaddr("/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        let m_fromStringP2P = try Multiaddr("/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")

        /// The descriptions should not be the same
        #expect(m_fromData.description != m_fromStringIPFS.description)
        #expect(m_fromData.description == m_fromStringP2P.description)

        /// The binary data should be the same across p2p and deprecated ipfs codecs
        #expect(try m_fromData.binaryPacked() == m_fromStringP2P.binaryPacked())
        #expect(try m_fromData.binaryPacked() != m_fromStringIPFS.binaryPacked())
    }

    @Test func testCreateMultiaddrFromBytes_P2PAddressBase32() throws {
        let bytes =
            [
                0xa5, 0x03, 0x22, 0x12, 0x20, 0x73, 0xd7, 0x7b, 0x46, 0xc9, 0x4f, 0x21, 0x52, 0xc8, 0x07, 0x51, 0x16,
                0xbf, 0x54, 0xd3, 0x17, 0x73, 0xd5, 0x73, 0x03, 0x0b, 0xba, 0x13, 0xe7, 0xdb, 0x7d, 0x39, 0xf1, 0x2e,
                0x55, 0xb8, 0x7f,
            ] as [UInt8]
        let data = Data(bytes: bytes, count: bytes.count)

        let addrBase32 = try Multiaddr("/p2p/bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4")
        let base58Str = "/p2p/QmW8rAgaaA6sRydK1k6vonShQME47aDxaFidbtMevWs73t"
        let addrBase58 = try Multiaddr("/p2p/QmW8rAgaaA6sRydK1k6vonShQME47aDxaFidbtMevWs73t")

        // We preserve the encoding of the multihash when possible
        #expect(addrBase32.description == "/p2p/bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4")
        // The two should be equal regardless of their string encodings
        #expect(addrBase32 == addrBase58)
        #expect(try addrBase32 == Multiaddr(data))
        #expect(try addrBase58 == Multiaddr(data))
        // We preserve the encoding of the multihash when possible
        #expect(addrBase58.description == base58Str)
        // When instantiated from bytes we default to base58 encoding
        #expect(try Multiaddr(data).description == base58Str)
        #expect(try addrBase32.binaryPacked() == addrBase58.binaryPacked())
    }

    @Test func testCreateMultiaddrFromString_WithoutAddressValue() throws {
        let m = try! Multiaddr("/dns6/foo.com/tcp/443/https")
        let expectedAddress1 = try Address(addrProtocol: .dns6, address: "foo.com")
        let expectedAddress2 = try Address(addrProtocol: .tcp, address: "443")
        let expectedAddress3 = try Address(addrProtocol: .https, address: nil)

        #expect(m.addresses[0] == expectedAddress1)
        #expect(m.addresses[1] == expectedAddress2)
        #expect(m.addresses[2] == expectedAddress3)
    }

    @Test func testCreateMultiaddrFromString_AddressValueHasMultipleSlashes() throws {
        let m = try Multiaddr("/dns4/foo.com/tcp/80/http/")  // bar/baz.jpg
        let expectedAddress1 = try Address(addrProtocol: .dns4, address: "foo.com")
        let expectedAddress2 = try Address(addrProtocol: .tcp, address: "80")
        let expectedAddress3 = try Address(addrProtocol: .http, address: nil)  //"bar/baz.jpg")

        #expect(m.addresses[0] == expectedAddress1)
        #expect(m.addresses[1] == expectedAddress2)
        #expect(m.addresses[2] == expectedAddress3)
    }

    @Test func testCreateMultiaddrFromString_AddressValueHasColons() throws {
        let m = try Multiaddr("/ip6/::1/tcp/3217")
        let expectedAddress1 = try Address(addrProtocol: .ip6, address: "::1")
        let expectedAddress2 = try Address(addrProtocol: .tcp, address: "3217")

        #expect(m.addresses[0] == expectedAddress1)
        #expect(m.addresses[1] == expectedAddress2)
    }

    @Test func testEncapsulated_BasedOnStringEquality() throws {
        let m1 = try Multiaddr("/ip4/127.0.0.1")
        let m2 = try Multiaddr("/udt")

        let encapsulated = m1.encapsulate(m2)
        #expect(String(describing: encapsulated) == "/ip4/127.0.0.1/udt")

        let m3 = try Multiaddr("/ip4/127.0.0.1")
        let encapsulated2 = try m3.encapsulate("/udp/1234")
        #expect(String(describing: encapsulated2) == "/ip4/127.0.0.1/udp/1234")
    }

    @Test func testEncapsulated_BasedOnObjectEquality() throws {
        let m1 = try Multiaddr("/ip4/127.0.0.1")
        let m2 = try Multiaddr("/udt")

        let expected = try Multiaddr("/ip4/127.0.0.1/udt")
        #expect(m1.encapsulate(m2) == expected)
    }

    @Test func testDecapsulate() throws {
        let full = try Multiaddr("/ip4/1.2.3.4/tcp/80")
        let m1 = try Multiaddr("/tcp/80")
        let m2 = try Multiaddr("/ip4/1.2.3.4")

        #expect(full.decapsulate(m1) == m2)

        let m3 = try Multiaddr("/dns4/foo.com/tcp/80/http/")  //bar/baz.jpg")
        let decapsulated = m3.decapsulate(m1)
        #expect(try Multiaddr("/dns4/foo.com") == decapsulated)
    }

    @Test(
        "testCreateMultiaddrFromString_FailsWithInvalidStrings",
        arguments: [
            "notAProtocol",
            "/ip4/tcp/alsoNotAProtocol",
            "////ip4/tcp/21432141///",
            "////ip4///////tcp////",
        ]
    )
    func testCreateMultiaddrFromString_FailsWithInvalidStrings(_ input: String) throws {
        #expect(throws: MultiaddrError.self) {
            try Multiaddr(input)
        }
    }

    @Test func testBinaryPackedReturnsCorrectValue_For16BitProtocolPort() throws {
        let expected = "0601bb"
        let m = try Multiaddr("/tcp/443")
        let actual = try m.binaryPacked().hexString()
        #expect(actual == expected)
    }

    @Test func testBinaryPackedReturnsCorrectValue_ForIPv4Address() throws {
        let expected = "04c000022a"
        let m = try Multiaddr("/ip4/192.0.2.42")
        let actual = try m.binaryPacked().hexString()
        #expect(actual == expected)
    }

    @Test func testBinaryPackedThrowsError_ForInvalidIPv4Address() throws {
        #expect(throws: MultiaddrError.parseIPv4AddressFail) {
            try Multiaddr("/ip4/555.55.55.5").binaryPacked()
        }
    }

    @Test func testBinaryPacked_ForOnionAddress_EncodesCorrectly() throws {
        let expected = "bc039a18087306369043091f0050"
        let m = try Multiaddr("/onion/timaq4ygg2iegci7:80")
        let actual = try m.binaryPacked().hexString()
        #expect(actual == expected)
    }

    @Test func testBinaryPacked_ForP2PAddress_EncodesCorrectly() throws {
        let expected = "a503221220d52ebb89d85b02a284948203a62ff28389c57c9f42beec4ec20db76a68911c0b"
        let m = try Multiaddr("/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        let actual = try m.binaryPacked().hexString()
        #expect(actual == expected)
    }

    @Test func testIPv4FromBytes() throws {
        let addy = try Address(addrProtocol: .ip4, addressData: BaseEncoding.decode("c0a80001", as: .base16).data)
        #expect(addy.description == "/ip4/192.168.0.1")
    }

    @Test func testIPv6FromBytes() throws {
        let addy = try Address(
            addrProtocol: .ip6,
            addressData: BaseEncoding.decode("abcd0000000100020003000400050006", as: .base16).data
        )
        #expect(addy.description == "/ip6/abcd:0:1:2:3:4:5:6")
    }

    @Test func testIPv6FromString() throws {
        let addy = try Address(addrProtocol: .ip6, address: "ABCD::1:2:3:4:5:6")
        #expect(
            try IPv6.data(for: addy.address!).asString(base: .base16Upper) == "ABCD0000000100020003000400050006"
        )
    }

    @Test func testIPv4FromString() throws {
        let addy = try Address(addrProtocol: .ip4, address: "192.168.0.1")
        #expect(try IPv4.data(for: addy.address!).asString(base: .base16) == "c0a80001")
    }

    @Test func testIPv4InvalidString() throws {
        #expect(throws: MultiaddrError.parseIPv4AddressFail) { try Multiaddr(.ip4, address: "555.168.0.1") }
    }

    @Test func testIPv6InvalidString() throws {
        #expect(throws: MultiaddrError.parseIPv6AddressFail) { try Multiaddr(.ip6, address: "FFFF::GGGG") }
    }

    @Test(
        "Decode Certhash",
        arguments: [
            "046883835291020fa1cd03d103d203221220a78c594f830726e17fba30224d448d5c4a4434e9e5a14f24b3822d14da46d19bd203221220855beff35231e37b3c4970b3e16e0e100eba09adc3e1ad5473a16c97f258b61e"
        ]
    )
    func testDecodeEmbeddedCerthashFromBytes(_ input: String) throws {
        let addy1Bytes = Data(hex: input)

        let ma = try Multiaddr(addy1Bytes)

        let addy1Packed = try ma.binaryPacked()
        #expect(addy1Bytes == addy1Packed)
    }

    @Test func testConstructCerthashMultiaddr() throws {
        let ma1 = try Multiaddr(
            "/ip4/104.131.131.82/udp/4001/quic-v1/webtransport/certhash/f1220a78c594f830726e17fba30224d448d5c4a4434e9e5a14f24b3822d14da46d19b/certhash/f1220855beff35231e37b3c4970b3e16e0e100eba09adc3e1ad5473a16c97f258b61e"
        )
        let expectedPackedAddress1 = Data(
            hex:
                "046883835291020fa1cd03d103d203221220a78c594f830726e17fba30224d448d5c4a4434e9e5a14f24b3822d14da46d19bd203221220855beff35231e37b3c4970b3e16e0e100eba09adc3e1ad5473a16c97f258b61e"
        )
        #expect(try ma1.binaryPacked() == expectedPackedAddress1)
        let unpacked1 = try Multiaddr(expectedPackedAddress1)
        #expect(ma1 == unpacked1)
        #expect(try ma1.binaryPacked() == unpacked1.binaryPacked())

        let ma2 = try Multiaddr(
            "/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash/b2uaraocy6yrdblb4sfptaddgimjmmpy"
        )
        let expectedPackedAddress2 = Data(hex: "047f000001910204d2cd03d103d20313d501103858f62230ac3c915f300c664312c63f")
        #expect(try ma2.binaryPacked() == expectedPackedAddress2)
        let unpacked2 = try Multiaddr(expectedPackedAddress2)
        #expect(ma2 == unpacked2)
        #expect(try ma2.binaryPacked() == unpacked2.binaryPacked())

        let ma3 = try Multiaddr(
            "/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash/b2uaraocy6yrdblb4sfptaddgimjmmpy/certhash/zQmbWTwYGcmdyK9CYfNBcfs9nhZs17a6FQ4Y8oea278xx41"
        )
        let expectedPackedAddress3 = Data(
            hex:
                "047f000001910204d2cd03d103d20313d501103858f62230ac3c915f300c664312c63fd203221220c3ab8ff13720e8ad9047dd39466b3c8974e592c2fa383d4a3960714caef0c4f2"
        )
        #expect(try ma3.binaryPacked() == expectedPackedAddress3)
        let unpacked3 = try Multiaddr(expectedPackedAddress3)
        #expect(ma3 == unpacked3)
        #expect(try ma3.binaryPacked() == unpacked3.binaryPacked())
    }

    @Test func testFailesWithInvalidCerthash() throws {
        #expect(throws: MultiaddrError.parseAddressFail) {
            try Multiaddr("/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash")
        }
        #expect(throws: MultiaddrError.parseAddressFail) {
            try Multiaddr("/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash/b2uaraocy6yrdblb4sfptaddgimjmmp")
        }
    }

    @Test(
        "Invalid Multiaddr",
        arguments: [
            "/ip4",
            "/ip4/::1",
            "/ip4/fdpsofodsajfdoisa",
            "/ip4/1.2.3.4/ipcidr/256",
            "/ip6/::1/ipcidr/1026",
            "/ip6",
            "/ip6zone",
            "/ip6zone/",
            "/ip6zone//ip6/fe80::1",
            "/udp",
            "/tcp",
            "/sctp",
            "/udp/65536",
            "/tcp/65536",
            "/quic/65536",
            "/quic-v1/65536",
            "/onion/9imaq4ygg2iegci7:80",
            "/onion/aaimaq4ygg2iegci7:80",
            "/onion/timaq4ygg2iegci7:0",
            "/onion/timaq4ygg2iegci7:-1",
            "/onion/timaq4ygg2iegci7",
            "/onion/timaq4ygg2iegci@:666",
            "/onion3/9ww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:80",
            "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd7:80",
            "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:0",
            "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:-1",
            "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd",
            "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyy@:666",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA7:80",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA:0",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA:0",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA:-1",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA@:666",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA7:80",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA:0",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA:0",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA:-1",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA@:666",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzu",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzu77",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzu:80",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuq:-1",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzu@",
            "/udp/1234/sctp",
            "/udp/1234/udt/1234",
            "/udp/1234/utp/1234",
            "/ip4/127.0.0.1/udp/jfodsajfidosajfoidsa",
            "/ip4/127.0.0.1/udp",
            "/ip4/127.0.0.1/tcp/jfodsajfidosajfoidsa",
            "/ip4/127.0.0.1/tcp",
            "/ip4/127.0.0.1/quic/1234",
            "/ip4/127.0.0.1/quic-v1/1234",
            "/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash",
            // 1 character missing from certhash
            "/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash/b2uaraocy6yrdblb4sfptaddgimjmmp",
            "/ip4/127.0.0.1/ipfs",
            "/ip4/127.0.0.1/ipfs/tcp",
            "/ip4/127.0.0.1/p2p",
            "/ip4/127.0.0.1/p2p/tcp",
            "/unix",
            "/ip4/1.2.3.4/tcp/80/unix",
            "/ip4/127.0.0.1/tcp/9090/http/p2p-webcrt-direct",
            "/",
            "",
        ]
    )
    func testInvalidMutliaddr(_ input: String) throws {
        #expect(throws: Error.self) { try Multiaddr(input) }
    }

    @Test(
        "Valid Multiaddr",
        arguments: [
            "/ip4/1.2.3.4",
            "/ip4/0.0.0.0",
            "/ip4/192.0.2.0/ipcidr/24",
            "/ip6/::1",
            "/ip6/2601:9:4f81:9700:803e:ca65:66e8:c21",
            "/ip6/2601:9:4f81:9700:803e:ca65:66e8:c21/udp/1234/quic",
            "/ip6/2601:9:4f81:9700:803e:ca65:66e8:c21/udp/1234/quic-v1",
            "/ip6/2001:db8::/ipcidr/32",
            "/ip6zone/x/ip6/fe80::1",
            "/ip6zone/x%y/ip6/fe80::1",
            "/ip6zone/x%y/ip6/::",
            "/ip6zone/x/ip6/fe80::1/udp/1234/quic",
            "/ip6zone/x/ip6/fe80::1/udp/1234/quic-v1",
            "/onion/timaq4ygg2iegci7:1234",
            "/onion/timaq4ygg2iegci7:80/http",
            "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:1234",
            "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:80/http",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA/http",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA/udp/8080",
            "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA/tcp/8080",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuq",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuqzwas",
            //"/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuqzwassw", // Base32Pad stray bits error
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuq/http",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuq/tcp/8080",
            "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuq/udp/8080",
            "/udp/0",
            "/tcp/0",
            "/sctp/0",
            "/udp/1234",
            "/tcp/1234",
            "/sctp/1234",
            "/udp/65535",
            "/tcp/65535",
            "/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC",
            "/ipfs/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7",
            "/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC",
            "/p2p/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7",
            "/p2p/bafzbeigvf25ytwc3akrijfecaotc74udrhcxzh2cx3we5qqnw5vgrei4bm",
            "/p2p/12D3KooWCryG7Mon9orvQxcS1rYZjotPgpwoJNHHKcLLfE4Hf5mV",
            "/p2p/k51qzi5uqu5dhb6l8spkdx7yxafegfkee5by8h7lmjh2ehc2sgg34z7c15vzqs",
            "/p2p/bafzaajaiaejcalj543iwv2d7pkjt7ykvefrkfu7qjfi6sduakhso4lay6abn2d5u",
            "/udp/1234/sctp/1234",
            "/udp/1234/udt",
            "/udp/1234/utp",
            "/tcp/1234/http",
            "/tcp/1234/tls/http",
            "/tcp/1234/https",
            "/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234",
            "/ipfs/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7/tcp/1234",
            "/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234",
            "/p2p/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7/tcp/1234",
            "/ip4/127.0.0.1/udp/1234",
            "/ip4/127.0.0.1/udp/0",
            "/ip4/127.0.0.1/tcp/1234",
            "/ip4/127.0.0.1/tcp/1234/",
            "/ip4/127.0.0.1/udp/1234/quic",
            "/ip4/127.0.0.1/udp/1234/quic-v1",
            "/ip4/127.0.0.1/udp/1234/quic-v1/webtransport",
            "/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash/b2uaraocy6yrdblb4sfptaddgimjmmpy",
            "/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash/b2uaraocy6yrdblb4sfptaddgimjmmpy/certhash/zQmbWTwYGcmdyK9CYfNBcfs9nhZs17a6FQ4Y8oea278xx41",
            "/ip4/127.0.0.1/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC",
            "/ip4/127.0.0.1/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234",
            "/ip4/127.0.0.1/ipfs/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7",
            "/ip4/127.0.0.1/ipfs/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7/tcp/1234",
            "/ip4/127.0.0.1/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC",
            "/ip4/127.0.0.1/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234",
            "/ip4/127.0.0.1/p2p/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7",
            "/ip4/127.0.0.1/p2p/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7/tcp/1234",
            "/unix/a/b/c/d/e",
            "/unix/stdio",
            "/ip4/1.2.3.4/tcp/80/unix/a/b/c/d/e/f",
            "/ip4/127.0.0.1/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234/unix/stdio",
            "/ip4/127.0.0.1/ipfs/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7/tcp/1234/unix/stdio",
            "/ip4/127.0.0.1/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234/unix/stdio",
            "/ip4/127.0.0.1/p2p/k2k4r8oqamigqdo6o7hsbfwd45y70oyynp98usk7zmyfrzpqxh1pohl7/tcp/1234/unix/stdio",
            "/ip4/127.0.0.1/tcp/9090/http/p2p-webrtc-direct",
            "/ip4/127.0.0.1/tcp/127/ws",
            "/ip4/127.0.0.1/tcp/127/ws",
            "/ip4/127.0.0.1/tcp/127/tls",
            "/ip4/127.0.0.1/tcp/127/tls/ws",
            "/ip4/127.0.0.1/tcp/127/noise",
            "/ip4/127.0.0.1/tcp/127/wss",
            "/ip4/127.0.0.1/tcp/127/wss",
            "/ip4/127.0.0.1/tcp/127/webrtc-direct",
            "/ip4/127.0.0.1/tcp/127/webrtc",
        ]
    )
    func testValidMutliaddr(_ address: String) throws {

        let addy = try Multiaddr(address)
        #expect(
            addy.description == (address.hasSuffix("/") ? String(address.dropLast()) : address),
            "Failed to preserve string address: \(address)"
        )
        #expect(try Multiaddr(addy.binaryPacked()) == addy, "Failed to roundtrip address: \(address)")
    }

    @Test func testEquality() throws {
        let m1 = try Multiaddr("/ip4/127.0.0.1/udp/1234")
        let m2 = try Multiaddr("/ip4/127.0.0.1/tcp/1234")
        let m3 = try Multiaddr("/ip4/127.0.0.1/tcp/1234")
        let m4 = try Multiaddr("/ip4/127.0.0.1/tcp/1234/")

        #expect(m1 != m2)
        #expect(m2 != m1)
        #expect(m1 == m1)
        #expect(m2 == m3)
        #expect(m3 == m2)
        #expect(m2 == m4)
        #expect(m3 == m4)
    }

    @Test func testStringAddressToBinaryPacked() throws {
        #expect(try Multiaddr("/ip4/127.0.0.1/udp/1234").binaryPacked().toHexString() == "047f000001910204d2")
        #expect(try Multiaddr("/ip4/127.0.0.1/tcp/4321").binaryPacked().toHexString() == "047f0000010610e1")
        #expect(
            try Multiaddr("/ip4/127.0.0.1/udp/1234/ip4/127.0.0.1/tcp/4321").binaryPacked().toHexString()
                == "047f000001910204d2047f0000010610e1"
        )
        #expect(
            try Multiaddr("/onion/aaimaq4ygg2iegci:80").binaryPacked().toHexString() == "bc030010c0439831b48218480050"
        )
        #expect(
            try Multiaddr("/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:1234").binaryPacked()
                .toHexString() == "bd03adadec040be047f9658668b11a504f3155001f231a37f54c4476c07fb4cc139ed7e30304d2"
        )
        #expect(
            try Multiaddr(
                "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA"
            ).binaryPacked().toHexString()
                == "be0383038d3fc8c976a86ae4e78ba378e75ec41bc9ab1542a9cb422581987e118f5cb0c024f3639d6ad9b3aff613672f07bfbbbfc2f920ef910534ecaa6ff9c03e0fa4872a764d2fce6d4cfc5a5a9800cd95944cc9ef0241f753fe71494a175f334b35682459acadc4076428ab49b5a83a49d2ea2366b06461e4a559b0111fa750e0de0c138a94d1231ed5979572ff53922905636221994bdabc44bd0c17fef11622b16432db3f193400af53cc61aa9bfc0c4c8d874b41a6e18732f0b60f5662ef1a89c80589dd8366c90bb58bb85ead56356aba2a244950ca170abbd01094539014f84bdd383e4a10e00cee63dfc3e809506e2d9b54edbdca1bace6eaa119e68573d30533791fba830f5d80be5c051a77c09415e3b8fe3139400848be5244b8ae96bb0c4a24f819cba0488f34985eac741d3359180bd72cafa1559e4c19f54ea8cedbb6a5afde4319396eb92aab340c60a50cc2284580cb3ad09017e8d9abc60269b3d8d687680bd86ce834412273d4f2e3bf68dd3d6fe87e2426ac658cd5c77fd5c0aa000000"
        )
        #expect(
            try Multiaddr("/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuq").binaryPacked()
                .toHexString() == "bf0320efbcd45d0c5dc79781ac6f20ea5055a036afb48d45a52e7d68ec7d4338919e69"
        )
    }

    @Test func testBinaryPackedToStrings() throws {
        #expect(try Multiaddr(Data(hex: "047f000001910204d2")).description == "/ip4/127.0.0.1/udp/1234")
        #expect(try Multiaddr(Data(hex: "047f0000010610e1")).description == "/ip4/127.0.0.1/tcp/4321")
        #expect(
            try Multiaddr(Data(hex: "047f000001910204d2047f0000010610e1")).description
                == "/ip4/127.0.0.1/udp/1234/ip4/127.0.0.1/tcp/4321"
        )
        #expect(
            try Multiaddr(Data(hex: "bc030010c0439831b48218480050")).description == "/onion/aaimaq4ygg2iegci:80"
        )
        #expect(
            try Multiaddr(Data(hex: "bd03adadec040be047f9658668b11a504f3155001f231a37f54c4476c07fb4cc139ed7e30304d2"))
                .description == "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:1234"
        )
        #expect(
            try Multiaddr(
                Data(
                    hex:
                        "be0383038d3fc8c976a86ae4e78ba378e75ec41bc9ab1542a9cb422581987e118f5cb0c024f3639d6ad9b3aff613672f07bfbbbfc2f920ef910534ecaa6ff9c03e0fa4872a764d2fce6d4cfc5a5a9800cd95944cc9ef0241f753fe71494a175f334b35682459acadc4076428ab49b5a83a49d2ea2366b06461e4a559b0111fa750e0de0c138a94d1231ed5979572ff53922905636221994bdabc44bd0c17fef11622b16432db3f193400af53cc61aa9bfc0c4c8d874b41a6e18732f0b60f5662ef1a89c80589dd8366c90bb58bb85ead56356aba2a244950ca170abbd01094539014f84bdd383e4a10e00cee63dfc3e809506e2d9b54edbdca1bace6eaa119e68573d30533791fba830f5d80be5c051a77c09415e3b8fe3139400848be5244b8ae96bb0c4a24f819cba0488f34985eac741d3359180bd72cafa1559e4c19f54ea8cedbb6a5afde4319396eb92aab340c60a50cc2284580cb3ad09017e8d9abc60269b3d8d687680bd86ce834412273d4f2e3bf68dd3d6fe87e2426ac658cd5c77fd5c0aa000000"
                )
            ).description
                == "/garlic64/jT~IyXaoauTni6N4517EG8mrFUKpy0IlgZh-EY9csMAk82Odatmzr~YTZy8Hv7u~wvkg75EFNOyqb~nAPg-khyp2TS~ObUz8WlqYAM2VlEzJ7wJB91P-cUlKF18zSzVoJFmsrcQHZCirSbWoOknS6iNmsGRh5KVZsBEfp1Dg3gwTipTRIx7Vl5Vy~1OSKQVjYiGZS9q8RL0MF~7xFiKxZDLbPxk0AK9TzGGqm~wMTI2HS0Gm4Ycy8LYPVmLvGonIBYndg2bJC7WLuF6tVjVquiokSVDKFwq70BCUU5AU-EvdOD5KEOAM7mPfw-gJUG4tm1TtvcobrObqoRnmhXPTBTN5H7qDD12AvlwFGnfAlBXjuP4xOUAISL5SRLiulrsMSiT4GcugSI80mF6sdB0zWRgL1yyvoVWeTBn1TqjO27alr95DGTluuSqrNAxgpQzCKEWAyzrQkBfo2avGAmmz2NaHaAvYbOg0QSJz1PLjv2jdPW~ofiQmrGWM1cd~1cCqAAAA"
        )
        #expect(
            try Multiaddr(Data(hex: "bf0320efbcd45d0c5dc79781ac6f20ea5055a036afb48d45a52e7d68ec7d4338919e69"))
                .description == "/garlic32/566niximlxdzpanmn4qouucvua3k7neniwss47li5r6ugoertzuq"
        )
    }

    @Test func testMultiaddrSplitDescription() throws {
        #expect(
            try Multiaddr("/ip4/1.2.3.4/udp/1234").addresses.map { $0.description } == [
                "/ip4/1.2.3.4",
                "/udp/1234",
            ]
        )
        #expect(
            try Multiaddr("/ip4/1.2.3.4/tcp/1/ip4/2.3.4.5/udp/2").addresses.map { $0.description } == [
                "/ip4/1.2.3.4",
                "/tcp/1",
                "/ip4/2.3.4.5",
                "/udp/2",
            ]
        )
        #expect(
            try Multiaddr("/ip4/1.2.3.4/utp/ip4/2.3.4.5/udp/2/udt").addresses.map { $0.description } == [
                "/ip4/1.2.3.4",
                "/utp",
                "/ip4/2.3.4.5",
                "/udp/2",
                "/udt",
            ]
        )
    }

    @Test func testGetValueForProto() throws {
        let ma = try Multiaddr(
            "/ip4/127.0.0.1/utp/tcp/5555/udp/1234/tls/utp/ipfs/QmbHVEEepCi7rn7VL7Exxpd2Ci9NNB6ifvqwhsrbRMgQFP"
        )
        #expect(ma.getFirstAddress(forCodec: .ip4)?.address == "127.0.0.1")
        #expect(ma.getFirstAddress(forCodec: .utp)?.address == nil)
        #expect(ma.getFirstAddress(forCodec: .tls)?.address == nil)
        #expect(ma.getFirstAddress(forCodec: .tcp)?.address == "5555")
        #expect(ma.getFirstAddress(forCodec: .udp)?.address == "1234")
        #expect(ma.getFirstAddress(forCodec: .ipfs)?.address == "QmbHVEEepCi7rn7VL7Exxpd2Ci9NNB6ifvqwhsrbRMgQFP")
        #expect(ma.getFirstAddress(forCodec: .p2p)?.address == "QmbHVEEepCi7rn7VL7Exxpd2Ci9NNB6ifvqwhsrbRMgQFP")

        #expect(ma.getFirstAddress(forCodec: .ip6) == nil)
    }

    @Test(
        "Test Round Trip",
        arguments: [
            "/unix/a/b/c/d",
            "/ip6/::ffff:127.0.0.1/tcp/111",
            "/ip4/127.0.0.1/tcp/123",
            "/ip4/127.0.0.1/tcp/123/tls",
            "/ip4/127.0.0.1/udp/123",
            "/ip4/127.0.0.1/udp/123/ip6/::",
            "/ip4/127.0.0.1/udp/1234/quic-v1/webtransport/certhash/uEiDDq4_xNyDorZBH3TlGazyJdOWSwvo4PUo5YHFMrvDE8g",
            "/p2p/QmbHVEEepCi7rn7VL7Exxpd2Ci9NNB6ifvqwhsrbRMgQFP",
            "/p2p/QmbHVEEepCi7rn7VL7Exxpd2Ci9NNB6ifvqwhsrbRMgQFP/unix/a/b/c",
        ]
    )
    func testRoundTrip(_ address: String) throws {
        let ma = try Multiaddr(address)
        #expect(ma.description == address)
        #expect(try Multiaddr(ma.binaryPacked()) == ma)
    }

    /// https://github.com/multiformats/multicodec/pull/283
    @Test func testIPFSvP2P() throws {
        let p2pAddress = "/p2p/QmbHVEEepCi7rn7VL7Exxpd2Ci9NNB6ifvqwhsrbRMgQFP"
        let ipfsAddress = "/ipfs/QmbHVEEepCi7rn7VL7Exxpd2Ci9NNB6ifvqwhsrbRMgQFP"

        let p2p = try Multiaddr(p2pAddress)
        let ipfs = try Multiaddr(ipfsAddress)

        #expect(p2p == ipfs)
        #expect(ipfs.description != p2p.description)
    }

}
