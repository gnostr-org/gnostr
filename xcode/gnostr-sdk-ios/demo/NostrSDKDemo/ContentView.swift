//
//  ContentView.swift
//  NostrSDKDemo
//
//  Created by Joel Klabo on 6/10/23.
//

import SwiftUI
import GnostrSDK 
import UIKit
import Combine
#if os(macOS)
import AppKit
#endif

struct SomeView: View {

    @Binding var relay: Relay?

    @State private var relayURLString = "wss://nos.lol"
    @State private var relayError: String?
    @State private var state: Relay.State = .notConnected
    @State private var stateCancellable: AnyCancellable?

    var body: some View {
        VStack(spacing: 12) {
            if relay?.state == .connected {
                Text("Connected to: \(relayURLString)")
                    .font(.footnote)
                Button(role: .destructive) {
                    relay?.disconnect()
                } label: {
                    Text("Disconnect")
                }
            } else {
                TextField(text: $relayURLString) {
                    Text("wss://nos.lol")
                }
                .textFieldStyle(.roundedBorder)
                .autocapitalization(.none)
                .autocorrectionDisabled()

                Button("Connect") {
                    attemptRelayConnect()
                }
                Text(relayError ?? status(state))
            }
        }
        .padding()
        .onAppear {
            attemptRelayConnect()
        }
    }

    private func attemptRelayConnect() {
        if let relayURL = URL(string: relayURLString.lowercased()) {
            do {
                relay = try Relay(url: relayURL)
                relay?.connect()
                stateCancellable = relay?.$state
                    .receive(on: DispatchQueue.main)
                    .sink { newState in
                        state = newState
                    }
            } catch {
                relayError = error.localizedDescription
            }
        } else {
            relayError = "Invalid URL String"
        }
    }

    private func status(_ state: Relay.State?) -> String {
        guard let state else {
            return "No status"
        }
        switch state {
        case .notConnected:
            return "Disconnected"
        case .connecting:
            return "Connecting"
        case .connected:
            return "Connected"
        case .error(let error):
            return error.localizedDescription
        }
    }
}

struct ContentView: View {

    @EnvironmentObject private var relayPool: RelayPool
    @EnvironmentObject private var identityStore: DemoIdentityStore
    @EnvironmentObject private var gitSettingsStore: DemoGitSettingsStore
    @EnvironmentObject private var appPrimeStore: DemoAppPrimeStore
    @EnvironmentObject private var repositoryHostStore: DemoRepositoryHostStore
    @State private var selectedDestination: SidebarDestination = .configureRelays
    @State private var navigationPath = NavigationPath()
    @State private var navigationResetToken = UUID()
    @State private var orientation: AppOrientation = .landscape

    var body: some View {
        layout
            #if os(macOS)
            .background(MacKeyEventMonitor(orientation: $orientation))
            #endif
    }

    @ViewBuilder
    private var layout: some View {
        switch orientation {
        case .landscape:
            HStack(spacing: 0) {
                sidebar
                    .frame(width: 64)

                Divider()

                NavigationStack(path: $navigationPath) {
                    detailView(for: selectedDestination)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .background(Color(.systemBackground))
                }
                .id(navigationResetToken)
            }
        case .portrait:
            VStack(spacing: 0) {
                sidebar
                    .frame(height: 280)

                Divider()

                NavigationStack(path: $navigationPath) {
                    detailView(for: selectedDestination)
                        .frame(maxWidth: .infinity, maxHeight: .infinity)
                        .background(Color(.systemBackground))
                }
                .id(navigationResetToken)
            }
        }
    }

