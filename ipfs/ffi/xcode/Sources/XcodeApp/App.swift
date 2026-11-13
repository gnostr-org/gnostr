import SwiftUI
import WebKit
import Kubo

@main
struct KuboXcodeApp: App {
    var body: some Scene {
        WindowGroup {
            ContentView()
        }
    }
}

struct ContentView: View {
    @State private var version: String = Kubo.version()
    @State private var repoPath: String = FileManager.default.temporaryDirectory.appendingPathComponent("kubo-xcode-demo").path
    @State private var command: String = "swarm peers"
    @State private var output: String = ""
    @State private var busy: Bool = false
    @State private var node: Kubo.Node? = nil
    @State private var peerId: String = ""
    @State private var addrs: String = ""
    @State private var peerCount: Int = 0
    @State private var logTimer: Timer? = nil
    @State private var peerTimer: Timer? = nil
    @State private var webUrl: String = ""

    var body: some View {
        HSplitView {
            VStack(alignment: .leading, spacing: 12) {
                Text("Kubo IPFS FFI Demo").font(.title)
                Text("Kubo version: \(version)").font(.caption)
                TextField("Repo path", text: $repoPath)
                HStack {
                    Button("Init Repo") { run("init") }
                    Button("Start Node (online)") { startNode() }
                    Button("Stop Node") { stopNode() }
                }
                if !peerId.isEmpty {
                    Text("Peer ID: \(peerId)").font(.caption)
                    Text("Listening: \(addrs)").font(.caption)
                    Text("Connected peers: \(peerCount)").font(.caption).bold()
                }
                HStack {
                    Button("Swarm Peers") { showPeers() }
                    Button("Bootstrap") { dialBootstrap() }
                    Button("ID") { run("id") }
                    Button("API: version") { runApi("version") }
                }
                HStack {
                    TextField("Command", text: $command)
                    Button("Run") { run(command) }.disabled(busy)
                }
                if busy { ProgressView().scaleEffect(0.7) }
                ScrollView {
                    Text(output).font(.system(.body, design: .monospaced)).frame(maxWidth: .infinity, alignment: .leading)
                }
                .frame(minHeight: 200)
                .background(Color.black.opacity(0.05))
                .cornerRadius(8)
            }
            .padding()
            .frame(minWidth: 460)

            VStack {
                if !webUrl.isEmpty {
                    WebView(urlString: webUrl)
                } else {
                    Text("WebUI will appear here after starting a node").foregroundColor(.secondary)
                }
            }
            .frame(minWidth: 400)
        }
        .frame(minWidth: 900, minHeight: 600)
        .onAppear { startLogPolling() }
    }

    private func log(_ s: String) {
        let ts = ISO8601DateFormatter().string(from: Date())
        output = "[\(ts)] \(s)\n" + output
        let logFile = FileManager.default.currentDirectoryPath + "/kubo-xcode.log"
        if let h = FileHandle(forWritingAtPath: logFile) {
            h.seekToEndOfFile()
            h.write(("[\(ts)] \(s)\n").data(using: .utf8)!)
            h.closeFile()
        }
    }

    private func startLogPolling() {
        Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
            let chunk = Kubo.getLogs()
            if !chunk.isEmpty {
                output = chunk + output
            }
        }
    }

    private func startPeerPolling() {
        Timer.scheduledTimer(withTimeInterval: 4.0, repeats: true) { _ in
            guard let n = self.node else { return }
            let peers = n.swarmPeers()
            DispatchQueue.main.async {
                self.peerCount = peers.count
                if peers.count > 0 {
                    self.log("peers(\(peers.count))")
                }
            }
        }
    }

    private func startNode() {
        busy = true
        DispatchQueue.global().async {
            do {
                if !FileManager.default.fileExists(atPath: repoPath) {
                    try Kubo.initRepo(repoPath)
                }
                let n = try Kubo.Node(repoPath: repoPath, online: true)
                let apiUrl = n.startApi("/ip4/127.0.0.1/tcp/0") ?? ""
                DispatchQueue.main.async {
                    self.node = n
                    self.peerId = n.peerId ?? ""
                    self.addrs = n.listeningAddrs ?? ""
                    let parts = apiUrl.split(separator: "/")
                    var host = "127.0.0.1"
                    var port = "5001"
                    var i = 0
                    while i < parts.count {
                        if parts[i] == "ip4" && i + 1 < parts.count { host = String(parts[i+1]) }
                        if parts[i] == "tcp" && i + 1 < parts.count { port = String(parts[i+1]) }
                        i += 1
                    }
                    self.webUrl = "http://\(host):\(port)/webui"
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
        node?.stop()
        node = nil
        peerId = ""
        addrs = ""
        peerCount = 0
        webUrl = ""
        log("[node stopped]")
    }

    private func dialBootstrap() {
        busy = true
        log("$ dialing bootstrap peers")
        DispatchQueue.global().async {
            guard let n = self.node else {
                DispatchQueue.main.async { self.log("no node running"); self.busy = false }
                return
            }
            let bootstraps = [
                "/dnsaddr/bootstrap.libp2p.io/p2p/QmNnooDu7bfjPFoTZYxMNLWUQJyrVwtbZg5gBMjTezGAJN",
            ]
            for p in bootstraps {
                let ok = n.connect(p)
                DispatchQueue.main.async { self.log("connect \(p): \(ok ? "OK" : "FAIL")") }
            }
            let peers = n.swarmPeers()
            DispatchQueue.main.async {
                self.peerCount = peers.count
                self.log("peers(\(peers.count))")
                self.busy = false
            }
        }
    }

    private func showPeers() {
        guard let n = node else { log("no node running"); return }
        let peers = n.swarmPeers()
        peerCount = peers.count
        log("peers(\(peers.count)):\n\(peers.joined(separator: "\n"))")
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
}

struct WebView: NSViewRepresentable {
    let urlString: String
    func makeNSView(context: Context) -> WKWebView {
        let wv = WKWebView()
        if let url = URL(string: urlString) {
            wv.load(URLRequest(url: url))
        }
        return wv
    }
    func updateNSView(_ nsView: WKWebView, context: Context) {
        if let url = URL(string: urlString), nsView.url != url {
            nsView.load(URLRequest(url: url))
        }
    }
}