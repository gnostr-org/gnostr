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

import Foundation
import Multibase
import Multicodec
import Testing

@testable import Multiaddr

@Suite("Multiaddr Protocol Tests")
struct ProtocolTests {

    @Test func testVarIntEncoding() {
        let proto1 = MultiaddrProtocol.ip6
        let expectedPackedValueAsHex1 = "29"
        let varIntEncodedBytes1 = proto1.packedCode().hexString()
        #expect(varIntEncodedBytes1 == expectedPackedValueAsHex1)

        let proto2 = MultiaddrProtocol.ip4
        let expectedPackedValueAsHex2 = "04"
        let varIntEncodedBytes2 = proto2.packedCode().hexString()
        #expect(varIntEncodedBytes2 == expectedPackedValueAsHex2)
    }

    //    it('create multiaddr', () => {
    //        udpAddr = multiaddr('/ip4/127.0.0.1/udp/1234')
    //        expect(udpAddr instanceof multiaddr).to.equal(true)
    //      })
    @Test func testCreateMultiaddr() throws {
        let ma = try Multiaddr("/ip4/127.0.0.1/udp/1234")
        #expect(ma.description == "/ip4/127.0.0.1/udp/1234")
    }

    //    it('clone multiaddr', () => {
    //        const udpAddrClone = multiaddr(udpAddr)
    //        expect(udpAddrClone !== udpAddr).to.equal(true)
    //      })
    @Test func testClone() throws {
        var m1 = try Multiaddr("/ip4/127.0.0.1")
        var cloned = m1
        #expect(m1 == cloned)

        let m2 = try Multiaddr("/udp/1234")
        cloned = cloned.encapsulate(m2)
        #expect(m1 != cloned)

        m1 = m1.encapsulate(m2)
        let expected = try Multiaddr("/ip4/127.0.0.1/udp/1234")
        #expect(m1 == expected)
        #expect(cloned == expected)
    }

    //      it('reconstruct with buffer', () => {
    //        expect(multiaddr(udpAddr.bytes).bytes === udpAddr.bytes).to.equal(false)
    //        expect(multiaddr(udpAddr.bytes).bytes).to.deep.equal(udpAddr.bytes)
    //      })
    @Test func testCreateFromBuffer() throws {
        let udpAddr = try Multiaddr("/ip4/127.0.0.1/udp/1234")
        let fromBytes = try Multiaddr(udpAddr.binaryPacked())
        #expect(udpAddr == fromBytes)
    }

    @Test func testCreateFromString() throws {
        let udpAddr = try Multiaddr("/ip4/127.0.0.1/udp/1234")
        let fromString = try Multiaddr(udpAddr.description)
        #expect(udpAddr == fromString)
    }

    /// - TODO: Support empty Multiaddr initialization
    //    func testCreateFromEmptyString() throws {
    //        let emptyAddr = try? Multiaddr("/")
    //        XCTAssertNotNil(emptyAddr)
    //        dump(emptyAddr)
    //    }