    private var sidebar: some View {
        VStack(spacing: 0) {
            HStack {
                Image("GnostrIcon")
                    .resizable()
                    .scaledToFit()
                    .frame(width: 24, height: 24)
                    .accessibilityHidden(true)

                Spacer(minLength: 0)
            }
            .padding(.horizontal, 16)
            .padding(.vertical, 12)
            .background(Color(.clear))

            List {
                ForEach(SidebarDestination.sidebarCases) { destination in
                    Button {
                        navigationPath = NavigationPath()
                        navigationResetToken = UUID()
                        selectedDestination = destination
                    } label: {
                        HStack(spacing: 12) {
                            Image(systemName: destination.imageName)
                        }
                    }
                    .buttonStyle(.plain)
                    .listRowBackground(selectedDestination == destination ? Color(.tertiarySystemFill) : Color.clear)
                }
            }
            .listStyle(.sidebar)

            Spacer(minLength: 0)

            Button {
                navigationPath = NavigationPath()
                navigationResetToken = UUID()
                selectedDestination = .settings
            } label: {
                HStack(spacing: 12) {
                    Image(systemName: SidebarDestination.settings.imageName)
                }
                .frame(maxWidth: .infinity)
                .padding(.vertical, 12)
            }
            .buttonStyle(.plain)
            .background(selectedDestination == .settings ? Color(.tertiarySystemFill) : Color.clear)
        }
        .frame(maxHeight: .infinity, alignment: .top)
    }

    @ViewBuilder
    private func detailView(for destination: SidebarDestination) -> some View {
        switch destination {
        case .configureRelays:
            RelaysView()
        case .hostedRepositories:
            HostedRepositoriesView()
        case .nip0034Viewer:
            QueryRelayDemoView()
        case .nip04DirectMessage:
            LegacyDirectMessageDemoView()
        case .nip44Encrypt:
            EncryptMessageDemoView()
        case .nip44Decrypt:
            DecryptMessageDemoView()
        case .keyGeneration:
            GenerateKeyDemoView()
        case .nip05:
            NIP05VerficationDemoView()
        case .settings:
            SettingsView()
        }
    }
}

private enum AppOrientation {
    case portrait
    case landscape
}

#if os(macOS)
private struct MacKeyEventMonitor: NSViewRepresentable {
    @Binding var orientation: AppOrientation

    func makeCoordinator() -> Coordinator {
        Coordinator(orientation: $orientation)
    }

    func makeNSView(context: Context) -> NSView {
        context.coordinator.installMonitor()
        return NSView(frame: .zero)
    }

    func updateNSView(_ nsView: NSView, context: Context) {
        context.coordinator.orientation = $orientation
    }

    static func dismantleNSView(_ nsView: NSView, coordinator: Coordinator) {
        coordinator.removeMonitor()
    }

    final class Coordinator {
        var orientation: Binding<AppOrientation>
        private var monitor: Any?

        init(orientation: Binding<AppOrientation>) {
            self.orientation = orientation
        }

        func installMonitor() {
            guard monitor == nil else { return }

            monitor = NSEvent.addLocalMonitorForEvents(matching: .keyDown) { [weak self] event in
                self?.log(event: event)

                guard let self, event.modifierFlags.contains(.command) else {
                    return event
                }

                switch event.keyCode {
                case 123:
                    self.orientation.wrappedValue = .portrait
                    return nil
                case 124:
                    self.orientation.wrappedValue = .landscape
                    return nil
                default:
                    return event
                }
            }
        }

        func removeMonitor() {
            if let monitor {
                NSEvent.removeMonitor(monitor)
            }
            monitor = nil
        }

        private func log(event: NSEvent) {
            let modifiers = event.modifierFlags.intersection([.command, .shift, .option, .control, .capsLock])
            print("[MacKeyEvent] type=\(event.type.rawValue) keyCode=\(event.keyCode) characters=\(event.charactersIgnoringModifiers ?? "nil") modifiers=\(modifiers.rawValue)")
        }
    }
}
#endif

private enum SidebarDestination: String, CaseIterable, Identifiable {
    case configureRelays
    case hostedRepositories
    case nip0034Viewer
    case nip04DirectMessage
    case nip44Encrypt
    case nip44Decrypt
    case keyGeneration
    case nip05
    case settings

    var id: String { rawValue }

    var labelText: String {
        switch self {
        case .configureRelays: "Configure Relays"
        case .hostedRepositories: "Hosted Repos"
        case .nip0034Viewer: "NIP-0034 Viewer"
        case .nip04DirectMessage: "NIP-04 Direct Message"
        case .nip44Encrypt: "NIP-44 Encrypt"
        case .nip44Decrypt: "NIP-44 Decrypt"
        case .keyGeneration: "Key Generation"
        case .nip05: "NIP-05"
        case .settings: "49:Settings"
        }
    }

