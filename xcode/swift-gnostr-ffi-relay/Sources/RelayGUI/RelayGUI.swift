import Foundation
import SwiftUI
import FFRelay
#if canImport(UIKit)
import UIKit
#elseif canImport(AppKit)
import AppKit
#endif

enum RelayStatusIndicatorState: String, Sendable {
    case green
    case yellow
    case red
}

public protocol RelayControlling: Sendable {
    func status() async throws -> RelayProcessState
    func start() async throws -> RelayProcessState
    func stop() async throws -> RelayProcessState
    func restart() async throws -> RelayProcessState
}

public final class RelayControlClient: RelayControlling, @unchecked Sendable {
    private let bridge: RustRelayBridge

    public init(bridge: RustRelayBridge = .shared) {
        self.bridge = bridge
    }

    public func status() async throws -> RelayProcessState {
        try unwrap(bridge.relayStatus(), message: "relay status unavailable")
    }

    public func start() async throws -> RelayProcessState {
        try unwrap(bridge.relayStart(), message: "relay start unavailable")
    }

    public func stop() async throws -> RelayProcessState {
        try unwrap(bridge.relayStop(), message: "relay stop unavailable")
    }

    public func restart() async throws -> RelayProcessState {
        try unwrap(bridge.relayRestart(), message: "relay restart unavailable")
    }

    private func unwrap(_ state: RelayProcessState?, message: String) throws -> RelayProcessState {
        guard let state else {
            throw NSError(domain: "RelayControlClient", code: -1, userInfo: [
                NSLocalizedDescriptionKey: message
            ])
        }
        return state
    }
}

@MainActor
public final class RelayDashboardViewModel: ObservableObject {
    @Published public var host: String
    @Published public var port: String
    @Published public var defaultConfiguration: RelayConfiguration
    @Published public var configFileContents: String
    @Published public var isConfigEditorVisible = false
    @Published public private(set) var isConfigFileEditable = false
    @Published public private(set) var listenEndpoint: String
    @Published public private(set) var statusMessage: String
    @Published public private(set) var configFileStatus: String
    @Published public private(set) var isRunning = false
    @Published public var isConsoleExpanded = false
    @Published public private(set) var logLines: [String] = []

    private let configBaseDirectoryPath: String
    private let relayControl: any RelayControlling

    public init(
        host: String = "127.0.0.1",
        port: String = "8080",
        autoStart: Bool = true,
        configBaseDirectoryPath: String? = nil,
        relayControl: any RelayControlling = RelayControlClient()
    ) {
        self.host = host
        self.port = port
        self.configBaseDirectoryPath = configBaseDirectoryPath ?? Self.defaultConfigBaseDirectoryPath()
        self.relayControl = relayControl
        self.defaultConfiguration = RelayConfiguration.rustDefault() ?? RelayConfiguration()
        self.listenEndpoint = RelayEndpoints.listenEndpoint(host: host, port: UInt16(port) ?? 8080)
        self.statusMessage = RustRelayBridge.shared.isAvailable ? "Relay FFI available" : "Relay FFI unavailable"
        self.configFileContents = ""
        self.configFileStatus = "Config file not loaded"
        loadConfigFileContents(allowEditing: false)
        appendLog("Relay dashboard ready")
        appendLog(statusMessage)
        if autoStart {
            Task { await startRelay() }
        }
    }

    public func refresh() {
        defaultConfiguration = RelayConfiguration.rustDefault() ?? RelayConfiguration()
        listenEndpoint = RelayEndpoints.listenEndpoint(host: host, port: UInt16(port) ?? 8080)
        appendLog("Refreshed relay defaults")
        appendLog("Listen endpoint: \(listenEndpoint)")
    }

    public func updateLogging(_ value: String) {
        defaultConfiguration.logging = value
        appendLog("Logging updated to \(value)")
    }

