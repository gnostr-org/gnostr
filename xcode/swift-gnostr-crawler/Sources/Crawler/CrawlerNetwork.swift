import Foundation
import SwiftUI
import WebKit
import GnostrTypes

public struct CrawlerNetworkBucketSnapshot: Codable, Hashable, Sendable, Identifiable {
    public var source: String
    public var bucket: String
    public var relays: [String]

    public var id: String { "\(self.source):\(self.bucket)" }

    public init(source: String, bucket: String, relays: [String]) {
        self.source = source
        self.bucket = bucket
        self.relays = relays
    }
}

public struct CrawlerNetworkSourceSnapshot: Codable, Hashable, Sendable {
    public var name: String
    public var rootRelays: [String]
    public var buckets: [CrawlerNetworkBucketSnapshot]

    public init(name: String, rootRelays: [String], buckets: [CrawlerNetworkBucketSnapshot]) {
        self.name = name
        self.rootRelays = rootRelays
        self.buckets = buckets
    }
}

public struct CrawlerNetworkSnapshot: Codable, Hashable, Sendable {
    public var refreshedAt: Date
    public var crawlerRuntime: RelayProcessState?
    public var crawlerCrawl: RelayProcessState?
    public var relayDiscovery: [RelayDiscoveryEntry]
    public var crawler: CrawlerNetworkSourceSnapshot
    public var p2p: CrawlerNetworkSourceSnapshot
    public var errors: [String]

    public init(
        refreshedAt: Date,
        crawlerRuntime: RelayProcessState?,
        crawlerCrawl: RelayProcessState?,
        relayDiscovery: [RelayDiscoveryEntry],
        crawler: CrawlerNetworkSourceSnapshot,
        p2p: CrawlerNetworkSourceSnapshot,
        errors: [String]
    ) {
        self.refreshedAt = refreshedAt
        self.crawlerRuntime = crawlerRuntime
        self.crawlerCrawl = crawlerCrawl
        self.relayDiscovery = relayDiscovery
        self.crawler = crawler
        self.p2p = p2p
        self.errors = errors
    }

    public static var empty: CrawlerNetworkSnapshot {
        CrawlerNetworkSnapshot(
            refreshedAt: Date(timeIntervalSince1970: 0),
            crawlerRuntime: nil,
            crawlerCrawl: nil,
            relayDiscovery: [],
            crawler: .init(name: "crawler", rootRelays: [], buckets: []),
            p2p: .init(name: "p2p", rootRelays: [], buckets: []),
            errors: []
        )
    }
}

@MainActor
public final class CrawlerNetworkStore: ObservableObject {
    @Published public private(set) var snapshot: CrawlerNetworkSnapshot = .empty
    @Published public private(set) var isPolling: Bool = false

    public let baseURL: URL
    private let crawlerClient: CrawlerClient
    private let refreshIntervalNanoseconds: UInt64
    private let crawlerConfigRoot: URL
    private let p2pConfigRoot: URL
    private var pollingTask: Task<Void, Never>?
    private var didBootstrapCrawler = false

    public init(
        baseURL: URL = URL(string: "http://127.0.0.1:3030")!,
        session: URLSession = .shared,
        refreshIntervalNanoseconds: UInt64 = 5_000_000_000,
        crawlerConfigRoot: URL = CrawlerNetworkFileSystem.crawlerConfigDirectory(),
        p2pConfigRoot: URL = CrawlerNetworkFileSystem.p2pConfigDirectory()
    ) {
        self.baseURL = baseURL
        self.crawlerClient = CrawlerClient(baseURL: baseURL, session: session)
        self.refreshIntervalNanoseconds = refreshIntervalNanoseconds
        self.crawlerConfigRoot = crawlerConfigRoot
        self.p2pConfigRoot = p2pConfigRoot
        self.startPolling()
        Task { await self.refresh() }
    }

    deinit {
        self.pollingTask?.cancel()
    }

