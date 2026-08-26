import Foundation
import SwiftUI
import RustyLib
#if canImport(WebKit)
import WebKit
#endif

private func serviceIconColor(for status: String) -> Color {
    let lowercased = status.lowercased()
    if lowercased.contains("running") {
        return .green
    }
    if lowercased.contains("starting") || lowercased.contains("stopping") {
        return .yellow
    }
    return .red
}

@MainActor
public final class CrawlerNetworkStore: ObservableObject {
    @Published public private(set) var status: String = crawlerServiceStatus()
    @Published public private(set) var logs: String = crawlerServiceLogs()

    public let port: UInt16

    public init(port: UInt16 = 3030) {
        self.port = port
    }

    public var isRunning: Bool {
        self.status.localizedCaseInsensitiveContains("running")
    }

    public var iconColor: Color {
        serviceIconColor(for: status)
    }

    public func refresh() async {
        let status = await Task.detached(priority: .userInitiated) {
            crawlerServiceStatus()
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            crawlerServiceLogs()
        }.value
        self.status = status
        self.logs = logs
    }

    public func startCrawlerServe() async {
        let port = self.port
        let status = await Task.detached(priority: .userInitiated) {
            crawlerServiceStart(port: port)
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            crawlerServiceLogs()
        }.value
        self.status = status
        self.logs = logs
        await self.refresh()
    }

    public func stopCrawlerServe() async {
        let status = await Task.detached(priority: .userInitiated) {
            crawlerServiceStop()
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            crawlerServiceLogs()
        }.value
        self.status = status
        self.logs = logs
        await self.refresh()
    }

    public var serviceURL: URL {
        URL(string: "http://127.0.0.1:\(port)/")!
    }
}

@MainActor
public final class SniperServiceStore: ObservableObject {
    @Published public private(set) var status: String = sniperServiceStatus()
    @Published public private(set) var logs: String = sniperServiceLogs()

    public init() {}

    public var isRunning: Bool {
        self.status.localizedCaseInsensitiveContains("running")
    }

    public var iconColor: Color {
        serviceIconColor(for: status)
    }

    public func refresh() async {
        let status = await Task.detached(priority: .userInitiated) {
            sniperServiceStatus()
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            sniperServiceLogs()
        }.value
        self.status = status
        self.logs = logs
    }

    public func startSniperService() async {
        let status = await Task.detached(priority: .userInitiated) {
            sniperServiceStart()
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            sniperServiceLogs()
        }.value
        self.status = status
        self.logs = logs
        await self.refresh()
    }

    public func stopSniperService() async {
        let status = await Task.detached(priority: .userInitiated) {
            sniperServiceStop()
        }.value
        let logs = await Task.detached(priority: .userInitiated) {
            sniperServiceLogs()
        }.value
        self.status = status
        self.logs = logs
        await self.refresh()
    }
}

struct CrawlerServiceWebView: View {
    let url: URL

    var body: some View {
        #if canImport(WebKit)
        WebView(url: url)
        #else
        Text("Web view is unavailable on this platform.")
            .frame(maxWidth: .infinity, maxHeight: .infinity)
        #endif
    }
}

#if canImport(WebKit)
    #if os(macOS)
struct WebView: NSViewRepresentable {
    let url: URL

    func makeNSView(context: Context) -> WKWebView {
        WKWebView()
    }

    func updateNSView(_ nsView: WKWebView, context: Context) {
        let request = URLRequest(url: url)
        if nsView.url != url {
            nsView.load(request)
        }
    }
}
#elseif os(iOS) || os(tvOS) || os(visionOS)
struct WebView: UIViewRepresentable {
    let url: URL

    func makeUIView(context: Context) -> WKWebView {
        WKWebView()
    }

    func updateUIView(_ uiView: WKWebView, context: Context) {
        let request = URLRequest(url: url)
        if uiView.url != url {
            uiView.load(request)
        }
    }
}
#endif
#endif

public struct SniperServiceDashboard: View {
    @ObservedObject var store: SniperServiceStore
    @State private var logFontSize: CGFloat = 12
    @Environment(\.dismiss) private var dismiss

    public init(store: SniperServiceStore) {
        self.store = store
    }