    public func updateConfigFilePath(_ value: String) {
        defaultConfiguration.configFilePath = value
        loadConfigFileContents()
        appendLog("Config file updated to \(value)")
        appendLog("System path: \(resolvedConfigFilePath)")
    }

    public func loadConfigFileContents() {
        loadConfigFileContents(allowEditing: true)
    }

    private func loadConfigFileContents(allowEditing: Bool) {
        do {
            if let contents = try readConfigFile() {
                configFileContents = contents
                configFileStatus = "Loaded \(resolvedConfigFilePath)"
            } else {
                configFileContents = Self.defaultConfigTemplate()
                configFileStatus = "Loaded template for \(resolvedConfigFilePath)"
            }
            isConfigFileEditable = allowEditing
            appendLog(configFileStatus)
        } catch {
            configFileStatus = "Load failed: \(error.localizedDescription)"
            isConfigFileEditable = false
            appendLog(configFileStatus)
        }
    }

    public func saveConfigFileContents() {
        do {
            try writeConfigFile(configFileContents)
            configFileStatus = "Saved \(resolvedConfigFilePath)"
            isConfigFileEditable = false
            appendLog(configFileStatus)
        } catch {
            configFileStatus = "Save failed: \(error.localizedDescription)"
            appendLog(configFileStatus)
        }
    }

    public func updateEndpoint() {
        listenEndpoint = RelayEndpoints.listenEndpoint(host: host, port: UInt16(port) ?? 8080)
        appendLog("Endpoint updated to \(listenEndpoint)")
    }

    public func startRelay() async {
        isRunning = true
        statusMessage = "Relay starting..."
        appendLog("Start pressed")
        appendLog(statusMessage)
        do {
            let state = try await relayControl.start()
            isRunning = state.running
            statusMessage = state.message
            appendLog(state.message)
        } catch {
            isRunning = false
            statusMessage = "Relay start failed: \(error.localizedDescription)"
            appendLog(statusMessage)
        }
    }

    public func stopRelay() async {
        isRunning = false
        statusMessage = "Relay stopping..."
        appendLog("Stop pressed")
        appendLog(statusMessage)
        do {
            let state = try await relayControl.stop()
            isRunning = state.running
            statusMessage = state.message
            appendLog(state.message)
        } catch {
            statusMessage = "Relay stop failed: \(error.localizedDescription)"
            appendLog(statusMessage)
        }
    }

    public func restartRelay() async {
        statusMessage = "Relay restarting..."
        appendLog("Restart pressed")
        appendLog(statusMessage)
        do {
            let state = try await relayControl.restart()
            isRunning = state.running
            statusMessage = state.message
            appendLog(state.message)
        } catch {
            isRunning = false
            statusMessage = "Relay restart failed: \(error.localizedDescription)"
            appendLog(statusMessage)
        }
    }

    public func clearLog() {
        logLines.removeAll()
        appendLog("Console cleared")
    }

    public func toggleConsoleVisibility() {
        isConsoleExpanded.toggle()
        appendLog(isConsoleExpanded ? "Console expanded" : "Console collapsed")
    }

    public func toggleConfigEditorVisibility() {
        isConfigEditorVisible.toggle()
        appendLog(isConfigEditorVisible ? "Config editor shown" : "Config editor hidden")
    }

    var statusIndicatorState: RelayStatusIndicatorState {
        Self.statusIndicatorState(for: statusMessage, isRunning: isRunning)
    }

    static func statusIndicatorState(for statusMessage: String, isRunning: Bool) -> RelayStatusIndicatorState {
        if statusMessage.localizedCaseInsensitiveContains("unavailable") {
            return .red
        }
        if statusMessage.localizedCaseInsensitiveContains("stopping")
            || statusMessage.localizedCaseInsensitiveContains("stopped")
        {
            return .red
        }
        if statusMessage.localizedCaseInsensitiveContains("restarting")
            || statusMessage.localizedCaseInsensitiveContains("starting")
        {
            return .yellow
        }
        if statusMessage.localizedCaseInsensitiveContains("spawned detached relay")
            || statusMessage.localizedCaseInsensitiveContains("already running")
            || statusMessage.localizedCaseInsensitiveContains("running")
        {
            return .green
        }
        return isRunning ? .green : .yellow
    }

