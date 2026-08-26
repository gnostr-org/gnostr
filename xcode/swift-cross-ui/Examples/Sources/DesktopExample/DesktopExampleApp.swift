import DefaultBackend
import Foundation
import SwiftCrossUI

enum DesktopExamplePanel: String, CaseIterable, Hashable {
    case overview = "Overview"
    case controls = "Controls"
    case activity = "Activity"
}

@MainActor
@ObservableObject
final class DesktopExampleViewModel {
    var selectedPanel: DesktopExamplePanel? = .overview
    var message = "Hello from SwiftCrossUI"
    var counter = 0
    var enabled = true
    var activityLog: [String] = []

    var platformLabel: String {
        #if os(macOS)
            return "macOS"
        #elseif os(Linux)
            return "Linux"
        #else
            return "Other"
        #endif
    }

    init() {
        log("Desktop example started")
    }

    func incrementCounter() {
        counter += 1
        log("Counter incremented to \(counter)")
    }

    func resetCounter() {
        counter = 0
        log("Counter reset")
    }

    func appendMessage() {
        log("Message: \(message)")
    }

    func log(_ entry: String) {
        let formatter = Self.timestampFormatter
        activityLog.insert("[\(formatter.string(from: Date()))] \(entry)", at: 0)
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}

@main
struct DesktopExample: App {
    @State var model = DesktopExampleViewModel()

    var body: some Scene {
        WindowGroup("SwiftCrossUI Desktop Example") {
            ContentView()
                .environment(model)
        }
        .defaultSize(width: 880, height: 640)
    }
}

struct ContentView: View {
    @SwiftCrossUI.Environment(DesktopExampleViewModel.self) var model

    var body: some View {
        VStack(alignment: .leading, spacing: 16) {
            header
            controls
            panelPicker
            selectedPanel
        }
        .padding(16)
    }

    var header: some View {
        VStack(alignment: .leading, spacing: 6) {
            Text("SwiftCrossUI Desktop Example")
                .font(.largeTitle)
                .fontWeight(.bold)
            Text("Cross-platform desktop demo for macOS and Linux.")
            Text("Platform: \(model.platformLabel)")
                .font(.caption)
        }
    }

    var controls: some View {
        HStack {
            Button("Increment") { model.incrementCounter() }
            Button("Reset") { model.resetCounter() }
            Button("Log message") { model.appendMessage() }
            Spacer()
            Toggle("Enabled", isOn: Binding(
                get: { model.enabled },
                set: { model.enabled = $0 }
            ))
        }
    }

    var panelPicker: some View {
        HStack {
            Text("Panel")
            Picker(
                of: DesktopExamplePanel.allCases,
                selection: model.$selectedPanel
            )
        }
    }

    @ViewBuilder
    var selectedPanel: some View {
        switch model.selectedPanel ?? .overview {
            case .overview:
                overviewPanel
            case .controls:
                controlsPanel
            case .activity:
                activityPanel
        }
    }

    var overviewPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Overview").font(.headline)
            Text("Counter: \(model.counter)")
            Text("Status: \(model.enabled ? "Enabled" : "Disabled")")
        }
        .padding(12)
    }

    var controlsPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Controls").font(.headline)
            TextField(
                "Message",
                text: Binding(
                    get: { model.message },
                    set: { model.message = $0 }
                )
            )
            Text("Current message: \(model.message)")
        }
        .padding(12)
    }

    var activityPanel: some View {
        VStack(alignment: .leading, spacing: 8) {
            Text("Activity").font(.headline)
            if model.activityLog.isEmpty {
                Text("No activity yet.")
            }
            ForEach(model.activityLog, id: \.self) { entry in
                Text(entry)
                    .font(.caption.monospaced())
            }
        }
        .padding(12)
    }
}
