import Testing
import LibP2P

@testable import LibP2PRelay

@Suite("Relay Tests")
struct LibP2PRelayTests {
    @Test func testReservationStateDefaultsToFalse() throws {
        let app = Application(.testing)
        defer { app.shutdown() }

        let coordinator = RelayCoordinator(application: app)
        #expect(coordinator.isReserved(peer: try PeerID(.Ed25519)) == false)
    }
}
