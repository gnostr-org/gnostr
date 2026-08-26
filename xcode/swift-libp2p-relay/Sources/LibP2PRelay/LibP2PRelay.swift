import Foundation
import LibP2P
import Multiaddr
import NIOCore

enum RelayWire {
    static let protocolID = "/libp2p/circuit/relay/0.2.0"

    enum Command: UInt8, Sendable {
        case reserve = 0
        case connect = 1
    }

    struct Request: Sendable {
        let command: Command
        let peerID: String?
    }

    struct Response: Sendable {
        let accepted: Bool
        let relayAddress: Multiaddr?
    }

    static func encode(_ request: Request) throws -> ByteBuffer {
        var buffer = ByteBufferAllocator().buffer(capacity: 128)
        buffer.writeInteger(request.command.rawValue)
        if let peerID = request.peerID {
            let bytes = Array(peerID.utf8)
            buffer.writeInteger(UInt8(1))
            buffer.writeInteger(UInt16(bytes.count))
            buffer.writeBytes(bytes)
        } else {
            buffer.writeInteger(UInt8(0))
        }
        return buffer
    }

    static func decodeRequest(_ buffer: ByteBuffer) throws -> Request {
        var copy = buffer
        guard let commandRaw: UInt8 = copy.readInteger() else { throw Errors.invalidMessage }
        guard let command = Command(rawValue: commandRaw) else { throw Errors.invalidMessage }
        guard let hasPeerID: UInt8 = copy.readInteger() else { throw Errors.invalidMessage }
        var peerID: String?
        if hasPeerID == 1 {
            guard let length: UInt16 = copy.readInteger() else { throw Errors.invalidMessage }
            guard let bytes = copy.readBytes(length: Int(length)), let value = String(bytes: bytes, encoding: .utf8) else {
                throw Errors.invalidMessage
            }
            peerID = value
        }
        return Request(command: command, peerID: peerID)
    }

    static func encode(_ response: Response) throws -> ByteBuffer {
        var buffer = ByteBufferAllocator().buffer(capacity: 256)
        buffer.writeInteger(response.accepted ? UInt8(1) : UInt8(0))
        if let relayAddress = response.relayAddress {
            let bytes = try relayAddress.binaryPacked()
            buffer.writeInteger(UInt8(1))
            buffer.writeInteger(UInt16(bytes.count))
            buffer.writeBytes(bytes)
        } else {
            buffer.writeInteger(UInt8(0))
        }
        return buffer
    }

    enum Errors: Error {
        case invalidMessage
    }
}

public final class RelayCoordinator: @unchecked Sendable {
    struct Reservation {
        let relayPeer: PeerID
        let relayAddress: Multiaddr
        let createdAt: Date
    }

    private let application: Application
    private let queue = DispatchQueue(label: "LibP2PRelay.reservations")
    private var reservations: [String: Reservation] = [:]

    init(application: Application) {
        self.application = application
    }

    func install() {
        self.application.events.on(self, event: .identifiedPeer(self.onIdentifiedPeer(_:)))
        self.application.events.on(self, event: .disconnected(self.onDisconnected(_:_:)))
        self.application.group("libp2p") { libp2p in
            libp2p.group("circuit", handlers: [.varIntLengthPrefixed]) { circuit in
                circuit.group("relay", handlers: [.varIntLengthPrefixed]) { relay in
                    relay.on("0.2.0", handlers: [.varIntLengthPrefixed]) { req in
                        try self.handle(req)
                    }
                }
            }
        }
        self.application.logger.notice("Installed relay/circuit reservation support")
    }

    public func isReserved(peer: PeerID) -> Bool {
        self.queue.sync { self.reservations[peer.b58String] != nil }
    }

    private func relayAddress(for peer: PeerID) -> Multiaddr? {
        guard let base = self.application.peerInfo.addresses.first else { return nil }
        guard let withPeer = try? base.encapsulate(proto: .p2p, address: self.application.peerID.b58String) else {
            return nil
        }
        return try? withPeer.encapsulate(proto: .p2p_circuit, address: nil)
    }

    private func recordReservation(for peer: PeerID) {
        guard let relayAddress = self.relayAddress(for: peer) else { return }
        self.queue.sync {
            self.reservations[peer.b58String] = Reservation(relayPeer: self.application.peerID, relayAddress: relayAddress, createdAt: .init())
        }
        _ = self.application.peers.add(address: relayAddress, toPeer: peer, on: self.application.eventLoopGroup.any())
    }

    private func onIdentifiedPeer(_ identifiedPeer: IdentifiedPeer) {
        self.application.peers.getProtocols(forPeer: identifiedPeer.peer, on: self.application.eventLoopGroup.any()).whenSuccess { protocols in
            guard protocols.contains(where: { $0.stringValue.contains("/libp2p/circuit/relay") || $0.stringValue.contains("/libp2p/circuit/relay/0.2.0") }) else {
                return
            }
            self.recordReservation(for: identifiedPeer.peer)
        }
    }

    private func onDisconnected(_ connection: Connection, _ peerID: PeerID?) {
        guard let peerID else { return }
        self.queue.sync {
            self.reservations.removeValue(forKey: peerID.b58String)
        }
    }

    private func handle(_ req: Request) throws -> Response<ByteBuffer> {
        guard let remotePeer = req.remotePeer else { return .close }

        switch req.event {
        case .ready:
            return .stayOpen

        case .data(let payload):
            let request = try RelayWire.decodeRequest(payload)
            switch request.command {
            case .reserve:
                self.recordReservation(for: remotePeer)
                return .respondThenClose(try RelayWire.encode(.init(accepted: true, relayAddress: self.relayAddress(for: remotePeer))))

            case .connect:
                if let target = request.peerID {
                    self.application.logger.notice("Relay connect request for \(target) via \(remotePeer.b58String)")
                }
                return .respondThenClose(try RelayWire.encode(.init(accepted: true, relayAddress: self.relayAddress(for: remotePeer))))
            }

        case .closed:
            return .close

        case .error(let error):
            req.logger.error("Relay error: \(error)")
            return .close
        }
    }
}
