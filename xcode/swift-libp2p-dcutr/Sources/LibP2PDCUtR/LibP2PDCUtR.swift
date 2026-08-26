import LibP2P
import SwiftProtobuf

enum DCUtRWire {
    static let protocolID = "/libp2p/dcutr/1.0.0"

    static func encode(_ message: HolePunch) throws -> ByteBuffer {
        let data = try message.serializedData()
        var buffer = ByteBufferAllocator().buffer(capacity: data.count)
        buffer.writeBytes(data)
        return buffer
    }

    static func decode(_ buffer: ByteBuffer) throws -> HolePunch {
        try HolePunch(serializedBytes: Data(buffer.readableBytesView))
    }
}

final class DCUtRCoordinator: @unchecked Sendable {
    struct Attempt {
        var relayConnection: Connection?
        var remotePeerInfo: PeerInfo?
        var connectSentAt: Date?
        var connectReceivedAt: Date?
        var waitingForSync: Bool = false
    }

    private let application: Application
    private let queue = DispatchQueue(label: "LibP2PDCUtR.attempts")
    private var attempts: [String: Attempt] = [:]

    init(application: Application) {
        self.application = application
    }

    func install() {
        self.application.events.on(self, event: .connected(self.onConnected(_:)))
        self.application.events.on(self, event: .disconnected(self.onDisconnected(_:_:)))
        self.application.events.on(self, event: .identifiedPeer(self.onIdentifiedPeer(_:)))
        self.application.group("libp2p") { libp2p in
            libp2p.group("dcutr", handlers: [.varIntLengthPrefixed]) { dcutr in
                dcutr.on("1.0.0", handlers: [.varIntLengthPrefixed]) { req -> Response<ByteBuffer> in
                    self.handle(req)
                }
            }
        }
        self.application.logger.notice("Installed DCUtR hole punching support")
    }

    private func isRelayConnection(_ connection: Connection) -> Bool {
        guard let addr = connection.remoteAddr else { return false }
        return addr.protocols().contains(where: { $0 == .p2p_circuit })
    }

    private func attempt(for peer: PeerID) -> Attempt {
        self.queue.sync { self.attempts[peer.b58String] ?? Attempt() }
    }

    private func setAttempt(_ attempt: Attempt, for peer: PeerID) {
        self.queue.sync { self.attempts[peer.b58String] = attempt }
    }

    private func clearAttempt(for peer: PeerID) {
        _ = self.queue.sync { self.attempts.removeValue(forKey: peer.b58String) }
    }

    private func mergePeerInfo(_ lhs: PeerInfo?, with rhs: PeerInfo) -> PeerInfo {
        guard let lhs else { return rhs }
        let addresses = Array(Set(lhs.addresses).union(rhs.addresses))
        return PeerInfo(peer: rhs.peer, addresses: addresses)
    }

    func hasRelayReservation(in peerInfo: PeerInfo) -> Bool {
        peerInfo.addresses.contains { $0.protocols().contains(.p2p_circuit) }
    }

    private func refreshPeerInfo(for peer: PeerID) {
        self.application.peers.getPeerInfo(byID: peer.b58String, on: self.application.eventLoopGroup.any()).whenSuccess { peerInfo in
            self.queue.sync {
                let merged = self.mergePeerInfo(self.attempts[peer.b58String]?.remotePeerInfo, with: peerInfo)
                var attempt = self.attempts[peer.b58String] ?? Attempt()
                attempt.remotePeerInfo = merged
                self.attempts[peer.b58String] = attempt
            }
            self.startPunchIfReady(for: peer)
        }
    }

    private func startPunchIfReady(for peer: PeerID) {
        let eventLoop = self.application.eventLoopGroup.any()
        self.application.peers.getPeerInfo(byID: peer.b58String, on: eventLoop).whenSuccess { peerInfo in
            guard self.hasRelayReservation(in: peerInfo) else { return }

            var relayConnection: Connection?
            self.queue.sync {
                guard
                    let attempt = self.attempts[peer.b58String],
                    attempt.connectSentAt == nil,
                    let connection = attempt.relayConnection
                else {
                    return
                }
                relayConnection = connection
            }

            guard let relayConnection else { return }
            self.initiatePunch(for: peer, relayConnection: relayConnection)
        }
    }

    private func registerRelayConnection(_ connection: Connection, for peer: PeerID) {
        var attempt = self.attempt(for: peer)
        guard attempt.relayConnection == nil else { return }
        attempt.relayConnection = connection
        self.setAttempt(attempt, for: peer)
    }

    private func localObservedAddresses() -> [Multiaddr] {
        self.application.peerInfo.addresses.filter { !($0.protocols().contains(.p2p_circuit)) }
    }

    private func makePayload(type: HolePunch.Kind) throws -> ByteBuffer {
        let message = HolePunch(type: type, obsAddrs: try self.localObservedAddresses().map { try $0.binaryPacked() })
        return try DCUtRWire.encode(message)
    }

    private func parsePeerInfo(from message: HolePunch, fallbackPeer: PeerID?) throws -> PeerInfo {
        var addrs: [Multiaddr] = []
        for raw in message.obsAddrs {
            do {
                let ma = try Multiaddr(raw)
                if ma.protocols().contains(where: { $0 == .p2p_circuit }) { continue }
                addrs.append(ma)
            } catch {
                self.application.logger.warning("Skipping invalid hole punch address: \(error)")
            }
        }
        let peer = fallbackPeer ?? self.application.peerID
        return PeerInfo(peer: peer, addresses: addrs)
    }

