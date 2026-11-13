import SwiftUI
import WebKit
import Kubo

struct ContentView: View {
    @State private var version: String = Kubo.version()
    @State private var repoPath: String = FileManager.default.temporaryDirectory.appendingPathComponent("kubo-xcode-demo").path
    @State private var command: String = ""
    @State private var output: String = ""
    @State private var busy: Bool = false
    @State private var node: Kubo.Node? = nil
    @State private var peerId: String = ""
    @State private var addrs: String = ""
    @State private var peerCount: Int = 0
    @State private var logTimer: Timer? = nil
    @State private var peerTimer: Timer? = nil
    @State private var webUrl: String = ""
    @State private var logExpanded: Bool = false
    @State private var selectedSection: SidebarSection = .status

    enum SidebarSection: String, CaseIterable, Identifiable {
        case status = "Status"
        case peers = "Peers"
        case files = "Files"
        case settings = "Settings"
        case diagnostics = "Diagnostics"
        case webui = "WebUI"
        var id: String { rawValue }
    }

    var body: some View {
        NavigationSplitView {
            List(selection: $selectedSection) {
                Section("Kubo") {
                    ForEach(SidebarSection.allCases) { section in
                        Label(section.rawValue, systemImage: icon(for: section)).tag(section)
                    }
                }
            }
            .listStyle(.sidebar)
            .navigationTitle("Kubo IPFS")
        } detail: {
            VStack(alignment: .leading, spacing: 0) {
                // Header bar
                HStack {
                    Text(selectedSection.rawValue).font(.title2).bold()
                    Spacer()
                    Text(version).font(.caption).foregroundColor(.secondary)
                }
                .padding()
                .background(Color(NSColor.controlBackgroundColor))

                HStack {
                    TextField("Repo path", text: $repoPath).textFieldStyle(.roundedBorder)
                    Button("Init") { run("init") }
                    Button("Start") { startNode() }
                    Button("Stop") { stopNode() }
                    if busy { ProgressView().scaleEffect(0.7) }
                }
                .padding(.horizontal)

                ScrollView {
                    VStack(alignment: .leading, spacing: 16) {
                        // Repo + node controls
                        // Section-specific actions
                        GroupBox(selectedSection.rawValue) {
                            VStack(alignment: .leading, spacing: 8) {
                                actions(for: selectedSection)
                            }
                            .padding(8)
                            .frame(maxWidth: .infinity, alignment: .leading)
                        }

                        // Command runner
                        if !webUrl.isEmpty && selectedSection == .webui {
                            WebView(urlString: webUrl).frame(minHeight: 360)
                        }

                        // Log terminal (foldable)
                        DisclosureGroup("Log", isExpanded: $logExpanded) {
                            ScrollView {
                                Text(output).font(.system(.caption, design: .monospaced)).frame(maxWidth: .infinity, alignment: .leading)
                            }
                            .frame(minHeight: 220)
                            .background(Color.black.opacity(0.85))
                            .foregroundColor(.green)
                            .cornerRadius(6)
                            .padding(8)
                        }
                    }
                    .padding()
                }
            }
        }
        .onAppear { startLogPolling(); autoStart() }
        .safeAreaInset(edge: .bottom) {
            HStack {
                Button(action: { logExpanded.toggle() }) {
                    Label(logExpanded ? "Hide Log" : "Show Log", systemImage: logExpanded ? "chevron.down" : "chevron.up")
                }
                Spacer()
                if !webUrl.isEmpty {
                    Button(action: { if let url = URL(string: webUrl) { NSWorkspace.shared.open(url) } }) {
                        Label("Open in Browser", systemImage: "safari")
                    }
                }
                Button(action: { if let url = URL(string: "https://github.com/ipfs/ipfs-webui") { NSWorkspace.shared.open(url) } }) {
                    Label("WebUI Repo", systemImage: "link")
                }
            }
            .padding(8)
            .background(.bar)
        }
    }