    public func refresh() async {
        var errors: [String] = []
        let crawlerRuntime = RustCrawlerBridge.shared.crawlerRuntimeStatus()
        let crawlerCrawl = RustCrawlerBridge.shared.crawlerCrawlStatus()
        let crawlerRootRelays = self.loadCrawlerRootRelays()
        let p2pRootRelays = CrawlerNetworkFileSystem.loadRelayEntries(
            in: self.p2pConfigRoot,
            source: "p2p"
        )
        var relayDiscovery: [RelayDiscoveryEntry] = []

        if crawlerRuntime?.running == true {
            do {
                relayDiscovery = try await self.crawlerClient.relayDiscovery()
            } catch {
                errors.append("crawler discovery: \(error.localizedDescription)")
            }
        }

        self.snapshot = CrawlerNetworkSnapshot(
            refreshedAt: Date(),
            crawlerRuntime: crawlerRuntime,
            crawlerCrawl: crawlerCrawl,
            relayDiscovery: relayDiscovery,
            crawler: .init(
                name: "crawler",
                rootRelays: crawlerRootRelays,
                buckets: CrawlerNetworkFileSystem.loadRelayBuckets(in: self.crawlerConfigRoot, source: "crawler")
            ),
            p2p: .init(
                name: "p2p",
                rootRelays: p2pRootRelays,
                buckets: CrawlerNetworkFileSystem.loadRelayBuckets(in: self.p2pConfigRoot, source: "p2p")
            ),
            errors: errors
        )
    }

    public func startCrawlerServe() async {
        _ = RustCrawlerBridge.shared.startCrawlerRuntime()
        _ = RustCrawlerBridge.shared.startCrawlerCrawl()
        await self.refresh()
    }

    public func stopCrawlerServe() async {
        _ = RustCrawlerBridge.shared.stopCrawlerCrawl()
        _ = RustCrawlerBridge.shared.stopCrawlerRuntime()
        await self.refresh()
    }

    public func startPolling() {
        guard self.pollingTask == nil else { return }
        self.isPolling = true
        self.pollingTask = Task { [weak self] in
            guard let self else { return }
            await self.pollLoop()
        }
    }

    public func stopPolling() {
        self.pollingTask?.cancel()
        self.pollingTask = nil
        self.isPolling = false
    }

    private func pollLoop() async {
        while !Task.isCancelled {
            await self.refresh()
            if Task.isCancelled {
                break
            }
            try? await Task.sleep(nanoseconds: self.refreshIntervalNanoseconds)
        }
    }

    private func loadCrawlerRootRelays() -> [String] {
        CrawlerNetworkFileSystem.loadRelayEntries(
            in: self.crawlerConfigRoot,
            source: "crawler"
        )
    }
}

public enum CrawlerNetworkFileSystem {
    public static func crawlerConfigDirectory() -> URL {
        self.configDirectory(product: "crawler")
    }

    public static func p2pConfigDirectory() -> URL {
        self.configDirectory(product: "p2p")
    }

    public static func configDirectory(product: String) -> URL {
        let env = ProcessInfo.processInfo.environment
        if let xdg = env["XDG_CONFIG_HOME"], !xdg.isEmpty {
            return URL(fileURLWithPath: xdg, isDirectory: true)
                .appendingPathComponent("gnostr", isDirectory: true)
                .appendingPathComponent(product, isDirectory: true)
        }
        if let home = env["HOME"], !home.isEmpty {
            return URL(fileURLWithPath: home, isDirectory: true)
                .appendingPathComponent(".config", isDirectory: true)
                .appendingPathComponent("gnostr", isDirectory: true)
                .appendingPathComponent(product, isDirectory: true)
        }
        return URL(fileURLWithPath: ".", isDirectory: true)
    }

    static func loadRelayEntries(in rootDirectory: URL, source: String) -> [String] {
        guard let root = self.loadBucket(directory: rootDirectory, bucket: "relays", source: source) else {
            return []
        }
        return root.relays
    }

