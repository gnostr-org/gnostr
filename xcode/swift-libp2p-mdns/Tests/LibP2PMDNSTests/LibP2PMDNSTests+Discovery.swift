import DNS
import LibP2P
import Multiaddr
import PeerID
import Testing

@testable import LibP2PMDNS

extension LibP2PMDNSTests {
    @Test func testExtractMultiaddressesFromAdditionalRecords() throws {
        let app = Application(.testing, peerID: try PeerID(.Ed25519))
        defer { app.shutdown() }

        let peerID = try PeerID(.Ed25519)
        let serviceName = "\(peerID.b58String).p2p.local"
        let records: [any ResourceRecord] = [
            DNS.ServiceRecord(name: serviceName, ttl: 120, port: 4001, server: "host.local"),
            DNS.TextRecord(name: serviceName, ttl: 120, attributes: ["tcp": "4001"]),
            DNS.HostRecord<IPv4>(name: "host.local", ttl: 120, ip: IPv4("127.0.0.1")!),
        ]

        let mdns = MulticastPeerDiscovery(app: app)
        let addrs = mdns.extractMultiaddressFromAdditionalRecords(records)

        #expect(
            addrs.contains(
                try Multiaddr("/ip4/127.0.0.1/tcp/4001/p2p/\(peerID.b58String)")
            )
        )
    }
}