    @Test func testCreateBasic() throws {
        //        const udpAddrStr = '/ip4/127.0.0.1/udp/1234'
        //        const udpAddrBuf = uint8ArrayFromString('047f000001910204d2', 'base16')
        //        const udpAddr = multiaddr(udpAddrStr)
        //
        //        expect(udpAddr.toString()).to.equal(udpAddrStr)
        //        expect(udpAddr.bytes).to.deep.equal(udpAddrBuf)
        //
        //        expect(udpAddr.protoCodes()).to.deep.equal([4, 273])
        //        expect(udpAddr.protoNames()).to.deep.equal(['ip4', 'udp'])
        //        expect(udpAddr.protos()).to.deep.equal([multiaddr.protocols.codes[4], multiaddr.protocols.codes[273]])
        //        expect(udpAddr.protos()[0] === multiaddr.protocols.codes[4]).to.equal(false)
        //
        //        const udpAddrbytes2 = udpAddr.encapsulate('/udp/5678')
        //        expect(udpAddrbytes2.toString()).to.equal('/ip4/127.0.0.1/udp/1234/udp/5678')
        //        expect(udpAddrbytes2.decapsulate('/udp').toString()).to.equal('/ip4/127.0.0.1/udp/1234')
        //        expect(udpAddrbytes2.decapsulate('/ip4').toString()).to.equal('/')
        //        expect(function () { udpAddr.decapsulate('/').toString() }).to.throw()
        //        expect(multiaddr('/').encapsulate(udpAddr).toString()).to.equal(udpAddr.toString())
        //        expect(multiaddr('/').decapsulate('/').toString()).to.equal('/')
        let udpAddrStr = "/ip4/127.0.0.1/udp/1234"
        let udpAddrBuf = try BaseEncoding.decode("047f000001910204d2", as: .base16).data

        let udpAddr = try Multiaddr(udpAddrStr)

        /// Expect the string description to equal our udp addr string
        #expect(udpAddr.description == udpAddrStr)

        /// Expect the data representation to equal the hex data above
        #expect(try udpAddr.binaryPacked() == udpAddrBuf)

        /// Protocols
        #expect(udpAddr.protocols() == [.ip4, .udp])
        #expect(udpAddr.protoNames() == ["ip4", "udp"])
        #expect(udpAddr.protoCodes() == [4, 273])

        /// Decapsulation via new Multiaddress
        let udpAddrBytes2 = try udpAddr.encapsulate("/udp/5678")
        #expect(udpAddrBytes2.description == "/ip4/127.0.0.1/udp/1234/udp/5678")
        // TODO: Should Support just "/udp"
        #expect(udpAddrBytes2.decapsulate(try Multiaddr("/udp/5678")).description == "/ip4/127.0.0.1/udp/1234")
        // TODO: Should Support just "/ip4"
        #expect(udpAddrBytes2.decapsulate(try Multiaddr("/ip4/127.0.0.1")).description == "/")

        /// Decapsulation via String Protocol
        let udpAddrBytes3 = try udpAddr.encapsulate("/udp/5678")
        #expect(udpAddrBytes3.description == "/ip4/127.0.0.1/udp/1234/udp/5678")
        #expect(udpAddrBytes3.decapsulate("/udp").description == "/ip4/127.0.0.1/udp/1234")
        //Decapsulate isn't mutating so it must be called twice...
        #expect(udpAddrBytes3.decapsulate("/udp").decapsulate("/udp").description == "/ip4/127.0.0.1")
        #expect(udpAddrBytes3.decapsulate("/ip4").description == "/")

        /// Decapsulation via MultiaddrProtocol (akak Codec)
        let udpAddrBytes4 = try udpAddr.encapsulate(proto: .udp, address: "5678")
        #expect(udpAddrBytes4.description == "/ip4/127.0.0.1/udp/1234/udp/5678")
        #expect(udpAddrBytes4.decapsulate(.udp).description == "/ip4/127.0.0.1/udp/1234")
        //Decapsulate isn't mutating so it must be called twice...
        #expect(udpAddrBytes4.decapsulate(.udp).decapsulate(.udp).description == "/ip4/127.0.0.1")
        #expect(udpAddrBytes4.decapsulate(.ip4).description == "/")

        /// - TODO: Support Instantiating with a single "/"
        //#expect(try Multiaddr("/").encapsulate(udpAddr).description == udpAddrStr)
    }

    @Test func testIPFSAddress() throws {
        let ipfsAddr = try Multiaddr("/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        let ip6Addr = try Multiaddr("/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
        let tcpAddr = try Multiaddr("/tcp/8000")
        let wsAddr = try Multiaddr("/ws")

        #expect(
            ip6Addr.encapsulate(tcpAddr)
                .encapsulate(wsAddr)
                .encapsulate(ipfsAddr).description == [ip6Addr, tcpAddr, wsAddr, ipfsAddr].map { "\($0)" }.joined()
        )

        #expect(
            ip6Addr.encapsulate(tcpAddr)
                .encapsulate(wsAddr)
                .encapsulate(ipfsAddr)
                //.decapsulate("/ipfs").description,
                .decapsulate(.ipfs).description == [ip6Addr, tcpAddr, wsAddr].map { "\($0)" }.joined()
        )

