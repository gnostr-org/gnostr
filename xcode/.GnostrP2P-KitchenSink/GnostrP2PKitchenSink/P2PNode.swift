//
//  P2PNode.swift
//  GnostrP2P-KitchenSink
//

import CryptoKit
import Foundation
import LibP2P
import LibP2PDCUtR
import LibP2PKadDHT
import LibP2PMDNS
import LibP2PNoise
import LibP2PYAMUX
import NIO
import SwiftUI

#if os(iOS)
    import UIKit
#endif

@MainActor
protocol P2PNodeDelegate: AnyObject {
    func p2pNode(_ node: P2PNode, didDiscoverPeer peer: PeerInfo)
    func p2pNode(_ node: P2PNode, didDisconnectPeer peer: PeerID, connection: Connection)
    func p2pNode(_ node: P2PNode, didUpdateListenAddresses addresses: [String])
    func p2pNodeDidStart(_ node: P2PNode)
    func p2pNodeDidStop(_ node: P2PNode)
}

@MainActor
final class P2PNode: ObservableObject {
    enum State: String {
        case stopped
        case starting
        case running
        case stopping
    }

    struct PeerSummary: Identifiable, Hashable {
        var id: String { peerID }
        let peerID: String
        let addresses: [String]
    }

    private enum RuntimeProfile: String {
        case macOS
        case macCatalyst
        case iPad
        case iPhone
        case madeForiPad
    }

    @Published private(set) var state: State = .stopped
    @Published private(set) var listenAddresses: [String] = []
    @Published private(set) var discoveredPeers: [PeerSummary] = []
    @Published private(set) var lastError: String?
    @Published private(set) var logs: [String] = []

    let peerID: PeerID
    let displayName: String

    weak var delegate: P2PNodeDelegate?

    private var app: Application?
    private var eventLoopGroup: MultiThreadedEventLoopGroup?
    private var runTask: Task<Void, Never>?
    private var dialedPeerIDs = Set<String>()

    init() {
        peerID = Self.makePeerID()
        displayName = "\(Self.platformLabel)-\(peerID.b58String)"
    }

    var runtimeProfile: String {
        Self.runtimeProfile.rawValue
    }

    var listenPort: Int {
        Self.listenPort
    }

    var peerIDString: String {
        peerID.b58String
    }

    var isRunning: Bool {
        state == .running
    }

    var application: Application? {
        app
    }

    func clearLogs() {
        logs.removeAll()
    }

    // MARK: - Lifecycle

    func start() {
        guard runTask == nil, state == .stopped else { return }

        lastError = nil
        state = .starting
        log("P2PNode.start() peerID=\(peerID.b58String) displayName=\(displayName) platformLabel=\(Self.platformLabel) listenPort=\(Self.listenPort)")

        let group = MultiThreadedEventLoopGroup(numberOfThreads: System.coreCount)
        eventLoopGroup = group
        let app = Self.makeApplication(peerID: peerID, eventLoopGroup: group)
        self.app = app

        app.discovery.onPeerDiscovered(app) { [weak self] peer in
            let peerID = peer.peer.b58String
            let addresses = peer.addresses.map(\.description)
            DispatchQueue.main.async { [weak self] in
                self?.log("onPeerDiscovered peerID=\(peerID) addresses=\(addresses)")
                self?.recordDiscoveredPeer(peerID: peerID, addresses: addresses)
                self?.dialDiscoveredPeer(peer)
                if let self {
                    self.delegate?.p2pNode(self, didDiscoverPeer: peer)
                }
            }
        }

        app.events.on(self, event: .disconnected({ [weak self] connection, peer in
            Task { @MainActor in
                guard let self, let peer else { return }
                self.handleDisconnectedPeer(connection: connection, peer: peer)
                self.delegate?.p2pNode(self, didDisconnectPeer: peer, connection: connection)
            }
        }))

        app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) { [weak self, weak app] in
            guard let self, let app else { return }
            let addresses = app.listenAddresses.compactMap { address -> String? in
                guard let fullAddress = try? address.encapsulate(proto: .p2p, address: app.peerID.b58String) else {
                    return nil
                }
                return fullAddress.description
            }

            Task { @MainActor in
                self.listenAddresses = addresses
                self.delegate?.p2pNode(self, didUpdateListenAddresses: addresses)
                self.log("listenAddressesUpdated count=\(addresses.count) addresses=\(addresses)")
                if !addresses.isEmpty {
                    self.log("Listening on: \(addresses.joined(separator: ", "))")
                }
                if self.state == .starting {
                    self.state = .running
                    self.log("Node is running peerID=\(self.peerID.b58String)")
                    self.delegate?.p2pNodeDidStart(self)
                }
            }
        }