    public var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            controls
            Divider()
            lifecycleBody
            Divider()
            logBody
        }
        .background(.background)
        .task {
            await self.store.refresh()
        }
        .onReceive(Timer.publish(every: 1.0, on: .main, in: .common).autoconnect()) { _ in
            Task {
                await self.store.refresh()
            }
        }
        .textSelection(.enabled)
    }

    private var header: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text("Sniper Service")
                    .font(.headline)
                Text(store.status)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            Button("Start") {
                Task { await store.startSniperService() }
            }
            Button("Stop") {
                Task { await store.stopSniperService() }
            }
            Button("Refresh") {
                Task { await store.refresh() }
            }
            Button("Done") {
                dismiss()
            }
        }
        .padding()
        .background(.regularMaterial)
    }

    private var controls: some View {
        HStack {
            Button("Start") {
                Task { await store.startSniperService() }
            }
            Button("Stop") {
                Task { await store.stopSniperService() }
            }
            Button("Refresh") {
                Task { await store.refresh() }
            }
            Spacer()
            Button {
                adjustLogFontSize(by: -1)
            } label: {
                Image(systemName: "minus")
            }
            .accessibilityLabel("Smaller sniper text")
            Text("\(Int(logFontSize))")
                .font(.caption.monospacedDigit())
                .foregroundStyle(.secondary)
                .frame(minWidth: 28)
            Button {
                adjustLogFontSize(by: 1)
            } label: {
                Image(systemName: "plus")
            }
            .accessibilityLabel("Larger sniper text")
        }
        .padding()
    }

    private var logBody: some View {
        ScrollViewReader { proxy in
            ScrollView {
                Text(store.logs.isEmpty ? "No sniper logs yet." : store.logs)
                    .font(.system(size: logFontSize, weight: .regular, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()
                    .id("sniper-log-tail")
            }
            .textSelection(.enabled)
            .onChange(of: store.logs) { _ in
                withAnimation {
                    proxy.scrollTo("sniper-log-tail", anchor: .bottom)
                }
            }
        }
    }

    private var lifecycleBody: some View {
        ScrollViewReader { proxy in
            ScrollView {
                Text(sniperLifecycleTranscript.isEmpty ? "No sniper lifecycle yet." : sniperLifecycleTranscript)
                    .font(.system(size: logFontSize, weight: .regular, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()
                    .id("sniper-lifecycle-tail")
            }
            .textSelection(.enabled)
            .onChange(of: store.logs) { _ in
                withAnimation {
                    proxy.scrollTo("sniper-lifecycle-tail", anchor: .bottom)
                }
            }
        }
        .frame(minHeight: 180)
    }

    private var sniperLifecycleTranscript: String {
        store.logs
            .split(separator: "\n", omittingEmptySubsequences: false)
            .map(String.init)
            .filter { $0.hasPrefix("sniper lifecycle:") }
            .joined(separator: "\n")
    }

    private func adjustLogFontSize(by delta: CGFloat) {
        logFontSize = min(24, max(10, logFontSize + delta))
    }
}

public struct CrawlerNetworkDashboard: View {
    @ObservedObject var store: CrawlerNetworkStore
    @State private var logFontSize: CGFloat = 12
    @State private var webViewID = UUID()
    @Environment(\.dismiss) private var dismiss

    public init(store: CrawlerNetworkStore) {
        self.store = store
    }

    public var body: some View {
        VStack(spacing: 0) {
            header
            Divider()
            controls
            Divider()
            logBody
            Divider()
            webBody
        }
        .background(.background)
        .task {
            await self.store.refresh()
        }
        .onChange(of: store.status) { _ in
            webViewID = UUID()
        }
        .textSelection(.enabled)
    }

    private var header: some View {
        HStack(spacing: 12) {
            VStack(alignment: .leading, spacing: 2) {
                Text("Crawler Service")
                    .font(.headline)
                Text(store.status)
                    .font(.caption)
                    .foregroundStyle(.secondary)
                    .lineLimit(2)
            }

            Spacer()

            Button("Start") {
                Task { await store.startCrawlerServe() }
            }
            Button("Stop") {
                Task { await store.stopCrawlerServe() }
            }
            Button("Refresh") {
                Task { await store.refresh() }
            }
            Button("Done") {
                dismiss()
            }
        }
        .padding()
        .background(.regularMaterial)
    }

    private var controls: some View {
        HStack {
            Button("Start") {
                Task { await store.startCrawlerServe() }
            }
            Button("Stop") {
                Task { await store.stopCrawlerServe() }
            }
            Button("Refresh") {
                Task { await store.refresh() }
            }
            Spacer()
            Button {
                adjustLogFontSize(by: -1)
            } label: {
                Image(systemName: "minus")
            }
            .accessibilityLabel("Smaller crawler text")
            Text("\(Int(logFontSize))")
                .font(.caption.monospacedDigit())
                .foregroundStyle(.secondary)
                .frame(minWidth: 28)
            Button {
                adjustLogFontSize(by: 1)
            } label: {
                Image(systemName: "plus")
            }
            .accessibilityLabel("Larger crawler text")
        }
        .padding()
    }

    private var logBody: some View {
        ScrollViewReader { proxy in
            ScrollView {
                Text(store.logs.isEmpty ? "No crawler logs yet." : store.logs)
                    .font(.system(size: logFontSize, weight: .regular, design: .monospaced))
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .padding()
                    .id("crawler-log-tail")
            }
            .textSelection(.enabled)
            .onChange(of: store.logs) { _ in
                withAnimation {
                    proxy.scrollTo("crawler-log-tail", anchor: .bottom)
                }
            }
        }
    }

    private var webBody: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Crawler Web Page")
                .font(.headline)
                .padding(.horizontal)
                .padding(.top, 8)

            CrawlerServiceWebView(url: store.serviceURL)
                .id(webViewID)
                .frame(maxWidth: .infinity, maxHeight: .infinity)
                .background(.secondary.opacity(0.08))
        }
        .frame(minHeight: 260)
    }

    private func adjustLogFontSize(by delta: CGFloat) {
        logFontSize = min(24, max(10, logFontSize + delta))
    }
}