    var imageName: String {
        switch self {
        case .configureRelays:
            "network"
        case .hostedRepositories:
            "folder"
        case .nip0034Viewer:
            "list.bullet.rectangle.portrait"
        case .nip04DirectMessage, .nip44Encrypt, .nip44Decrypt:
            "list.bullet"
        case .keyGeneration:
            "key"
        case .nip05:
            "checkmark.seal"
        case .settings:
            "gearshape"
        }
    }

    var useAssetImage: Bool {
        false
    }

    static var sidebarCases: [SidebarDestination] {
        allCases.filter { $0 != .settings }
    }
}

struct ContentView_Previews: PreviewProvider {
    static var previews: some View {
        ContentView()
            .environmentObject(RelayPool(relays: []))
            .environmentObject(DemoIdentityStore())
            .environmentObject(DemoGitSettingsStore())
            .environmentObject(RelayDirectoryStore())
            .environmentObject(DemoAppPrimeStore())
            .environmentObject(DemoRepositoryHostStore())
    }
}

private struct SettingsView: View {
    @EnvironmentObject private var relayPool: RelayPool
    @EnvironmentObject private var identityStore: DemoIdentityStore
    @EnvironmentObject private var gitSettingsStore: DemoGitSettingsStore
    @EnvironmentObject private var appPrimeStore: DemoAppPrimeStore
    @EnvironmentObject private var repositoryHostStore: DemoRepositoryHostStore
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass
    @StateObject private var metadataLoader = PubkeyMetadataLoader()

    @State private var name = "name"
    @State private var displayName = "displayName"
    @State private var about = "about"
    @State private var website = "website"
    @State private var pictureURL = "pictureURL"
    @State private var bannerURL = "bannerURL"
    @State private var nostrAddress = "nostrAddress"
    @State private var isBot = false
    @State private var lud06 = "lud06"
    @State private var lud16 = "lud16"
    @State private var populatedPubkey: String?
    @State private var isPrivateKeyRevealed = false

    private var titleLineLimit: Int {
        horizontalSizeClass == .compact ? 1 : 2
    }