    static func loadRelayBuckets(in rootDirectory: URL, source: String) -> [CrawlerNetworkBucketSnapshot] {
        var buckets: [CrawlerNetworkBucketSnapshot] = []

        if let root = self.loadBucket(directory: rootDirectory, bucket: "relays", source: source) {
            buckets.append(root)
        }

        let fileManager = FileManager.default
        guard let contents = try? fileManager.contentsOfDirectory(
            at: rootDirectory,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles]
        ) else {
            return buckets.sorted { $0.bucket < $1.bucket }
        }

        for entry in contents {
            guard (try? entry.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) == true else {
                continue
            }
            if let bucket = self.loadBucket(directory: entry, bucket: entry.lastPathComponent, source: source) {
                buckets.append(bucket)
            }
        }

        buckets.sort { $0.bucket < $1.bucket }
        return buckets.dedup(by: { $0.id == $1.id })
    }

    static func loadBucket(directory: URL, bucket: String, source: String) -> CrawlerNetworkBucketSnapshot? {
        let relays = self.loadBucketRelays(directory: directory)
        guard !relays.isEmpty else { return nil }
        return CrawlerNetworkBucketSnapshot(source: source, bucket: bucket, relays: relays)
    }

    static func loadBucketRelays(directory: URL) -> [String] {
        let fileManager = FileManager.default
        var relays = Set<String>()

        func addRelayList(fileURL: URL, fileType: String) {
            guard let content = try? String(contentsOf: fileURL, encoding: .utf8) else { return }
            relays.formUnion(parseRelayEntries(from: content, fileType: fileType))
        }

        let yamlPath = directory.appendingPathComponent("relays.yaml")
        let ymlPath = directory.appendingPathComponent("relays.yml")
        let jsonPath = directory.appendingPathComponent("relays.json")
        let txtPath = directory.appendingPathComponent("relays.txt")

        if fileManager.fileExists(atPath: yamlPath.path) {
            addRelayList(fileURL: yamlPath, fileType: "yaml")
        }
        if fileManager.fileExists(atPath: ymlPath.path) {
            addRelayList(fileURL: ymlPath, fileType: "yaml")
        }
        if fileManager.fileExists(atPath: jsonPath.path) {
            addRelayList(fileURL: jsonPath, fileType: "json")
        }
        if fileManager.fileExists(atPath: txtPath.path) {
            addRelayList(fileURL: txtPath, fileType: "txt")
        }

        guard let contents = try? fileManager.contentsOfDirectory(
            at: directory,
            includingPropertiesForKeys: [.isDirectoryKey],
            options: [.skipsHiddenFiles]
        ) else {
            return relays.sorted()
        }

        for entry in contents {
            let isDirectory = (try? entry.resourceValues(forKeys: [.isDirectoryKey]).isDirectory) == true
            guard !isDirectory else { continue }
            let name = entry.lastPathComponent
            guard name.hasSuffix(".json"), name != "relays.json" else { continue }
            let host = String(name.dropLast(".json".count))
            relays.insert(websocketHTTPURL(host))
        }

        return relays.sorted()
    }

    static func parseRelayEntries(from content: String, fileType: String) -> [String] {
        switch fileType {
        case "json":
            if let data = content.data(using: .utf8),
               let values = try? JSONDecoder().decode([String].self, from: data) {
                return values.compactMap(normalizeRelayEntry)
            }
            return []
        case "txt":
            return content
                .split(whereSeparator: { $0.isWhitespace || $0.isNewline })
                .compactMap { normalizeRelayEntry(String($0)) }
        default:
            return content
                .split(whereSeparator: { $0.isNewline })
                .compactMap { normalizeRelayEntry(String($0)) }
        }
    }

    static func websocketHTTPURL(_ url: String) -> String {
        if !url.contains("://") {
            return "https://\(url)"
        }
        return url.replacingOccurrences(of: "wss://", with: "https://")
            .replacingOccurrences(of: "ws://", with: "http://")
    }

    private static func normalizeRelayEntry(_ line: String) -> String? {
        var value = line.trimmingCharacters(in: .whitespacesAndNewlines)
        if value.hasPrefix("- ") {
            value.removeFirst(2)
            value = value.trimmingCharacters(in: CharacterSet.whitespacesAndNewlines)
        } else if value.hasPrefix("-") {
            value.removeFirst()
            value = value.trimmingCharacters(in: CharacterSet.whitespacesAndNewlines)
        }
        if let comma = value.firstIndex(of: ",") {
            value = String(value[..<comma]).trimmingCharacters(in: .whitespacesAndNewlines)
        }
        if value.isEmpty {
            return nil
        }
        if !value.contains("://") {
            let potential = "wss://\(value)"
            if URL(string: potential) != nil {
                value = potential
            }
        }
        guard value.hasPrefix("wss://") || value.hasPrefix("ws://") else {
            return nil
        }
        guard let url = URL(string: value) else {
            return nil
        }
        return url.absoluteString
    }
}

