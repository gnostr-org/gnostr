//
//  ContentView.swift
//  GnostrP2P-KitchenSink
//

import Foundation
import SwiftUI

enum KitchenSinkTab: String, CaseIterable, Hashable {
    case overview = "Overview"
    case p2p = "P2P"
    case chat = "Chat"
    case controls = "Controls"
    case lists = "Lists"
    case presentation = "Presentation"
    case activity = "Activity"
}

@MainActor
final class KitchenSinkViewModel: ObservableObject {
    @Published var selectedTab: KitchenSinkTab = .overview
    @Published var isEnabled = true
    @Published var counter = 0
    @Published var sliderValue = 42.0
    @Published var stepperValue = 3
    @Published var progressValue = 0.35
    @Published var favoriteMode = "Balanced"
    @Published var username = ""
    @Published var password = ""
    @Published var notes = "A kitchen sink app for iOS, iPadOS, and Catalyst."
    @Published var selectedDate = Date()
    @Published var selectedColor = Color.blue
    @Published var newItemText = ""
    @Published var items = ["Alpha", "Beta", "Gamma"]
    @Published var selectedItem: String?
    @Published var isSheetPresented = false
    @Published var isAlertPresented = false
    @Published var isConfirmationPresented = false
    @Published var activityLog: [String] = []

    init() {
        log("Kitchen sink ready")
    }

    var platformLabel: String {
        #if targetEnvironment(macCatalyst)
            return "Mac Catalyst"
        #elseif os(iOS)
            return "iOS / iPadOS"
        #elseif os(macOS)
            return "macOS"
        #else
            return "Other"
        #endif
    }

    func incrementCounter() {
        counter += 1
        log("Counter -> \(counter)")
    }

    func resetCounter() {
        counter = 0
        log("Counter reset")
    }

    func randomizeControls() {
        sliderValue = Double.random(in: 0...100)
        stepperValue = Int.random(in: 0...10)
        progressValue = Double.random(in: 0...1)
        selectedColor = [.red, .orange, .yellow, .green, .blue, .purple].randomElement() ?? .blue
        selectedDate = Date().addingTimeInterval(Double.random(in: -86_400...86_400))
        log("Randomized controls")
    }

    func addItem() {
        let trimmed = newItemText.trimmingCharacters(in: .whitespacesAndNewlines)
        guard !trimmed.isEmpty else { return }
        items.insert(trimmed, at: 0)
        selectedItem = trimmed
        newItemText = ""
        log("Added item \(trimmed)")
    }

    func removeSelectedItem() {
        guard let selectedItem,
              let index = items.firstIndex(of: selectedItem)
        else { return }
        items.remove(at: index)
        self.selectedItem = items.first
        log("Removed item \(selectedItem)")
    }

    func log(_ message: String) {
        let formatter = Self.timestampFormatter
        activityLog.insert("[\(formatter.string(from: Date()))] \(message)", at: 0)
    }

    private static let timestampFormatter: DateFormatter = {
        let formatter = DateFormatter()
        formatter.dateFormat = "HH:mm:ss"
        return formatter
    }()
}

struct ContentView: View {
    @StateObject private var model = KitchenSinkViewModel()
    @StateObject private var p2p = P2PService()