    var body: some View {
        Form {
            Section("110:") {//}
                //Spacer(minLength: 1)
                Section("") {
                    labeledTextField("111:Private Key",
                                     text: $identityStore.privateKeyInput,
                                     prompt: "nsec or hex",
                                     secure: true,
                                     isRevealed: $isPrivateKeyRevealed)

                    if let publicKeyHex = identityStore.publicKeyHex {
                        labeledTextField("118:Public Key",
                                         text: .constant(publicKeyHex),
                                         prompt: "npub or hex",
                                         secure: false,
                                         isEnabled: false)
                    } else {
                        Text("Enter a valid private key to load profile metadata.")
                            .font(.caption)
                            .foregroundColor(.primary)
                    }
                }

                Section("User Metadata") {
                    SettingsMetadataEditorView(name: $name,
                                               displayName: $displayName,
                                               about: $about,
                                               website: $website,
                                               pictureURL: $pictureURL,
                                               bannerURL: $bannerURL,
                                               nostrAddress: $nostrAddress,
                                               isBot: $isBot,
                                               lud06: $lud06,
                                               lud16: $lud16)
                }

                Section("Git Settings") {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("App Repos Root")
                            .font(.caption.weight(.semibold))

                        TextField("Application Support/NostrSDKDemo/HostedRepos", text: $gitSettingsStore.appRepositoriesRootPath)
                            .textInputAutocapitalization(.never)
                            .autocorrectionDisabled()

                        Text("Hosted repositories will be cloned into this folder instead of the default location.")
                            .font(.caption)
                            .foregroundColor(.secondary)

                        Button("Reset to Default") {
                            gitSettingsStore.resetAppRepositoriesRootPath()
                        }
                        .buttonStyle(.borderless)
                    }

                    NavigationLink(destination: HostedRepositoriesView()) {
                        Text("Hosted Repositories")
                    }

                    Text("Repository priming runs at app launch and collects clone tags from NIP-34 events.")
                        .font(.caption)
                        .foregroundColor(.secondary)
                }
            }
        }
        .safeAreaInset(edge: .top, spacing: 0) {
            SettingsNavHeaderView(metadata: metadataLoader.metadata)
                .padding(.horizontal)
                .padding(.top, 8)
        }
        .onAppear {
            metadataLoader.attach(relayPool: relayPool)
            refreshMetadata()
        }
        .onChange(of: identityStore.privateKeyInput) { _ in
            populatedPubkey = nil
            refreshMetadata()
        }
        .onReceive(metadataLoader.$metadata) { metadata in
            guard let metadata, let publicKeyHex = identityStore.publicKeyHex else { return }
            guard populatedPubkey != publicKeyHex else { return }
            populatedPubkey = publicKeyHex
            apply(metadata: metadata)
        }
    }

    private func refreshMetadata() {
            guard let publicKeyHex = identityStore.publicKeyHex else {
                metadataLoader.update(publicKeyInput: identityStore.privateKeyInput, isValid: false)
                return
            }

            metadataLoader.update(publicKeyInput: publicKeyHex, isValid: true)
    }

    private func apply(metadata: MetadataEvent) {
            name = metadata.name ?? "metadata.name"
            displayName = metadata.displayName ?? "metadata.displayName"
            about = metadata.about ?? "metadata.about"
            website = metadata.websiteURL?.absoluteString ?? "metadata.websiteURL?.absoluteString"
            pictureURL = metadata.pictureURL?.absoluteString ?? "metadata.pictureURL?.absoluteString"
            bannerURL = metadata.bannerPictureURL?.absoluteString ?? "metadata.bannerPictureURL?.absoluteString"
            nostrAddress = metadata.nostrAddress ?? "metadata.nostrAddress"
            isBot = metadata.isBot ?? false
            lud06 = metadata.lightningURLString ?? "metadata.lightningURLString"
            lud16 = metadata.lightningAddress ?? "metadata.lightningAddress"
    }

    private func labeledTextField(_ label: String,
                                  text: Binding<String>,
                                  prompt: String,
                                  secure: Bool = false,
                                  isRevealed: Binding<Bool> = .constant(false),
                                  isEnabled: Bool = true) -> some View {
            VStack(alignment: .leading, spacing: 4) {
                Text(label)
                    .font(.caption.weight(.semibold))
                    .foregroundColor(.primary)
                if secure {
                    HStack(spacing: 8) {
                        Group {
                            if isRevealed.wrappedValue {
                                TextField(prompt, text: text)
                            } else {
                                SecureField(prompt, text: text)
                            }
                        }
                        .textFieldStyle(.roundedBorder)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()

                        Button {
                            isRevealed.wrappedValue.toggle()
                        } label: {
                            Image(systemName: isRevealed.wrappedValue ? "eye.slash" : "eye")
                                .foregroundColor(.primary)
                        }
                        .buttonStyle(.plain)
                    }
                } else {
                    TextField(prompt, text: text)
                        .textFieldStyle(.roundedBorder)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
                }
            }
            .disabled(isEnabled == false)
    }

    private func labeledTextEditor(_ label: String, text: Binding<String>, prompt: String) -> some View {
            VStack(alignment: .leading, spacing: 4) {
                Text(label)
                    .font(.caption.weight(.semibold))
                    .foregroundColor(.primary)
                TextEditor(text: text)
                    .frame(minHeight: 88)
                    .padding(6)
                    .overlay(
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .stroke(Color(.separator).opacity(0.15))
                    )
                    .overlay(
                        Group {
                            if text.wrappedValue.isEmpty {
                                Text(prompt)
                                    .foregroundColor(.primary)
                                    .padding(.leading, 12)
                                    .padding(.top, 14)
                            }
                        },
                        alignment: .topLeading
                    )
            }
    }

    private func labeledValue(_ label: String, value: String) -> some View {
            VStack(alignment: .leading, spacing: 4) {
                Text(label)
                    .font(.caption.weight(.semibold))
                    .foregroundColor(.primary)

                Text(value)
                    .font(.system(size: 16, weight: .regular, design: .monospaced))
                    .foregroundColor(.primary)
                    .lineLimit(1)
                    .minimumScaleFactor(0.6)
                    .allowsTightening(true)
                    .textSelection(.enabled)
                    .padding(.horizontal, 10)
                    .padding(.vertical, 8)
                    .frame(maxWidth: .infinity, alignment: .leading)
                    .background(
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .fill(Color(.secondarySystemBackground))
                    )
                    .overlay(
                        RoundedRectangle(cornerRadius: 8, style: .continuous)
                            .stroke(Color(.separator).opacity(0.15))
                    )
            }
    }
}