    @ViewBuilder
    private func actions(for section: SidebarSection) -> some View {
        switch section {
        case .status:
            HStack {
                Button("ID") { run("id") }
                Button("Version") { run("version") }
                Button("Repo Stat") { run("repo stat") }
                Button("API: version") { runApi("version") }
            }
        case .peers:
            HStack {
                Button("Swarm Peers") { showPeers() }
                Button("Dial Bootstrap") { dialBootstrap() }
                Button("Bootstrap List") { run("bootstrap list") }
            }
        case .files:
            HStack {
                Button("Files ls /") { run("files ls /") }
                Button("Pin ls") { run("pin ls") }
                Button("Repo ls") { run("repo ls") }
            }
        case .settings:
            HStack {
                Button("Config show") { run("config show") }
                Button("API: config") { runApi("config/show") }
            }
        case .diagnostics:
            HStack {
                Button("Diag") { run("diag sys") }
                Button("Log level") { run("log level") }
                Button("Stats bw") { run("stats bw") }
            }
        case .webui:
            if webUrl.isEmpty {
                Text("Start a node to load the WebUI").foregroundColor(.secondary)
            } else {
                EmptyView()
            }
        }
    }

    private func icon(for section: SidebarSection) -> String {
        switch section {
        case .status: return "info.circle"
        case .peers: return "network"
        case .files: return "doc"
        case .settings: return "gear"
        case .diagnostics: return "stethoscope"
        case .webui: return "globe"
        }
    }

    private func log(_ s: String) {
        let ts = ISO8601DateFormatter().string(from: Date())
        output = "[\(ts)] \(s)\n" + output
    }

    private func autoStart() {
        DispatchQueue.global().async {
            do {
                if !FileManager.default.fileExists(atPath: repoPath) { try Kubo.initRepo(repoPath) }
                let n = try Kubo.Node(repoPath: repoPath, online: true)
                if let res = Bundle.main.resourcePath {
                    let webuiDir = res + "/webui"
                    if FileManager.default.fileExists(atPath: webuiDir) {
                        n.addStatic(prefix: "/webui-src", dir: webuiDir)
                    }
                }
                let apiUrl = n.startApi("/ip4/127.0.0.1/tcp/0") ?? ""
                let parts = apiUrl.split(separator: "/")
                var host = "127.0.0.1"; var port = "5001"; var i = 0
                while i < parts.count {
                    if parts[i] == "ip4" && i + 1 < parts.count { host = String(parts[i+1]) }
                    if parts[i] == "tcp" && i + 1 < parts.count { port = String(parts[i+1]) }
                    i += 1
                }
                DispatchQueue.main.async {
                    self.node = n
                    self.peerId = n.peerId ?? ""
                    self.addrs = n.listeningAddrs ?? ""
                    self.webUrl = "http://\(host):\(port)/webui/"
                    self.selectedSection = .webui
                    // Add a welcome file to MFS so the WebUI Files page shows content
                    let tmpFile = self.repoPath + "/README.md"
                    try? "# Kubo FFI Demo\n\nHello from Kubo FFI.".write(toFile: tmpFile, atomically: true, encoding: .utf8)
                    if let out = try? n.run("add --to-files /README.md \(tmpFile)") {
                        self.log("mfs: \(out)")
                    }
                    // Add the bundled kubo repo to MFS
                    if let res = Bundle.main.resourcePath {
                        let kuboDir = res + "/kubo"
                        if FileManager.default.fileExists(atPath: kuboDir) {
                            if let out2 = try? n.run("add -r --to-files /kubo \(kuboDir)") {
                                self.log("mfs kubo: \(out2)")
                            }
                        }
                    }
                    self.log("[auto] node started peer=\(self.peerId)")
                    self.log("[auto] listening: \(self.addrs)")
                    self.log("[auto] webui: \(self.webUrl)")
                    self.startPeerPolling()
                    self.dialBootstrap()
                }
            } catch {
                DispatchQueue.main.async { self.log("Error: \(error)") }
            }
        }
    }