        runTask = Task.detached(priority: .background) { [weak self, app, group] in
            do {
                try await app.execute()
            } catch {
                await MainActor.run { [weak self] in
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run { [weak self] in
                guard let self else { return }
                self.state = .stopped
                self.runTask = nil
                self.app = nil
                self.listenAddresses = []
                self.dialedPeerIDs.removeAll()
                self.log("Node stopped peerID=\(self.peerID.b58String)")
                self.delegate?.p2pNodeDidStop(self)
            }

            try? await group.shutdownGracefully()
            await MainActor.run { [weak self] in
                self?.eventLoopGroup = nil
                self?.log("EventLoopGroup shutdown peerID=\(self?.peerID.b58String ?? "?")")
            }
        }
    }

    func stop() {
        Task { await stopAsync() }
    }

    func restart() {
        Task { await stopAsync(); start() }
    }

    private func stopAsync() async {
        guard let app else { return }

        state = .stopping
        log("P2PNode.stopAsync() peerID=\(peerID.b58String)")

        let group = eventLoopGroup
        let runTaskToAwait = runTask
        self.app = nil
        self.runTask = nil
        self.eventLoopGroup = nil

        do {
            try await app.asyncShutdown()
        } catch {
            await MainActor.run { [weak self] in
                self?.lastError = error.localizedDescription
                self?.log("Error: \(error.localizedDescription)")
            }
        }

        await runTaskToAwait?.value

        if let group {
            try? await group.shutdownGracefully()
            log("EventLoopGroup shutdown (async) peerID=\(peerID.b58String)")
        }

        await MainActor.run { [weak self] in
            guard let self else { return }
            self.listenAddresses = []
            self.state = .stopped
            self.runTask = nil
            self.app = nil
            self.eventLoopGroup = nil
            self.dialedPeerIDs.removeAll()
            self.log("Node stopped (async shutdown complete) peerID=\(self.peerID.b58String)")
            self.delegate?.p2pNodeDidStop(self)
        }
    }

    // MARK: - Peer management

    private func recordDiscoveredPeer(peerID: String, addresses: [String]) {
        let peer = PeerSummary(peerID: peerID, addresses: addresses)
        if !discoveredPeers.contains(peer) {
            discoveredPeers.append(peer)
            log("recordDiscoveredPeer peerID=\(peerID) addresses=\(addresses)")
        }
    }

    private func handleDisconnectedPeer(connection: Connection, peer: PeerID) {
        let peerID = peer.b58String
        guard peerID != self.peerID.b58String else { return }
        dialedPeerIDs.remove(peerID)
        log("handleDisconnectedPeer peerID=\(peerID) remoteAddr=\(connection.remoteAddr?.description ?? "nil")")
    }

    private func dialDiscoveredPeer(_ peer: PeerInfo) {
        guard let app else { return }
        let peerID = peer.peer.b58String
        guard peerID != self.peerID.b58String else { return }
        guard !dialedPeerIDs.contains(peerID) else { return }
        dialedPeerIDs.insert(peerID)

        let addresses = peer.addresses.map(\.description)
        log("dialDiscoveredPeer peerID=\(peerID) addresses=\(addresses)")
        do {
            try app.newStream(to: peer, forProtocol: "/ipfs/id/1.0.0")
            log("dialDiscoveredPeer success peerID=\(peerID)")
        } catch {
            lastError = error.localizedDescription
            log("dialDiscoveredPeer failure peerID=\(peerID) error=\(error.localizedDescription)")
        }
    }

    // MARK: - Helpers

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        let line = "[\(formatter.string(from: Date()))] \(message)"
        print(line)
        logs.insert(line, at: 0)
        if logs.count > 500 {
            logs = Array(logs.prefix(500))
        }
    }

    // MARK: - Configuration

    private static func makeApplication(peerID: PeerID, eventLoopGroup: EventLoopGroup) -> Application {
        let app = Application(.testing, peerID: peerID, eventLoopGroupProvider: .shared(eventLoopGroup))
        app.logger.logLevel = .notice
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.pubsub.use(.gossipsub(emitSelf: false))
        app.dcutr.use(.dcutr)
        app.discovery.use(.mdns)
        app.discovery.use(.kadDHT)
        app.listen(.tcp(host: "0.0.0.0", port: Self.listenPort))
        return app
    }

    private static var runtimeProfile: RuntimeProfile {
        #if targetEnvironment(macCatalyst)
            return .macCatalyst
        #elseif os(macOS)
            return .macOS
        #elseif os(iOS)
            if ProcessInfo.processInfo.isiOSAppOnMac {
                return .madeForiPad
            }
            switch UIDevice.current.userInterfaceIdiom {
                case .pad:
                    return .iPad
                case .mac:
                    return .madeForiPad
                default:
                    return .iPhone
            }
        #else
            return .iPhone
        #endif
    }

    private static var listenPort: Int {
        if let value = ProcessInfo.processInfo.environment["P2P_LISTEN_PORT"],
           let port = Int(value),
           port > 0 {
            return port
        }

        switch runtimeProfile {
            case .macOS:
                return 10000
            case .iPhone:
                return 10001
            case .iPad:
                return 10002
            case .macCatalyst:
                return 10003
            case .madeForiPad:
                return 10004
        }
    }

    private static var platformLabel: String {
        switch runtimeProfile {
            case .macOS:
                return "macos"
            case .macCatalyst:
                return "maccatalyst"
            case .iPad:
                return "ipad"
            case .iPhone:
                return "ios"
            case .madeForiPad:
                return "made-for-ipad"
        }
    }

    private static func makePeerID() -> PeerID {
        let seed = Data(SHA256.hash(data: Data("\(platformLabel)\(instanceSeed())".utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        return try! PeerID(marshaledPrivateKey: privateKey.marshal())
    }

    private static func instanceSeed() -> String {
        let defaults = UserDefaults.standard
        let key = "GnostrP2PKitchenSink.peerSeed"

        if let existing = defaults.string(forKey: key), !existing.isEmpty {
            return existing
        }

        let seed = UUID().uuidString
        defaults.set(seed, forKey: key)
        return seed
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}