private struct SettingsNavHeaderView: View {
    let metadata: MetadataEvent?

    var body: some View {
        ContextAwareHeaderView(
            title: metadata?.displayName ?? metadata?.name ?? "Settings",
            subtitle: metadata?.about ?? "Profile, relays, and git settings.",
            systemImage: "gearshape",
            bannerURL: metadata?.bannerPictureURL,
            bannerHeight: 180,
            accessory: {
               SettingsAvatarView(metadata: metadata, size: 52)
            }
        )
        .frame(maxWidth: .infinity, alignment: .leading)
    }
}

private struct SettingsAvatarView: View {
    let metadata: MetadataEvent?
    let size: CGFloat

    var body: some View {
        ZStack {
            Circle()
                .fill(Color(.secondarySystemBackground))

            if let pictureURL = metadata?.pictureURL {
                SettingsRemoteImageView(url: pictureURL)
            } else {
                Image("GnostrIcon")
                    .resizable()
                    .scaledToFit()
                    .padding(size * 0.2)
            }
        }
        .frame(width: size, height: size)
        .clipShape(Circle())
        .overlay(Circle().stroke(Color(.separator).opacity(0.2), lineWidth: 1))
    }
}

private struct SettingsRemoteImageView: UIViewRepresentable {
    let url: URL

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> UIImageView {
        let imageView = UIImageView()
        imageView.contentMode = .scaleAspectFill
        imageView.clipsToBounds = true
        imageView.backgroundColor = .systemBackground
        context.coordinator.load(url: url, into: imageView)
        return imageView
    }

    func updateUIView(_ uiView: UIImageView, context: Context) {
        context.coordinator.load(url: url, into: uiView)
    }

    final class Coordinator {
        private var task: Task<Void, Never>?
        private var currentURL: URL?

        func load(url: URL, into imageView: UIImageView) {
            currentURL = url

            task?.cancel()
            // Load remote header imagery off the main actor so the settings screen stays responsive.
            task = Task.detached(priority: .background) { [weak imageView] in
                do {
                    let (data, _) = try await URLSession.shared.data(from: url)
                    guard let image = UIImage(data: data) else { return }

                    await MainActor.run {
                        guard let imageView, self.currentURL == url else { return }
                        imageView.image = image
                    }
                } catch {
                }
            }
        }
    }
}

private struct SettingsMetadataEditorView: View {
    @Binding var name: String
    @Binding var displayName: String
    @Binding var about: String
    @Binding var website: String
    @Binding var pictureURL: String
    @Binding var bannerURL: String
    @Binding var nostrAddress: String
    @Binding var isBot: Bool
    @Binding var lud06: String
    @Binding var lud16: String

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            ResponsiveTextFieldRow(label: "Name", text: $name, prompt: "name")
            ResponsiveTextFieldRow(label: "Display Name", text: $displayName, prompt: "display name")
            ResponsiveTextEditorRow(label: "About", text: $about, prompt: "about")
            ResponsiveTextFieldRow(label: "Website", text: $website, prompt: "https://...")
            ResponsiveTextFieldRow(label: "Picture URL", text: $pictureURL, prompt: "https://...")
            ResponsiveTextFieldRow(label: "Banner URL", text: $bannerURL, prompt: "https://...")
            ResponsiveTextFieldRow(label: "NIP-05", text: $nostrAddress, prompt: "name@example.com")
            ResponsiveToggleRow(label: "Bot", isOn: $isBot)
            ResponsiveTextFieldRow(label: "LUD-06", text: $lud06, prompt: "lnurl...")
            ResponsiveTextFieldRow(label: "LUD-16", text: $lud16, prompt: "name@domain.com")
        }
    }
}

private struct ResponsiveTextFieldRow: View {
    let label: String
    @Binding var text: String
    let prompt: String

