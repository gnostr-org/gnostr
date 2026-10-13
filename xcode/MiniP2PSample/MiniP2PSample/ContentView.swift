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

    private var app: Application?
    private var runTask: Task<Void, Never>?
    private var shutdownTask: Task<Void, Never>?
    private var diagnosticsTask: Task<Void, Never>?
    private var gossipTask: Task<Void, Never>?
    private var pingTask: Task<Void, Never>?
    private var startedAt: Date?
    private var gossipSubscription: PubSub.SubscriptionHandler?
    private var discoveredPeersByID: [String: PeerInfo] = [:]
    private var gossipPeersByID = Set<String>()
    private var pingBackoffUntilByPeerID: [String: Date] = [:]
    private var pingCursor = 0

    let peerID: PeerID
    private let networkLogger = Logger(subsystem: "MiniP2PSample", category: "network")

    init() {
        peerID = Self.makePeerID()
    }

    var runtimeProfile: String {
        Self.runtimeProfile.rawValue
    }

    var listenPort: Int {
        Self.listenPort
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

    func start() {
        guard runTask == nil, shutdownTask == nil else { return }

        lastError = nil
        state = .starting
        log("Starting node")
        startedAt = Date()
        log("Runtime profile: \(runtimeProfile)")
        log("Listen port: \(listenPort)")
        log("Peer ID: \(peerIDString)")

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

        app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) { [weak self] in
            guard let self else { return }
            Task { @MainActor in
                if self.state == .starting {
                    self.state = .running
                    self.log("Node is running")
                }
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
                self?.gossipSubscription?.unsubscribe()
                self?.gossipSubscription = nil
                self?.gossipPeersByID.removeAll()
                self?.pingBackoffUntilByPeerID.removeAll()
                self?.state = .stopped
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
        guard let app else { return nil }

        state = .stopping
        log("Stopping node")
        logNetworkSnapshot(reason: "stopping")
        diagnosticsTask?.cancel()
        diagnosticsTask = nil
        gossipTask?.cancel()
        gossipTask = nil
        pingTask?.cancel()
        pingTask = nil
        gossipSubscription?.unsubscribe()
        gossipSubscription = nil
        gossipPeersByID.removeAll()
        pingBackoffUntilByPeerID.removeAll()
        self.app = nil
        self.runTask = nil

        let shutdownTask = Task.detached(priority: .background) { [weak self] in
            do {
                try await app.asyncShutdown()
            } catch {
                await MainActor.run {
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run {
                self?.listenAddresses = []
                self?.state = .stopped
                self?.runTask = nil
                self?.shutdownTask = nil
                self?.app = nil
                self?.discoveredPeersByID.removeAll()
                self?.startedAt = nil
                self?.log("Node stopped")
            }
        }

        self.shutdownTask = shutdownTask
        return shutdownTask
    }

    func restart() {
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

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .trace
        app.discovery.use(.mdns)
        app.discovery.use(.kadDHT)
        app.pubsub.use(.gossipsub(emitSelf: false))
        app.listen(.tcp(host: "0.0.0.0", port: Self.listenPort))
        return app
    }

    private func configureNetworking(_ app: Application) {
        app.discovery.onPeerDiscovered(app) { [weak self] peer in
            DispatchQueue.main.async {
                self?.recordDiscoveredPeer(peer)
                self?.dialDiscoveredPeer(peer)
            }
        }

        app.events.on(self, event: .disconnected({ [weak self] connection, peer in
            Task { @MainActor in
                self?.handleDisconnectedPeer(connection: connection, peer: peer)
            }
        }))
    }

    private func subscribeToGossip(_ app: Application) {
        do {
            let subscription = try app.pubsub.gossipsub.subscribe(
                .init(
                    topic: Self.gossipTopic,
                    signaturePolicy: .strictSign,
                    validator: .acceptAll,
                    messageIDFunc: .concatFromAndSequenceFields
                )
            )

            subscription.on = { [weak self] event in
                switch event {
                case .newPeer(let peer):
                    Task { @MainActor in
                        guard peer.b58String != self?.peerIDString else { return }
                        self?.gossipPeersByID.insert(peer.b58String)
                        self?.log(
                            "Peer joined gossip topic \(Self.gossipTopic)",
                            peerID: peer.b58String,
                            addresses: self?.peerAddressesString(for: peer.b58String)
                        )
                    }
                case .data(let message):
                    let text = String(data: message.data, encoding: .utf8) ?? "non-utf8 payload"
                    let author = message.from.asString(base: .base58btc)
                    Task { @MainActor in
                        guard author != self?.peerIDString else { return }
                        if let heartbeat = Self.parseHeartbeatPayload(text) {
                            self?.log(
                                "Peer heartbeat #\(heartbeat.sequence) on \(Self.gossipTopic): \(heartbeat.payload)",
                                peerID: author,
                                addresses: self?.peerAddressesString(for: author)
                            )
                        } else {
                            self?.log(
                                "Gossip on \(Self.gossipTopic): \(text)",
                                peerID: author,
                                addresses: self?.peerAddressesString(for: author)
                            )
                        }
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

    private func startGossipHeartbeatLoop(for app: Application) {
        gossipTask?.cancel()
        gossipTask = Task.detached(priority: .background) { [weak self] in
            var counter = 0
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 1_000_000_000)
                guard !Task.isCancelled else { break }
                counter += 1
                let payload = "gnostr-dev heartbeat #\(counter) from \(app.peerID.b58String)"
                let result = app.pubsub.gossipsub.publish(
                    topic: Self.gossipTopic,
                    bytes: payload.data(using: .utf8)!.byteArray,
                    on: app.eventLoopGroup.next()
                )
                await MainActor.run {
                    self?.log("Published gossip heartbeat #\(counter) to \(Self.gossipTopic)")
                }
                _ = result
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
        log(
            wasInGossipMesh
                ? "Peer left gossip topic \(Self.gossipTopic)"
                : "Peer disconnected",
            peerID: peerID,
            addresses: peerAddressesString(for: peerID)
        )
    }

    private func pingKnownPeers(using app: Application) {
        guard let identify = app.identify as? Identify else {
            return
        }

        let candidates = discoveredPeersByID.keys.filter { gossipPeersByID.contains($0) }
        guard !candidates.isEmpty else {
            return
        }

        let orderedCandidates = candidates.sorted()
        guard !orderedCandidates.isEmpty else { return }
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
        let addresses = peer.addresses.map(\.description).joined(separator: ", ")
        log("Discovered peer", peerID: peerID, addresses: peer.addresses.map(\.description))
    }

    private func dialDiscoveredPeer(_ peer: PeerInfo) {
        guard peer.peer.b58String != peerIDString else { return }
        do {
            try app?.newStream(to: peer, forProtocol: "/ipfs/id/1.0.0")
            log(
                "Dialed discovered peer",
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
                    self?.logNetworkSnapshot(reason: "periodic")
                }
            }
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
        let formatted = "[\(formatter.string(from: Date()))] \(message)\(peerContext(peerID: peerID ?? peerIDString, addresses: addresses))"
        activityLog.append(formatted)
        networkLogger.info("\(formatted, privacy: .public)")
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

    private static func parseHeartbeatPayload(_ payload: String) -> (sequence: Int, payload: String)? {
        let prefix = "gnostr-dev heartbeat #"
        guard payload.hasPrefix(prefix) else { return nil }

        let remainder = payload.dropFirst(prefix.count)
        guard let separator = remainder.firstIndex(of: " "),
              let sequence = Int(remainder[..<separator]) else {
            return nil
        }

        return (sequence, String(remainder))
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}

struct ContentView: View {
    @StateObject private var service = P2PService()

    var body: some View {
        NavigationStack {
            GeometryReader { proxy in
                VStack(spacing: 16) {
                    stickyHeader

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
                    Button("Clear Log") {
                        service.clearActivityLog()
                    }
                    .disabled(service.activityLog.isEmpty)
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
        VStack(alignment: .leading, spacing: 16) {
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
                GridItem(.adaptive(minimum: 160), spacing: 12, alignment: .leading)
            ], alignment: .leading, spacing: 12) {
                statChip(title: "Runtime", value: service.runtimeProfile)
                statChip(title: "Listen port", value: "\(service.listenPort)")
                statChip(title: "State", value: service.stateLabel)
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

            Button("Stop node") {
                service.stop()
            }
            .buttonStyle(.bordered)
            .disabled(!service.isRunning)

            Spacer(minLength: 12)

            Button("Restart") {
                service.restart()
            }
            .buttonStyle(.bordered)
            .disabled(service.state == .stopped)
        }
        .padding(16)
        .background(Color.secondary.opacity(0.06), in: RoundedRectangle(cornerRadius: 20, style: .continuous))
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
    }

    private func addressRow(_ address: String) -> some View {
        Text(address)
            .font(.caption.monospaced())
            .frame(maxWidth: .infinity, alignment: .leading)
            .padding(12)
            .background(Color.primary.opacity(0.04), in: RoundedRectangle(cornerRadius: 12, style: .continuous))
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
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
