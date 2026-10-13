import Foundation
import LibP2P
import Multiaddr
import NIOCore
import Testing

@testable import LibP2PRelay

@Suite("Relay Tests")
struct LibP2PRelayTests {
    @Test func testReservationStateDefaultsToFalse() throws {
        let app = Application(.testing)
        defer { app.shutdown() }

        let coordinator = RelayCoordinator(application: app)
        #expect(coordinator.isReserved(peer: try PeerID(.Ed25519)) == false)
    }

    @Test func testEncodeAndDecodeReserveRequest() throws {
        let request = RelayWire.Request(command: .reserve, peerID: nil)
        let buffer = try RelayWire.encode(request)
        let decoded = try RelayWire.decodeRequest(buffer)

        #expect(decoded.command == .reserve)
        #expect(decoded.peerID == nil)
    }

    @Test func testEncodeAndDecodeConnectRequestWithPeerID() throws {
        let peerID = try PeerID(.Ed25519)
        let request = RelayWire.Request(command: .connect, peerID: peerID.b58String)
        let buffer = try RelayWire.encode(request)
        let decoded = try RelayWire.decodeRequest(buffer)

        #expect(decoded.command == .connect)
        #expect(decoded.peerID == peerID.b58String)
    }

    @Test func testEncodeAndDecodeAcceptedResponseWithAddress() throws {
        let address = try Multiaddr("/ip4/127.0.0.1/tcp/4001")
        let response = RelayWire.Response(accepted: true, relayAddress: address)
        let buffer = try RelayWire.encode(response)

        var copy = buffer
        guard let accepted: UInt8 = copy.readInteger() else {
            Issue.record("Failed to read accepted flag")
            return
        }
        #expect(accepted == 1)

        guard let hasAddress: UInt8 = copy.readInteger() else {
            Issue.record("Failed to read hasAddress flag")
            return
        }
        #expect(hasAddress == 1)

        guard let length: UInt16 = copy.readInteger() else {
            Issue.record("Failed to read address length")
            return
        }

        guard let addressBytes = copy.readBytes(length: Int(length)) else {
            Issue.record("Failed to read address bytes")
            return
        }

        let decodedAddress = try Multiaddr(Data(addressBytes))
        #expect(decodedAddress == address)
    }

    @Test func testEncodeAndDecodeRejectedResponse() throws {
        let response = RelayWire.Response(accepted: false, relayAddress: nil)
        let buffer = try RelayWire.encode(response)

        var copy = buffer
        guard let accepted: UInt8 = copy.readInteger() else {
            Issue.record("Failed to read accepted flag")
            return
        }
        #expect(accepted == 0)

        guard let hasAddress: UInt8 = copy.readInteger() else {
            Issue.record("Failed to read hasAddress flag")
            return
        }
        #expect(hasAddress == 0)
    }

    @Test func testDecodeRequestThrowsForInvalidMessage() throws {
        var buffer = ByteBufferAllocator().buffer(capacity: 1)
        buffer.writeInteger(UInt8(255))  // Invalid command

        #expect(throws: RelayWire.Errors.invalidMessage) {
            try RelayWire.decodeRequest(buffer)
        }
    }
}
