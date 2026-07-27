import Foundation
import XCTest
@testable import LibP2PDCUtR
import LibP2P

final class LibP2PDCUtRTests: XCTestCase {
    func testWireRoundTripsConnectMessage() throws {
        let original = HolePunch(
            type: .connect,
            obsAddrs: [
                try Multiaddr("/ip4/127.0.0.1/tcp/10000").binaryPacked(),
                try Multiaddr("/ip4/192.168.1.2/tcp/10001").binaryPacked(),
            ]
        )

        let decoded = try DCUtRWire.decode(DCUtRWire.encode(original))

        XCTAssertEqual(decoded.type, .connect)
        XCTAssertEqual(decoded.obsAddrs, original.obsAddrs)
    }

    func testDialablePeerInfoFiltersCircuitAddresses() throws {
        let peer = try PeerID()
        let peerInfo = PeerInfo(
            peer: peer,
            addresses: [
                try Multiaddr("/ip4/127.0.0.1/tcp/10000"),
                try Multiaddr("/ip4/127.0.0.1/tcp/10001/p2p-circuit"),
                try Multiaddr("/dns4/example.com/tcp/10002"),
            ]
        )

        let app = Application(.testing)
        defer { app.shutdown() }

        let dialable = DCUtRCoordinator(application: app).dialablePeerInfo(in: peerInfo)

        XCTAssertEqual(dialable.peer, peer)
        XCTAssertEqual(dialable.addresses, [try Multiaddr("/ip4/127.0.0.1/tcp/10000")])
    }

    func testHasRelayReservationRequiresCircuitAddress() throws {
        let app = Application(.testing)
        defer { app.shutdown() }

        let coordinator = DCUtRCoordinator(application: app)
        let peer = try PeerID()

        let noRelay = PeerInfo(
            peer: peer,
            addresses: [
                try Multiaddr("/ip4/127.0.0.1/tcp/10000"),
            ]
        )
        XCTAssertFalse(coordinator.hasRelayReservation(in: noRelay))

        let withRelay = PeerInfo(
            peer: peer,
            addresses: [
                try Multiaddr("/ip4/127.0.0.1/tcp/10000"),
                try Multiaddr("/ip4/127.0.0.1/tcp/10001/p2p-circuit"),
            ]
        )
        XCTAssertTrue(coordinator.hasRelayReservation(in: withRelay))
    }
}