    var body: some View {
        TabView(selection: $model.selectedTab) {
            overviewTab
                .tabItem { Label("Overview", systemImage: "house") }
                .tag(KitchenSinkTab.overview)

            p2pTab
                .tabItem { Label("P2P", systemImage: "network") }
                .tag(KitchenSinkTab.p2p)

            chatTab
                .tabItem { Label("Chat", systemImage: "message") }
                .tag(KitchenSinkTab.chat)

            controlsTab
                .tabItem { Label("Controls", systemImage: "slider.horizontal.3") }
                .tag(KitchenSinkTab.controls)

            listsTab
                .tabItem { Label("Lists", systemImage: "list.bullet") }
                .tag(KitchenSinkTab.lists)

            presentationTab
                .tabItem { Label("Presentation", systemImage: "square.on.square") }
                .tag(KitchenSinkTab.presentation)

            activityTab
                .tabItem { Label("Activity", systemImage: "text.bubble") }
                .tag(KitchenSinkTab.activity)
        }
        .task {
            p2p.start()
        }
        .sheet(isPresented: $model.isSheetPresented) {
            KitchenSinkSheetView(counter: model.counter)
        }
        .alert("Kitchen Sink Alert", isPresented: $model.isAlertPresented) {
            Button("OK", role: .cancel) {}
        } message: {
            Text("SwiftUI alert, sheet, picker, list, and form demos are all available here.")
        }
        .confirmationDialog("Choose a mode", isPresented: $model.isConfirmationPresented, titleVisibility: .visible) {
            Button("Balanced") {
                model.favoriteMode = "Balanced"
                model.log("Mode -> Balanced")
            }
            Button("Performance") {
                model.favoriteMode = "Performance"
                model.log("Mode -> Performance")
            }
            Button("Debug") {
                model.favoriteMode = "Debug"
                model.log("Mode -> Debug")
            }
            Button("Cancel", role: .cancel) {}
        }
    }