    private func startLogPolling() {
        Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
            let chunk = Kubo.getLogs()
            if !chunk.isEmpty { output = chunk + output }
        }
    }

    private func startPeerPolling() {
        peerTimer?.invalidate()
        peerTimer = Timer.scheduledTimer(withTimeInterval: 4.0, repeats: true) { _ in
            guard let n = self.node else { return }
            let peers = n.swarmPeers()
            DispatchQueue.main.async {
                self.peerCount = peers.count
                if peers.count > 0 { self.log("peers(\(peers.count))") }
            }
        }
    }

    private func startNode() {
        busy = true
        DispatchQueue.global().async {
            do {
                if !FileManager.default.fileExists(atPath: repoPath) { try Kubo.initRepo(repoPath) }
                let n = try Kubo.Node(repoPath: repoPath, online: true)
                if let res = Bundle.main.resourcePath {
                    let webuiDir = res + "/webui"
                    if FileManager.default.fileExists(atPath: webuiDir) {
                        n.addStatic(prefix: "/webui-src", dir: webuiDir)
                    }
                }
                let apiUrl = n.startApi("/ip4/127.0.0.1/tcp/0") ?? ""
                let parts = apiUrl.split(separator: "/")
                var host = "127.0.0.1"; var port = "5001"; var i = 0
                while i < parts.count {
                    if parts[i] == "ip4" && i + 1 < parts.count { host = String(parts[i+1]) }
                    if parts[i] == "tcp" && i + 1 < parts.count { port = String(parts[i+1]) }
                    i += 1
                }
                DispatchQueue.main.async {
                    self.node = n
                    self.peerId = n.peerId ?? ""
                    self.addrs = n.listeningAddrs ?? ""
                    self.webUrl = "http://\(host):\(port)/webui/"
                    self.selectedSection = .webui
                    if let out = try? n.run("files write /welcome.txt 'Hello from Kubo FFI' --create") {
                        self.log("mfs: \(out)")
                    }
                    self.log("[node started] peer=\(self.peerId)")
                    self.log("listening: \(self.addrs)")
                    self.log("webui: \(self.webUrl)")
                    self.busy = false
                    self.startPeerPolling()
                    self.dialBootstrap()
                }
            } catch {
                DispatchQueue.main.async { self.log("Error: \(error)"); self.busy = false }
            }
        }
    }

    private func stopNode() {
        peerTimer?.invalidate()
        peerTimer = nil
        node?.stop()
        node = nil
        peerId = ""; addrs = ""; peerCount = 0; webUrl = ""
        log("[node stopped]")
    }

    private func dialBootstrap() {
        busy = true
        log("$ dialing bootstrap peers")
        DispatchQueue.global().async {
            guard let n = self.node else { DispatchQueue.main.async { self.log("no node"); self.busy = false }; return }
            let bootstraps = ["/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN"]
            for p in bootstraps {
                let ok = n.connect(p)
                DispatchQueue.main.async { self.log("connect \(p): \(ok ? "OK" : "FAIL")") }
            }
            let peers = n.swarmPeers()
            DispatchQueue.main.async { self.peerCount = peers.count; self.log("peers(\(peers.count))"); self.busy = false }
        }
    }

    private func showPeers() {
        guard let n = node else { log("no node running"); return }
        let peers = n.swarmPeers()
        peerCount = peers.count
        log("peers(\(peers.count)):\n\(peers.joined(separator: "\n"))")
    }

    private func run(_ cmd: String) {
        busy = true
        log("$ \(cmd)")
        DispatchQueue.global().async {
            do {
                let k = Kubo(repoPath: repoPath)
                let out = try k.run(cmd)
                DispatchQueue.main.async { self.log(out); self.busy = false }
            } catch {
                DispatchQueue.main.async { self.log("Error: \(error)"); self.busy = false }
            }
        }
    }

    private func runApi(_ endpoint: String) {
        busy = true
        log("$ api \(endpoint)")
        guard let node = node else { log("no node"); busy = false; return }
        Task {
            do {
                let out = try await node.apiCall(endpoint)
                await MainActor.run { self.log(out); self.busy = false }
            } catch {
                await MainActor.run { self.log("Error: \(error)"); self.busy = false }
            }
        }
    }
}

struct WebView: NSViewRepresentable {
    let urlString: String
    func makeNSView(context: Context) -> WKWebView {
        let wv = WKWebView()
        if let url = URL(string: urlString) { wv.load(URLRequest(url: url)) }
        return wv
    }
    func updateNSView(_ nsView: WKWebView, context: Context) {
        if let url = URL(string: urlString), nsView.url != url {
            nsView.load(URLRequest(url: url))
        }
    }
}