public struct CrawlerServerWebView: View {
    public let url: URL

    public init(url: URL) {
        self.url = url
    }

    public var body: some View {
        CrawlerServerWebViewRepresentable(url: self.url)
    }
}

#if canImport(UIKit)
private struct CrawlerServerWebViewRepresentable: UIViewRepresentable {
    let url: URL

    func makeUIView(context: Context) -> WKWebView {
        let webView = WKWebView()
        webView.navigationDelegate = context.coordinator
        webView.allowsBackForwardNavigationGestures = true
        return webView
    }

    func updateUIView(_ webView: WKWebView, context: Context) {
        context.coordinator.load(url: self.url, in: webView)
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    final class Coordinator: NSObject, WKNavigationDelegate {
        private var currentURL: URL?

        func load(url: URL, in webView: WKWebView) {
            guard self.currentURL != url else { return }
            self.currentURL = url
            webView.load(URLRequest(url: url))
        }
    }
}
#elseif canImport(AppKit)
private struct CrawlerServerWebViewRepresentable: NSViewRepresentable {
    let url: URL

    func makeNSView(context: Context) -> WKWebView {
        let webView = WKWebView()
        webView.navigationDelegate = context.coordinator
        return webView
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        context.coordinator.load(url: self.url, in: webView)
    }

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    final class Coordinator: NSObject, WKNavigationDelegate {
        private var currentURL: URL?

        func load(url: URL, in webView: WKWebView) {
            guard self.currentURL != url else { return }
            self.currentURL = url
            webView.load(URLRequest(url: url))
        }
    }
}
#endif

public struct CrawlerBucketsWebView: View {
    public let snapshot: CrawlerNetworkSnapshot

    public init(snapshot: CrawlerNetworkSnapshot) {
        self.snapshot = snapshot
    }

    public var body: some View {
        CrawlerBucketsWebViewRepresentable(html: Self.makeHTML(snapshot: self.snapshot))
    }

    private static func makeHTML(snapshot: CrawlerNetworkSnapshot) -> String {
        func escape(_ string: String) -> String {
            string
                .replacingOccurrences(of: "&", with: "&amp;")
                .replacingOccurrences(of: "<", with: "&lt;")
                .replacingOccurrences(of: ">", with: "&gt;")
                .replacingOccurrences(of: "\"", with: "&quot;")
        }

        func renderBuckets(_ source: CrawlerNetworkSourceSnapshot) -> String {
            let rootRelays = source.rootRelays.isEmpty
                ? "<p class=\"muted\">No root relays.</p>"
                : "<ul>" + source.rootRelays.map { "<li><code>\(escape($0))</code></li>" }.joined() + "</ul>"

            let buckets = source.buckets.isEmpty
                ? "<p class=\"muted\">No buckets.</p>"
                : source.buckets.map { bucket in
                    let relays = bucket.relays.map { "<li><code>\(escape($0))</code></li>" }.joined()
                    return """
                    <section class="bucket">
                      <h3>\(escape(bucket.bucket))</h3>
                      <p class="muted">\(
                        escape(bucket.source)
                      )</p>
                      <ul>\(relays)</ul>
                    </section>
                    """
                }.joined()

            return """
            <article class="source">
              <h2>\(escape(source.name))</h2>
              <h3>Root relays</h3>
              \(rootRelays)
              <h3>Buckets</h3>
              \(buckets)
            </article>
            """
        }

        let errors = snapshot.errors.isEmpty
            ? "<p class=\"muted\">No snapshot errors.</p>"
            : "<ul>" + snapshot.errors.map { "<li>\(escape($0))</li>" }.joined() + "</ul>"

        return """
        <!doctype html>
        <html>
        <head>
          <meta charset="utf-8">
          <meta name="viewport" content="width=device-width, initial-scale=1">
          <style>
            body { font-family: -apple-system, BlinkMacSystemFont, sans-serif; margin: 0; padding: 16px; background: #0f1115; color: #f4f4f5; }
            h1, h2, h3 { margin: 0 0 8px 0; }
            .muted { color: #a1a1aa; }
            .meta { margin-bottom: 16px; }
            .source, .bucket { border: 1px solid #2a2f3a; border-radius: 12px; padding: 12px; margin-bottom: 12px; background: #171923; }
            ul { margin: 8px 0 0 20px; }
            code { white-space: pre-wrap; word-break: break-word; }
          </style>
        </head>
        <body>
          <h1>Relay Buckets</h1>
          <div class="meta">
            <div>Updated: \(escape(Self.format(date: snapshot.refreshedAt)))</div>
          </div>
          \(renderBuckets(snapshot.crawler))
          \(renderBuckets(snapshot.p2p))
          <section class="source">
            <h2>Errors</h2>
            \(errors)
          </section>
        </body>
        </html>
        """
    }

