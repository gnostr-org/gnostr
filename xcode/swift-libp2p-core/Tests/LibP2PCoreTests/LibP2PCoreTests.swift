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
import LibP2PCrypto
import Testing

@testable import LibP2PCore

@Suite("Libp2p Core Tests")
struct LibP2PCoreTests {
    @Test func testPeerIDRSA2048RoundTrip() throws {
        let peer = try PeerID(.RSA(bits: .B2048))
        let marshaled = try peer.marshalPublicKey()
        let reconstructed = try PeerID(marshaledPublicKey: Data(marshaled))
        #expect(peer == reconstructed)
        #expect(peer.b58String == reconstructed.b58String)
        #expect(reconstructed.type == .isPublic)
    }

    @Test func testPeerIDEd25519RoundTrip() throws {
        let peer = try PeerID(.Ed25519)
        let marshaled = try peer.marshalPublicKey()
        let reconstructed = try PeerID(marshaledPublicKey: Data(marshaled))
        #expect(peer == reconstructed)
        #expect(reconstructed.type == .isPublic)
    }

    @Test func testPeerIDSecp256k1RoundTrip() throws {
        let peer = try PeerID(.Secp256k1)
        let marshaled = try peer.marshalPublicKey()
        let reconstructed = try PeerID(marshaledPublicKey: Data(marshaled))
        #expect(peer == reconstructed)
        #expect(reconstructed.type == .isPublic)
    }

    @Test func testPeerIDFromHexID() throws {
        let peer = try PeerID(.Ed25519)
        let hex = peer.hexString
        let fromHex = try PeerID(fromHexID: hex)
        #expect(peer == fromHex)
    }

    @Test func testPeerIDFromBytesID() throws {
        let peer = try PeerID(.Ed25519)
        let bytes = peer.id
        let fromBytes = try PeerID(fromBytesID: bytes)
        #expect(peer == fromBytes)
    }

    @Test func testPeerIDFromCIDRoundTrip() throws {
        let peer = try PeerID(.Ed25519)
        let cid = peer.cidString
        let fromCID = try PeerID(cid: cid)
        #expect(peer == fromCID)
    }

    @Test func testPeerIDDescriptionsAreNonEmpty() throws {
        let peer = try PeerID(.Ed25519)
        #expect(!peer.description.isEmpty)
        #expect(!peer.shortDescription.isEmpty)
        #expect(!peer.debugDescription.isEmpty)
    }

    @Test func testMultiaddrEncapsulateAndDecapsulate() throws {
        let base = try Multiaddr("/ip4/127.0.0.1/tcp/4001")
        let peer = try PeerID(.Ed25519)
        let withPeer = try base.encapsulate(proto: .p2p, address: peer.b58String)
        #expect(withPeer.getPeerIDString() == peer.b58String)

        let decapsulated = withPeer.decapsulate(.p2p)
        #expect(decapsulated == base)
    }

    @Test func testMultiaddrProtocols() throws {
        let ma = try Multiaddr("/ip4/127.0.0.1/tcp/4001/ws")
        let names = ma.protoNames()
        #expect(names.contains("ip4"))
        #expect(names.contains("tcp"))
        #expect(names.contains("ws"))

        let codes = ma.protoCodes()
        #expect(codes.contains { $0 == MultiaddrProtocol.ip4.code })
    }

    @Test func testMultiaddrEquality() throws {
        let a = try Multiaddr("/ip4/127.0.0.1/tcp/4001")
        let b = try Multiaddr("/ip4/127.0.0.1/tcp/4001")
        let c = try Multiaddr("/ip4/127.0.0.1/tcp/4002")
        #expect(a == b)
        #expect(a != c)
    }
}