    private var overviewTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "GnostrP2P Kitchen Sink",
                    subtitle: "A cross-platform SwiftUI app for iOS, iPadOS, and Mac Catalyst."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        statRow(label: "Platform", value: model.platformLabel)
                        statRow(label: "Counter", value: "\(model.counter)")
                        statRow(label: "Mode", value: model.favoriteMode)
                        statRow(label: "Items", value: "\(model.items.count)")
                        statRow(label: "P2P state", value: p2pStateLabel)
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Notes")
                            .font(.headline)
                        TextEditor(text: $model.notes)
                            .frame(minHeight: 140)
                            .overlay(RoundedRectangle(cornerRadius: 8).stroke(.quaternary))
                    }
                }
            }
            .padding(16)
        }
    }

    private var p2pTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "P2P",
                    subtitle: "Start a libp2p node, discover peers, and inspect listen addresses."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 10) {
                        statRow(label: "Runtime profile", value: p2p.runtimeProfile)
                        statRow(label: "Peer ID", value: p2p.peerIDString)
                            .textSelection(.enabled)
                        statRow(label: "Listen port", value: "\(p2p.listenPort)")
                        statRow(label: "State", value: p2pStateLabel)

                        HStack {
                            Button("Start node") { p2p.start() }
                            Button("Stop node") { p2p.stop() }
                                .disabled(!p2p.isRunning)
                            Button("Restart") { p2p.restart() }
                        }

                        HStack {
                            TextField(
                                "Ping message",
                                text: Binding(
                                    get: { p2p.draftMessage },
                                    set: { p2p.draftMessage = $0 }
                                )
                            )
                            Button("Queue ping") { p2p.sendLocalPing() }
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Listen addresses")
                            .font(.title3.weight(.semibold))
                        if p2p.listenAddresses.isEmpty {
                            Text("Start the node to populate listen addresses.")
                                .font(.callout)
                        }
                        ForEach(p2p.listenAddresses, id: \.self) { address in
                            Text(address)
                                .font(.callout.monospaced())
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Discovered peers")
                            .font(.title3.weight(.semibold))
                        if p2p.discoveredPeers.isEmpty {
                            Text("No peers discovered yet.")
                                .font(.callout)
                        }
                        ForEach(p2p.discoveredPeers) { peer in
                            VStack(alignment: .leading, spacing: 4) {
                                Text(peer.peerID)
                                    .font(.callout.monospaced())
                                ForEach(peer.addresses, id: \.self) { address in
                                    Text(address)
                                        .font(.callout)
                                }
                            }
                            .padding(.vertical, 6)
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Activity")
                            .font(.title3.weight(.semibold))
                        if let lastError = p2p.lastError {
                            Text("Last error: \(lastError)")
                                .foregroundStyle(.red)
                        }
                        Button("Clear") {
                            p2p.clearActivityLog()
                        }
                        ForEach(p2p.activityLog, id: \.self) { line in
                            Text(line)
                                .font(.callout.monospaced())
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private var chatTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "Chat",
                    subtitle: "Join a pubsub topic, send messages, and watch live updates."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        statRow(label: "Node state", value: p2pStateLabel)
                        statRow(label: "Current topic", value: p2p.chatTopic)
                        statRow(label: "Display name", value: p2p.chatDisplayName)

                        HStack {
                            TextField("Topic", text: $p2p.chatTopic)
                                .textFieldStyle(.roundedBorder)
                                .onSubmit {
                                    p2p.joinChatTopic()
                                }
                            Button("Join topic") {
                                p2p.joinChatTopic()
                            }
                        }

                        HStack {
                            TextField("Display name", text: $p2p.chatDisplayName)
                                .textFieldStyle(.roundedBorder)
                        }

                        HStack {
                            TextField("Message", text: $p2p.chatDraftMessage)
                                .textFieldStyle(.roundedBorder)
                            Button("Send") {
                                p2p.sendChatMessage()
                            }
                            .disabled(p2p.chatDraftMessage.trimmingCharacters(in: .whitespacesAndNewlines).isEmpty)
                        }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Text("Messages")
                            .font(.title3.weight(.semibold))
                        if p2p.chatMessages.isEmpty {
                            Text("No chat messages yet. Join \(p2p.chatTopic) and send one.")
                                .font(.callout)
                        }
                        LazyVStack(alignment: .leading, spacing: 10) {
                            ForEach(p2p.chatMessages) { entry in
                                VStack(alignment: .leading, spacing: 8) {
                                    HStack {
                                        Text(entry.topic)
                                            .font(.headline)
                                        Text(entry.kind)
                                            .font(.caption2.bold())
                                            .padding(.horizontal, 6)
                                            .padding(.vertical, 2)
                                            .background(.blue.opacity(0.18))
                                            .clipShape(Capsule())
                                        if entry.isLocal {
                                            Text("local")
                                                .font(.caption2.bold())
                                                .padding(.horizontal, 6)
                                                .padding(.vertical, 2)
                                                .background(.green.opacity(0.2))
                                                .clipShape(Capsule())
                                        }
                                        Spacer()
                                        Text(entry.timestamp, style: .time)
                                            .font(.callout)
                                            .foregroundStyle(.secondary)
                                    }
                                    Text(entry.author)
                                        .font(.subheadline.monospaced())
                                        .foregroundStyle(.secondary)
                                    Text(entry.text)
                                        .font(.body)
                                }
                                .padding(12)
                                .frame(maxWidth: .infinity, alignment: .leading)
                                .background(
                                    RoundedRectangle(cornerRadius: 14, style: .continuous)
                                        .fill(Color.secondary.opacity(0.08))
                                )
                            }
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private var controlsTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "Controls",
                    subtitle: "Buttons, pickers, sliders, text fields, and date controls."
                )

                GroupBox {
                    Form {
                        Toggle("Enabled", isOn: $model.isEnabled)

                        Picker("Favorite mode", selection: $model.favoriteMode) {
                            Text("Balanced").tag("Balanced")
                            Text("Performance").tag("Performance")
                            Text("Debug").tag("Debug")
                        }

                        ColorPicker("Accent color", selection: $model.selectedColor)

                        Slider(value: $model.sliderValue, in: 0...100)
                        Stepper("Stepper value: \(model.stepperValue)", value: $model.stepperValue, in: 0...10)

                        DatePicker("Selected date", selection: $model.selectedDate)

                        TextField("Username", text: $model.username)
                        SecureField("Password", text: $model.password)
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        ProgressView(value: model.progressValue)
                        HStack {
                            Button("Increment") { model.incrementCounter() }
                            Button("Reset") { model.resetCounter() }
                            Button("Randomize") { model.randomizeControls() }
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private var listsTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "Lists",
                    subtitle: "Add, select, and remove items."
                )

                listsPanel
            }
            .padding(16)
        }
    }

    private var listsPanel: some View {
        GroupBox {
            VStack(alignment: .leading, spacing: 12) {
                HStack {
                    TextField("New item", text: $model.newItemText)
                    Button("Add") { model.addItem() }
                }

                selectedItemPicker
                itemList
            }
        }
    }

    private var selectedItemPicker: some View {
        Picker("Selected item", selection: $model.selectedItem) {
            Text("None").tag(String?.none)
            ForEach(model.items, id: \.self) { item in
                Text(item).tag(Optional(item))
            }
        }
    }

    private var itemList: some View {
        List {
            ForEach(model.items, id: \.self) { item in
                itemRow(item)
            }
            .onDelete(perform: deleteItems)
        }
        .frame(minHeight: 240)
    }

    @ViewBuilder
    private func itemRow(_ item: String) -> some View {
        Button {
            model.selectedItem = item
            model.log("Selected \(item)")
        } label: {
            HStack {
                Text(item)
                Spacer()
                if model.selectedItem == item {
                    Image(systemName: "checkmark.circle.fill")
                        .foregroundStyle(.tint)
                }
            }
        }
    }

    private func deleteItems(at offsets: IndexSet) {
        for index in offsets.sorted(by: >) {
            let removed = model.items[index]
            model.items.remove(at: index)
            model.log("Deleted \(removed)")
        }
        model.selectedItem = model.items.first
    }

    private var presentationTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "Presentation",
                    subtitle: "Sheet, alert, and confirmation dialog demos."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 12) {
                        Button("Show sheet") { model.isSheetPresented = true }
                        Button("Show alert") { model.isAlertPresented = true }
                        Button("Show confirmation dialog") { model.isConfirmationPresented = true }
                    }
                }

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Current mode: \(model.favoriteMode)")
                        Text("Selected color")
                        RoundedRectangle(cornerRadius: 10)
                            .fill(model.selectedColor)
                            .frame(height: 48)
                    }
                }
            }
            .padding(16)
        }
    }

    private var activityTab: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 16) {
                titleBlock(
                    title: "Activity",
                    subtitle: "Recent UI actions and state changes."
                )

                GroupBox {
                    VStack(alignment: .leading, spacing: 8) {
                        Button("Add log entry") {
                            model.log("Manual log entry")
                        }
                        ForEach(model.activityLog, id: \.self) { entry in
                            Text(entry)
                                .font(.callout.monospaced())
                                .frame(maxWidth: .infinity, alignment: .leading)
                        }
                    }
                }
            }
            .padding(16)
        }
    }

    private func titleBlock(title: String, subtitle: String) -> some View {
        VStack(alignment: .leading, spacing: 8) {
            Text(title)
                .font(.largeTitle.bold())
            Text(subtitle)
                .font(.title3)
                .foregroundStyle(.secondary)
        }
    }

    private func statRow(label: String, value: String) -> some View {
        HStack {
            Text(label)
                .font(.subheadline.weight(.semibold))
            Spacer()
            Text(value)
                .font(.body)
        }
    }

    private var p2pStateLabel: String {
        p2p.state.rawValue.capitalized
    }
}

private struct KitchenSinkSheetView: View {
    let counter: Int

    var body: some View {
        VStack(spacing: 16) {
            Text("Kitchen Sink Sheet")
                .font(.title.bold())
            Text("Counter snapshot: \(counter)")
            Button("Dismiss") {
                dismiss()
            }
        }
        .padding(24)
        .frame(minWidth: 320, minHeight: 220)
    }

    @Environment(\.dismiss) private var dismiss
}