    private func appendLog(_ message: String) {
        let line = "[\(Self.timestamp())] \(message)"
        logLines.insert(line, at: 0)
        NSLog("%@", line)
    }

    private func readConfigFile() throws -> String? {
        let url = configFileURL
        guard FileManager.default.fileExists(atPath: url.path) else {
            return nil
        }
        return try String(contentsOf: url, encoding: .utf8)
    }

    private func writeConfigFile(_ contents: String) throws {
        let url = configFileURL
        try FileManager.default.createDirectory(
            at: url.deletingLastPathComponent(),
            withIntermediateDirectories: true
        )
        try contents.write(to: url, atomically: true, encoding: .utf8)
    }

    private var configFileURL: URL {
        URL(fileURLWithPath: resolvedConfigFilePath)
    }

    var resolvedConfigFilePath: String {
        RelayConfiguration.resolvedConfigFilePath(
            defaultConfiguration.configFilePath,
            relativeTo: configBaseDirectoryPath
        )
    }

    private static func defaultConfigBaseDirectoryPath() -> String {
        let bundleID = Bundle.main.bundleIdentifier ?? "gnostr"
        let supportURL = FileManager.default.urls(for: .applicationSupportDirectory, in: .userDomainMask).first
            ?? FileManager.default.temporaryDirectory
        return supportURL.appendingPathComponent(bundleID, isDirectory: true).path
    }

    private static func defaultConfigTemplate() -> String {
        guard let url = Bundle.module.url(forResource: "relay", withExtension: "toml"),
              let contents = try? String(contentsOf: url, encoding: .utf8) else {
            return ""
        }
        return contents
    }

    private static func timestamp() -> String {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter.string(from: Date())
    }
}

public struct RelayDashboardView: View {
    @ObservedObject private var model: RelayDashboardViewModel

    public init(model: RelayDashboardViewModel = RelayDashboardViewModel()) {
        _model = ObservedObject(wrappedValue: model)
    }

