import LibP2P
import Multiaddr
import XCTest

@testable import LibP2PAutoNAT

final class LibP2PAutoNATTests: XCTestCase {
    func testWireRoundTripsDialRequest() throws {
        let message = AutoNATWire.DialRequest(
            token: [1, 2, 3, 4],
            addresses: [
                try Multiaddr("/ip4/127.0.0.1/tcp/10000"),
                try Multiaddr("/ip4/192.168.1.2/tcp/10001"),
            ]
        )

        let decoded = try AutoNATWire.decodeRequest(AutoNATWire.encode(message))
        XCTAssertEqual(decoded.token, message.token)
        XCTAssertEqual(decoded.addresses, message.addresses)
    }

    func testWireRoundTripsDialResponse() throws {
        let message = AutoNATWire.DialResponse(
            token: [5, 6, 7, 8],
            status: .ok,
            dialedAddress: try Multiaddr("/ip4/127.0.0.1/tcp/10000")
        )

        let decoded = try AutoNATWire.decodeResponse(AutoNATWire.encode(message))
        XCTAssertEqual(decoded.token, message.token)
        XCTAssertEqual(decoded.status, message.status)
        XCTAssertEqual(decoded.dialedAddress, message.dialedAddress)
    }

    func testStatusDefaultsToUnknown() async throws {
        let app = try await Application.make(.testing, peerID: .ephemeral(type: .Ed25519))

        let coordinator = AutoNATCoordinator(application: app)
        XCTAssertEqual(coordinator.status, .unknown)

        try await app.asyncShutdown()
    }
}