        #expect(
            ip6Addr.encapsulate(tcpAddr)
                .encapsulate(ipfsAddr)
                .encapsulate(wsAddr)
                .decapsulate(.ws).description == [ip6Addr, tcpAddr, ipfsAddr].map { "\($0)" }.joined()
        )
    }

    @Test func testIP4() throws {
        let str = "/ip4/127.0.0.1"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(try Multiaddr(.ip4, address: "127.0.0.1") == addr)
        // Different IP4 Address
        #expect(try Multiaddr(.ip4, address: "127.0.0.2") != addr)
    }

    @Test func testIP6() throws {
        let str = "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(try Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095") == addr)
        // Different IP6 Address
        #expect(try Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096") != addr)
    }

    @Test func testIP4_TCP() throws {
        let str = "/ip4/127.0.0.1/tcp/5000"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try Multiaddr("/ip4/127.0.0.1/tcp/5000") == addr)
        // Built via encapsulation
        #expect(try Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "5000") == addr)
        // Different IP4 Address
        #expect(try addr != Multiaddr(.ip4, address: "127.0.0.2").encapsulate(proto: .tcp, address: "5000"))
        // Different Port
        #expect(try addr != Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "5001"))
    }

    @Test func testIP6_TCP() throws {
        let str = "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/5000"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/5000"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095").encapsulate(
                    proto: .tcp,
                    address: "5000"
                )
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096").encapsulate(
                    proto: .tcp,
                    address: "5000"
                )
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095").encapsulate(
                    proto: .tcp,
                    address: "5001"
                )
        )
    }

    @Test func testIP4_UDP() throws {
        let str = "/ip4/127.0.0.1/udp/5000"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip4/127.0.0.1/udp/5000"))
        // Built via encapsulation
        #expect(try addr == Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .udp, address: "5000"))
        // Different IP4 Address
        #expect(try addr != Multiaddr(.ip4, address: "127.0.0.2").encapsulate(proto: .udp, address: "5000"))
        // Different Port
        #expect(try addr != Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .udp, address: "5001"))
    }

    @Test func testIP6_UDP() throws {
        let str = "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/5000"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/5000"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095").encapsulate(
                    proto: .udp,
                    address: "5000"
                )
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096").encapsulate(
                    proto: .udp,
                    address: "5000"
                )
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095").encapsulate(
                    proto: .udp,
                    address: "5001"
                )
        )
    }

    @Test func testIP4_P2P_TCP() throws {
        let str = "/ip4/127.0.0.1/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(
            try addr == Multiaddr("/ip4/127.0.0.1/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234")
        )
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip4, address: "127.0.0.1")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1234")
        )
        // Different IP4 Address
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.2")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1234")
        )
        // Different PeerID
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKD")
                .encapsulate(proto: .tcp, address: "1234")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1235")
        )
    }

    /// We need support for ipfs in order to support pre p2p protocols
    @Test func testIP4_IPFS_TCP() throws {
        let str = "/ip4/127.0.0.1/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(
            try addr == Multiaddr("/ip4/127.0.0.1/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234")
        )
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip4, address: "127.0.0.1")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1234")
        )
        // Different IP4 Address
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.2")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1234")
        )
        // Different PeerID
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKD")
                .encapsulate(proto: .tcp, address: "1234")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1235")
        )
    }

    @Test func testIP6_P2P_TCP() throws {
        let str =
            "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(
            try addr
                == Multiaddr(
                    "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/tcp/1234"
                )
        )
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1234")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1234")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
                .encapsulate(proto: .tcp, address: "1235")
        )
        // Different PeerID
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKD")
                .encapsulate(proto: .tcp, address: "1234")
        )
    }

    func testIP4_UDP_UTP() throws {
        let str = "/ip4/127.0.0.1/udp/5000/utp"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip4/127.0.0.1/udp/5000/utp"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .udp, address: "5000").encapsulate(
                    proto: .utp,
                    address: nil
                )
        )
        // Different IP4 Address
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.2").encapsulate(proto: .udp, address: "5000").encapsulate(
                    proto: .utp,
                    address: nil
                )
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .udp, address: "5001").encapsulate(
                    proto: .utp,
                    address: nil
                )
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.utp).description == "/ip4/127.0.0.1/udp/5000")
        //Decapsulating udp drops both udp and utp protos
        #expect(addr.decapsulate(.udp).description == "/ip4/127.0.0.1")
    }

    @Test func testIP6_UDP_UTP() throws {
        let str = "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/5000/utp"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/5000/utp"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095").encapsulate(
                    proto: .udp,
                    address: "5000"
                ).encapsulate(proto: .utp, address: "")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096").encapsulate(
                    proto: .udp,
                    address: "5000"
                ).encapsulate(proto: .utp, address: "")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095").encapsulate(
                    proto: .udp,
                    address: "5001"
                ).encapsulate(proto: .utp, address: "")
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.utp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/5000")
        //Decapsulating udp drops both udp and utp protos
        #expect(addr.decapsulate(.udp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    @Test func testIP4_TCP_HTTP() throws {
        let str = "/ip4/127.0.0.1/tcp/8000/http"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip4/127.0.0.1/tcp/8000/http"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "8000").encapsulate(
                    proto: .http,
                    address: ""
                )
        )
        // Different IP4 Address
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.2").encapsulate(proto: .tcp, address: "8000").encapsulate(
                    proto: .http,
                    address: ""
                )
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "8001").encapsulate(
                    proto: .http,
                    address: ""
                )
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.http).description == "/ip4/127.0.0.1/tcp/8000")
        //Decapsulating udp drops both udp and http protos
        #expect(addr.decapsulate(.tcp).description == "/ip4/127.0.0.1")
    }

    @Test func testIP4_TCP_UNIX() throws {
        let str = "/ip4/127.0.0.1/tcp/80/unix/a/b/c/d/e/f"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip4/127.0.0.1/tcp/80/unix/a/b/c/d/e/f"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "80").encapsulate(
                    proto: .unix,
                    address: "a/b/c/d/e/f"
                )
        )
        // Different IP4 Address
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.2").encapsulate(proto: .tcp, address: "80").encapsulate(
                    proto: .unix,
                    address: "a/b/c/d/e/f"
                )
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "81").encapsulate(
                    proto: .unix,
                    address: "a/b/c/d/e/f"
                )
        )
        // Different Unix Address
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "80").encapsulate(
                    proto: .unix,
                    address: "a/b/c/d/e/g"
                )
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.unix).description == "/ip4/127.0.0.1/tcp/80")
        //Decapsulating udp drops both udp and http protos
        #expect(addr.decapsulate(.tcp).description == "/ip4/127.0.0.1")
    }

    @Test func testIP6_TCP_HTTP() throws {
        let str = "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/http"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/http"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .http, address: "")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .http, address: "")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8001")
                .encapsulate(proto: .http, address: "")
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.http).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000")
        //Decapsulating udp drops both udp and utp protos
        #expect(addr.decapsulate(.tcp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    @Test func testIP6_TCP_UNIX() throws {
        let str = "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/unix/a/b/c/d/e/f"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/unix/a/b/c/d/e/f"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .unix, address: "a/b/c/d/e/f")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .unix, address: "a/b/c/d/e/f")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8001")
                .encapsulate(proto: .unix, address: "a/b/c/d/e/f")
        )
        // Different Unix Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .unix, address: "a/b/c/d/e/g")
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.unix).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000")
        //Decapsulating udp drops both udp and utp protos
        #expect(addr.decapsulate(.tcp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    //      it('ip4 + tcp + https', () => {
    //        const str = '/ip4/127.0.0.1/tcp/8000/https'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testIP4_TCP_HTTPS() throws {
        let str = "/ip4/127.0.0.1/tcp/8000/https"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip4/127.0.0.1/tcp/8000/https"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "8000").encapsulate(
                    proto: .https,
                    address: nil
                )
        )
        // Different IP4 Address
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.2").encapsulate(proto: .tcp, address: "8000").encapsulate(
                    proto: .https,
                    address: nil
                )
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "8001").encapsulate(
                    proto: .https,
                    address: nil
                )
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.https).description == "/ip4/127.0.0.1/tcp/8000")
        //Decapsulating udp drops both udp and http protos
        #expect(addr.decapsulate(.tcp).description == "/ip4/127.0.0.1")
    }

    //      it('ip6 + tcp + https', () => {
    //        const str = '/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/https'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testIP6_TCP_HTTPS() throws {
        let str = "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/https"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/https"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .https, address: "")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .https, address: "")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8001")
                .encapsulate(proto: .https, address: "")
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.https).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000")
        //Decapsulating udp drops both udp and utp protos
        #expect(addr.decapsulate(.tcp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    //      it('ip4 + tcp + websockets', () => {
    //        const str = '/ip4/127.0.0.1/tcp/8000/ws'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testIP4_TCP_WS() throws {
        let str = "/ip4/127.0.0.1/tcp/8000/ws"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip4/127.0.0.1/tcp/8000/ws"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "8000").encapsulate(
                    proto: .ws,
                    address: ""
                )
        )
        // Different IP4 Address
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.2").encapsulate(proto: .tcp, address: "8000").encapsulate(
                    proto: .ws,
                    address: ""
                )
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip4, address: "127.0.0.1").encapsulate(proto: .tcp, address: "8001").encapsulate(
                    proto: .ws,
                    address: ""
                )
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.ws).description == "/ip4/127.0.0.1/tcp/8000")
        //Decapsulating udp drops both udp and http protos
        #expect(addr.decapsulate(.tcp).description == "/ip4/127.0.0.1")
    }

    //      it('ip6 + tcp + websockets', () => {
    //        const str = '/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testIP6_TCP_WS() throws {
        let str = "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(try addr == Multiaddr("/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws"))
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .ws, address: "")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .ws, address: "")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8001")
                .encapsulate(proto: .ws, address: "")
        )
        //Decapsulating utp
        #expect(addr.decapsulate(.ws).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000")
        //Decapsulating udp drops both udp and utp protos
        #expect(addr.decapsulate(.tcp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    //      it('ip6 + tcp + websockets + ipfs', () => {
    //        const str = '/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str.replace('/ipfs/', '/p2p/'))
    //      })
    @Test func testIP6_TCP_WS_IPFS() throws {
        let str =
            "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(
            try addr
                == Multiaddr(
                    "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
                )
        )
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .ws, address: "")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .ws, address: "")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8001")
                .encapsulate(proto: .ws, address: "")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        //Decapsulating ws
        #expect(addr.decapsulate(.ws).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000")
        //Decapsulating tcp drops both tcp, ws and ipfs protos
        #expect(addr.decapsulate(.tcp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    //      it('ip6 + tcp + websockets + p2p', () => {
    //        const str = '/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testIP6_TCP_WS_P2P() throws {
        let str =
            "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(
            try addr
                == Multiaddr(
                    "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000/ws/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
                )
        )
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .ws, address: "")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .tcp, address: "8000")
                .encapsulate(proto: .ws, address: "")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .tcp, address: "8001")
                .encapsulate(proto: .ws, address: "")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        //Decapsulating ws
        #expect(addr.decapsulate(.ws).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/tcp/8000")
        //Decapsulating tcp drops both tcp, ws and ipfs protos
        #expect(addr.decapsulate(.tcp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    //      it('ip6 + udp + quic + ipfs', () => {
    //        const str = '/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/4001/quic/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str.replace('/ipfs/', '/p2p/'))
    //      })
    @Test func testIP6_UDP_QUIC_IPFS() throws {
        let str =
            "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/4001/quic/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(
            try addr
                == Multiaddr(
                    "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/4001/quic/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
                )
        )
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .udp, address: "4001")
                .encapsulate(proto: .quic, address: "")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .udp, address: "4001")
                .encapsulate(proto: .quic, address: "")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .udp, address: "4000")
                .encapsulate(proto: .quic, address: "")
                .encapsulate(proto: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        //Decapsulating quic
        #expect(addr.decapsulate(.quic).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/4001")
        //Decapsulating udp drops both udp, quic and ipfs protos
        #expect(addr.decapsulate(.udp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    //      it('ip6 + udp + quic + p2p', () => {
    //        const str = '/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/4001/quic/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    //
    @Test func testIP6_UDP_QUIC_P2P() throws {
        let str =
            "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/4001/quic/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        // Two Multiaddresses initialized with the same string should be equal
        #expect(
            try addr
                == Multiaddr(
                    "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/4001/quic/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
                )
        )
        // Built via encapsulation
        #expect(
            try addr
                == Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .udp, address: "4001")
                .encapsulate(proto: .quic, address: "")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        // Different IP6 Address
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7096")
                .encapsulate(proto: .udp, address: "4001")
                .encapsulate(proto: .quic, address: "")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        // Different Port
        #expect(
            try addr
                != Multiaddr(.ip6, address: "2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
                .encapsulate(proto: .udp, address: "4000")
                .encapsulate(proto: .quic, address: "")
                .encapsulate(proto: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        )
        //Decapsulating quic
        #expect(addr.decapsulate(.quic).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095/udp/4001")
        //Decapsulating udp drops both udp, quic and p2p protos
        #expect(addr.decapsulate(.udp).description == "/ip6/2001:8a0:7ac5:4201:3ac9:86ff:fe31:7095")
    }

    //      it('unix', () => {
    //        const str = '/unix/a/b/c/d/e'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testUNIX() throws {
        let str = "/unix/a/b/c/d/e"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
    }

    //      it('p2p', () => {
    //        const str = '/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testP2P_Multihash_Initialization() throws {
        let str = "/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
            ]
        )
    }

    //      it('p2p', () => {
    //        const str = '/p2p/bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal('/p2p/QmW8rAgaaA6sRydK1k6vonShQME47aDxaFidbtMevWs73t')
    //      })
    /// - TODO: Support B32 P2P/IPFS Instantiation
    @Test func testP2P_CID_Initialization() throws {
        let str = "/p2p/bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == "/p2p/bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4")
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .p2p, address: "QmW8rAgaaA6sRydK1k6vonShQME47aDxaFidbtMevWs73t")
            ]
        )
    }

    //      it('ipfs', () => {
    //        const str = '/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str.replace('/ipfs/', '/p2p/'))
    //      })
    /// IPFS Overload no longer exists (these should no longer be equal)
    /// - Note: https://github.com/multiformats/multicodec/pull/283
    @Test func testIPFS() throws {
        let str = "/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
            ]
        )
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
            ]
        )
    }

    //      it('onion', () => {
    //        const str = '/onion/timaq4ygg2iegci7:1234'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testOnion() throws {
        let str = "/onion/timaq4ygg2iegci7:1234"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(try addr.addresses == [Address(addrProtocol: .onion, address: "timaq4ygg2iegci7:1234")])
    }

    //      it('onion bad length', () => {
    //        const str = '/onion/timaq4ygg2iegci:80'
    //        expect(() => multiaddr(str)).to.throw()
    //      })
    @Test func testOnion_BadLength() throws {
        let str = "/onion/timaq4ygg2iegci:80"
        // Throws with bad Onion Length Error
        #expect(throws: MultiaddrError.invalidOnionHostAddress) { try Multiaddr(str) }
    }

    //      it('onion bad port', () => {
    //        const str = '/onion/timaq4ygg2iegci7:-1'
    //        expect(() => multiaddr(str)).to.throw()
    //      })
    @Test func testOnion_BadPort() throws {
        let str = "/onion/timaq4ygg2iegci7:-1"
        // Throws with bad Onion Port Error
        #expect(throws: MultiaddrError.invalidPortValue) { try Multiaddr(str) }
        //XCTAssertThrowsError(try Multiaddr(str))
    }

    //      it('onion no port', () => {
    //        const str = '/onion/timaq4ygg2iegci7'
    //        expect(() => multiaddr(str)).to.throw()
    //      })
    @Test func testOnion_NoPort() throws {
        let str = "/onion/timaq4ygg2iegci7"
        // Throws with bad Onion Port Error
        #expect(throws: MultiaddrError.invalidFormat) { try Multiaddr(str) }
        //XCTAssertThrowsError(try Multiaddr(str))
    }

    //      it('onion3', () => {
    //        const str = '/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:1234'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    /// - TODO: Support Onion3 Address Instantiation
    @Test func testOnion3() throws {
        let str = "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:1234"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(
                    addrProtocol: .onion3,
                    address: "vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:1234"
                )
            ]
        )
    }

    //      it('onion3 bad length', () => {
    //        const str = '/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopyyd:1234'
    //        expect(() => multiaddr(str)).to.throw()
    //      })
    @Test func testOnion3_BadLength() throws {
        let str = "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopyyd:1234"
        #expect(throws: MultiaddrError.invalidOnionHostAddress) { try Multiaddr(str) }
        //XCTAssertThrowsError(try Multiaddr(str))
    }

    //      it('onion3 bad port', () => {
    //        const str = '/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd:-1'
    //        expect(() => multiaddr(str)).to.throw()
    //      })
    @Test func testOnion3_BadPort() throws {
        let str = "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopyyd:-1"
        #expect(throws: MultiaddrError.invalidOnionHostAddress) { try Multiaddr(str) }
        //XCTAssertThrowsError(try Multiaddr(str))
    }

    //      it('onion3 no port', () => {
    //        const str = '/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopnpyyd'
    //        expect(() => multiaddr(str)).to.throw()
    //      })
    @Test func testOnion3_NoPort() throws {
        let str = "/onion3/vww6ybal4bd7szmgncyruucpgfkqahzddi37ktceo3ah7ngmcopyyd"
        #expect(throws: MultiaddrError.invalidFormat) { try Multiaddr(str) }
        //XCTAssertThrowsError(try Multiaddr(str))
    }

    //      it('p2p-circuit', () => {
    //        const str = '/p2p-circuit/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    /// - TODO: Support P2P_Curcuit Protocol
    @Test func testP2PCircuit() throws {
        let str = "/p2p-circuit/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .p2p_circuit, address: nil),
                Address(addrProtocol: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"),
            ]
        )
    }

    //      it('p2p-circuit p2p', () => {
    //        const str = '/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/p2p-circuit'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testP2P_P2PCircuit() throws {
        let str = "/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/p2p-circuit"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"),
                Address(addrProtocol: .p2p_circuit, address: ""),
            ]
        )
    }

    //      it('p2p-circuit ipfs', () => {
    //        const str = '/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/p2p-circuit'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str.replace('/ipfs/', '/p2p/'))
    //      })
    @Test func testIPFS_P2PCircuit() throws {
        let str = "/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC/p2p-circuit"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"),
                Address(addrProtocol: .p2p_circuit, address: ""),
            ]
        )
    }

    //      it('p2p-webrtc-star', () => {
    //        const str = '/ip4/127.0.0.1/tcp/9090/ws/p2p-webrtc-star/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testIP4_TCP_WEBRTCSTAR_P2P() throws {
        let str = "/ip4/127.0.0.1/tcp/9090/ws/p2p-webrtc-star/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .ip4, address: "127.0.0.1"),
                Address(addrProtocol: .tcp, address: "9090"),
                Address(addrProtocol: .ws, address: ""),
                Address(addrProtocol: .p2p_webrtc_star, address: ""),
                Address(addrProtocol: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"),
            ]
        )
    }

    //      it('p2p-webrtc-star ipfs', () => {
    //        const str = '/ip4/127.0.0.1/tcp/9090/ws/p2p-webrtc-star/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str.replace('/ipfs/', '/p2p/'))
    //      })
    @Test func testIP4_TCP_WEBRTCSTAR_IPFS() throws {
        let str = "/ip4/127.0.0.1/tcp/9090/ws/p2p-webrtc-star/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .ip4, address: "127.0.0.1"),
                Address(addrProtocol: .tcp, address: "9090"),
                Address(addrProtocol: .ws, address: ""),
                Address(addrProtocol: .p2p_webrtc_star, address: ""),
                Address(addrProtocol: .ipfs, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"),
            ]
        )
    }

    @Test func testIP4_TCP_WEBRTCSTAR_IPFS_CID() throws {
        let str =
            "/ip4/127.0.0.1/tcp/9090/ws/p2p-webrtc-star/ipfs/bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(
            addr.description
                == "/ip4/127.0.0.1/tcp/9090/ws/p2p-webrtc-star/ipfs/bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4"
        )
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .ip4, address: "127.0.0.1"),
                Address(addrProtocol: .tcp, address: "9090"),
                Address(addrProtocol: .ws, address: ""),
                Address(addrProtocol: .p2p_webrtc_star, address: ""),
                Address(addrProtocol: .ipfs, address: "QmW8rAgaaA6sRydK1k6vonShQME47aDxaFidbtMevWs73t"),
            ]
        )
    }
    //      it('p2p-webrtc-direct', () => {
    //        const str = '/ip4/127.0.0.1/tcp/9090/http/p2p-webrtc-direct'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testIP4_TCP_HTTP_WEBRTCDIRECT() throws {
        let str = "/ip4/127.0.0.1/tcp/9090/http/p2p-webrtc-direct"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .ip4, address: "127.0.0.1"),
                Address(addrProtocol: .tcp, address: "9090"),
                Address(addrProtocol: .http, address: ""),
                Address(addrProtocol: .p2p_webrtc_direct, address: ""),
            ]
        )
    }

    //      it('p2p-websocket-star', () => {
    //        const str = '/ip4/127.0.0.1/tcp/9090/ws/p2p-websocket-star'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    @Test func testIP4_TCP_WS_WEBSOCKETSTAR() throws {
        let str = "/ip4/127.0.0.1/tcp/9090/ws/p2p-websocket-star"
        let addr = try Multiaddr(str)
        // Description should equal initialization string
        #expect(addr.description == str)
        #expect(
            try addr.addresses == [
                Address(addrProtocol: .ip4, address: "127.0.0.1"),
                Address(addrProtocol: .tcp, address: "9090"),
                Address(addrProtocol: .ws, address: ""),
                Address(addrProtocol: .p2p_websocket_star, address: ""),
            ]
        )
    }

    //      it('memory + p2p', () => {
    //        const str = '/memory/test/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC'
    //        const addr = multiaddr(str)
    //        expect(addr).to.have.property('bytes')
    //        expect(addr.toString()).to.equal(str)
    //      })
    //    func testMEMORY_TEST_P2P() throws {
    //        let str = "/memory/test/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
    //        let addr = try Multiaddr(str)
    //        // Description should equal initialization string
    //        XCTAssertEqual(addr.description, str)
    //        XCTAssertEqual(addr.addresses, [
    //            Address(addrProtocol: .memory, address: ""),
    //            Address(addrProtocol: .test, address: ""),
    //            Address(addrProtocol: .p2p, address: "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
    //        ])
    //    }

    /// - MARK: PeerID Extraction Tests

    @Test func testGetPeerID_P2P() throws {
        let ma = try Multiaddr("/p2p-circuit/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        #expect(ma.getPeerIDString() == "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
    }

    @Test func testGetPeerID_P2PCircuit() throws {
        let ma = try Multiaddr(
            "/ip4/0.0.0.0/tcp/8080/p2p/QmZR5a9AAXGqQF2ADqoDdGS8zvqv8n3Pag6TDDnTNMcFW6/p2p-circuit/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC"
        )
        #expect(ma.getPeerIDString() == "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
    }

    @Test func testGetPeerID_IPFS() throws {
        let ma = try Multiaddr("/p2p-circuit/ipfs/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        #expect(ma.getPeerIDString() == "QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
    }

    @Test func testGetPeerID_P2P_CIDv1_BASE32() throws {
        let ma = try Multiaddr("/p2p-circuit/p2p/bafzbeigweq4zr4x4ky2dvv7nanbkw6egutvrrvzw6g3h2rftp7gidyhtt4")
        #expect(ma.getPeerIDString() == "bafzbeigweq4zr4x4ky2dvv7nanbkw6egutvrrvzw6g3h2rftp7gidyhtt4")
    }

    @Test func testGetPeerID_P2P_CIDv1_BASE32_Nonb58_chars() throws {
        let ma = try Multiaddr("/p2p-circuit/p2p/bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4")
        #expect(ma.getPeerIDString() == "bafzbeidt255unskpefjmqb2rc27vjuyxopkxgaylxij6pw35hhys4vnyp4")
    }

    @Test func testGetPeerID_From_Address_Without_A_PeerID() throws {
        let ma = try Multiaddr("/ip4/0.0.0.0/tcp/1234/utp")
        #expect(ma.getPeerIDString() == nil)
    }

    /// - MARK: Path Extraction Tests
    //    it('should return a path for unix', () => {
    //      expect(
    //        multiaddr('/unix/tmp/p2p.sock').getPath()
    //      ).to.eql('/tmp/p2p.sock')
    //    })
    @Test func testGetPathForUnix() throws {
        let ma = try Multiaddr("/unix/tmp/p2p.sock")
        #expect(ma.getPath() == "/tmp/p2p.sock")
    }
    //
    //    it('should return a path for unix when other protos exist', () => {
    //      expect(
    //        multiaddr('/ip4/0.0.0.0/tcp/1234/unix/tmp/p2p.sock').getPath()
    //      ).to.eql('/tmp/p2p.sock')
    //    })
    @Test func testGetPathForUnix_Multiple_Protos() throws {
        let ma = try Multiaddr("/ip4/0.0.0.0/tcp/1234/unix/tmp/p2p.sock")
        #expect(ma.getPath() == "/tmp/p2p.sock")
    }
    //
    //    it('should not return a path when no path proto exists', () => {
    //      expect(
    //        multiaddr('/ip4/0.0.0.0/tcp/1234/p2p-circuit/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC').getPath()
    //      ).to.eql(null)
    //    })
    @Test func testGetPathForUnix_No_Path() throws {
        let ma = try Multiaddr("/ip4/0.0.0.0/tcp/1234/p2p-circuit/p2p/QmcgpsyWgH8Y8ajJz1Cu72KnS5uo2Aa2LpzU7kinSupNKC")
        #expect(ma.getPath() == nil)
    }

}

extension Data {
    func hexString() -> String {
        map { String(format: "%02x", $0) }.joined()
    }
}
