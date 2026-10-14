//
//  ContentView.swift
//  MiniP2PSample
//
//  Created by Lightech on 10/24/2048.
//

import Foundation
import CryptoKit
import LibP2P
import LibP2PKadDHT
import LibP2PMDNS
import LibP2PNoise
import LibP2PPubSub
import LibP2PYAMUX
import os
import SwiftUI
#if os(iOS)
import UIKit
#endif

// MARK: - Message model matching `gnostr chat` JSON

/// Mirrors the `Msg` JSON used by the Rust `gnostr chat` command.
private struct ChatMsgPayload: Codable {
    var from: String
    var content: [String]
    var kind: String
}

fileprivate enum MsgKind: String, CaseIterable, Identifiable {
    case chat = "Chat"
    case join = "Join"
    case leave = "Leave"
    case system = "System"
    case raw = "Raw"
    case heartbeat = "Heartbeat"
    case command = "Command"
    case debug = "Debug"
    case unknown = "Unknown"

    var id: String { rawValue }

    var label: String {
        switch self {
        case .chat: return "Chat"
        case .join: return "Join"
        case .leave: return "Leave"
        case .system: return "System"
        case .raw: return "Raw"
        case .heartbeat: return "Heartbeat"
        case .command: return "Command"
        case .debug: return "Debug"
        case .unknown: return "Unknown"
        }
    }

    var tint: Color {
        switch self {
        case .chat: return .blue
        case .join: return .green
        case .leave: return .orange
        case .system: return .red
        case .raw: return .purple
        case .heartbeat: return .cyan
        case .command: return .indigo
        case .debug: return .gray
        case .unknown: return .secondary
        }
    }

    static func parse(_ value: String) -> MsgKind {
        MsgKind(rawValue: value) ?? .unknown
    }
}

fileprivate struct ChatMessage: Identifiable, Equatable {
    let id = UUID()
    let kind: MsgKind
    let from: String
    let body: String
    let peerID: String?
    let addresses: [String]
    let timestamp: Date
}

// MARK: - P2P Service

@MainActor
final class P2PService: ObservableObject {
    private static let gossipTopic = "gnostr-dev"

    private enum RuntimeProfile: String {
        case macOS
        case macCatalyst
        case iPad
        case iPhone
        case madeForiPad
    }

    enum State: String {
        case stopped
        case starting
        case running
        case stopping
    }

    @Published private(set) var listenAddresses: [String] = []
    @Published private(set) var state: State = .stopped
    @Published private(set) var lastError: String?
    @Published private(set) var activityLog: [String] = []
    @Published fileprivate var chatMessages: [ChatMessage] = []
    @Published var displayName: String = "" {
        didSet { UserDefaults.standard.set(displayName, forKey: "MiniP2PSample.displayName") }
    }

    private var app: Application?
    private var runTask: Task<Void, Never>?
    private var shutdownTask: Task<Void, Never>?
    private var diagnosticsTask: Task<Void, Never>?
    private var gossipTask: Task<Void, Never>?
    private var pingTask: Task<Void, Never>?
    private var joinTask: Task<Void, Never>?
    private var startedAt: Date?
    private var gossipSubscription: PubSub.SubscriptionHandler?
    private var gossipSubConfig: PubSub.SubscriptionConfig?
    private var discoveredPeersByID: [String: PeerInfo] = [:]
    private var gossipPeersByID = Set<String>()
    private var pingBackoffUntilByPeerID: [String: Date] = [:]
    private var pingCursor = 0
    private var hasAnnouncedJoin = false
    private var hasReannouncedSubscription = false

    let peerID: PeerID
    private let networkLogger = Logger(subsystem: "MiniP2PSample", category: "network")

    init() {
        peerID = Self.makePeerID()
        if let saved = UserDefaults.standard.string(forKey: "MiniP2PSample.displayName"), !saved.isEmpty {
            displayName = saved
        } else {
            displayName = "\(Self.platformPrefix)-\(peerID.b58String)"
        }
    }

    var runtimeProfile: String {
        Self.runtimeProfile.rawValue
    }

    var listenPort: Int {
        let tcpAddr = listenAddresses.first { $0.contains("/tcp/") }
        guard let addr = tcpAddr else { return 0 }
        let pattern = "/tcp/"
        guard let range = addr.range(of: pattern) else { return 0 }
        let after = addr[range.upperBound...]
        guard let end = after.firstIndex(where: { !$0.isNumber }) else {
            return Int(after) ?? 0
        }
        return Int(after[..<end]) ?? 0
    }

    var stateLabel: String {
        state.rawValue.capitalized
    }