    func dialablePeerInfo(in peerInfo: PeerInfo) -> PeerInfo {
        let addresses = peerInfo.addresses.filter { address in
            let protocols = address.protocols()
            return protocols.contains(.ip4)
                && protocols.contains(.tcp)
                && !protocols.contains(.p2p_circuit)
        }
        return PeerInfo(peer: peerInfo.peer, addresses: addresses)
    }

    private func initiatePunch(for peer: PeerID, relayConnection: Connection) {
        var attempt = self.attempt(for: peer)
        guard attempt.relayConnection == nil else { return }
        attempt.relayConnection = relayConnection
        attempt.connectSentAt = Date()
        self.setAttempt(attempt, for: peer)
        do {
            try self.application.newStream(to: peer, forProtocol: DCUtRWire.protocolID)
        } catch {
            self.application.logger.error("DCUtR: failed to open connect stream to \(peer.b58String): \(error)")
        }
    }

    private func sendSync(for peer: PeerID, remoteInfo: PeerInfo, halfRTT: TimeInterval) {
        var attempt = self.attempt(for: peer)
        attempt.remotePeerInfo = remoteInfo
        attempt.waitingForSync = true
        self.setAttempt(attempt, for: peer)
        let dialableInfo = self.dialablePeerInfo(in: remoteInfo)
        guard !dialableInfo.addresses.isEmpty else {
            self.application.logger.warning("DCUtR: no dialable address available for sync to \(peer.b58String)")
            return
        }
        let delay = max(0.0, halfRTT)
        self.application.eventLoopGroup.any().scheduleTask(in: .milliseconds(Int64(delay * 1000))) {
            do {
                try self.application.newStream(to: dialableInfo, forProtocol: DCUtRWire.protocolID)
            } catch {
                self.application.logger.error("DCUtR: failed to open sync stream to \(peer.b58String): \(error)")
            }
        }
    }

    private func dialDirect(for peer: PeerID, remoteInfo: PeerInfo) {
        let relayConnection = self.attempt(for: peer).relayConnection
        let dialableInfo = self.dialablePeerInfo(in: remoteInfo)
        guard !dialableInfo.addresses.isEmpty else {
            self.application.logger.warning("DCUtR: no dialable address available for direct punch to \(peer.b58String)")
            return
        }
        do {
            try self.application.newStream(to: dialableInfo, forProtocol: DCUtRWire.protocolID)
            self.application.eventLoopGroup.any().scheduleTask(in: .seconds(2)) {
                relayConnection?.close().whenComplete { _ in }
            }
            self.clearAttempt(for: peer)
        } catch {
            self.application.logger.error("DCUtR: direct dial failed for \(peer.b58String): \(error)")
        }
    }

    private func onConnected(_ connection: Connection) {
        guard self.isRelayConnection(connection), let peer = connection.remotePeer else { return }
        self.registerRelayConnection(connection, for: peer)
        self.refreshPeerInfo(for: peer)
        self.startPunchIfReady(for: peer)
    }

    private func onIdentifiedPeer(_ identifiedPeer: IdentifiedPeer) {
        self.refreshPeerInfo(for: identifiedPeer.peer)
        self.startPunchIfReady(for: identifiedPeer.peer)
    }

    private func onDisconnected(_ connection: Connection, _ peer: PeerID?) {
        guard let peer else { return }
        self.clearAttempt(for: peer)
        if self.isRelayConnection(connection) {
            self.application.logger.notice("DCUtR: relay connection closed for \(peer.b58String)")
        }
    }

    private func handle(_ req: Request) -> Response<ByteBuffer> {
        guard let peer = req.remotePeer else { return .close }
        switch req.event {
        case .ready:
            var attempt = self.attempt(for: peer)
            if req.streamDirection == .outbound, attempt.connectSentAt == nil {
                attempt.connectSentAt = Date()
                self.setAttempt(attempt, for: peer)
                do {
                    return .respond(try self.makePayload(type: .connect))
                } catch {
                    req.logger.error("DCUtR: failed to encode connect payload: \(error)")
                    return .close
                }
            }
            if attempt.waitingForSync {
                do {
                    return .respondThenClose(try self.makePayload(type: .sync))
                } catch {
                    req.logger.error("DCUtR: failed to encode sync payload: \(error)")
                    return .close
                }
            }
            return .stayOpen

        case .data(let payload):
            do {
                let message = try DCUtRWire.decode(payload)
                let remoteInfo = try self.parsePeerInfo(from: message, fallbackPeer: peer)
                switch message.type {
                case .connect:
                    var attempt = self.attempt(for: peer)
                    attempt.remotePeerInfo = self.mergePeerInfo(attempt.remotePeerInfo, with: remoteInfo)
                    attempt.connectReceivedAt = Date()
                    self.setAttempt(attempt, for: peer)

                    if req.streamDirection == .inbound {
                        return .respondThenClose(try self.makePayload(type: .connect))
                    }

                    if let sentAt = attempt.connectSentAt {
                        self.sendSync(for: peer, remoteInfo: remoteInfo, halfRTT: Date().timeIntervalSince(sentAt) / 2.0)
                    } else {
                        self.sendSync(for: peer, remoteInfo: remoteInfo, halfRTT: 0.05)
                    }
                    return .stayOpen

                case .sync:
                    self.dialDirect(for: peer, remoteInfo: remoteInfo)
                    return .close
                }
            } catch {
                req.logger.error("DCUtR: invalid punch payload: \(error)")
                return .close
            }

        case .closed:
            return .close

        case .error(let error):
            req.logger.error("DCUtR: stream error: \(error)")
            return .close
        }
    }
}

extension HolePunch {
    init(type: HolePunch.Kind, obsAddrs: [Data]) {
        self.init()
        self.type = type
        self.obsAddrs = obsAddrs
    }
}