    public var body: some View {
        ZStack {
            watermarkBackground
            VStack(alignment: .leading, spacing: 0) {
#if os(macOS) || targetEnvironment(macCatalyst)
                header
                    .padding(.top, 24)
                    .padding(.bottom, 8)
                    .background(headerBackground)
                    .zIndex(1)
#else
                header
                    .padding(.top, 24)
                    .padding(.bottom, 8)
                    .background(headerBackground)
                    .zIndex(1)
#endif
                card(title: nil, compact: false, content: configurationContent)
                .frame(maxWidth: .infinity, alignment: .topLeading)
                Spacer(minLength: 10)
                StickyFooter(
                    label: "Log console",
                    isExpanded: $model.isConsoleExpanded,
                    expandedContent: {
                        consoleContent
                    },
                    trailingActions: {
                        //Button("Clear") { model.clearLog() }
                        //  .buttonStyle(.borderless)
                    }
                )
            }
            .padding(.horizontal)
            .padding(.bottom)
        }
        .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .topLeading)
    }

    private var header: some View {
        HStack(alignment: .center, spacing: 16) {
            relayHeaderIcon()
                .resizable()
                .aspectRatio(contentMode: .fit)
                .frame(width: 56, height: 56)
            VStack(alignment: .leading, spacing: 8) {
                Text("Gnostr Relay")
                    .font(.largeTitle.bold())
                //Text("todo")
                  //  .foregroundColor(.secondary)
            }
            Spacer()
            controls
            trafficLightIndicator
        }
    }

    private func relayHeaderIcon() -> Image {
        guard let url = Bundle.module.url(forResource: "Icon", withExtension: "png"),
              let data = try? Data(contentsOf: url) else {
#if canImport(UIKit)
            return Image(uiImage: UIImage())
#elseif canImport(AppKit)
            return Image(nsImage: NSImage())
#else
            return Image(decorative: "")
#endif
        }

#if canImport(UIKit)
        return Image(uiImage: UIImage(data: data) ?? UIImage())
#elseif canImport(AppKit)
        return Image(nsImage: NSImage(data: data) ?? NSImage())
#else
        return Image(decorative: "")
#endif
    }

    private var configurationContent: some View {
        VStack(alignment: .leading, spacing: 12) {
            HStack() {
                Text("Config editor")
                    .font(.subheadline.weight(.semibold))
                Spacer()
                Button(action: { model.toggleConfigEditorVisibility() }) {
                    if #available(macOS 11.0, iOS 14.0, *) {
                        Image(systemName: model.isConfigEditorVisible ? "gearshape.fill" : "gearshape")
                    } else {
                        Text("⚙")
                    }
                }
                .buttonStyle(.plain)
                .font(.system(size: 20, weight: .regular))
                .frame(width: 30, height: 30, alignment: .center)
            }
            if model.isConfigEditorVisible {
                VStack(alignment: .leading, spacing: 12) {
                    VStack(alignment: .leading, spacing: 12) {
                        HStack {
                            TextField("Host", text: Binding(
                                get: { model.host },
                                set: { newValue in
                                    model.host = newValue
                                    model.updateEndpoint()
                                }
                            ))
                            .textFieldStyle(.roundedBorder)
                            TextField("Port", text: Binding(
                                get: { model.port },
                                set: { newValue in
                                    model.port = newValue
                                    model.updateEndpoint()
                                }
                            ))
                            .textFieldStyle(.roundedBorder)
                            .frame(width: 100)
                        }
                        VStack(alignment: .leading, spacing: 12) {
                            //TextField("Logging", text: Binding(
                            //    get: { model.defaultConfiguration.logging },
                            //    set: { model.updateLogging($0) }
                            //))
                            //.textFieldStyle(.roundedBorder)
                            row(label: "File status", value: model.configFileStatus)
                            configEditor
                                .disabled(!model.isConfigFileEditable)
                                .opacity(model.isConfigFileEditable ? 1.0 : 0.45)
                            HStack {
                                Button("Refresh defaults") { model.refresh() }
                                Button("Load file") { model.loadConfigFileContents() }
                                Button("Save file") { model.saveConfigFileContents() }
                                    .disabled(!model.isConfigFileEditable)
                            }
                        }
                        .padding(12)
                        .background(
                            RoundedRectangle(cornerRadius: 10, style: .continuous)
                                .fill(Color.secondary.opacity(0.08))
                        )
                    }
                }
            } else {
                Text("Click the gear to edit the relay config file.")
                    .foregroundColor(.secondary)
            }
        }
        .frame(maxWidth: .infinity, alignment: .leading)
    }

    @ViewBuilder
    private var configEditor: some View {
        if #available(macOS 11.0, iOS 14.0, *) {
            TextEditor(text: Binding(
                get: { model.configFileContents },
                set: { model.configFileContents = $0 }
            ))
            .font(.system(.body, design: .monospaced))
            .frame(minHeight: 240)
            .overlay(
                RoundedRectangle(cornerRadius: 8, style: .continuous)
                    .stroke(Color.secondary.opacity(0.3))
            )
        } else {
            TextField("Config contents", text: Binding(
                get: { model.configFileContents },
                set: { model.configFileContents = $0 }
            ))
            .textFieldStyle(.roundedBorder)
        }
    }

    private var controls: some View {
        HStack(spacing: 8) {
            Button("Start") { Task { await model.startRelay() } }
                .disabled(model.isRunning)
            Button("Stop") { Task { await model.stopRelay() } }
                .disabled(!model.isRunning)
            Button("Restart") { Task { await model.restartRelay() } }
        }
        .buttonStyle(.borderless)
    }

    private var consoleContent: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 8) {
                ForEach(Array(model.logLines.enumerated()), id: \.offset) { _, line in
                    Text(line)
                        .font(.system(.caption, design: .monospaced))
                        .frame(maxWidth: .infinity, alignment: .leading)
                }
            }
            .frame(maxWidth: .infinity, alignment: .leading)
        }
    }

    private var trafficLightIndicator: some View {
        Circle()
            .fill(statusIndicatorColor)
            .frame(width: 14, height: 14)
        .padding(8)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color.secondary.opacity(0.12))
        )
    }

    @ViewBuilder
    private var headerBackground: some View {
        Color.white.opacity(0.98)
    }

    private var watermarkBackground: some View {
        relayHeaderIcon()
            .resizable()
            .aspectRatio(contentMode: .fit)
            .frame(maxWidth: 520)
            .opacity(0.06)
            .blur(radius: 0.4)
            .allowsHitTesting(false)
            .frame(maxWidth: .infinity, maxHeight: .infinity, alignment: .center)
    }

    private var statusIndicatorColor: Color {
        switch model.statusIndicatorState {
        case .green: return .green
        case .yellow: return .yellow
        case .red: return .red
        }
    }

    private func row(label: String, value: String) -> some View {
        HStack(alignment: .top, spacing: 12) {
            Text(label)
                .frame(width: 92, alignment: .leading)
                .foregroundColor(.secondary)
            Text(value)
                .font(.system(.body, design: .monospaced))
        }
    }

    private func card(title: String?, compact: Bool, content: some View) -> some View {
        VStack(alignment: .leading, spacing: 12) {
            if let title {
                Text(title)
                    .font(compact ? .subheadline.weight(.semibold) : .headline)
            }
            content
        }
        .padding(compact ? 10 : 16)
        .background(
            RoundedRectangle(cornerRadius: 12, style: .continuous)
                .fill(Color.secondary.opacity(0.12))
        )
    }
}