    var isRunning: Bool {
        state == .running
    }

    var peerIDString: String {
        peerID.b58String
    }

    func clearActivityLog() {
        activityLog.removeAll()
    }

    func clearChat() {
        chatMessages.removeAll()
    }

    func start() {
        guard app == nil, state == .stopped else {
            log("Start ignored: node is \(state.rawValue)")
            return
        }

        lastError = nil
        state = .starting
        hasAnnouncedJoin = false
        log("Starting node")
        startedAt = Date()
        log("Runtime profile: \(runtimeProfile)")
        log("Listen port: auto (OS-assigned)")
        log("Peer ID: \(peerIDString)")
        log("Chat handle: \(displayName)")

        let app = Self.makeApplication(peerID: peerID)
        self.app = app
        log("Configured libp2p logger at trace level")
        configureNetworking(app)
        subscribeToGossip(app)
        listenAddresses = captureListenAddressStrings(for: app)
        if !listenAddresses.isEmpty {
            log("Listening on: \(listenAddresses.joined(separator: ", "))")
        }
        logNetworkSnapshot(reason: "startup")
        startDiagnosticsLoop(for: app)
        startGossipHeartbeatLoop(for: app)
        startPingLoop(for: app)
        startJoinAnnouncementLoop(for: app)

        app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) { [weak self] in
            guard let self else { return }
            Task { @MainActor in
                if self.state == .starting {
                    self.state = .running
                    self.log("Node is running")
                }
                self.refreshListenAddresses(for: app)
            }
        }

        app.eventLoopGroup.next().scheduleTask(in: .seconds(1)) { [weak self] in
            guard let self else { return }
            Task { @MainActor in
                self.refreshListenAddresses(for: app)
                if !self.listenAddresses.isEmpty {
                    self.log("Listening on: \(self.listenAddresses.joined(separator: ", "))")
                }
            }
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 2) { [weak self] in
            guard let self else { return }
            if let envAddrs = ProcessInfo.processInfo.environment["P2P_DIAL_ADDRS"] {
                self.log("P2P_DIAL_ADDRS env var: \(envAddrs)")
                for addr in envAddrs.split(separator: ",").map({ $0.trimmingCharacters(in: .whitespaces) }) {
                    guard !addr.isEmpty else { continue }
                    self.log("Auto-dialing env address: \(addr)")
                    self.dialAddress(addr)
                }
            } else {
                self.log("P2P_DIAL_ADDRS env var not set")
            }
        }

        DispatchQueue.main.asyncAfter(deadline: .now() + 6) { [weak self] in
            guard let self else { return }
            if let envMessage = ProcessInfo.processInfo.environment["P2P_CHAT_MESSAGE"], !envMessage.isEmpty {
                self.log("P2P_CHAT_MESSAGE env var set, auto-sending: \(envMessage)")
                self.sendChat(envMessage)
            } else {
                self.log("P2P_CHAT_MESSAGE env var not set")
            }
        }

