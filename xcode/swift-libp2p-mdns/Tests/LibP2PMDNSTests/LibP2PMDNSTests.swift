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

import DNS
import LibP2P
import Multiaddr
import NIO
import Network
import PeerID
import Testing

@testable import LibP2PMDNS

@Suite("Libp2p mDNS Tests")
struct LibP2PMDNSTests {
    // Try and get our machines internal IP Address
    let internalIPAddress = try! System.enumerateDevices().first(where: { device in
        guard device.name == "en0" && device.address != nil else { return false }
        guard let ma = try? device.address?.toMultiaddr().tcpAddress else { return false }

        return ma.ip4
    }).map { try! $0.address!.toMultiaddr().tcpAddress!.address }!

    /// System.enumerateDevices
    @Test func testSystemDevices() throws {
        for device in try! System.enumerateDevices() {
            print("Description: \(device)")
            print("Interface Index: \(device.interfaceIndex)")
            print("Name: \(device.name)")
            print("Address: \(String(describing: device.address))")
            print("Broadcast Address: \(String(describing: device.broadcastAddress))")
            print("Netmask: \(String(describing: device.netmask))")
        }
    }

    @Test func testSystemDevicesEN0() throws {
        let devices = try System.enumerateDevices().filter({ device in
            guard device.name == "en0" && device.address != nil else { return false }
            guard let ma = try? device.address?.toMultiaddr().tcpAddress else { return false }

            return ma.ip4
        })
        for device in devices {
            print("Description: \(device)")
            print("Interface Index: \(device.interfaceIndex)")
            print("Name: \(device.name)")
            print("Address: \(try! device.address!.toMultiaddr().tcpAddress!.address)")
            print("Broadcast Address: \(String(describing: device.broadcastAddress))")
            print("Netmask: \(String(describing: device.netmask))")
        }
    }

    @Test func testFixSameHostAddresses() {
        let hostIP4 = "192.168.1.21"
        let hostIP6 = "f4:d4:88:5c:bf:7b"
        let mas = [
            try! Multiaddr("/ip4/192.168.1.21/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"),
            try! Multiaddr("/ip4/127.0.0.1/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"),
            try! Multiaddr("/ip4/192.168.1.23/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"),
            //try! Multiaddr("/ip6/f4:d4:88:5c:bf:7b/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"),
            //try! Multiaddr("/ip6/b4:44:28:5d:af:7c/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV")
        ]

        let fixed = mas.compactMap { ma -> Multiaddr? in
            guard let addy = ma.addresses.first else { return nil }
            switch addy.codec {
            case .ip4:
                if addy.addr == hostIP4 {
                    do {
                        var new = try Multiaddr(.ip4, address: "127.0.0.1")
                        for proto in ma.addresses.dropFirst() {
                            new = try new.encapsulate(proto: proto.codec, address: proto.addr)
                        }
                        return new
                    } catch {
                        return nil
                    }
                } else {
                    return ma
                }
            case .ip6:
                if addy.addr == hostIP6 {
                    do {
                        var new = try Multiaddr(.ip6, address: "::1")
                        for proto in ma.addresses.dropFirst() {
                            new = try new.encapsulate(proto: proto.codec, address: proto.addr)
                        }
                        return new
                    } catch {
                        return nil
                    }
                } else {
                    return ma
                }
            default:
                return nil
            }
        }

        // Our host machines ip address was changed to 127.0.0.1
        #expect(
            fixed[0].description == "/ip4/127.0.0.1/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"
        )
        // 127.0.0.1 was left unchanged
        #expect(
            fixed[1].description == "/ip4/127.0.0.1/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"
        )
        // Another local address (not our host machine) was left unchanged
        #expect(
            fixed[2].description == "/ip4/192.168.1.23/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"
        )
    }

    @Test func testFixSameHostAddresses2() {
        let hostIP4 = "192.168.1.21"
        let hostIP6 = "f4:d4:88:5c:bf:7b"
        let mas = [
            try! Multiaddr("/ip4/192.168.1.21/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"),
            try! Multiaddr("/ip4/127.0.0.1/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"),
            try! Multiaddr("/ip4/192.168.1.23/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"),
            //try! Multiaddr("/ip6/f4:d4:88:5c:bf:7b/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"),
            //try! Multiaddr("/ip6/b4:44:28:5d:af:7c/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV")
        ]

        let fixed = mas.compactMap { ma -> Multiaddr? in
            guard let addy = ma.addresses.first else { return nil }
            switch addy.codec {
            case .ip4:
                return ma.replace(address: hostIP4, with: "127.0.0.1", forCodec: .ip4)
            case .ip6:
                return ma.replace(address: hostIP6, with: "::1", forCodec: .ip6)
            default:
                return nil
            }
        }

        // Our host machines ip address was changed to 127.0.0.1
        #expect(
            fixed[0].description == "/ip4/127.0.0.1/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"
        )
        // 127.0.0.1 was left unchanged
        #expect(
            fixed[1].description == "/ip4/127.0.0.1/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"
        )
        // Another local address (not our host machine) was left unchanged
        #expect(
            fixed[2].description == "/ip4/192.168.1.23/tcp/1234/p2p/QmX99Bk1CoBc787KY36tcy43AbXoo1HiG1WTDpenmJecNV"
        )
    }

    @Test func testCreateTextRecord() throws {
        let txtRecord = DNS.TextRecord(
            name: "chat-room-name",
            ttl: 3600,
            attributes: ["port": "3338", "test": "something"]
        )
        var txtBytes = Data()
        var labels = Labels()
        try txtRecord.serialize(onto: &txtBytes, labels: &labels)

        // The parameter serialization is not deterministic
        //XCTAssertEqual(
        //    txtBytes.asString(base: .base16),
        //    "0e636861742d726f6f6d2d6e616d65000010000100000e10001909706f72743d333333380e746573743d736f6d657468696e67"
        //)
        #expect(txtBytes.count == 51)
    }

    @Test func testCreateTextRecord2() throws {
        var txtRecord = TXTRecordRef()
        var buffer = Data()
        TXTRecordCreate(&txtRecord, 1, &buffer)
        var value = "1234"
        TXTRecordSetValue(&txtRecord, "test", UInt8(value.bytes.count), &value)

        #expect(TXTRecordGetLength(&txtRecord) == 10)

        TXTRecordDeallocate(&txtRecord)
    }
}
