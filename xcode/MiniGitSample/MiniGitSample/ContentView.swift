//
//  ContentView.swift
//  MiniGit Sample App
//
//  Created by Lightech on 10/24/2048.
//

import CryptoKit
import Foundation
import LibP2P
import LibP2PPubSub
import LibP2PNoise
import LibP2PYAMUX
import LibP2PDCUtR
import LibP2PMDNS
import LibP2PKadDHT
import SwiftUI
import GnostrGit
#if os(iOS)
import UIKit
#endif

let documentURL = try! FileManager.default.url(for: .documentDirectory, in: .userDomainMask, appropriateFor: nil, create: true)

let remoteRepoLocation = "https://github.com/randymcmillan/ChatAppExample-iOS.git"

func repoLocation(for remoteURL: String) -> URL {
    let repoFolderName = URL(string: remoteURL)?
        .deletingPathExtension()
        .lastPathComponent ?? "MiniGit-SampleApp"
    return documentURL.appendingPathComponent(repoFolderName)
}

let localRepoLocation = repoLocation(for: remoteRepoLocation)

// Do not do this in a real application, put the credentials somewhere safe
// And possibly encrypt them or keychain them by subclassing CredentialsManager
let credentialManager = CredentialsManager(credentialsFileUrl: documentURL.appendingPathComponent("gitcredentials"))

// For push/fetch to work, you might need to add the credential
var credentialAdded = false

func addCredential() {
    do {
        // TODO Change the info here
        try credentialManager.addOrUpdate(nil, Credential(id: "MyGithub", kind: .password, targetURL: "https://github.com/YOUR_USERNAME/", userName: "YOUR_USERNAME", password: "YOUR_ACCESS_TOKEN"))
        credentialAdded = true
        print("Credential added.")
    } catch let error {
        print("Fail to add credential:", error)
    }
}
let repository = GitRepository(localRepoLocation, credentialManager)

struct RepoAnnouncement: Codable, Identifiable, Hashable {
    var id: String { "\(senderPeerID)|\(cloneURL)" }

    let senderPeerID: String
    let cloneURL: String
    let repositoryName: String
    let listenAddresses: [String]
    let timestamp: TimeInterval
}

struct PeerSummary: Identifiable, Hashable {
    let peerID: String
    let addresses: [String]
    let protocols: [String]

    var id: String { peerID }
}

struct RepoSnapshotRequest: Codable, Sendable {
    let repositoryName: String
}

struct RepoSnapshotFile: Codable, Identifiable, Hashable, Sendable {
    var id: String { path }

    let path: String
    let contents: Data
}

struct RepoSnapshot: Codable, Hashable, Sendable {
    let repositoryName: String
    let createdAt: TimeInterval
    let files: [RepoSnapshotFile]
}

private struct PeerIDReference: Codable {
    let id: String
}

@MainActor
final class P2PService: ObservableObject {
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
    @Published private(set) var repoAnnouncements: [RepoAnnouncement] = []
    @Published private(set) var peers: [PeerSummary] = []

    private var app: Application?
    private var runTask: Task<Void, Never>?
    private var peerRefreshTask: Task<Void, Never>?
    private var announcementSubscription: PubSub.SubscriptionHandler?

    let peerID: PeerID

    private static let repoTopic = "mini-git/repo-announcements"
    private static let gossipsubProtocol = SemVerProtocol("/meshsub/1.0.0")!
    private static let repoSnapshotProtocol = "/mini-git/repo-snapshot/1.0.0"
    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()

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

    var currentRepositoryName: String {
        localRepoLocation.lastPathComponent
    }

    func clearActivityLog() {
        activityLog.removeAll()
    }

    func broadcastCurrentRepo() {
        guard let app else { return }

        let announcement = RepoAnnouncement(
            senderPeerID: peerIDString,
            cloneURL: "p2p://\(peerIDString)\(Self.repoSnapshotProtocol)",
            repositoryName: currentRepositoryName,
            listenAddresses: listenAddresses,
            timestamp: Date().timeIntervalSince1970
        )

        do {
            let payload = try JSONEncoder().encode(announcement)
            let _ = app.pubsub.publish(payload.byteArray, toTopic: Self.repoTopic)
            log("Broadcast repo: \(announcement.repositoryName) -> \(announcement.cloneURL)")
        } catch {
            lastError = error.localizedDescription
            log("Error broadcasting repo: \(error.localizedDescription)")
        }
    }