    var body: some View {
        ViewThatFits(in: .horizontal) {
            HStack(alignment: .firstTextBaseline, spacing: 12) {
                    Text(label)
                        .font(.caption.weight(.semibold))
                        .foregroundColor(.primary)
                        .frame(width: 120, alignment: .leading)
                    TextField(prompt, text: $text)
                        .textFieldStyle(.roundedBorder)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
            }

            VStack(alignment: .leading, spacing: 4) {
                    Text(label)
                        .font(.caption.weight(.semibold))
                        .foregroundColor(.primary)
                    TextField(prompt, text: $text)
                        .textFieldStyle(.roundedBorder)
                        .textInputAutocapitalization(.never)
                        .autocorrectionDisabled()
            }
        }
    }
}

private struct ResponsiveTextEditorRow: View {
    let label: String
    @Binding var text: String
    let prompt: String

    var body: some View {
        ViewThatFits(in: .horizontal) {
            HStack(alignment: .top, spacing: 12) {
                    Text(label)
                        .font(.caption.weight(.semibold))
                        .foregroundColor(.primary)
                        .frame(width: 120, alignment: .leading)
                    editor
            }

            VStack(alignment: .leading, spacing: 4) {
                    Text(label)
                        .font(.caption.weight(.semibold))
                        .foregroundColor(.primary)
                    editor
            }
        }
    }

    private var editor: some View {
        TextEditor(text: $text)
            .frame(minHeight: 88)
            .padding(6)
            .overlay(
                    RoundedRectangle(cornerRadius: 8, style: .continuous)
                        .stroke(Color(.separator).opacity(0.15))
            )
            .overlay(
                    Group {
                        if text.isEmpty {
                            Text(prompt)
                                .foregroundColor(.primary)
                                .padding(.leading, 12)
                                .padding(.top, 14)
                        }
                    },
                    alignment: .topLeading
            )
    }
}

private struct ResponsiveToggleRow: View {
    let label: String
    @Binding var isOn: Bool

    var body: some View {
        HStack {
            Text(label)
                    .font(.caption.weight(.semibold))
                    .foregroundColor(.primary)
            //Spacer(minLength: 0)
            Toggle("", isOn: $isOn)
                    .labelsHidden()
                    .tint(.accentColor)
        }
    }
}

private struct SettingsProfileSummaryView: View {
    let metadata: MetadataEvent?
    let publicKeyHex: String?
    let titleLineLimit: Int

    @Environment(\.horizontalSizeClass) private var horizontalSizeClass

    private var aboutLineLimit: Int {
        horizontalSizeClass == .compact ? 2 : 3
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 10) {
            //Spacer(minLength: 0)
            HStack(alignment: .firstTextBaseline, spacing: 8) {
            //        Text("383:Public Key")
            //            .font(.caption2.weight(.bold))
            //            .foregroundColor(.primary)
            //        Spacer(minLength: 0)
            //        if let publicKeyHex {
            //            Text(publicKeyHex)
            //                .font(.caption.monospaced())
            //                .foregroundColor(.primary)
            //                .lineLimit(1)
            //                .minimumScaleFactor(0.6)
            //                .allowsTightening(true)
            //        } else {
            //            Text("Enter a valid private key")
            //                .font(.caption)
            //                .foregroundColor(.primary)
            //        }
            // //}

            if let metadata {
                    if let title = metadata.displayName ?? metadata.name ?? metadata.nostrAddress {
                        Text(title)
                            .font(.headline)
                            .foregroundColor(.primary)
                            .lineLimit(titleLineLimit)
                    }

                    if let nostrAddress = metadata.nostrAddress {
                        Text(nostrAddress)
                            .font(.caption)
                            .foregroundColor(.primary)
                    }

                    if let about = metadata.about, !about.isEmpty {
                        Text(about)
                            .font(.caption)
                            .foregroundColor(.primary)
                            .lineLimit(aboutLineLimit)
                    }

                    if let website = metadata.websiteURL?.absoluteString {
                        Text(website)
                            .font(.caption.monospaced())
                            .foregroundColor(.primary)
                            .lineLimit(1)
                            .minimumScaleFactor(0.6)
                            .allowsTightening(true)
                    }
            } else {
                    Text("Profile preview appears after a valid private key is entered.")
                        .font(.caption)
                        .foregroundColor(.primary)
            }
            }
        }
    }
}