        runTask = Task.detached(priority: .background) { [weak self, app] in
            do {
                await MainActor.run {
                    self?.log("Executing libp2p application")
                }
                try await app.execute()
            } catch {
                await MainActor.run {
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run {
                self?.log("libp2p application returned")
                self?.diagnosticsTask?.cancel()
                self?.diagnosticsTask = nil
                self?.gossipTask?.cancel()
                self?.gossipTask = nil
                self?.pingTask?.cancel()
                self?.pingTask = nil
                self?.joinTask?.cancel()
                self?.joinTask = nil
                self?.gossipSubscription?.unsubscribe()
                self?.gossipSubscription = nil
                self?.gossipPeersByID.removeAll()
                self?.pingBackoffUntilByPeerID.removeAll()
                if self?.state != .stopped && self?.state != .stopping {
                    self?.state = .stopped
                }
                self?.runTask = nil
                self?.shutdownTask = nil
                self?.app = nil
                self?.discoveredPeersByID.removeAll()
                self?.startedAt = nil
                self?.log("Node stopped")
            }
        }
    }

    @discardableResult
    func stop() -> Task<Void, Never>? {
        guard let app, state != .stopped, state != .stopping else { return nil }

        state = .stopping
        log("Stopping node")
        logNetworkSnapshot(reason: "stopping")
        diagnosticsTask?.cancel()
        diagnosticsTask = nil
        gossipTask?.cancel()
        gossipTask = nil
        pingTask?.cancel()
        pingTask = nil
        joinTask?.cancel()
        joinTask = nil

        if !gossipPeersByID.isEmpty {
            publishLeave(app: app)
        }

        gossipSubscription?.unsubscribe()
        gossipSubscription = nil
        gossipPeersByID.removeAll()
        pingBackoffUntilByPeerID.removeAll()

        let capturedApp = app
        self.app = nil
        self.runTask = nil

        let shutdownTask = Task.detached(priority: .background) { [weak self] in
            // Use the synchronous shutdown path to avoid the asyncShutdown storage-clear
            // ordering bug that triggers "ConnectionManager not initialized" in the
            // swift-libp2p dependency.
            capturedApp.shutdown()

            await MainActor.run {
                self?.listenAddresses = []
                self?.state = .stopped
                self?.shutdownTask = nil
                self?.discoveredPeersByID.removeAll()
                self?.startedAt = nil
                self?.log("Node stopped")
            }
        }

        self.shutdownTask = shutdownTask
        return shutdownTask
    }

    func restart() {
        guard state != .starting else { return }
        let shutdownTask = stop() ?? self.shutdownTask
        guard let shutdownTask else {
            start()
            return
        }

        Task.detached(priority: .background) { [weak self] in
            _ = await shutdownTask.value
            await MainActor.run {
                self?.start()
            }
        }
    }

    func dialAddress(_ address: String) {
        guard let app = app, isRunning else {
            log("Cannot dial: node is not running")
            return
        }
        let trimmed = address.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        do {
            let ma = try Multiaddr(trimmed)
            try app.newStream(to: ma, forProtocol: "/ipfs/id/1.0.0")
            log("Dialed multiaddr \(ma.description)")
        } catch {
            lastError = error.localizedDescription
            log("Failed to dial \(trimmed): \(error.localizedDescription)")
        }
    }

    func sendChat(_ text: String) {
        let trimmed = text.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }

        if trimmed.hasPrefix("/raw ") {
            let body = String(trimmed.dropFirst(5))
            publish(kind: .raw, body: body)
            return
        }
        if trimmed == "/join" {
            publish(kind: .join, body: "joined \(Self.gossipTopic)")
            return
        }
        if trimmed == "/heartbeat" {
            publishHeartbeat()
            return
        }

        publish(kind: .chat, body: trimmed)
    }

