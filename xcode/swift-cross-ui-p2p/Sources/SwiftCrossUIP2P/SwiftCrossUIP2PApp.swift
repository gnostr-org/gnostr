#if canImport(CryptoKit)
    import CryptoKit
#elseif canImport(Crypto)
    import Crypto
#else
    #error("Neither CryptoKit nor Crypto is available on this platform.")
#endif
import DefaultBackend
import Foundation
import LibP2P
import LibP2PDCUtR
import LibP2PKadDHT
import LibP2PNoise
import LibP2PYAMUX
import SwiftCrossUI

#if os(macOS) || os(iOS)
    import LibP2PMDNS
#endif

#if os(iOS) && !targetEnvironment(macCatalyst)
    import GnostrGit
    import XGit
#endif

#if os(iOS)
    import UIKit
#endif

struct GitLineSnapshot: Identifiable, Hashable, Sendable {
    var id: String { "\(kind)|\(text)" }
    let kind: String
    let text: String
}

struct GitHunkSnapshot: Identifiable, Hashable, Sendable {
    var id: String { header }
    let header: String
    let lines: [GitLineSnapshot]
}

struct GitFileSnapshot: Identifiable, Hashable, Sendable {
    var id: String { path }
    let path: String
    let hunks: [GitHunkSnapshot]
}

struct GitCommitSnapshot: Identifiable, Hashable, Sendable {
    var id: String { oid }
    let oid: String
    let shortOID: String
    let summary: String
    let author: String
    let time: String
    let refs: [String]
}

struct GitRemoteSnapshot: Identifiable, Hashable, Sendable {
    var id: String { name }
    let name: String
    let url: String
}

struct GitRepoSnapshot: Hashable, Sendable {
    let path: String
    let exists: Bool
    let currentBranch: String
    let repositoryState: String
    let remotes: [GitRemoteSnapshot]
    let commits: [GitCommitSnapshot]
    let stagedChanges: [GitFileSnapshot]
    let unstagedChanges: [GitFileSnapshot]
    let selectedCommit: GitCommitSnapshot?
    let selectedCommitDiff: [GitFileSnapshot]
    let refreshedAt: String
    let error: String?
}

@main
struct SwiftCrossUIP2PApp: App {
    @State var model = P2PDemoViewModel()

    var body: some Scene {
        WindowGroup("SwiftCrossUI P2P") {
            ContentView()
                .environment(model)
        }
        .defaultSize(width: 980, height: 760)
    }
}

@MainActor
@ObservableObject
final class P2PDemoViewModel {
    enum State: String {
        case stopped
        case starting
        case running
        case stopping
    }

    enum Demo: String, CaseIterable, Hashable {
        case overview = "Overview"
        case discovery = "Discovery"
        case catalog = "Module Catalog"
        case protocolLab = "Protocol Lab"
        case repoTopology = "Repo Topology"
        case gitRepo = "Git Repo Viewer"
    }

    struct PeerSummary: Identifiable, Hashable {
        var id: String { peerID }
        let peerID: String
        let addresses: [String]
    }

    var selectedDemo: Demo? = .overview
    var state: State = .stopped
    var listenAddresses: [String] = []
    var discoveredPeers: [PeerSummary] = []
    var activityLog: [String] = []
    var lastError: String?
    var draftMessage = "Hello from SwiftCrossUI P2P"
    var gitRepositoryPath = ""
    var gitIsLoading = false
    var gitHasRepository = false
    var gitCurrentBranch = ""
    var gitRepositoryState = ""
    var gitRemotes: [GitRemoteSnapshot] = []
    var gitCommits: [GitCommitSnapshot] = []
    var gitStagedChanges: [GitFileSnapshot] = []
    var gitUnstagedChanges: [GitFileSnapshot] = []
    var gitSelectedCommitOID: String?
    var gitSelectedCommitDiff: [GitFileSnapshot] = []
    var gitLastRefreshed = ""
    var gitLastError: String?

    private var app: Application?
    private var runTask: Task<Void, Never>?
    private var gitRefreshTask: Task<Void, Never>?

    let peerID: PeerID
    let runtimeProfile: String
    let listenPort: Int