public struct StickyFooter<ExpandedContent: View, TrailingActions: View>: View {
    @Binding private var isExpanded: Bool
    private let label: String
    private let expandedMaxHeight: CGFloat?
    private let expandedContent: ExpandedContent
    private let trailingActions: TrailingActions

    public init(
        label: String,
        isExpanded: Binding<Bool>,
        expandedMaxHeight: CGFloat? = nil,
        @ViewBuilder expandedContent: () -> ExpandedContent,
        @ViewBuilder trailingActions: () -> TrailingActions
    ) {
        self.label = label
        self._isExpanded = isExpanded
        self.expandedMaxHeight = expandedMaxHeight
        self.expandedContent = expandedContent()
        self.trailingActions = trailingActions()
    }

    public var body: some View {
        VStack(spacing: 0) {
            if isExpanded {
                ScrollView(showsIndicators: false) {
                    expandedContent
                        .frame(maxWidth: .infinity, alignment: .topLeading)
                }
                .frame(maxHeight: expandedMaxHeight ?? .infinity, alignment: .topLeading)
                .padding(.bottom, 8)
                Divider()
            }
            footerBar
        }
        .padding(12)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color.secondary.opacity(0.12))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .stroke(Color.secondary.opacity(0.25))
        )
    }

    private var footerBarHeight: CGFloat { 44 }

    private var footerBar: some View {
        HStack {
            Text(label)
                .font(.subheadline.weight(.semibold))
            Spacer()
            Button(action: { isExpanded.toggle() }) {
                if #available(macOS 11.0, iOS 14.0, *) {
                    Image(systemName: isExpanded ? "chevron.down" : "chevron.up")
                } else {
                    Text(isExpanded ? "⌄" : "⌃")
                }
            }
            .buttonStyle(.plain)
            trailingActions
        }
        .frame(height: footerBarHeight)
    }
}
