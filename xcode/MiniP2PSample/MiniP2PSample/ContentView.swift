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
    private var diagnosticsTask: Task<Void, Never>?
    private var gossipTask: Task<Void, Never>?
    private var pingTask: Task<Void, Never>?
    private var startedAt: Date?
    private var gossipSubscription: PubSub.SubscriptionHandler?
    private var discoveredPeersByID: [String: PeerInfo] = [:]

    let peerID: PeerID
    private let networkLogger = Logger(subsystem: "MiniP2PSample", category: "network")

    init() {
        peerID = Self.makePeerID(for: Self.runtimeProfile)
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
        guard runTask == nil else { return }

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
        logNetworkSnapshot(app: app, reason: "startup")
        startDiagnosticsLoop(for: app)
        startGossipHeartbeatLoop(for: app)
        startPingLoop(for: app)

        app.eventLoopGroup.next().scheduleTask(in: .milliseconds(100)) { [weak self, weak app] in
            guard let self, let app else { return }
            let addresses = self.listenAddressStrings(for: app)

            Task { @MainActor in
                self.listenAddresses = addresses
                if !addresses.isEmpty {
                    self.log("Listening on: \(addresses.joined(separator: ", "))")
                }
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
                self?.state = .stopped
                self?.runTask = nil
                self?.app = nil
                self?.discoveredPeersByID.removeAll()
                self?.startedAt = nil
                self?.log("Node stopped")
            }
        }
    }

    func stop() {
        guard let app else { return }

        state = .stopping
        log("Stopping node")
        logNetworkSnapshot(app: app, reason: "stopping")
        diagnosticsTask?.cancel()
        diagnosticsTask = nil
        gossipTask?.cancel()
        gossipTask = nil
        pingTask?.cancel()
        pingTask = nil
        gossipSubscription?.unsubscribe()
        gossipSubscription = nil
        self.app = nil
        self.runTask = nil

        Task.detached(priority: .background) { [weak self] in
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
                self?.app = nil
                self?.discoveredPeersByID.removeAll()
                self?.startedAt = nil
                self?.log("Node stopped")
            }
        }
    }

    func restart() {
        stop()
        DispatchQueue.main.asyncAfter(deadline: .now() + 0.2) {
            self.start()
        }
    }

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .trace
        app.discovery.use(.mdns)
        app.discovery.use(.kadDHT)
        app.pubsub.use(.gossipsub(emitSelf: true))
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
                        self?.log("Gossip peer for \(Self.gossipTopic): \(peer.b58String)")
                    }
                case .data(let message):
                    let text = String(data: message.data, encoding: .utf8) ?? "non-utf8 payload"
                    let author = message.from.asString(base: .base58btc)
                    Task { @MainActor in
                        self?.log("Gossip on \(Self.gossipTopic) from \(author): \(text)")
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

    private func pingKnownPeers(using app: Application) {
        guard let identify = app.identify as? Identify else {
            return
        }

        guard !discoveredPeersByID.isEmpty else {
            return
        }

        for peerInfo in discoveredPeersByID.values where peerInfo.peer.b58String != peerIDString {
            let peer = peerInfo.peer
            let pingFuture = identify.ping(peer: peer)
            pingFuture.whenSuccess { [weak self] duration in
                Task { @MainActor in
                    self?.log("Ping \(peer.b58String.prefix(12))… = \(duration)")
                }
            }
            pingFuture.whenFailure { [weak self] error in
                Task { @MainActor in
                    self?.log("Ping failed for \(peer.b58String.prefix(12))…: \(error.localizedDescription)")
                }
            }
        }
    }

    private func recordDiscoveredPeer(_ peer: PeerInfo) {
        let peerID = peer.peer.b58String
        guard discoveredPeersByID[peerID] == nil else { return }
        discoveredPeersByID[peerID] = peer
        let addresses = peer.addresses.map(\.description).joined(separator: ", ")
        log("Discovered peer \(peerID)")
        if !addresses.isEmpty {
            log("Peer addresses: \(addresses)")
        }
    }

    private func dialDiscoveredPeer(_ peer: PeerInfo) {
        guard peer.peer.b58String != peerIDString else { return }
        do {
            try app?.newStream(to: peer, forProtocol: "/ipfs/id/1.0.0")
            log("Dialed discovered peer \(peer.peer.b58String)")
        } catch {
            log("Failed to dial peer \(peer.peer.b58String): \(error.localizedDescription)")
        }
    }

    private func startDiagnosticsLoop(for app: Application) {
        diagnosticsTask?.cancel()
        diagnosticsTask = Task.detached(priority: .background) { [weak self] in
            while !Task.isCancelled {
                try? await Task.sleep(nanoseconds: 5_000_000_000)
                guard !Task.isCancelled else { break }
                await MainActor.run {
                    self?.logNetworkSnapshot(app: app, reason: "periodic")
                }
            }
        }
    }

    private func listenAddressStrings(for app: Application) -> [String] {
        app.listenAddresses.compactMap { address -> String? in
            guard let fullAddress = try? address.encapsulate(proto: .p2p, address: app.peerID.b58String) else {
                return nil
            }
            return fullAddress.description
        }
    }

    private func logNetworkSnapshot(app: Application, reason: String) {
        let addresses = listenAddressStrings(for: app)
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

    private static func makePeerID(for profile: RuntimeProfile) -> PeerID {
        let seed = Data(SHA256.hash(data: Data("MiniP2PSample.peerid.\(profile.rawValue)".utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        return try! PeerID(marshaledPrivateKey: privateKey.marshal())
    }

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        let formatted = "[\(formatter.string(from: Date()))] \(message)"
        activityLog.append(formatted)
        networkLogger.info("\(formatted, privacy: .public)")
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
            ScrollView {
                VStack(alignment: .leading, spacing: 16) {
                    headerCard
                    actionBar

                    if let lastError = service.lastError {
                        statusBanner(title: "Last error", message: lastError, tint: .red)
                    }

                    sectionCard(title: "Network activity", subtitle: "Recent network lifecycle events") {
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
                }
                .padding(16)
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