    init() {
        runtimeProfile = Self.runtimeProfile.rawValue
        listenPort = Self.listenPort
        peerID = Self.makePeerID(for: Self.runtimeProfile)
        gitRepositoryPath = Self.defaultRepositoryPath()
        refreshGitRepository()
    }

    var peerIDString: String {
        peerID.b58String
    }

    var isRunning: Bool {
        state == .running
    }

    func start() {
        guard runTask == nil else { return }

        lastError = nil
        state = .starting
        log("Starting libp2p node")

        let app = Self.makeApplication(peerID: peerID)
        self.app = app

        app.discovery.onPeerDiscovered(app) { [weak self] peer in
            let peerID = peer.peer.b58String
            let addresses = peer.addresses.map(\.description)
            DispatchQueue.main.async { [weak self] in
                self?.recordDiscoveredPeer(
                    peerID: peerID,
                    addresses: addresses
                )
            }
        }

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
            }
        }

        runTask = Task { [app] in
            do {
                try await app.execute()
            } catch {
                await MainActor.run { [weak self] in
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run { [weak self] in
                self?.state = .stopped
                self?.runTask = nil
                self?.app = nil
                self?.listenAddresses = []
                self?.log("Node stopped")
            }
        }
    }

    func stop() {
        guard let app else { return }

        state = .stopping
        log("Stopping libp2p node")
        self.app = nil
        self.runTask = nil

        Task { [app] in
            do {
                try await app.asyncShutdown()
            } catch {
                await MainActor.run { [weak self] in
                    self?.lastError = error.localizedDescription
                    self?.log("Error: \(error.localizedDescription)")
                }
            }

            await MainActor.run { [weak self] in
                self?.listenAddresses = []
                self?.state = .stopped
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

    func sendLocalPing() {
        log("Ping: \(draftMessage)")
    }

    func useWorkspaceRepositoryRoot() {
        gitRepositoryPath = Self.defaultRepositoryPath()
        refreshGitRepository()
    }

    func refreshGitRepository() {
        gitRefreshTask?.cancel()
        let path = gitRepositoryPath
        let selectedCommitOID = gitSelectedCommitOID
        gitIsLoading = true
        gitLastError = nil
        #if os(iOS) && !targetEnvironment(macCatalyst)
            gitRefreshTask = Task.detached(priority: .background) { [path, selectedCommitOID] in
                let snapshot = GitRepoSnapshotLoader.loadGitSnapshot(
                    path: path,
                    selectedCommitOID: selectedCommitOID
                )
                await MainActor.run { [weak self] in
                    self?.applyGitSnapshot(snapshot)
                }
            }
        #else
            applyGitSnapshot(
                GitRepoSnapshot(
                    path: path,
                    exists: false,
                    currentBranch: "",
                    repositoryState: "Unavailable",
                    remotes: [],
                    commits: [],
                    stagedChanges: [],
                    unstagedChanges: [],
                    selectedCommit: nil,
                    selectedCommitDiff: [],
                    refreshedAt: Self.timestampFormatter.string(from: Date()),
                    error: "Git repo viewer is available on iOS and Mac Catalyst."
                )
            )
        #endif
    }

    func selectGitCommit(_ oid: String) {
        gitSelectedCommitOID = oid
        refreshGitRepository()
    }

    private func recordDiscoveredPeer(peerID: String, addresses: [String]) {
        let peer = PeerSummary(peerID: peerID, addresses: addresses)
        if !discoveredPeers.contains(peer) {
            discoveredPeers.append(peer)
            log("Discovered peer \(peerID)")
        }
    }

    private func log(_ message: String) {
        let formatter = Self.timestampFormatter
        activityLog.append("[\(formatter.string(from: Date()))] \(message)")
    }

    private func applyGitSnapshot(_ snapshot: GitRepoSnapshot) {
        gitIsLoading = false
        gitHasRepository = snapshot.exists
        gitCurrentBranch = snapshot.currentBranch
        gitRepositoryState = snapshot.repositoryState
        gitRemotes = snapshot.remotes
        gitCommits = snapshot.commits
        gitStagedChanges = snapshot.stagedChanges
        gitUnstagedChanges = snapshot.unstagedChanges
        gitLastRefreshed = snapshot.refreshedAt
        gitLastError = snapshot.error

        if gitSelectedCommitOID == nil {
            gitSelectedCommitOID = snapshot.selectedCommit?.oid
        }

        if let selectedCommit = gitSelectedCommitOID {
            if let commit = snapshot.commits.first(where: { $0.oid == selectedCommit }) {
                gitSelectedCommitOID = commit.oid
            } else {
                gitSelectedCommitOID = snapshot.selectedCommit?.oid
            }
        }

        gitSelectedCommitDiff = snapshot.selectedCommitDiff
        if !snapshot.exists {
            gitCommits = []
            gitStagedChanges = []
            gitUnstagedChanges = []
            gitSelectedCommitDiff = []
        }
    }

    private static func defaultRepositoryPath() -> String {
        let fileRoot = URL(fileURLWithPath: #filePath)
            .deletingLastPathComponent()
            .deletingLastPathComponent()
        let cwd = URL(fileURLWithPath: FileManager.default.currentDirectoryPath)

        for start in [fileRoot, cwd] {
            var candidate = start
            while true {
                if FileManager.default.fileExists(atPath: candidate.appendingPathComponent(".git").path) {
                    return candidate.path
                }

                let parent = candidate.deletingLastPathComponent()
                if parent.path == candidate.path {
                    break
                }
                candidate = parent
            }
        }

        return cwd.path
    }

    private static func makeApplication(peerID: PeerID) -> Application {
        let app = Application(.testing, peerID: peerID)
        app.logger.logLevel = .notice
        app.security.use(.noise)
        app.muxers.use(.yamux)
        app.dcutr.use(.dcutr)
        #if os(macOS) || os(iOS)
            // TODO: Linux mDNS needs a Swift-facing Avahi backend or conditional wrapper; dnssd is Apple-only.
            // mDNS is local LAN discovery only; keep it on Apple platforms where dnssd exists.
            app.discovery.use(.mdns)
        #endif
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
                return 12000
            case .iPhone:
                return 12001
            case .iPad:
                return 12002
            case .macCatalyst:
                return 12003
            case .madeForiPad:
                return 12004
        }
    }

    private static func makePeerID(for profile: RuntimeProfile) -> PeerID {
        let seed = Data(SHA256.hash(data: Data("SwiftCrossUIP2P.peerid.\(profile.rawValue)".utf8)))
        let privateKey = try! Curve25519.Signing.PrivateKey(rawRepresentation: seed)
        return try! PeerID(marshaledPrivateKey: privateKey.marshal())
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()

    private enum RuntimeProfile: String {
        case macOS
        case macCatalyst
        case iPad
        case iPhone
        case madeForiPad
    }
}

enum GitRepoSnapshotLoader {
    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()

    private static let commitDateFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "yyyy-MM-dd HH:mm"
        return formatter
    }()

    #if os(iOS) && !targetEnvironment(macCatalyst)
    static func loadGitSnapshot(path: String, selectedCommitOID: String?) -> GitRepoSnapshot {
        let url = URL(fileURLWithPath: path, isDirectory: true)
        let credentialsURL = FileManager.default.temporaryDirectory
            .appendingPathComponent("swift-cross-ui-p2p-git-credentials.json")
        let credentialManager = CredentialsManager(credentialsFileUrl: credentialsURL)
        let repository = GitRepository(url, credentialManager)
        repository.open()

        guard repository.hasRepo else {
            return GitRepoSnapshot(
                path: path,
                exists: false,
                currentBranch: "",
                repositoryState: "No repository",
                remotes: [],
                commits: [],
                stagedChanges: [],
                unstagedChanges: [],
                selectedCommit: nil,
                selectedCommitDiff: [],
                refreshedAt: timestampFormatter.string(from: Date()),
                error: "No git repository found at \(path)"
            )
        }

        repository.updateStatus()
        repository.updateCommitGraph()

        let commits = repository.commitGraph.commits.prefix(30).map { commit in
            makeCommitSnapshot(commit)
        }
        let selectedCommit = commits.first(where: { $0.oid == selectedCommitOID }) ?? commits.first
        let staged = makeDiffSnapshots(repository.status.stagedChanges)
        let unstaged = makeDiffSnapshots(repository.status.unstagedChanges)
        let remotes = repository.getRemotes().map {
            GitRemoteSnapshot(name: $0.name, url: $0.url)
        }

        var selectedCommitDiff: [GitFileSnapshot] = []
        if let selectedCommit,
           let sourceCommit = repository.commitGraph.commits.first(where: {
               $0.oid.description() == selectedCommit.oid
           }),
           let parent = sourceCommit.parents.first {
            let diffReceiver = GitDiff()
            repository.diff(parent, sourceCommit, diffReceiver)
            selectedCommitDiff = makeDiffSnapshots(diffReceiver.changes)
        }

        return GitRepoSnapshot(
            path: path,
            exists: true,
            currentBranch: repository.status.currentBranch,
            repositoryState: String(describing: repository.status.state),
            remotes: remotes,
            commits: commits,
            stagedChanges: staged,
            unstagedChanges: unstaged,
            selectedCommit: selectedCommit,
            selectedCommitDiff: selectedCommitDiff,
            refreshedAt: timestampFormatter.string(from: Date()),
            error: nil
        )
    }

    private static func makeCommitSnapshot(_ commit: GitCommit) -> GitCommitSnapshot {
        GitCommitSnapshot(
            oid: commit.oid.description(),
            shortOID: commit.oid.shortDescription,
            summary: commit.summary,
            author: "\(commit.author.name) <\(commit.author.email)>",
            time: commitDateFormatter.string(from: commit.time),
            refs: commit.refs.map(\.shorthand)
        )
    }

    private static func makeDiffSnapshots(_ diff: Diff) -> [GitFileSnapshot] {
        diff.deltas.map { delta in
            GitFileSnapshot(
                path: delta.path,
                hunks: delta.hunks.map { hunk in
                    GitHunkSnapshot(
                        header: hunk.header,
                        lines: hunk.lines.map {
                            GitLineSnapshot(kind: $0.kind, text: $0.textTrimmed)
                        }
                    )
                }
            )
        }
    }
    #endif
}

struct ContentView: View {
    @SwiftCrossUI.Environment(P2PDemoViewModel.self) var model

    var body: some View {
        NavigationSplitView(
            sidebar: {
                VStack(alignment: .leading, spacing: 0) {
                    sidebarHeader

                    List(P2PDemoViewModel.Demo.allCases, id: { $0 }, selection: model.$selectedDemo) { demo in
                        Text(demo.rawValue)
                    }
                }
            },
            detail: {
                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        header
                        controls
                        selectedDemo
                        activityPanel
                    }
                    .padding(16)
                }
            }
        )
    }

    var sidebarHeader: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("Demos")
                .font(.headline)
            Text("Choose a libp2p scene")
                .font(.caption)
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(.horizontal, 16)
        .padding(.vertical, 12)
    }

    var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("SwiftCrossUI P2P")
                .font(.largeTitle)
                .fontWeight(.bold)
            Text("Cross-platform kitchen sink for libp2p demos")
            Text("Peer ID: \(model.peerIDString)")
                .font(.caption.monospaced())
                .textSelectionEnabled()
        }
    }

    var controls: some View {
        HStack {
            Button("Start") { model.start() }
            Button("Stop") { model.stop() }
                .disabled(!model.isRunning)
            Button("Restart") { model.restart() }
            Spacer()
            Text("State: \(model.state.rawValue.capitalized)")
        }
    }

    @ViewBuilder
    var selectedDemo: some View {
        switch model.selectedDemo ?? P2PDemoViewModel.Demo.overview {
            case .overview:
                overviewPanel
            case .discovery:
                discoveryPanel
            case .catalog:
                catalogPanel
            case .protocolLab:
                protocolLabPanel
            case .repoTopology:
                repoTopologyPanel
            case .gitRepo:
                gitRepoPanel
        }
    }

    var overviewPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Overview").font(.headline)
            Text("Runtime profile: \(model.runtimeProfile)")
            Text("Listen port: \(model.listenPort)")
            Text("Listening addresses:")
            ForEach(model.listenAddresses, id: \.self) { address in
                Text(address).font(.caption.monospaced())
            }
            if model.listenAddresses.isEmpty {
                Text("Start the node to populate listen addresses.")
            }
        }
        .padding(12)
    }

    var discoveryPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Discovery").font(.headline)
            Text("Peers discovered via mDNS and DHT.")
            ForEach(model.discoveredPeers, id: \.id) { peer in
                VStack(alignment: .leading, spacing: 4) {
                    Text(peer.peerID).font(.caption.monospaced())
                    ForEach(peer.addresses, id: \.self) { address in
                        Text(address).font(.caption)
                    }
                }
                .padding(.vertical, 4)
            }
            if model.discoveredPeers.isEmpty {
                Text("No peers discovered yet.")
            }
        }
        .padding(12)
    }

    var catalogPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Module Catalog").font(.headline)
            Text("This scaffold is wired for the same stack used by the samples.")
            moduleRow(name: "LibP2P", detail: "Core runtime and application lifecycle")
            moduleRow(name: "Noise", detail: "Secure transport handshake")
            moduleRow(name: "Yamux", detail: "Stream multiplexing")
            moduleRow(name: "mDNS", detail: "Local peer discovery")
            moduleRow(name: "KadDHT", detail: "Distributed peer discovery")
            moduleRow(name: "DCUtR", detail: "Hole punching support")
            HStack {
                TextField(
                    "Message",
                    text: Binding(
                        get: { model.draftMessage },
                        set: { model.draftMessage = $0 }
                    )
                )
                Button("Queue ping") { model.sendLocalPing() }
            }
        }
        .padding(12)
    }

    var protocolLabPanel: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Protocol Lab").font(.headline)
            Text("A living legend of the stack in use right now.")
            protocolChip(title: "Noise", detail: "secure transport")
            protocolChip(title: "Yamux", detail: "multiplexed streams")
            protocolChip(title: "mDNS", detail: "LAN peer discovery")
            protocolChip(title: "KadDHT", detail: "distributed lookup")
            protocolChip(title: "DCUtR", detail: "relay-assisted hole punching")
            protocolChip(title: "Git", detail: "repo browsing + history rendering")
            Text("Peer count: \(model.discoveredPeers.count)")
            Text("Listen addresses: \(model.listenAddresses.count)")
            if let firstAddress = model.listenAddresses.first {
                Text(firstAddress).font(.caption.monospaced())
            }
        }
        .padding(12)
    }

    var repoTopologyPanel: some View {
        VStack(alignment: .leading, spacing: 10) {
            Text("Repo Topology").font(.headline)
            Text("A compact story of what this repo looks like.")
            Text("Commits loaded: \(model.gitCommits.count)")
            Text("Remotes configured: \(model.gitRemotes.count)")
            Text("Staged files: \(model.gitStagedChanges.count)")
            Text("Unstaged files: \(model.gitUnstagedChanges.count)")
            if let head = model.gitCommits.first {
                VStack(alignment: .leading, spacing: 4) {
                    Text("HEAD").font(.caption)
                    Text("\(head.shortOID) — \(head.summary)")
                    Text(head.author).font(.caption)
                }
            }
            if !model.gitCommits.isEmpty {
                let prefix = model.gitCommits.prefix(8)
                VStack(alignment: .leading, spacing: 4) {
                    Text("Recent commits").font(.headline)
                    ForEach(Array(prefix.enumerated()), id: \.offset) { index, commit in
                        HStack(alignment: .top, spacing: 8) {
                            Text("#\(index + 1)").font(.caption.monospaced())
                            VStack(alignment: .leading, spacing: 2) {
                                Text(commit.shortOID + " " + commit.summary)
                                Text(commit.refs.isEmpty ? "No refs" : commit.refs.joined(separator: ", "))
                                    .font(.caption)
                            }
                        }
                    }
                }
            }
        }
        .padding(12)
    }

    func protocolChip(title: String, detail: String) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Text(title)
                .font(.body)
                .fontWeight(.bold)
                .frame(width: 90, alignment: .leading)
            Text(detail)
        }
    }

    var gitRepoPanel: some View {
        let selectedCommit = model.gitCommits.first(where: { $0.oid == model.gitSelectedCommitOID })

        return VStack(alignment: .leading, spacing: 10) {
            Text("Git Repo Viewer").font(.headline)
            TextField(
                "Repository path",
                text: model.$gitRepositoryPath
            )
            HStack {
                Button("Use workspace root") {
                    model.useWorkspaceRepositoryRoot()
                }
                Button("Refresh") {
                    model.refreshGitRepository()
                }
                Spacer()
                Text(model.gitIsLoading ? "Loading..." : "Ready")
            }
            HStack {
                Text("Repository: \(model.gitHasRepository ? "found" : "missing")")
                Spacer()
                Text("Branch: \(model.gitCurrentBranch.isEmpty ? "unknown" : model.gitCurrentBranch)")
                Spacer()
                Text("State: \(model.gitRepositoryState.isEmpty ? "unknown" : model.gitRepositoryState)")
            }
            if let lastError = model.gitLastError {
                Text(lastError).foregroundColor(.red)
            }
            if !model.gitRemotes.isEmpty {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Remotes").font(.headline)
                    ForEach(model.gitRemotes) { remote in
                        Text("\(remote.name): \(remote.url)")
                            .font(.caption)
                    }
                }
            }
            VStack(alignment: .leading, spacing: 4) {
                Text("Commits").font(.headline)
                ForEach(model.gitCommits) { commit in
                    VStack(alignment: .leading, spacing: 2) {
                        Button(commit.shortOID + "  " + commit.summary) {
                            model.selectGitCommit(commit.oid)
                        }
                        Text("\(commit.author) • \(commit.time)")
                            .font(.caption)
                        if !commit.refs.isEmpty {
                            Text(commit.refs.joined(separator: ", "))
                                .font(.caption)
                        }
                    }
                }
            }
            if let selectedCommit {
                VStack(alignment: .leading, spacing: 4) {
                    Text("Selected commit").font(.headline)
                    Text(selectedCommit.oid).font(.caption.monospaced())
                    Text(selectedCommit.summary)
                    if !selectedCommit.refs.isEmpty {
                        Text("Refs: \(selectedCommit.refs.joined(separator: ", "))")
                            .font(.caption)
                    }
                }
            }
            diffSection(title: "Staged changes", files: model.gitStagedChanges)
            diffSection(title: "Unstaged changes", files: model.gitUnstagedChanges)
            diffSection(title: "Selected commit diff", files: model.gitSelectedCommitDiff)
            Text("Last refreshed: \(model.gitLastRefreshed)")
                .font(.caption)
        }
        .padding(12)
    }

    func diffSection(title: String, files: [GitFileSnapshot]) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title).font(.headline)
            if files.isEmpty {
                Text("No changes")
                    .font(.caption)
            }
            ForEach(files) { file in
                VStack(alignment: .leading, spacing: 4) {
                    Text(file.path)
                        .font(.body)
                        .fontWeight(.bold)
                    ForEach(file.hunks) { hunk in
                        VStack(alignment: .leading, spacing: 2) {
                            Text(hunk.header).font(.caption.monospaced())
                            ForEach(hunk.lines) { line in
                                Text("\(line.kind) \(line.text)")
                                    .font(.caption.monospaced())
                            }
                        }
                        .padding(.leading, 8)
                    }
                }
                .padding(.vertical, 4)
            }
        }
        .padding(12)
    }

    func moduleRow(name: String, detail: String) -> some View {
        HStack(alignment: .top, spacing: 10) {
            Text(name)
                .font(.body)
                .fontWeight(.bold)
                .frame(width: 90, alignment: .leading)
            Text(detail)
        }
    }

    var activityPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            HStack {
                Text("Activity").font(.headline)
                Spacer()
                Button("Clear") {
                    model.activityLog.removeAll()
                }
            }
            ForEach(model.activityLog, id: \.self) { line in
                Text(line)
                    .font(.caption.monospaced())
            }
            if let lastError = model.lastError {
                Text("Last error: \(lastError)")
                    .foregroundColor(.red)
            }
        }
        .padding(12)
    }
}
