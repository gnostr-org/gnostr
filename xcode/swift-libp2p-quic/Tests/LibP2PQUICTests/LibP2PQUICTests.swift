import LibP2P
import Multiaddr
import Testing

@testable import LibP2PQUIC

@Suite("LibP2P QUIC Tests", .serialized)
final class LibP2PQUICTests {
    @Test
    func testCanDialQuicAddresses() throws {
        let app = try Application(.testing, peerID: PeerID(.Ed25519))
        let quic = QUIC(application: app)

        #expect(quic.canDial(address: try Multiaddr("/ip4/127.0.0.1/udp/4001/quic")))
        #expect(quic.canDial(address: try Multiaddr("/ip4/127.0.0.1/udp/4001/quic-v1")))
        #expect(!quic.canDial(address: try Multiaddr("/ip4/127.0.0.1/tcp/4001")))
    }

    @Test
    func testTransportProviderRegisters() async throws {
        let app = try await Application.make(.testing, peerID: .ephemeral())
        app.transports.use(.quic)

        #expect(app.transports.available.contains("quic"))
        #expect(app.transports.transport(for: QUIC.self) != nil)
    }
}