    private static func format(date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .medium
        return formatter.string(from: date)
    }
}

#if canImport(UIKit)
private struct CrawlerBucketsWebViewRepresentable: UIViewRepresentable {
    let html: String

    func makeUIView(context: Context) -> WKWebView {
        WKWebView()
    }

    func updateUIView(_ webView: WKWebView, context: Context) {
        webView.loadHTMLString(self.html, baseURL: nil)
    }
}
#elseif canImport(AppKit)
private struct CrawlerBucketsWebViewRepresentable: NSViewRepresentable {
    let html: String

    func makeNSView(context: Context) -> WKWebView {
        WKWebView()
    }

    func updateNSView(_ webView: WKWebView, context: Context) {
        webView.loadHTMLString(self.html, baseURL: nil)
    }
}
#endif

public struct CrawlerNetworkDashboard: View {
    @ObservedObject private var store: CrawlerNetworkStore
    @ObservedObject private var logStore = CrawlerLogStore(maxLines: 200)
    @Environment(\.presentationMode) private var presentationMode

    public init(store: CrawlerNetworkStore) {
        self._store = ObservedObject(wrappedValue: store)
    }

    public var body: some View {
        VStack(spacing: 0) {
            HStack(alignment: .top, spacing: 12) {
                VStack(alignment: .leading, spacing: 2) {
                    Text("Crawler Network")
                        .font(.headline)
                    Text(crawlerHeaderSubtitle)
                        .font(.caption)
                        .foregroundColor(.secondary)
                        .lineLimit(1)
                }

                Spacer()

                Button("Done") {
                    self.presentationMode.wrappedValue.dismiss()
                }
            }
            .padding()

            VStack(alignment: .leading, spacing: 8) {
                Text("Relay Buckets")
                    .font(.headline)
                Text("Local snapshot from crawler and P2P config files")
                    .font(.caption)
                    .foregroundColor(.secondary)
                CrawlerBucketsWebView(snapshot: self.store.snapshot)
                    .frame(maxWidth: .infinity)
                    .frame(minHeight: 320)
                    .cornerRadius(12)
            }
            .padding([.horizontal, .bottom])

            List {
                Section(header: Text("Runtime")) {
                    statusRow(title: "crawler runtime", state: self.store.snapshot.crawlerRuntime)
                    statusRow(title: "crawler crawl", state: self.store.snapshot.crawlerCrawl)
                    Text("refreshed \(Self.format(date: self.store.snapshot.refreshedAt))")
                }

                Section(header: Text("Serve")) {
                    HStack {
                        Button("Start Serve") {
                            Task { await self.store.startCrawlerServe() }
                        }
                        .disabled(self.store.snapshot.crawlerRuntime?.running == true)

                        Button("Stop Serve") {
                            Task { await self.store.stopCrawlerServe() }
                        }
                        .disabled(self.store.snapshot.crawlerRuntime?.running != true)
                    }
                    Text(self.store.snapshot.crawlerRuntime?.message ?? "crawler runtime not running")
                        .font(.caption)
                }

                Section(header: Text("Logs")) {
                    if self.logStore.lines.isEmpty {
                        Text("No crawler logs yet.")
                            .font(.caption)
                            .foregroundColor(.secondary)
                    } else {
                        ForEach(Array(self.logStore.lines.enumerated()), id: \.offset) { _, line in
                            Text(line)
                                .font(.system(size: 11, weight: .regular, design: .monospaced))
                        }
                    }
                }

                if !self.store.snapshot.relayDiscovery.isEmpty {
                    Section(header: Text("Discovered relays")) {
                        ForEach(self.store.snapshot.relayDiscovery, id: \.url) { entry in
                            relayCard(
                                title: entry.url,
                                subtitle: entry.name,
                                body: entry.description,
                                tags: entry.supportedNips.map { "NIP \($0)" }
                            )
                        }
                    }
                }

                sourceSection(self.store.snapshot.crawler)
                sourceSection(self.store.snapshot.p2p)

                if !self.store.snapshot.errors.isEmpty {
                    Section(header: Text("Errors")) {
                        ForEach(self.store.snapshot.errors, id: \.self) { error in
                            Text(error).font(.caption)
                        }
                    }
                }
            }
        }
        .onAppear {
            self.store.startPolling()
        }
        .onDisappear {
            self.store.stopPolling()
        }
    }