    // MARK: - Application setup

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .trace
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.connectionManager.setIdleTimeout(.seconds(30))
        app.discovery.use(.mdns)
        app.dht.use(.kadDHT(mode: .client))
        app.pubsub.use(.gossipsub(emitSelf: false))
        app.listen(.tcp(host: "0.0.0.0", port: 0))
        return app
    }

    private func configureNetworking(_ app: Application) {
        app.discovery.onPeerDiscovered(app) { [weak self] peer in
            DispatchQueue.main.async {
                self?.recordDiscoveredPeer(peer)
                self?.dialDiscoveredPeer(peer)
            }
        }

        app.events.on(self, event: .identifiedPeer({ [weak self] identified in
            Task { @MainActor in
                guard let self else { return }
                guard identified.peer.b58String != self.peerIDString else { return }
                self.log("Peer identified", peerID: identified.peer.b58String, addresses: nil)
            }
        }))

        app.events.on(self, event: .remotePeerProtocolChange({ [weak self] change in
            Task { @MainActor in
                guard let self else { return }
                let peerID = change.peer.b58String
                guard peerID != self.peerIDString else { return }
                let addrs = self.peerAddressesString(for: peerID)
                let protocols = change.protocols.map(\.stringValue).joined(separator: ", ")
                self.log(
                    "Remote peer protocols updated: \(protocols)",
                    peerID: peerID,
                    addresses: addrs
                )
            }
        }))

        app.events.on(self, event: .upgraded({ [weak self] connection in
            Task { @MainActor in
                self?.log(
                    "Connection upgraded",
                    peerID: connection.remotePeer?.b58String,
                    addresses: connection.remoteAddr.map { [$0.description] }
                )
            }
        }))

        app.events.on(self, event: .disconnected({ [weak self] connection, peer in
            Task { @MainActor in
                self?.handleDisconnectedPeer(connection: connection, peer: peer)
            }
        }))
    }

    // MARK: - Gossipsub subscription

    private func subscribeToGossip(_ app: Application) {
        do {
            let config = PubSub.SubscriptionConfig(
                topic: Self.gossipTopic,
                signaturePolicy: .strictSign,
                validator: .acceptAll,
                messageIDFunc: .concatFromAndSequenceFields
            )
            self.gossipSubConfig = config

            let subscription = try app.pubsub.gossipsub.subscribe(config)

            subscription.on = { [weak self] event in
                switch event {
                case .newPeer(let peer):
                    Task { @MainActor in
                        guard let self else { return }
                        guard peer.b58String != self.peerIDString else { return }
                        self.gossipPeersByID.insert(peer.b58String)
                        let addrs = self.peerAddressesString(for: peer.b58String)
                        self.log(
                            "Peer joined gossip topic \(Self.gossipTopic)",
                            peerID: peer.b58String,
                            addresses: addrs
                        )
                        self.appendChat(
                            kind: .system,
                            from: "topic",
                            body: "peer \(peer.b58String) joined \(Self.gossipTopic)",
                            peerID: peer.b58String,
                            addresses: addrs
                        )

                        if !self.hasReannouncedSubscription, let cfg = self.gossipSubConfig {
                            self.hasReannouncedSubscription = true
                            self.log("Re-announcing subscription to \(Self.gossipTopic) to all known peers")
                            do {
                                _ = try app.pubsub.gossipsub.subscribe(cfg)
                                self.log("Re-announce subscription succeeded")
                            } catch {
                                self.log("Re-announce subscription failed: \(error.localizedDescription)")
                            }
                        }
                    }
                case .data(let message):
                    let author = message.from.asString(base: .base58btc)
                    let addrs = self?.peerAddressesString(for: author)
                    let data = Data(message.data)
                    Task { @MainActor in
                        guard author != self?.peerIDString else { return }
                        self?.handleIncomingGossip(data: data, from: author, addresses: addrs)
                    }
                case .error(let error):
                    Task { @MainActor in
                        self?.lastError = error.localizedDescription
                        self?.log("Gossip error: \(error.localizedDescription)")
                    }
                }
                return app.eventLoopGroup.next().makeSucceededVoidFuture()
            }

            gossipSubscription = subscription
            log("Subscribed to gossip topic \(Self.gossipTopic)")
        } catch {
            lastError = error.localizedDescription
            log("Failed to subscribe to gossip topic \(Self.gossipTopic): \(error.localizedDescription)")
        }
    }

    private func handleIncomingGossip(data: Data, from author: String, addresses: [String]?) {
        if let payload = try? JSONDecoder().decode(ChatMsgPayload.self, from: data) {
            let kind = MsgKind.parse(payload.kind)
            let body = payload.content.first ?? ""
            let displayBody: String
            switch kind {
            case .heartbeat:
                displayBody = "[heartbeat] \(body)"
            case .join:
                displayBody = "[join] \(body)"
            case .leave:
                displayBody = "[leave] \(body)"
            default:
                displayBody = body
            }
            log(
                "\(kind.label) on \(Self.gossipTopic) from \(payload.from): \(displayBody)",
                peerID: author,
                addresses: addresses
            )
            appendChat(kind: kind, from: payload.from, body: body, peerID: author, addresses: addresses)
        } else {
            let text = String(data: data, encoding: .utf8) ?? "non-utf8 payload"
            log(
                "Raw gossip on \(Self.gossipTopic): \(text)",
                peerID: author,
                addresses: addresses
            )
            appendChat(kind: .raw, from: author, body: text, peerID: author, addresses: addresses)
        }
    }

    private func appendChat(kind: MsgKind, from: String, body: String, peerID: String?, addresses: [String]?) {
        let message = ChatMessage(
            kind: kind,
            from: from,
            body: body,
            peerID: peerID,
            addresses: addresses ?? [],
            timestamp: Date()
        )
        chatMessages.append(message)
    }

    // MARK: - Publishing

    private func publish(kind: MsgKind, body: String) {
        guard let app = app, isRunning else {
            log("Cannot publish \(kind.label): node is not running")
            return
        }

        let payload = ChatMsgPayload(from: displayName, content: [body, ""], kind: kind.rawValue)
        guard let data = try? JSONEncoder().encode(payload) else {
            log("Failed to encode \(kind.label) payload")
            return
        }

        let bytes = Array(data)
        let result = app.pubsub.gossipsub.publish(
            topic: Self.gossipTopic,
            bytes: bytes,
            on: app.eventLoopGroup.next()
        )

        let local = ChatMessage(
            kind: kind,
            from: displayName,
            body: body,
            peerID: peerIDString,
            addresses: listenAddresses,
            timestamp: Date()
        )
        chatMessages.append(local)
        log("Published \(kind.label) to \(Self.gossipTopic): \(body)", peerID: peerIDString, addresses: listenAddresses)

        _ = result
    }

    private func publishJoin(app: Application) {
        guard let selfApp = self.app, selfApp === app else { return }
        publish(kind: .join, body: "joined \(Self.gossipTopic) on \(listenAddresses.joined(separator: ", "))")
        hasAnnouncedJoin = true
    }

    private func publishLeave(app: Application) {
        guard let selfApp = self.app, selfApp === app else { return }
        publish(kind: .leave, body: "left \(Self.gossipTopic)")
    }

    private func publishHeartbeat() {
        let body = "heartbeat #\(heartbeatCounter) from \(displayName) addrs=\(listenAddresses.joined(separator: ", "))"
        heartbeatCounter += 1
        publish(kind: .heartbeat, body: body)
    }

    private var heartbeatCounter = 0

    // MARK: - Background loops

    private func startGossipHeartbeatLoop(for app: Application) {
        gossipTask?.cancel()
        gossipTask = Task.detached(priority: .background) { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 10_000_000_000)
                guard !Task.isCancelled else { break }
                let hasPeers = await MainActor.run {
                    guard let self else { return false }
                    return !self.gossipPeersByID.isEmpty && self.state == .running
                }
                guard hasPeers else { continue }
                await MainActor.run {
                    self?.publishHeartbeat()
                }
            }
        }
    }

    private func startJoinAnnouncementLoop(for app: Application) {
        joinTask?.cancel()
        joinTask = Task.detached(priority: .background) { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 500_000_000)
                guard !Task.isCancelled else { break }
                let shouldAnnounce = await MainActor.run {
                    guard let self else { return false }
                    return self.state == .running && !self.hasAnnouncedJoin && !self.gossipPeersByID.isEmpty
                }
                guard shouldAnnounce else { continue }
                await MainActor.run {
                    self?.publishJoin(app: app)
                }
            }
        }
    }

    private func startPingLoop(for app: Application) {
        pingTask?.cancel()
        pingTask = Task.detached(priority: .background) { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 1_000_000_000)
                guard !Task.isCancelled else { break }
                await MainActor.run {
                    self?.pingKnownPeers(using: app)
                }
            }
        }
    }

    private func handleDisconnectedPeer(connection: Connection, peer: PeerID?) {
        guard let peer else {
            log("Peer disconnected", addresses: connection.remoteAddr.map { [$0.description] })
            return
        }

        guard peer.b58String != peerIDString else { return }
        let peerID = peer.b58String
        let wasInGossipMesh = gossipPeersByID.remove(peerID) != nil
        let addrs = peerAddressesString(for: peerID)
        log(
            wasInGossipMesh
                ? "Peer left gossip topic \(Self.gossipTopic)"
                : "Peer disconnected",
            peerID: peerID,
            addresses: addrs
        )
        if wasInGossipMesh {
            appendChat(
                kind: .system,
                from: "topic",
                body: "peer \(peerID) left \(Self.gossipTopic)",
                peerID: peerID,
                addresses: addrs
            )
        }
    }

    private func pingKnownPeers(using app: Application) {
        guard let identify = app.identify as? Identify else {
            return
        }

        let candidates = discoveredPeersByID.keys.filter { gossipPeersByID.contains($0) }
        guard !candidates.isEmpty else { return }

        let orderedCandidates = candidates.sorted()
        let index = pingCursor % orderedCandidates.count
        pingCursor = (pingCursor + 1) % orderedCandidates.count
        let peerID = orderedCandidates[index]
        guard peerID != peerIDString else { return }
        guard pingBackoffUntilByPeerID[peerID].map({ $0 > Date() }) != true else { return }
        guard let peerInfo = discoveredPeersByID[peerID] else { return }

        let peer = peerInfo.peer
        let pingFuture = identify.ping(peer: peer)
        pingFuture.whenSuccess { [weak self] duration in
            Task { @MainActor in
                self?.pingBackoffUntilByPeerID.removeValue(forKey: peerID)
                self?.log(
                    "Ping success (\(duration))",
                    peerID: peer.b58String,
                    addresses: peerInfo.addresses.map(\.description)
                )
            }
        }
        pingFuture.whenFailure { [weak self] error in
            Task { @MainActor in
                self?.pingBackoffUntilByPeerID[peerID] = Date().addingTimeInterval(10)
                self?.log(
                    "Ping failed: \(error.localizedDescription)",
                    peerID: peer.b58String,
                    addresses: peerInfo.addresses.map(\.description)
                )
            }
        }
    }

    private func recordDiscoveredPeer(_ peer: PeerInfo) {
        let peerID = peer.peer.b58String
        guard discoveredPeersByID[peerID] == nil else { return }
        discoveredPeersByID[peerID] = peer
        let addresses = peer.addresses.map(\.description)
        log("Discovered peer", peerID: peerID, addresses: addresses)
    }

    private func dialDiscoveredPeer(_ peer: PeerInfo) {
        guard peer.peer.b58String != peerIDString else { return }
        do {
            try app?.newStream(to: peer, forProtocol: "/ipfs/id/1.0.0")
            log(
                "Dialed discovered peer for identify",
                peerID: peer.peer.b58String,
                addresses: peer.addresses.map(\.description)
            )
        } catch {
            log(
                "Failed to dial peer: \(error.localizedDescription)",
                peerID: peer.peer.b58String,
                addresses: peer.addresses.map(\.description)
            )
        }
    }

    private func startDiagnosticsLoop(for app: Application) {
        diagnosticsTask?.cancel()
        diagnosticsTask = Task.detached(priority: .background) { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 5_000_000_000)
                guard !Task.isCancelled else { break }
                await MainActor.run {
                    self?.refreshListenAddresses(for: app)
                    self?.logNetworkSnapshot(reason: "periodic")
                }
            }
        }
    }

    private func refreshListenAddresses(for app: Application?) {
        guard let app, self.app === app else { return }
        let addresses = captureListenAddressStrings(for: app)
        if addresses != listenAddresses {
            listenAddresses = addresses
        }
    }

    private func captureListenAddressStrings(for app: Application) -> [String] {
        app.listenAddresses.compactMap { address -> String? in
            guard let fullAddress = try? address.encapsulate(proto: .p2p, address: app.peerID.b58String) else {
                return nil
            }
            return fullAddress.description
        }
    }

    private func logNetworkSnapshot(reason: String) {
        let addresses = listenAddresses
        let elapsed: String
        if let startedAt {
            elapsed = String(format: "%.1fs", Date().timeIntervalSince(startedAt))
        } else {
            elapsed = "n/a"
        }

        let snapshot = "Network snapshot [\(reason)]: state=\(state.rawValue), runtime=\(runtimeProfile), peer=\(peerIDString), port=\(listenPort), elapsed=\(elapsed), listenAddresses=\(addresses.isEmpty ? "none" : addresses.joined(separator: ", "))"
        log(snapshot)
    }

    // MARK: - Utility

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

    private static var platformPrefix: String {
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
            return "ios-mac"
        }
    }

    private static var listenPort: Int {
        if let value = ProcessInfo.processInfo.environment["P2P_LISTEN_PORT"],
           let port = Int(value),
           port > 0 {
            return port
        }
        return 0
    }

    private static func makePeerID() -> PeerID {
        let seed = Data(SHA256.hash(data: Data(Self.instanceSeed().utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        return try! PeerID(marshaledPrivateKey: privateKey.marshal())
    }

    private static func instanceSeed() -> String {
        let defaults = UserDefaults.standard
        let key = "MiniP2PSample.peerSeed"

        if let existing = defaults.string(forKey: key), !existing.isEmpty {
            return existing
        }

        let seed = UUID().uuidString
        defaults.set(seed, forKey: key)
        return seed
    }

    private func log(_ message: String, peerID: String? = nil, addresses: [String]? = nil) {
        let formatter = Self.timestampFormatter
        let targetPeerID = peerID ?? peerIDString
        let formatted = "[\(formatter.string(from: Date()))] \(message)\(peerContext(peerID: targetPeerID, addresses: addresses))"
        activityLog.append(formatted)
        networkLogger.info("\(formatted, privacy: .public)")
        print(formatted)
    }

    private func peerContext(peerID: String?, addresses: [String]?) -> String {
        var components: [String] = []
        if let peerID, !peerID.isEmpty {
            components.append("peer=\(peerID)")
        }
        if let addresses, !addresses.isEmpty {
            components.append("addrs=\(addresses.joined(separator: ", "))")
        }
        guard !components.isEmpty else { return "" }
        return " [\(components.joined(separator: " | "))]"
    }

    private func peerAddressesString(for peerID: String) -> [String]? {
        discoveredPeersByID[peerID]?.addresses.map(\.description)
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}

// MARK: - UI

struct ContentView: View {
    @StateObject private var service = P2PService()
    @State private var draft: String = ""
    @State private var dialAddress: String = ""
    @State private var isNicknameFocused: Bool = false
    @FocusState private var messageFieldFocused: Bool

    var body: some View {
        NavigationStack {
            GeometryReader { proxy in
                VStack(spacing: 12) {
                    stickyHeader

                    chatCard

                    sectionCard(title: "Network activity", subtitle: "Recent network lifecycle events") {
                        ScrollView {
                            if service.activityLog.isEmpty {
                                emptyState("No activity yet")
                            } else {
                                VStack(alignment: .leading, spacing: 8) {
                                    ForEach(service.activityLog.indices, id: \.self) { index in
                                        logRow(service.activityLog[index])
                                    }
                                }
                            }
                        }
                        .frame(maxHeight: .infinity, alignment: .topLeading)
                    }
                    .frame(maxHeight: .infinity, alignment: .topLeading)

                    sectionCard(title: "Listening addresses", subtitle: "Encapsulated multiaddrs for this node") {
                        if service.listenAddresses.isEmpty {
                            emptyState("No addresses yet")
                        } else {
                            VStack(alignment: .leading, spacing: 8) {
                                ForEach(service.listenAddresses, id: \.self) { address in
                                    addressRow(address)
                                }
                            }
                        }
                    }
                    .padding(.horizontal, 16)
                    .padding(.bottom, 16)
                }
                .frame(width: proxy.size.width, height: proxy.size.height, alignment: .top)
            }
            .navigationTitle("MiniP2PSample")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .primaryAction) {
                    Menu {
                        Button("Clear activity log") {
                            service.clearActivityLog()
                        }
                        Button("Clear chat") {
                            service.clearChat()
                        }
                    } label: {
                        Label("Clear", systemImage: "ellipsis.circle")
                    }
                }
            }
        }
        .onAppear {
            service.start()
        }
        .onDisappear {
            service.stop()
        }
    }

    private var stickyHeader: some View {
        VStack(alignment: .leading, spacing: 12) {
            headerCard
            actionBar

            if let lastError = service.lastError {
                statusBanner(title: "Last error", message: lastError, tint: .red)
            }
        }
    }

    private var headerCard: some View {
        VStack(alignment: .leading, spacing: 14) {
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 4) {
                    Text("MiniP2PSample")
                        .font(.largeTitle.bold())
                    Text("Deterministic libp2p node for SwiftUI")
                        .font(.subheadline)
                        .foregroundStyle(.secondary)
                }

                Spacer(minLength: 12)

                statusBadge
            }

            LazyVGrid(columns: [
                GridItem(.adaptive(minimum: 140), spacing: 12, alignment: .leading)
            ], alignment: .leading, spacing: 12) {
                statChip(title: "Runtime", value: service.runtimeProfile)
                statChip(title: "Listen port", value: "\(service.listenPort)")
                statChip(title: "State", value: service.stateLabel)
                statChip(title: "Topic", value: "gnostr-dev")
            }

            VStack(alignment: .leading, spacing: 6) {
                Text("Peer ID")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                Text(service.peerIDString)
                    .font(.caption.monospaced())
                    .textSelection(.enabled)
                    .frame(maxWidth: .infinity, alignment: .leading)
            }
            .padding(12)
            .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 14, style: .continuous))

            VStack(alignment: .leading, spacing: 6) {
                Text("Chat handle")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                TextField("ios-12D3KooW...", text: $service.displayName)
                    .font(.caption.monospaced())
                    .textFieldStyle(.roundedBorder)
                    .textSelection(.enabled)
            }
            .padding(12)
            .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 14, style: .continuous))

            VStack(alignment: .leading, spacing: 6) {
                Text("Dial multiaddr")
                    .font(.caption.weight(.semibold))
                    .foregroundStyle(.secondary)
                HStack(spacing: 8) {
                    TextField("/ip4/.../tcp/.../p2p/...", text: $dialAddress)
                        .font(.caption.monospaced())
                        .textFieldStyle(.roundedBorder)
                        .textSelection(.enabled)
                        .onSubmit {
                            service.dialAddress(dialAddress)
                        }

                    Button("Dial") {
                        service.dialAddress(dialAddress)
                    }
                    .buttonStyle(.bordered)
                    .disabled(service.state != .running || dialAddress.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }
            .padding(12)
            .background(Color.secondary.opacity(0.08), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
        }
        .padding(16)
        .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 20, style: .continuous))
    }

    private var actionBar: some View {
        HStack(spacing: 12) {
            Button("Start node") {
                service.start()
            }
            .buttonStyle(.borderedProminent)
            .disabled(service.state != .stopped)

            Button("Stop node") {
                service.stop()
            }
            .buttonStyle(.bordered)
            .disabled(service.state == .stopped || service.state == .stopping)

            Spacer(minLength: 12)

            Button("Restart") {
                service.restart()
            }
            .buttonStyle(.bordered)
            .disabled(service.state == .stopped || service.state == .starting || service.state == .stopping)
        }
        .padding(16)
        .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 20, style: .continuous))
    }

    private var chatCard: some View {
        sectionCard(title: "gnostr-dev chat", subtitle: "Messages on the topic") {
            VStack(spacing: 8) {
                ScrollViewReader { proxy in
                    ScrollView {
                        LazyVStack(alignment: .leading, spacing: 8) {
                            ForEach(service.chatMessages) { message in
                                chatRow(message)
                            }
                        }
                        .padding(.vertical, 4)
                    }
                    .onChange(of: service.chatMessages.count) { _ in
                        if let last = service.chatMessages.last {
                            withAnimation {
                                proxy.scrollTo(last.id, anchor: .bottom)
                            }
                        }
                    }
                }

                HStack(spacing: 8) {
                    TextField("Message... (/raw, /join, /heartbeat)", text: $draft)
                        .textFieldStyle(.roundedBorder)
                        .focused($messageFieldFocused)
                        .onSubmit {
                            submitDraft()
                        }

                    Button {
                        submitDraft()
                    } label: {
                        Image(systemName: "arrow.up.circle.fill")
                            .font(.title2)
                    }
                    .disabled(draft.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                }
            }
        }
        .padding(.horizontal, 16)
    }

    private func submitDraft() {
        let text = draft
        draft = ""
        service.sendChat(text)
        messageFieldFocused = false
    }

    private func sectionCard<Content: View>(
        title: String,
        subtitle: String,
        @ViewBuilder content: () -> Content
    ) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            VStack(alignment: .leading, spacing: 3) {
                Text(title)
                    .font(.headline)
                Text(subtitle)
                    .font(.footnote)
                    .foregroundStyle(.secondary)
            }

            content()
        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 20, style: .continuous))
    }

    private func statChip(title: String, value: String) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.caption.weight(.semibold))
                .foregroundStyle(.secondary)
            Text(value)
                .font(.headline)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
    }

    private var statusBadge: some View {
        let tint = statusTint
        return Text(service.stateLabel)
            .font(.caption.weight(.semibold))
            .textCase(.uppercase)
            .foregroundStyle(tint)
            .padding(.horizontal, 12)
            .padding(.vertical, 8)
            .background(tint.opacity(0.14), in: Capsule())
    }

    private func statusBanner(title: String, message: String, tint: Color) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
                .font(.caption.weight(.semibold))
                .foregroundStyle(tint)
            Text(message)
                .font(.subheadline)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(14)
        .background(tint.opacity(0.10), in: RoundedRectangle(cornerRadius: 16, style: .continuous))
    }

    private func emptyState(_ message: String) -> some View {
        Text(message)
            .font(.subheadline)
            .foregroundStyle(.secondary)
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(12)
            .background(Color.primary.opacity(0.03), in: RoundedRectangle(cornerRadius: 14, style: .continuous))
    }

    private func logRow(_ message: String) -> some View {
        Text(message)
            .font(.caption.monospaced())
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(12)
            .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .textSelection(.enabled)
    }

    private func addressRow(_ address: String) -> some View {
        Text(address)
            .font(.caption.monospaced())
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(12)
            .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
            .textSelection(.enabled)
    }

    private func chatRow(_ message: ChatMessage) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            HStack(spacing: 6) {
                Text(message.kind.label)
                    .font(.caption2.weight(.bold))
                    .padding(.horizontal, 6)
                    .padding(.vertical, 2)
                    .background(message.kind.tint.opacity(0.15), in: Capsule())
                    .foregroundStyle(message.kind.tint)

                Text(message.from)
                    .font(.caption.weight(.semibold))
                    .textSelection(.enabled)

                Spacer()

                Text(timeString(message.timestamp))
                    .font(.caption2)
                    .foregroundStyle(.secondary)
            }

            Text(message.body)
                .font(.subheadline)
                .frame(maxWidth: .infinity, alignment: .leading)
                .textSelection(.enabled)

            if let peerID = message.peerID, !peerID.isEmpty {
                Text("peer: \(peerID)")
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                    .textSelection(.enabled)
            }

            if !message.addresses.isEmpty {
                Text("addrs: \(message.addresses.joined(separator: ", "))")
                    .font(.caption2.monospaced())
                    .foregroundStyle(.secondary)
                    .textSelection(.enabled)
            }
        }
        .padding(12)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(message.kind.tint.opacity(0.06), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
        .id(message.id)
    }

    private var statusTint: Color {
        switch service.state {
        case .running:
            return .green
        case .starting, .stopping:
            return .orange
        case .stopped:
            return .secondary
        }
    }

    private func timeString(_ date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter.string(from: date)
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