    func clone(announcement: RepoAnnouncement) {
        log("Cloning repo from peer \(announcement.senderPeerID)")
        Task {
            await self.cloneRepo(from: announcement)
        }
    }

    func refreshPeers() {
        guard let app else { return }
        Task.detached(priority: .background) { [weak self] in
            guard let self else { return }
            await self.refreshPeers(using: app)
        }
    }

    private func startPeerRefreshLoop(with app: Application) {
        self.peerRefreshTask?.cancel()
        self.peerRefreshTask = Task.detached(priority: .background) { [weak self] in
            guard let self else { return }
            while !Task.isCancelled {
                await self.refreshPeers(using: app)
                try? await Task.sleep(for: .seconds(2))
            }
        }
    }

    func start() {
        guard runTask == nil else { return }

        lastError = nil
        state = .starting
        log("Starting node")

        let app = Self.makeApplication(peerID: peerID)
        self.app = app
        self.configureRepoSnapshotRoute(app)
        self.configurePubSub(app)
        self.startPeerRefreshLoop(with: app)

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
                if !addresses.isEmpty {
                    self.log("Listening on: \(addresses.joined(separator: ", "))")
                }
                if self.state == .starting {
                    self.state = .running
                    self.log("Node is running")
                }
                self.broadcastCurrentRepo()
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
                self?.state = .stopped
                self?.runTask = nil
                self?.app = nil
                self?.log("Node stopped")
            }
        }
    }

    func stop() {
        guard let app else { return }

        state = .stopping
        log("Stopping node")
        self.app = nil
        self.runTask = nil
        self.peerRefreshTask?.cancel()
        self.peerRefreshTask = nil

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
                self?.peers = []
                self?.state = .stopped
                self?.runTask = nil
                self?.app = nil
                self?.log("Node stopped")
            }
        }
    }

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .notice
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.pubsub.use(.gossipsub(emitSelf: true))
        app.dcutr.use(.dcutr)
        app.discovery.use(.mdns)
        app.discovery.use(.kadDHT)
        app.listen(.tcp(host: "0.0.0.0", port: listenPort))
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

    private static func makePeerID(for profile: RuntimeProfile) -> PeerID {
        let seed = Data(SHA256.hash(data: Data("MiniGitSample.peerid.\(profile.rawValue)".utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        return try! PeerID(marshaledPrivateKey: privateKey.marshal())
    }

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        activityLog.append("[\(formatter.string(from: Date()))] \(message)")
    }

    private func configurePubSub(_ app: Application) {
        let subscription = try! app.pubsub.gossipsub.subscribe(
            .init(
                topic: Self.repoTopic,
                signaturePolicy: .strictSign,
                validator: .acceptAll,
                messageIDFunc: .concatFromAndSequenceFields
            )
        )

        let eventLoop = app.eventLoopGroup.next()
        subscription.on = { [weak self] event in
            switch event {
            case .newPeer(let peer):
                Task { @MainActor in
                    self?.log("Repo topic peer: \(peer.b58String)")
                }
            case .data(let message):
                guard let announcement = try? JSONDecoder().decode(RepoAnnouncement.self, from: message.data) else {
                    Task { @MainActor in
                        self?.log("Ignored invalid repo announcement")
                    }
                    return eventLoop.makeSucceededVoidFuture()
                }

                Task { @MainActor in
                    guard let self else { return }
                    guard announcement.senderPeerID != self.peerIDString else {
                        self.log("Saw own repo broadcast")
                        return
                    }
                    self.repoAnnouncements.removeAll { $0.id == announcement.id }
                    self.repoAnnouncements.insert(announcement, at: 0)
                    self.repoAnnouncements = Array(self.repoAnnouncements.prefix(10))
                    self.log("Discovered repo: \(announcement.repositoryName) from \(announcement.senderPeerID)")
                }
            case .error(let error):
                Task { @MainActor in
                    self?.lastError = error.localizedDescription
                    self?.log("Repo topic error: \(error.localizedDescription)")
                }
            }
            return eventLoop.makeSucceededVoidFuture()
        }
        self.announcementSubscription = subscription
        log("Subscribed to repo broadcasts")
    }

    private func configureRepoSnapshotRoute(_ app: Application) {
        app.group("mini-git") { routes in
            routes.on("repo-snapshot", "1.0.0") { [weak self] req -> Response<Data> in
                guard let self else { return .close }

                switch req.event {
                case .ready:
                    self.log("Repo snapshot request ready")
                    return .stayOpen

                case .data(let payload):
                    do {
                        let request = try JSONDecoder().decode(
                            RepoSnapshotRequest.self,
                            from: Data(payload.readableBytesView)
                        )
                        let snapshot = try self.makeRepoSnapshot()
                        self.log("Serving repo snapshot for \(request.repositoryName) with \(snapshot.files.count) files")
                        return .respondThenClose(try JSONEncoder().encode(snapshot))
                    } catch {
                        req.logger.error("Repo snapshot failed: \(error.localizedDescription)")
                        self.log("Repo snapshot failed: \(error.localizedDescription)")
                        return .close
                    }

                case .closed:
                    return .close

                case .error(let error):
                    req.logger.error("Repo snapshot stream error: \(error.localizedDescription)")
                    self.log("Repo snapshot stream error: \(error.localizedDescription)")
                    return .close
                }
            }
        }
    }

    private func refreshPeers(using app: Application) async {
        do {
            let peerIDs = try await app.peers.getPeers(supportingProtocol: Self.gossipsubProtocol).get()
            var rows: [PeerSummary] = []
            for peerIDString in peerIDs.sorted() {
                let peerInfo = try await app.peers.getPeerInfo(byID: peerIDString).get()
                let peerID = peerInfo.peer
                let addresses = peerInfo.addresses
                let protocols = try await app.peers.getProtocols(forPeer: peerID).get()
                rows.append(
                    PeerSummary(
                        peerID: peerID.b58String,
                        addresses: addresses.map(\.description),
                        protocols: protocols.map(\.stringValue)
                    )
                )
            }

            await MainActor.run {
                self.peers = rows
            }
        } catch {
            await MainActor.run {
                self.log("Peer refresh failed: \(error.localizedDescription)")
            }
        }
    }

    private func cloneRepo(from announcement: RepoAnnouncement) async {
            guard let app else { return }

            do {
                let peerID = try self.peerID(from: announcement.senderPeerID)
                let peerInfo = PeerInfo(
                    peer: peerID,
                    addresses: announcement.listenAddresses.compactMap { try? Multiaddr($0) }
                )
                try await app.peers.add(peerInfo: peerInfo)

                let request = try JSONEncoder().encode(RepoSnapshotRequest(repositoryName: announcement.repositoryName))
                let response = try await app.newRequest(
                    to: peerID,
                    forProtocol: Self.repoSnapshotProtocol,
                    withRequest: request
                ).get()
                let snapshot = try JSONDecoder().decode(RepoSnapshot.self, from: response)
                try self.restoreRepo(snapshot, to: localRepoLocation)

                repository.open()
                repository.updateCommitGraph()
                broadcastCurrentRepo()
                refreshPeers()
                log("Cloned repo snapshot from \(announcement.senderPeerID)")
            } catch {
                lastError = error.localizedDescription
                log("Clone failed: \(error.localizedDescription)")
            }
        }

    private func makeRepoSnapshot() throws -> RepoSnapshot {
            guard FileManager.default.fileExists(atPath: localRepoLocation.path) else {
                throw NSError(domain: "MiniGitSample", code: 404, userInfo: [NSLocalizedDescriptionKey: "No local repository to serve"])
            }

            var files: [RepoSnapshotFile] = []
            let enumerator = FileManager.default.enumerator(
                at: localRepoLocation,
                includingPropertiesForKeys: [.isDirectoryKey],
                options: []
            )

            while let item = enumerator?.nextObject() as? URL {
                let values = try item.resourceValues(forKeys: [.isDirectoryKey])
                guard values.isDirectory != true else { continue }
                let data = try Data(contentsOf: item)
                let relativePath = item.path.replacingOccurrences(
                    of: localRepoLocation.path + "/",
                    with: ""
                )
                files.append(RepoSnapshotFile(path: relativePath, contents: data))
            }

            return RepoSnapshot(
                repositoryName: currentRepositoryName,
                createdAt: Date().timeIntervalSince1970,
                files: files
            )
        }

    private func restoreRepo(_ snapshot: RepoSnapshot, to destination: URL) throws {
            let fileManager = FileManager.default
            if fileManager.fileExists(atPath: destination.path) {
                try fileManager.removeItem(at: destination)
            }

            try fileManager.createDirectory(at: destination, withIntermediateDirectories: true)

            for file in snapshot.files {
                let fileURL = destination.appendingPathComponent(file.path)
                let parent = fileURL.deletingLastPathComponent()
                try fileManager.createDirectory(at: parent, withIntermediateDirectories: true)
                try file.contents.write(to: fileURL, options: .atomic)
            }
        }

    private func peerID(from string: String) throws -> PeerID {
            try PeerID(fromJSON: JSONEncoder().encode(PeerIDReference(id: string)))
        }
}

struct ContentView: View {

    @ObservedObject var repo = repository
    @ObservedObject var commitGraph = repository.commitGraph
    @ObservedObject var remoteProgress = repository.remoteProgress
    @StateObject private var p2p = P2PService()

    var body: some View {
        NavigationView {
            List {
                Section("Git") {
                    NavigationLink(destination: repositoryDetail) {
                        Label("Repository", systemImage: "folder")
                    }
                }

                Section("P2P") {
                    NavigationLink(destination: peersDetail) {
                        Label("Peers", systemImage: "person.2")
                    }
                    NavigationLink(destination: repoBroadcastsDetail) {
                        Label("Repo broadcasts", systemImage: "dot.radiowaves.left.and.right")
                    }
                    NavigationLink(destination: networkActivityDetail) {
                        Label("Network activity", systemImage: "waveform")
                    }
                    NavigationLink(destination: listeningAddressesDetail) {
                        Label("Listening addresses", systemImage: "network")
                    }
                }

                if !p2p.peers.isEmpty {
                    Section("Peers") {
                        ForEach(p2p.peers.prefix(4)) { peer in
                            Text(peer.peerID)
                                .font(.caption.monospaced())
                        }
                    }
                }
            }
            .listStyle(.sidebar)

            repositoryDetail
        }
        .navigationViewStyle(DoubleColumnNavigationViewStyle())
        .padding(5)
        .onAppear {
            if !credentialAdded {
                addCredential()
            }
            repo.open()
            if repo.exists() {
                repo.updateCommitGraph()
            }
            p2p.start()
        }
        .onDisappear {
            p2p.stop()
        }
    }

    private var repositoryDetail: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("MiniGitSample").font(.largeTitle.bold())
            Text("On Mac Catalyst, you should be able to find the cloned repo in `~/Documents/\(localRepoLocation.lastPathComponent)/`.").italic()

            Button("Clone remote Git repo") {
                repo.clone(remoteRepoLocation)
            }

            if remoteProgress.inProgress {
                ProgressView(remoteProgress.operation)
            }

            if repo.hasRepo {
                if !remoteProgress.inProgress {
                    HStack {
                        Button("Push to origin") {
                            let allRemotes = repo.getRemotes()
                            let remoteOrigin = allRemotes[0]
                            repo.push(remoteOrigin, false)
                        }

                        Button("Fetch from origin") {
                            let allRemotes = repo.getRemotes()
                            let remoteOrigin = allRemotes[0]
                            repo.fetch(remoteOrigin)
                        }

                        Button("Merge origin/master into current branch") {
                            repo.updateCommitGraph()
                            for c in repo.commitGraph.commits {
                                for ref in c.refs where ref.name == "refs/remotes/origin/master" {
                                    repo.merge([ref])
                                }
                            }
                        }
                    }
                }

                List(commitGraph.commits) { commit in
                    VStack(alignment: .leading) {
                        Text(commit.message).bold()
                        Text(commit.author.name)
                    }
                }
                .listStyle(.plain)
            } else {
                Text("No repository cloned yet.")
                    .foregroundStyle(.secondary)
            }
        }
    }

    private var repoBroadcastsDetail: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Repo broadcasts").font(.largeTitle.bold())
            Text("The current repo is announced over gossipsub so peers can discover a p2p clone source.")

            HStack {
                Text("Swift p2p network").font(.headline)
                Spacer()
                Button("Broadcast now") {
                    p2p.broadcastCurrentRepo()
                }
            }

            HStack {
                Text("State: \(p2p.stateLabel)")
                Spacer()
                Button("Start p2p node") {
                    p2p.start()
                }
                Button("Stop p2p node") {
                    p2p.stop()
                }
                .disabled(!p2p.isRunning)
            }

            if let lastError = p2p.lastError {
                Text("Last error: \(lastError)")
                    .foregroundStyle(.red)
            }

            if p2p.repoAnnouncements.isEmpty {
                Text("No repo broadcasts yet")
                    .foregroundStyle(.secondary)
            } else {
                List(p2p.repoAnnouncements) { announcement in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(announcement.repositoryName).bold()
                        Text(announcement.cloneURL)
                            .font(.caption.monospaced())
                            .textSelection(.enabled)
                        Text("From: \(announcement.senderPeerID)")
                            .font(.caption2)
                        Text(announcement.listenAddresses.joined(separator: ", "))
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                        Button("Clone from peer") {
                            p2p.clone(announcement: announcement)
                        }
                    }
                }
                .frame(minHeight: 240)
            }
        }
    }

    private var peersDetail: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Peers").font(.largeTitle.bold())
                Spacer()
                Button("Refresh") {
                    p2p.refreshPeers()
                }
            }

            Text("Mac and iPhone peers should appear here when they share the gossipsub protocol and see each other on the network.")

            if p2p.peers.isEmpty {
                Text("No peers discovered yet")
                    .foregroundStyle(.secondary)
            } else {
                List(p2p.peers) { peer in
                    VStack(alignment: .leading, spacing: 4) {
                        Text(peer.peerID).bold()
                            .font(.caption.monospaced())
                            .textSelection(.enabled)
                        Text(peer.protocols.joined(separator: ", "))
                            .font(.caption2)
                        Text(peer.addresses.joined(separator: "\n"))
                            .font(.caption2)
                            .foregroundStyle(.secondary)
                            .textSelection(.enabled)
                    }
                }
            }
        }
    }

    private var networkActivityDetail: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack {
                Text("Network activity").font(.largeTitle.bold())
                Spacer()
                Button("Clear") {
                    p2p.clearActivityLog()
                }
            }

            if p2p.activityLog.isEmpty {
                Text("No activity yet")
                    .foregroundStyle(.secondary)
            } else {
                List(p2p.activityLog.indices, id: \.self) { index in
                    Text(p2p.activityLog[index])
                        .font(.caption.monospaced())
                }
            }
        }
    }

    private var listeningAddressesDetail: some View {
        VStack(alignment: .leading, spacing: 12) {
            Text("Listening addresses").font(.largeTitle.bold())
            Text("Runtime profile: \(p2p.runtimeProfile)")
            Text("Peer ID: \(p2p.peerIDString)")
                .font(.caption.monospaced())
                .textSelection(.enabled)
            Text("Listen port: \(p2p.listenPort)")

            if p2p.listenAddresses.isEmpty {
                Text("No addresses yet")
                    .foregroundStyle(.secondary)
            } else {
                List(p2p.listenAddresses, id: \.self) { address in
                    Text(address)
                        .font(.caption.monospaced())
                }
            }
        }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
    }
}