    private var crawlerHeaderSubtitle: String {
        let runtime = self.store.snapshot.crawlerRuntime?.message ?? "runtime unknown"
        let crawl = self.store.snapshot.crawlerCrawl?.message ?? "crawl unknown"
        return "\(runtime) • \(crawl)"
    }

    @ViewBuilder
    private func sourceSection(_ source: CrawlerNetworkSourceSnapshot) -> some View {
        Section(header: Text(source.name)) {
            if !source.rootRelays.isEmpty {
                VStack(alignment: .leading, spacing: 8) {
                    Text("root relays")
                        .font(.headline)
                    ForEach(source.rootRelays, id: \.self) { relay in
                        relayCard(title: relay, subtitle: source.name, body: "root relay", tags: [])
                    }
                }
            }

            ForEach(source.buckets) { bucket in
                VStack(alignment: .leading, spacing: 8) {
                    Text(bucket.bucket)
                        .font(.headline)
                    ForEach(bucket.relays, id: \.self) { relay in
                        relayCard(title: relay, subtitle: source.name, body: bucket.bucket, tags: [])
                    }
                }
            }
        }
    }

    @ViewBuilder
    private func relayCard(title: String, subtitle: String?, body: String?, tags: [String]) -> some View {
        VStack(alignment: .leading, spacing: 6) {
            Text(title)
                .font(.body.weight(.semibold))
            if let subtitle {
                Text(subtitle)
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
            if let body {
                Text(body)
                    .font(.caption)
            }
            if !tags.isEmpty {
                Text(tags.joined(separator: " • "))
                    .font(.caption)
                    .foregroundColor(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
        .padding(12)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(Color.secondary.opacity(0.12))
        )
    }

    @ViewBuilder
    private func statusRow(title: String, state: RelayProcessState?) -> some View {
        VStack(alignment: .leading, spacing: 4) {
            Text(title)
            Text(state?.message ?? "unknown")
                .font(.caption)
        }
    }

    private static func format(date: Date) -> String {
        let formatter = DateFormatter()
        formatter.dateStyle = .short
        formatter.timeStyle = .medium
        return formatter.string(from: date)
    }
}

private extension Array {
    func dedup(by areEquivalent: (Element, Element) -> Bool) -> [Element] {
        var result: [Element] = []
        for element in self {
            if result.contains(where: { areEquivalent($0, element) }) {
                continue
            }
            result.append(element)
        }
        return result
    }
}
