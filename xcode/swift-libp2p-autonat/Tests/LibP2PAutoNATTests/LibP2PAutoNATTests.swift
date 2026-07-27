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

    func testStatusDefaultsToUnknown() throws {
        let app = Application(.testing)
        defer { app.shutdown() }

        let coordinator = AutoNATCoordinator(application: app)
        XCTAssertEqual(coordinator.status, .unknown)
    }

    func testPendingProbeFailsAndCleansUp() throws {
        let app = Application(.testing)
        defer { app.shutdown() }

        let coordinator = AutoNATCoordinator(application: app)
        let peer = try PeerID(.Ed25519)
        let promise = app.eventLoopGroup.any().makePromise(of: AutoNATStatus.self)

        coordinator.debugInsertPendingProbe(peer: peer, promise: promise)
        XCTAssertEqual(coordinator.debugPendingCount(), 1)

        coordinator.debugFailPendingProbe(peer: peer, error: AutoNATCoordinator.ProbeError.timedOut)

        XCTAssertThrowsError(try promise.futureResult.wait())
        XCTAssertEqual(coordinator.debugPendingCount(), 0)
    }

    func testPendingProbeCompletesAndCleansUp() throws {
        let app = Application(.testing)
        defer { app.shutdown() }

        let coordinator = AutoNATCoordinator(application: app)
        let peer = try PeerID(.Ed25519)
        let promise = app.eventLoopGroup.any().makePromise(of: AutoNATStatus.self)

        coordinator.debugInsertPendingProbe(peer: peer, promise: promise)
        coordinator.debugCompletePendingProbe(peer: peer, status: .publicReachable)

        XCTAssertEqual(try promise.futureResult.wait(), .publicReachable)
        XCTAssertEqual(coordinator.debugPendingCount(), 0)
    }
}
