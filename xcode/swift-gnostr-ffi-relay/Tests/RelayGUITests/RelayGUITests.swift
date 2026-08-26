import Foundation
import Testing
import FFRelay
@testable import RelayGUI

final class MockRelayControl: RelayControlling, @unchecked Sendable {
    var calls: [String] = []

    var statusState = RelayProcessState(running: false, pid: nil, message: "relay stopped", diskUsageBytes: nil)
    var startState = RelayProcessState(running: true, pid: 41_001, message: "spawned detached relay pid 41001", diskUsageBytes: nil)
    var stopState = RelayProcessState(running: false, pid: 41_001, message: "stopped relay pid 41001", diskUsageBytes: nil)
    var restartState = RelayProcessState(running: true, pid: 41_002, message: "spawned detached relay pid 41002", diskUsageBytes: nil)

    func status() async throws -> RelayProcessState {
        calls.append("status")
        return statusState
    }

    func start() async throws -> RelayProcessState {
        calls.append("start")
        return startState
    }

    func stop() async throws -> RelayProcessState {
        calls.append("stop")
        return stopState
    }

    func restart() async throws -> RelayProcessState {
        calls.append("restart")
        return restartState
    }
}

@MainActor
@Test func relayDashboardViewModelBuildsEndpoint() {
    let model = RelayDashboardViewModel(host: "127.0.0.1", port: "8080")
    #expect(model.listenEndpoint == "ws://127.0.0.1:8080")
}

@MainActor
@Test func relayDashboardViewModelUpdatesConfigFields() {
    let model = RelayDashboardViewModel(autoStart: false)
    model.updateLogging("debug")
    model.updateConfigFilePath(".gnostr/custom-relay.toml")
    #expect(model.defaultConfiguration.logging == "debug")
    #expect(model.defaultConfiguration.configFilePath == ".gnostr/custom-relay.toml")
}

@MainActor
@Test func relayDashboardViewModelTogglesConfigEditor() {
    let model = RelayDashboardViewModel(autoStart: false)
    #expect(model.isConfigEditorVisible == false)
    model.toggleConfigEditorVisibility()
    #expect(model.isConfigEditorVisible == true)
}

@MainActor
@Test func relayDashboardViewModelTogglesConsoleVisibility() {
    let model = RelayDashboardViewModel(autoStart: false)
    #expect(model.isConsoleExpanded == false)
    model.toggleConsoleVisibility()
    #expect(model.isConsoleExpanded == true)
}

@MainActor
@Test func relayDashboardViewModelMapsStatusIndicatorState() {
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay FFI unavailable", isRunning: false) == .red)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay FFI available", isRunning: false) == .yellow)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay FFI available", isRunning: true) == .green)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay stopping...", isRunning: false) == .red)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "stopped relay pid 41001", isRunning: false) == .red)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "Relay restarting...", isRunning: false) == .yellow)
    #expect(RelayDashboardViewModel.statusIndicatorState(for: "spawned detached relay pid 41002", isRunning: true) == .green)
}

@MainActor
@Test func relayDashboardViewModelRestartsRelay() async {
    let control = MockRelayControl()
    let model = RelayDashboardViewModel(autoStart: false, relayControl: control)

    await model.restartRelay()

    #expect(control.calls == ["restart"])
    #expect(model.isRunning == true)
    #expect(model.statusMessage == "spawned detached relay pid 41002")
}

@MainActor
@Test func relayDashboardViewModelLoadsConfigFileContents() throws {
    let fileManager = FileManager.default
    let root = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    let configURL = root.appendingPathComponent(".gnostr/relay.toml")
    try fileManager.createDirectory(at: configURL.deletingLastPathComponent(), withIntermediateDirectories: true)
    try "logging = \"trace\"\n".write(to: configURL, atomically: true, encoding: .utf8)

    let model = RelayDashboardViewModel(autoStart: false, configBaseDirectoryPath: root.path)
    #expect(model.configFileContents.contains("logging = \"trace\""))
    #expect(model.configFileStatus.hasPrefix("Loaded"))
}

@MainActor
@Test func relayDashboardViewModelKeepsEditorLockedUntilLoadPressed() {
    let model = RelayDashboardViewModel(autoStart: false)
    #expect(model.isConfigFileEditable == false)
    model.loadConfigFileContents()
    #expect(model.isConfigFileEditable == true)
}

@MainActor
@Test func relayDashboardViewModelLoadsTemplateWhenConfigFileIsMissing() {
    let fileManager = FileManager.default
    let root = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)

    let model = RelayDashboardViewModel(autoStart: false, configBaseDirectoryPath: root.path)
    #expect(model.configFileContents.contains("# [network]"))
    #expect(model.configFileContents.contains("# max_message_length = 524288"))
    #expect(model.configFileStatus.contains("template"))
}

@MainActor
@Test func relayDashboardViewModelSavesConfigFileContents() throws {
    let fileManager = FileManager.default
    let root = fileManager.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    let model = RelayDashboardViewModel(autoStart: false, configBaseDirectoryPath: root.path)

    model.loadConfigFileContents()
    model.configFileContents = "logging = \"debug\"\n"
    model.saveConfigFileContents()

    let saved = try String(contentsOf: root.appendingPathComponent(".gnostr/relay.toml"), encoding: .utf8)
    #expect(saved.contains("logging = \"debug\""))
    #expect(model.configFileStatus.hasPrefix("Saved"))
    #expect(model.isConfigFileEditable == false)
}
