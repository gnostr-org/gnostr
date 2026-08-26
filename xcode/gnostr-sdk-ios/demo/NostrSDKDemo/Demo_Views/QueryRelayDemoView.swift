//
//  QueryRelayDemoView.swift
//  NostrSDKDemo
//
//  Created by Joel Klabo on 6/15/23.
//

import SwiftUI
import GnostrSDK
import Combine
import UIKit

private struct TinyWebImageView: UIViewRepresentable {
    let url: URL

    func makeCoordinator() -> Coordinator {
        Coordinator()
    }

    func makeUIView(context: Context) -> UIImageView {
        let imageView = UIImageView()
        imageView.contentMode = .scaleAspectFill
        imageView.clipsToBounds = true
        imageView.backgroundColor = .clear
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
            print("[QueryRelayDemo] image load attempt url=\(url.absoluteString)")

            if let cachedImage = RemoteImageLoader.cache.object(forKey: url as NSURL) {
                print("[QueryRelayDemo] image load cache hit url=\(url.absoluteString)")
                imageView.image = cachedImage
                return
            }

            task?.cancel()
            // Load preview images in the background and only publish the final bitmap on the main actor.
            task = Task.detached(priority: .background) { [weak imageView] in
                do {
                    let (data, _) = try await URLSession.shared.data(from: url)
                    guard let loadedImage = UIImage(data: data) else {
                        print("[QueryRelayDemo] image decode failed url=\(url.absoluteString)")
                        return
                    }

                    RemoteImageLoader.cache.setObject(loadedImage, forKey: url as NSURL)

                    await MainActor.run {
                        guard let imageView, self.currentURL == url else { return }
                        imageView.image = loadedImage
                    }
                } catch is CancellationError {
                } catch {
                    print("[QueryRelayDemo] image load failed url=\(url.absoluteString) error=\(error)")
                }
            }
        }
    }
}

private final class RemoteImageLoader {
    static let cache = NSCache<NSURL, UIImage>()
}

private final class RemoteImagePrefetcher {
    static let shared = RemoteImagePrefetcher()

    func prefetch(url: URL?) {
        guard let url else { return }
        guard RemoteImageLoader.cache.object(forKey: url as NSURL) == nil else { return }

        // Prefetch images off the main actor so event cards can render without waiting on network I/O.
        Task.detached(priority: .background) {
            print("[QueryRelayDemo] image prefetch attempt url=\(url.absoluteString)")
            do {
                let (data, _) = try await URLSession.shared.data(from: url)
                guard let loadedImage = UIImage(data: data) else {
                    print("[QueryRelayDemo] image decode failed url=\(url.absoluteString)")
                    return
                }

                RemoteImageLoader.cache.setObject(loadedImage, forKey: url as NSURL)
            } catch {
                print("[QueryRelayDemo] image prefetch failed url=\(url.absoluteString) error=\(error)")
            }
        }
    }
}

private func normalizedCloneURL(from value: String) -> URL? {
    if let url = URL(string: value), url.scheme != nil {
        return url
    }

    guard value.contains("@"), value.contains(":"), value.contains("://") == false else {
        return nil
    }

    let components = value.split(separator: ":", maxSplits: 1, omittingEmptySubsequences: false)
    guard components.count == 2 else { return nil }
    return URL(string: "ssh://\(components[0])/\(components[1])")
}

private struct EventCardView: View {
    let event: NostrEvent
    let metadata: MetadataEvent?
    let eventByID: [String: NostrEvent]
    let eventByCoordinate: [String: NostrEvent]
    let repoEventByRepoIDAndKind: [String: [Int: NostrEvent]]
    @EnvironmentObject private var repositoryHostStore: DemoRepositoryHostStore
    @Environment(\.verticalSizeClass) private var verticalSizeClass

    private struct TagItem: Hashable {
        let label: String
        let value: String
        let tagName: String
        let targetValue: String?
        let target: TagTarget?
    }

    private enum TagTarget: Hashable {
        case repo(String)
        case pubkey(String)
        case event(String)
        case coordinate(String)
        case clone(URL)
        case url(URL)
    }

    init(event: NostrEvent,
         metadata: MetadataEvent?,
         eventByID: [String: NostrEvent],
         eventByCoordinate: [String: NostrEvent],
         repoEventByRepoIDAndKind: [String: [Int: NostrEvent]]) {
        self.event = event
        self.metadata = metadata
        self.eventByID = eventByID
        self.eventByCoordinate = eventByCoordinate
        self.repoEventByRepoIDAndKind = repoEventByRepoIDAndKind
    }

    private var title: String? {
        tagValue("name") ?? tagValue("description") ?? tagValue("alt")
    }

    private var subtitle: String? {
        tagValue("description") ?? tagValue("alt")
    }

    private var repoID: String? {
        tagValue("d")
    }

    private var cloneURL: String? {
        tagValue("clone")
    }

    private var webURL: String? {
        tagValue("web")
    }

    private var relaysText: String? {
        values(for: "relays").first
    }

    private var maintainersText: String? {
        // Keep the raw maintainer edge visible as display text in tag chips.
        let values = values(for: "maintainers")
        return values.isEmpty ? nil : values.joined(separator: ", ")
    }

    private var maintainerPubkeys: [String] {
        var seen = Set<String>()
        return values(for: "maintainers").filter { seen.insert($0).inserted }
    }

    private var isPatch: Bool {
        event.kind.rawValue == 1617
    }

    private var relatedRepositoryEventTitle: String {
        switch event.kind.rawValue {
        case 1631:
            return "Related patch"
        case 30618:
            return "Related repository announcement"
        default:
            return "Related event"
        }
    }

    private var cardTags: [TagItem] {
        // Promote the semantic fields first, then append any remaining raw tags.
        let primaryTags: [TagItem?] = [
            tagItem(label: "Repo", value: repoID, tagName: "d"),
            tagItem(label: "Name", value: tagValue("name"), tagName: "name"),
            tagItem(label: "Description", value: tagValue("description"), tagName: "description"),
            tagItem(label: "Clone", value: shortDisplayValue(cloneURL), tagName: "clone", targetValue: cloneURL),
            tagItem(label: "Web", value: shortDisplayValue(webURL), tagName: "web", targetValue: webURL),
            tagItem(label: "Relays", value: relaysText, tagName: "relays"),
            tagItem(label: "Maintainers", value: maintainersText, tagName: "maintainers"),
            tagItem(label: "Alt", value: tagValue("alt"), tagName: "alt")
        ]

        let primaryLabels = Set(primaryTags.compactMap { $0?.label.lowercased() })
        let extraTags = event.tags.compactMap { tag -> TagItem? in
            let label = tag.name.capitalized
            guard primaryLabels.contains(label.lowercased()) == false else { return nil }
            return tagItem(label: label, value: tag.value, tagName: tag.name)
        }

        return (primaryTags.compactMap { $0 } + extraTags).prefix(8).map { $0 }
    }

    private var titleFont: Font {
        verticalSizeClass == .regular ? .system(size: 12, weight: .semibold, design: .default) : .headline
    }

    private var pubkeyFont: Font {
        verticalSizeClass == .regular ? .system(size: 7, weight: .regular, design: .monospaced) : .caption2.monospaced()
    }

    var body: some View {
        HStack(alignment: .top, spacing: 12) {
            avatar
                .frame(width: 72)
                .frame(maxHeight: .infinity, alignment: .top)

            VStack(alignment: .leading, spacing: 0) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text("ID: \(event.id)")
                        .font(.system(size: 13, weight: .semibold, design: .monospaced))
                        .foregroundColor(.primary)
                        .lineLimit(1)
                        .minimumScaleFactor(0.6)
                        .allowsTightening(true)
                        .layoutPriority(1)
                }

                if let title {
                    Text(title)
                        .font(titleFont)
                        .padding(.top, 1)
                }

                if isPatch {
                    Text("PATCH")
                        .font(.caption2.weight(.bold))
                        .foregroundColor(.primary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(
                            Capsule(style: .continuous)
                                .fill(Color.accentColor.opacity(0.18))
                        )
                }

                if let subtitle {
                    Text(subtitle)
                        .font(.caption)
                        .foregroundColor(.primary)
                        .lineLimit(2)
                }

                if let repoID {
                    Text(repoID)
                        .font(.caption.monospaced())
                        .foregroundColor(.primary)
                        .lineLimit(1)
                }

                HStack(alignment: .center, spacing: 8) {
                    Text("Kind \(event.kind.rawValue, format: .number.grouping(.never))")
                        .font(.caption2.weight(.bold))
                        .foregroundColor(.primary)
                        .padding(.horizontal, 8)
                        .padding(.vertical, 4)
                        .background(
                            Capsule(style: .continuous)
                                .fill(Color.accentColor.opacity(0.12))
                        )
                    Text(event.createdDate.formatted(date: .abbreviated, time: .shortened))
                        .font(.caption)
                        .foregroundColor(.primary)
                    Spacer(minLength: 0)
                }

                if !event.content.isEmpty {
                    Text(event.content)
                        .font(.callout.monospaced())
                        .padding(10)
                        .frame(maxWidth: .infinity, alignment: .leading)
                        .background(
                            RoundedRectangle(cornerRadius: 12, style: .continuous)
                                .fill(Color(.tertiarySystemFill))
                        )
                        .overlay(
                            RoundedRectangle(cornerRadius: 12, style: .continuous)
                                .stroke(Color(.separator).opacity(0.12))
                        )
                        .lineLimit(6)
                }

                if let cloneURL {
                    Text(cloneURL)
                        .font(.caption)
                        .foregroundColor(.primary)
                        .lineLimit(1)
                }

                if cardTags.isEmpty == false {
                    VStack(alignment: .leading, spacing: 10) {
                        ForEach(cardTags, id: \.self) { item in
                            if item.label == "Maintainers" {
                                MaintainersTagValueView(pubkeys: maintainerPubkeys)
                            } else {
                                tagRow(for: item)
                            }
                        }
                    }
                    .padding(.top, 2)
                }
            }

        }
        .padding(16)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .fill(Color(.secondarySystemBackground))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color(.separator).opacity(0.15))
        )
    }

    @ViewBuilder
    private var profileImage: some View {
        if let pictureURL = metadata?.pictureURL {
            TinyWebImageView(url: pictureURL)
                .frame(maxWidth: .infinity)
                .frame(height: 180)
                .clipShape(RoundedRectangle(cornerRadius: 14, style: .continuous))
        }
    }

    @ViewBuilder
    private var avatar: some View {
        if let pictureURL = metadata?.pictureURL {
            TinyWebImageView(url: pictureURL)
                .frame(width: 72, height: 72)
                .clipShape(Circle())
                .overlay(Circle().stroke(Color(.separator).opacity(0.2), lineWidth: 1))
        } else {
            avatarPlaceholder
        }
    }

    private var avatarPlaceholder: some View {
        ZStack {
            Circle()
                .fill(Color(.tertiarySystemFill))
            Image("GnostrIcon")
                .resizable()
                .scaledToFit()
                .padding(12)
        }
        .frame(width: 72, height: 72)
    }

    private func shortPubkey(_ pubkey: String) -> String {
        guard pubkey.count > 16 else { return pubkey }
        return "\(pubkey.prefix(8))...\(pubkey.suffix(8))"
    }

    private func tagValue(_ name: String) -> String? {
        event.tags.first(where: { $0.name == name })?.value
    }

    private func repositoryID(for event: NostrEvent) -> String? {
        event.tags.first(where: { $0.name == "d" })?.value
    }

    private func values(for name: String) -> [String] {
        event.tags.filter { $0.name == name }.map(\.value)
    }

    private func shortDisplayValue(_ value: String?) -> String? {
        guard let value, value.isEmpty == false else { return nil }
        if let url = URL(string: value), let host = url.host {
            return host
        }
        return value
    }

    @ViewBuilder
    private func tagRow(for item: TagItem) -> some View {
        if let target = item.target {
            switch target {
            case .repo(let repoID):
                if let linkedEvent = repositoryEvent(for: repoID) {
                    NavigationLink(destination: EventDetailView(event: linkedEvent,
                                                                metadata: metadata,
                                                                eventByID: eventByID,
                                                                eventByCoordinate: eventByCoordinate,
                                                                repoEventByRepoIDAndKind: repoEventByRepoIDAndKind,
                                                                referencedRepositoryAnnouncement: nil)) {
                        TagChipView(label: item.label, value: item.value)
                    }
                    .buttonStyle(.plain)
                } else {
                    TagChipView(label: item.label, value: item.value)
                }
            case .clone(let url):
                Button {
                    repositoryHostStore.cloneRepository(from: url)
                } label: {
                    TagChipView(label: item.label,
                                value: repositoryHostStore.isCloning(url) ? "Cloning..." : item.value)
                }
                .buttonStyle(.plain)
                .disabled(repositoryHostStore.isCloning(url))
            case .pubkey(let pubkey):
                NavigationLink(destination: MaintainerProfileView(pubkey: pubkey)) {
                    TagChipView(label: item.label, value: item.value)
                }
                .buttonStyle(.plain)
            case .event(let eventId):
                if let linkedEvent = eventByID[eventId] {
                    NavigationLink(destination: EventDetailView(event: linkedEvent,
                                                                metadata: metadata,
                                                                eventByID: eventByID,
                                                                eventByCoordinate: eventByCoordinate,
                                                                repoEventByRepoIDAndKind: repoEventByRepoIDAndKind,
                                                                referencedRepositoryAnnouncement: nil)) {
                        TagChipView(label: item.label, value: item.value)
                    }
                    .buttonStyle(.plain)
                } else {
                    TagChipView(label: item.label, value: item.value)
                }
            case .coordinate(let coordinateKey):
                if let linkedEvent = eventByCoordinate[coordinateKey] {
                    NavigationLink(destination: EventDetailView(event: linkedEvent,
                                                                metadata: metadata,
                                                                eventByID: eventByID,
                                                                eventByCoordinate: eventByCoordinate,
                                                                repoEventByRepoIDAndKind: repoEventByRepoIDAndKind,
                                                                referencedRepositoryAnnouncement: nil)) {
                        TagChipView(label: item.label, value: item.value)
                    }
                    .buttonStyle(.plain)
                } else {
                    TagChipView(label: item.label, value: item.value)
                }
            case .url(let url):
                Link(destination: url) {
                    TagChipView(label: item.label, value: item.value)
                }
            }
        } else {
            TagChipView(label: item.label, value: item.value)
        }
    }

    private func tagItem(label: String, value: String?, tagName: String, targetValue: String? = nil) -> TagItem? {
        guard let value, value.isEmpty == false else { return nil }
        return TagItem(label: label, value: value, tagName: tagName, targetValue: targetValue, target: linkTarget(forTagName: tagName, value: targetValue ?? value))
    }

    private func linkTarget(forTagName tagName: String, value: String) -> TagTarget? {
        switch tagName {
        case "d":
            return .repo(value)
        case TagName.pubkey.rawValue:
            return .pubkey(value)
        case TagName.event.rawValue:
            return .event(value)
        case TagName.eventCoordinates.rawValue:
            return .coordinate(value)
        case "clone":
            guard let url = normalizedCloneURL(from: value) else { return nil }
            return .clone(url)
        case TagName.webURL.rawValue, "web":
            guard let url = URL(string: value) else { return nil }
            return .url(url)
        default:
            guard let url = URL(string: value), url.scheme != nil else { return nil }
            return .url(url)
        }
    }

    private func repositoryEvent(for repoID: String) -> NostrEvent? {
        repoEventByRepoIDAndKind[repoID]?[30617] ?? repoEventByRepoIDAndKind[repoID]?[30618]
    }
}

private enum AuthorSource {
    case selfPubkey
    case followed
    case seen
}

private struct EventDetailView: View {
    let event: NostrEvent
    let metadata: MetadataEvent?
    let eventByID: [String: NostrEvent]
    let eventByCoordinate: [String: NostrEvent]
    let repoEventByRepoIDAndKind: [String: [Int: NostrEvent]]
    let referencedRepositoryAnnouncement: NostrEvent?
    @EnvironmentObject private var repositoryHostStore: DemoRepositoryHostStore
    @Environment(\.verticalSizeClass) private var verticalSizeClass

    private struct TagItem: Hashable {
        let label: String
        let value: String
        let tagName: String
        let targetValue: String?
        let target: TagTarget?
    }

    private enum TagTarget: Hashable {
        case repo(String)
        case pubkey(String)
        case event(String)
        case coordinate(String)
        case clone(URL)
        case url(URL)
    }

    private var title: String {
        tagValue("name") ?? tagValue("description") ?? tagValue("alt") ?? event.pubkey
    }

    private var pubkeyFont: Font {
        verticalSizeClass == .regular ? .system(size: 5, weight: .regular, design: .monospaced) : .caption2.monospaced()
    }

    private var repoID: String? {
        tagValue("d")
    }

    private var cloneURL: String? {
        tagValue("clone")
    }

    private var webURL: String? {
        tagValue("web")
    }

    private var relaysText: String? {
        values(for: "relays").first
    }

    private var maintainersText: String? {
        let values = values(for: "maintainers")
        return values.isEmpty ? nil : values.joined(separator: ", ")
    }

    private var maintainerPubkeys: [String] {
        // Preserve order while removing duplicate maintainer edges before linking.
        var seen = Set<String>()
        return values(for: "maintainers").filter { seen.insert($0).inserted }
    }

    private var isPatch: Bool {
        event.kind.rawValue == 1617
    }

    private var relatedRepositoryEventTitle: String {
        switch event.kind.rawValue {
        case 1631:
            return "Related patch"
        case 30618:
            return "Related repository announcement"
        default:
            return "Related event"
        }
    }

    private var detailTags: [TagItem] {
        // The Maintainers field is rendered separately so each pubkey stays clickable.
        [
            tagItem(label: "Repo", value: repoID, tagName: "d"),
            tagItem(label: "Name", value: tagValue("name"), tagName: "name"),
            tagItem(label: "Description", value: tagValue("description"), tagName: "description"),
            tagItem(label: "Alt", value: tagValue("alt"), tagName: "alt"),
            tagItem(label: "Clone", value: shortDisplayValue(cloneURL), tagName: "clone", targetValue: cloneURL),
            tagItem(label: "Web", value: shortDisplayValue(webURL), tagName: "web", targetValue: webURL),
            tagItem(label: "Relays", value: relaysText, tagName: "relays"),
            tagItem(label: "Maintainers", value: maintainersText, tagName: "maintainers")
        ]
        .compactMap { $0 }
    }

    private var maintainerTitle: String {
        metadata?.displayName ?? metadata?.name ?? title
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                banner

                if let referencedRepositoryAnnouncement {
                    VStack(alignment: .leading, spacing: 8) {
                        Text(relatedRepositoryEventTitle)
                            .font(.headline)
                        NavigationLink(destination: EventDetailView(event: referencedRepositoryAnnouncement,
                                                                    metadata: metadata,
                                                                    eventByID: eventByID,
                                                                    eventByCoordinate: eventByCoordinate,
                                                                    repoEventByRepoIDAndKind: repoEventByRepoIDAndKind,
                                                                    referencedRepositoryAnnouncement: nil)) {
                            EventCardView(event: referencedRepositoryAnnouncement,
                                          metadata: metadata,
                                          eventByID: eventByID,
                                          eventByCoordinate: eventByCoordinate,
                                          repoEventByRepoIDAndKind: repoEventByRepoIDAndKind)
                        }
                        .buttonStyle(.plain)
                    }
                }

                Text(event.kind.rawValue == 1631 ? "Merge event" : "Event")
                    .font(.headline)

                HStack(alignment: .center, spacing: 12) {
                    detailAvatar
                    VStack(alignment: .leading, spacing: 4) {
                        Text(maintainerTitle)
                            .font(.headline)

                        if isPatch {
                            Text("PATCH")
                                .font(.caption2.weight(.bold))
                                .foregroundColor(.primary)
                                .padding(.horizontal, 8)
                                .padding(.vertical, 4)
                                .background(
                                    Capsule(style: .continuous)
                                        .fill(Color.accentColor.opacity(0.18))
                                )
                        }

                        if let nostrAddress = metadata?.nostrAddress {
                            Text(nostrAddress)
                                .font(.caption)
                                .foregroundColor(.primary)
                        }

                        if let about = metadata?.about, !about.isEmpty {
                            Text(about)
                                .font(.caption)
                                .foregroundColor(.primary)
                        }
                    }
                }

                VStack(alignment: .leading, spacing: 12) {
                    HStack(alignment: .firstTextBaseline, spacing: 8) {
                        Text("ID")
                            .font(.caption2.weight(.bold))
                            .foregroundColor(.primary)
                        Text(event.id)
                            .font(.caption.monospaced())
                            .foregroundColor(.primary)
                            .lineLimit(1)
                            .minimumScaleFactor(0.6)
                            .allowsTightening(true)
                    }

                    HStack(alignment: .center, spacing: 8) {
                        Text("Kind \(event.kind.rawValue, format: .number.grouping(.never))")
                            .font(.caption2.weight(.bold))
                            .foregroundColor(.primary)
                            .padding(.horizontal, 8)
                            .padding(.vertical, 4)
                            .background(
                                Capsule(style: .continuous)
                                    .fill(Color.accentColor.opacity(0.12))
                            )
                        Text(event.createdDate.formatted(date: .long, time: .shortened))
                            .font(.caption)
                            .foregroundColor(.primary)
                        Spacer(minLength: 0)
                    }

                    if let repoID {
                        Text("Repository: \(repoID)")
                            .font(.body.monospaced())
                            .foregroundColor(.primary)
                    }

                    if !event.content.isEmpty {
                        Text(event.content)
                            .font(.callout.monospaced())
                            .padding()
                            .frame(maxWidth: .infinity, alignment: .leading)
                            .background(
                                RoundedRectangle(cornerRadius: 12, style: .continuous)
                                    .fill(Color(.tertiarySystemFill))
                            )
                            .overlay(
                                RoundedRectangle(cornerRadius: 12, style: .continuous)
                                    .stroke(Color(.separator).opacity(0.12))
                            )
                    }

                    if let cloneURL {
                        Text("Clone: \(cloneURL)")
                            .font(.body.monospaced())
                            .foregroundColor(.primary)
                            .lineLimit(1)
                            .minimumScaleFactor(0.5)
                            .allowsTightening(true)
                    }
                    if let webURL {
                        Text("Web: \(webURL)")
                            .font(.body)
                            .foregroundColor(.primary)
                    }
                    if let relaysText {
                        Text("Relays: \(relaysText)")
                            .font(.body)
                            .foregroundColor(.primary)
                    }
                    if let maintainersText {
                        MaintainersTagValueView(pubkeys: maintainerPubkeys)
                    }
                }
                .padding(14)
                .frame(maxWidth: .infinity, alignment: .leading)
                .background(
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .fill(Color(.secondarySystemBackground))
                )
                .overlay(
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .stroke(Color(.separator).opacity(0.15))
                )

                if detailTags.isEmpty == false {
                    VStack(alignment: .leading, spacing: 8) {
                        Text("Tags")
                            .font(.headline)

                        VStack(alignment: .leading, spacing: 8) {
                            ForEach(detailTags, id: \.self) { item in
                                if item.label == "Maintainers" {
                                    MaintainersTagValueView(pubkeys: maintainerPubkeys)
                                } else {
                                    tagRow(for: item)
                                }
                            }
                        }
                    }
                }
            }
            .padding()
        }
        .safeAreaInset(edge: .bottom) {
            Text("Signature: \(event.signature ?? "N/A")")
                .font(.caption.monospaced())
                .foregroundColor(.primary)
                .frame(maxWidth: .infinity, alignment: .leading)
                .padding(.horizontal)
                .padding(.vertical, 8)
                .background(.ultraThinMaterial)
        }
        .safeAreaInset(edge: .top, spacing: 0) {
            ContextAwareHeaderView(
                title: title,
                subtitle: "Repository event details",
                systemImage: "doc.text.magnifyingglass",
                bannerURL: metadata?.bannerPictureURL,
                bannerHeight: 180
            )
            .padding(.horizontal)
            .padding(.top, 8)
        }
    }

    private func tagValue(_ name: String) -> String? {
        event.tags.first(where: { $0.name == name })?.value
    }

    private func repositoryID(for event: NostrEvent) -> String? {
        event.tags.first(where: { $0.name == "d" })?.value
    }

    private func values(for name: String) -> [String] {
        event.tags.filter { $0.name == name }.map(\.value)
    }

    private func shortDisplayValue(_ value: String?) -> String? {
        guard let value, value.isEmpty == false else { return nil }
        if let url = URL(string: value), let host = url.host {
            return host
        }
        return value
    }

    @ViewBuilder
    private func tagRow(for item: TagItem) -> some View {
        if let target = item.target {
            switch target {
            case .repo(let repoID):
                if let linkedEvent = repositoryEvent(for: repoID) {
                    NavigationLink(destination: EventDetailView(event: linkedEvent,
                                                                metadata: metadata,
                                                                eventByID: eventByID,
                                                                eventByCoordinate: eventByCoordinate,
                                                                repoEventByRepoIDAndKind: repoEventByRepoIDAndKind,
                                                                referencedRepositoryAnnouncement: referencedRepositoryAnnouncement)) {
                        TagChipView(label: item.label, value: item.value)
                    }
                    .buttonStyle(.plain)
                } else {
                    TagChipView(label: item.label, value: item.value)
                }
            case .clone(let url):
                Button {
                    repositoryHostStore.cloneRepository(from: url)
                } label: {
                    TagChipView(label: item.label,
                                value: repositoryHostStore.isCloning(url) ? "Cloning..." : item.value)
                }
                .buttonStyle(.plain)
                .disabled(repositoryHostStore.isCloning(url))
            case .pubkey(let pubkey):
                NavigationLink(destination: MaintainerProfileView(pubkey: pubkey)) {
                    TagChipView(label: item.label, value: item.value)
                }
                .buttonStyle(.plain)
            case .event(let eventId):
                if let linkedEvent = eventByID[eventId] {
                    NavigationLink(destination: EventDetailView(event: linkedEvent,
                                                                metadata: metadata,
                                                                eventByID: eventByID,
                                                                eventByCoordinate: eventByCoordinate,
                                                                repoEventByRepoIDAndKind: repoEventByRepoIDAndKind,
                                                                referencedRepositoryAnnouncement: nil)) {
                        TagChipView(label: item.label, value: item.value)
                    }
                    .buttonStyle(.plain)
                } else {
                    TagChipView(label: item.label, value: item.value)
                }
            case .coordinate(let coordinateKey):
                if let linkedEvent = eventByCoordinate[coordinateKey] {
                    NavigationLink(destination: EventDetailView(event: linkedEvent,
                                                                metadata: metadata,
                                                                eventByID: eventByID,
                                                                eventByCoordinate: eventByCoordinate,
                                                                repoEventByRepoIDAndKind: repoEventByRepoIDAndKind,
                                                                referencedRepositoryAnnouncement: nil)) {
                        TagChipView(label: item.label, value: item.value)
                    }
                    .buttonStyle(.plain)
                } else {
                    TagChipView(label: item.label, value: item.value)
                }
            case .url(let url):
                Link(destination: url) {
                    TagChipView(label: item.label, value: item.value)
                }
            }
        } else {
            TagChipView(label: item.label, value: item.value)
        }
    }

    private func tagItem(label: String, value: String?, tagName: String, targetValue: String? = nil) -> TagItem? {
        guard let value, value.isEmpty == false else { return nil }
        return TagItem(label: label, value: value, tagName: tagName, targetValue: targetValue, target: linkTarget(forTagName: tagName, value: targetValue ?? value))
    }

    private func linkTarget(forTagName tagName: String, value: String) -> TagTarget? {
        switch tagName {
        case "d":
            return .repo(value)
        case TagName.pubkey.rawValue:
            return .pubkey(value)
        case TagName.event.rawValue:
            return .event(value)
        case TagName.eventCoordinates.rawValue:
            return .coordinate(value)
        case "clone":
            guard let url = normalizedCloneURL(from: value) else { return nil }
            return .clone(url)
        case TagName.webURL.rawValue, "web":
            guard let url = URL(string: value) else { return nil }
            return .url(url)
        default:
            guard let url = URL(string: value), url.scheme != nil else { return nil }
            return .url(url)
        }
    }

    private func repositoryEvent(for repoID: String) -> NostrEvent? {
        repoEventByRepoIDAndKind[repoID]?[30617] ?? repoEventByRepoIDAndKind[repoID]?[30618]
    }

    @ViewBuilder
    private var banner: some View {
        if let bannerURL = metadata?.bannerPictureURL {
            TinyWebImageView(url: bannerURL)
                .frame(maxWidth: .infinity)
                .frame(height: 140)
                .cornerRadius(16)
        }
    }

    private var bannerPlaceholder: some View {
        RoundedRectangle(cornerRadius: 16, style: .continuous)
            .fill(Color(.tertiarySystemFill))
            .frame(height: 140)
    }

    @ViewBuilder
    private var detailAvatar: some View {
        if let pictureURL = metadata?.pictureURL {
            TinyWebImageView(url: pictureURL)
                .frame(width: 96, height: 96)
                .clipShape(Circle())
                .overlay(Circle().stroke(Color(.separator).opacity(0.2), lineWidth: 1))
        } else {
            detailAvatarPlaceholder
        }
    }

    private var detailAvatarPlaceholder: some View {
        ZStack {
            Circle()
                .fill(Color(.tertiarySystemFill))
            Image("GnostrIcon")
                .resizable()
                .scaledToFit()
                .padding(16)
        }
        .frame(width: 96, height: 96)
    }
}

private struct MaintainerProfileView: View {
    @EnvironmentObject private var relayPool: RelayPool
    let pubkey: String

    @StateObject private var metadataLoader = PubkeyMetadataLoader()
    @State private var events: [NostrEvent] = []
    @State private var eventIDs: Set<String> = []
    @State private var eventsByID: [String: NostrEvent] = [:]
    @State private var eventsByCoordinate: [String: NostrEvent] = [:]
    @State private var repoEventByRepoIDAndKind: [String: [Int: NostrEvent]] = [:]
    private let subscriptionWorkQueue = DispatchQueue(label: "NostrSDKDemo.QueryRelayDemo.subscriptionWork", qos: .userInitiated)
    @State private var eventsCancellable: AnyCancellable?
    @State private var subscriptionId: String?
    @State private var trackedPubkey: String?

    private struct SummaryCard: View {
        let event: NostrEvent
        let title: String

        private var repoID: String? {
            event.tags.first(where: { $0.name == "d" })?.value
        }

        private var summary: String? {
            if let name = event.tags.first(where: { $0.name == "name" })?.value, name.isEmpty == false {
                return name
            }
            if let description = event.tags.first(where: { $0.name == "description" })?.value, description.isEmpty == false {
                return description
            }
            if event.content.isEmpty == false {
                return event.content
            }
            return nil
        }

        var body: some View {
            VStack(alignment: .leading, spacing: 8) {
                HStack(alignment: .firstTextBaseline, spacing: 8) {
                    Text(title)
                        .font(.caption.weight(.semibold))
                    Spacer(minLength: 0)
                    Text("Kind \(event.kind.rawValue)")
                        .font(.caption2.monospaced())
                }

                if let repoID {
                    Text(repoID)
                        .font(.caption2.monospaced())
                        .lineLimit(1)
                }

                if let summary {
                    Text(summary)
                        .font(.caption)
                        .lineLimit(3)
                }

                Text(event.id)
                    .font(.caption2.monospaced())
                    .foregroundColor(.secondary)
                    .lineLimit(1)
            }
            .foregroundColor(.primary)
            .padding(12)
            .frame(maxWidth: .infinity, alignment: .leading)
            .background(
                RoundedRectangle(cornerRadius: 14, style: .continuous)
                    .fill(Color(.secondarySystemBackground))
            )
            .overlay(
                RoundedRectangle(cornerRadius: 14, style: .continuous)
                    .stroke(Color(.separator).opacity(0.15))
            )
        }
    }

    private struct RepositoryAnnouncementCardView: View {
        let event: NostrEvent
        let metadata: MetadataEvent?
        let eventByID: [String: NostrEvent]
        let eventByCoordinate: [String: NostrEvent]
        let repoEventByRepoIDAndKind: [String: [Int: NostrEvent]]

        var body: some View {
            ScrollView {
                EventCardView(event: event,
                              metadata: metadata,
                              eventByID: eventByID,
                              eventByCoordinate: eventByCoordinate,
                              repoEventByRepoIDAndKind: repoEventByRepoIDAndKind)
                    .padding()
            }
            .safeAreaInset(edge: .top, spacing: 0) {
                ContextAwareHeaderView(
                    title: "30617 Event",
                    subtitle: "Repository announcement details.",
                    systemImage: "doc.text.magnifyingglass",
                    bannerHeight: 180
                )
                .padding(.horizontal)
                .padding(.top, 8)
            }
        }
    }

    var body: some View {
        ScrollView {
            VStack(alignment: .leading, spacing: 20) {
                PubkeyMetadataPreviewView(metadata: metadataLoader.metadata)
                    .frame(height: 240)

                VStack(alignment: .leading, spacing: 12) {
                    Text("30617 Repository announcements")
                        .font(.headline)

                    if events.isEmpty {
                        Text("No repository announcements loaded yet.")
                            .font(.caption)
                            .foregroundColor(.primary)
                    } else {
                        VStack(alignment: .leading, spacing: 8) {
                            ForEach(events, id: \.id) { event in
                                maintainerEventRow(for: event)
                            }
                        }
                    }
                }
            }
            .padding()
        }
        .safeAreaInset(edge: .top, spacing: 0) {
            ContextAwareHeaderView(
                title: metadataLoader.metadata?.displayName ?? metadataLoader.metadata?.name ?? "Maintainer",
                subtitle: metadataLoader.metadata?.about ?? "Repository announcements for this maintainer.",
                systemImage: "person.crop.circle",
                bannerURL: metadataLoader.metadata?.bannerPictureURL,
                bannerHeight: 180
            )
            .padding(.horizontal)
            .padding(.top, 8)
        }
        .onAppear {
            refresh()
        }
        .onChange(of: pubkey) { _ in
            refresh()
        }
        .onDisappear {
            if let subscriptionId {
                relayPool.closeSubscription(with: subscriptionId)
            }
        }
    }

    @ViewBuilder
    private func maintainerEventRow(for event: NostrEvent) -> some View {
        NavigationLink(destination: RepositoryAnnouncementCardView(event: event,
                                                                   metadata: metadataLoader.metadata,
                                                                   eventByID: eventsByID,
                                                                   eventByCoordinate: eventsByCoordinate,
                                                                   repoEventByRepoIDAndKind: repoEventByRepoIDAndKind)) {
            SummaryCard(event: event, title: "Open full 30617 event")
        }
        .buttonStyle(.plain)
    }

    private func refresh() {
        guard let normalizedPubkey = normalizedHexPubkey(from: pubkey) else {
            resetQueryState()
            metadataLoader.update(publicKeyInput: pubkey, isValid: false)
            return
        }

        metadataLoader.attach(relayPool: relayPool)
        metadataLoader.update(publicKeyInput: normalizedPubkey, isValid: true)

        if trackedPubkey != normalizedPubkey {
            trackedPubkey = normalizedPubkey
            resetQueryState()
            subscribe(pubkey: normalizedPubkey)
        }
    }

    private func subscribe(pubkey: String) {
        if let subscriptionId {
            relayPool.closeSubscription(with: subscriptionId)
        }

        guard let filter = Filter(authors: [pubkey], kinds: [30617]) else {
            subscriptionId = nil
            return
        }

        subscriptionId = relayPool.subscribe(with: filter)
        eventsCancellable = relayPool.events
            .receive(on: DispatchQueue.main)
            .sink { relayEvent in
                guard relayEvent.subscriptionId == subscriptionId else { return }
                let event = relayEvent.event
                if eventIDs.insert(event.id).inserted {
                    events.insert(event, at: 0)
                    cacheQueryEvent(event)
                }
            }
    }

    private func normalizedHexPubkey(from value: String) -> String? {
        if value.contains("npub") {
            return PublicKey(npub: value)?.hex
        }
        return PublicKey(hex: value)?.hex
    }

    private func resetQueryState() {
        events = []
        eventIDs.removeAll()
        eventsByID.removeAll()
        eventsByCoordinate.removeAll()
        repoEventByRepoIDAndKind.removeAll()
    }

    private func cacheQueryEvent(_ event: NostrEvent) {
        eventsByID[event.id] = event

        if let replaceableEvent = event as? ReplaceableEvent,
           let coordinates = replaceableEvent.replaceableEventCoordinates(relayURL: nil) {
            eventsByCoordinate[coordinates.tag.value] = event
        }

        guard let repoID = repoID(for: event) else { return }
        var eventsByKind = repoEventByRepoIDAndKind[repoID] ?? [:]
        if eventsByKind[event.kind.rawValue]?.createdAt ?? 0 <= event.createdAt {
            eventsByKind[event.kind.rawValue] = event
            repoEventByRepoIDAndKind[repoID] = eventsByKind
        }
    }

    private func repoID(for event: NostrEvent) -> String? {
        event.tags.first(where: { $0.name == "d" })?.value
    }

}

private struct TagChipView: View {
    let label: String
    let value: String

    var body: some View {
        ViewThatFits(in: .horizontal) {
            HStack(alignment: .firstTextBaseline, spacing: 6) {
                Text("\(label):")
                    .font(.caption2.weight(.semibold))
                    .foregroundColor(.primary)

                Text(value)
                    .font(.caption.monospaced())
                    .foregroundColor(.primary)
                    .lineLimit(1)
                    .minimumScaleFactor(0.8)
                    .allowsTightening(true)
                    .layoutPriority(1)
            }

            VStack(alignment: .leading, spacing: 2) {
                Text("\(label):")
                    .font(.caption2.weight(.semibold))
                    .foregroundColor(.primary)
                Text(value)
                    .font(.caption.monospaced())
                    .foregroundColor(.primary)
                    .fixedSize(horizontal: false, vertical: true)
                    .multilineTextAlignment(.leading)
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color(.tertiarySystemFill))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .stroke(Color(.separator).opacity(0.12))
        )
    }
}

private struct MaintainersTagValueView: View {
    let pubkeys: [String]

    var body: some View {
        ViewThatFits(in: .horizontal) {
            // Each maintainer pubkey is a traversal edge into that maintainer's profile.
            HStack(alignment: .firstTextBaseline, spacing: 6) {
                Text("Maintainers:")
                    .font(.caption2.weight(.semibold))
                    .foregroundColor(.primary)

                ForEach(Array(pubkeys.enumerated()), id: \.offset) { index, pubkey in
                    if index > 0 {
                        Text(",")
                            .font(.caption.monospaced())
                            .foregroundColor(.primary)
                    }

                    NavigationLink(destination: MaintainerProfileView(pubkey: pubkey)) {
                        Text(pubkey)
                            .font(.caption.monospaced())
                            .foregroundColor(.primary)
                            .lineLimit(1)
                            .minimumScaleFactor(0.8)
                            .allowsTightening(true)
                    }
                    .buttonStyle(.plain)
                }
            }

            VStack(alignment: .leading, spacing: 2) {
                Text("Maintainers:")
                    .font(.caption2.weight(.semibold))
                    .foregroundColor(.primary)

                ForEach(pubkeys, id: \.self) { pubkey in
                    NavigationLink(destination: MaintainerProfileView(pubkey: pubkey)) {
                        Text(pubkey)
                            .font(.caption.monospaced())
                            .foregroundColor(.primary)
                    }
                    .buttonStyle(.plain)
                }
            }
        }
        .padding(.horizontal, 12)
        .padding(.vertical, 10)
        .frame(maxWidth: .infinity, alignment: .leading)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color(.tertiarySystemFill))
        )
        .overlay(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .stroke(Color(.separator).opacity(0.12))
        )
    }
}

struct QueryRelayDemoView: View {

    @EnvironmentObject var relayPool: RelayPool
    @EnvironmentObject private var identityStore: DemoIdentityStore
    @EnvironmentObject private var relayDirectory: RelayDirectoryStore
    @EnvironmentObject private var appPrimeStore: DemoAppPrimeStore
    @EnvironmentObject private var repositoryHostStore: DemoRepositoryHostStore

    @State private var selectedFollowedAuthorPubkey: String = ""
    @State private var selectedSeenAuthorPubkey: String = ""
    @State private var selectedAuthorSource: AuthorSource = .selfPubkey
    @State private var events: [NostrEvent] = []
    @State private var eventIDs: Set<String> = []
    @State private var eventPubkeys: Set<String> = []
    @State private var eventsByID: [String: NostrEvent] = [:]
    @State private var eventsByCoordinate: [String: NostrEvent] = [:]
    @State private var metadataByPubkey: [String: MetadataEvent] = [:]
    @State private var eventsCancellable: AnyCancellable?
    @State private var errorString: String?
    @State private var subscriptionId: String?
    @State private var seenPrimeSubscriptionId: String?
    @State private var metadataSubscriptionId: String?
    @State private var trackedMetadataPubkeys: Set<String> = []
    @State private var seenAuthorEventIDsByPubkey: [String: [Int: Set<String>]] = [:]
    @State private var repoEventByRepoIDAndKind: [String: [Int: NostrEvent]] = [:]
    private let subscriptionWorkQueue = DispatchQueue(label: "NostrSDKDemo.QueryRelayDemo.subscriptionWork", qos: .userInitiated)

    private let kindOptions = [
        30617: "Repository announcements",
        30618: "Repository state announcements",
        1617: "Patches",
        1621: "Issues",
        1630: "Status (Open)",
        1631: "Status (Applied / Merged)",
        1632: "Status (Closed)",
        1633: "Status (Draft)"
    ]

    @State private var selectedKind = 30617

    var body: some View {
        VStack(spacing: 8) {
            VStack(alignment: .leading, spacing: 10) {
                HStack(alignment: .top, spacing: 12) {
                    VStack(alignment: .leading, spacing: 10) {
                        Text("Followed Author")
                            .font(.caption.weight(.semibold))
                            .foregroundColor(.primary)
                        Menu {
                            Button("Select self") {
                                selectedFollowedAuthorPubkey = ""
                                selectedAuthorSource = .selfPubkey
                                resetQueryState()
                                updateSubscription()
                                updateMetadataSubscription()
                            }
                            ForEach(identityStore.followedPubkeys, id: \.self) { pubkey in
                                Button(pubkey) {
                                    selectedFollowedAuthorPubkey = pubkey
                                    selectedSeenAuthorPubkey = ""
                                    selectedAuthorSource = .followed
                                    resetQueryState()
                                    updateSubscription()
                                    updateMetadataSubscription()
                                }
                            }
                        } label: {
                            HStack {
                                Text(selectedFollowedAuthorPubkey.isEmpty ? "Select followed author" : selectedFollowedAuthorPubkey)
                                Spacer(minLength: 0)
                                Image(systemName: "chevron.down")
                            }
                            .padding(.vertical, 8)
                        }
                        .disabled(identityStore.followedPubkeys.isEmpty)
                        if identityStore.followedPubkeys.isEmpty {
                            Text("Enter a valid private key in Settings to load followed public keys.")
                                .font(.caption)
                                .foregroundColor(.primary)
                        }
                    }

                    VStack(alignment: .leading, spacing: 10) {
                        Text("Seen Author")
                            .font(.caption.weight(.semibold))
                            .foregroundColor(.primary)
                        Menu {
                            Button("Select self") {
                                selectedSeenAuthorPubkey = ""
                                selectedAuthorSource = .selfPubkey
                                resetQueryState()
                                updateSubscription()
                                updateMetadataSubscription()
                            }
                            ForEach(seenAuthorPubkeys, id: \.self) { pubkey in
                                Button(seenAuthorLabel(for: pubkey)) {
                                    selectedSeenAuthorPubkey = pubkey
                                    selectedFollowedAuthorPubkey = ""
                                    selectedAuthorSource = .seen
                                    resetQueryState()
                                    updateSubscription()
                                    updateMetadataSubscription()
                                }
                            }
                        } label: {
                            HStack {
                                Text(selectedSeenAuthorPubkey.isEmpty ? "Select seen author" : seenAuthorLabel(for: selectedSeenAuthorPubkey))
                                Spacer(minLength: 0)
                                Image(systemName: "chevron.down")
                            }
                            .padding(.vertical, 8)
                        }
                        .disabled(seenAuthorPubkeys.isEmpty)
                        if seenAuthorPubkeys.isEmpty {
                            Text("Seen authors appear after NIP-34 events load.")
                                .font(.caption)
                                .foregroundColor(.primary)
                        }
                    }
                }

                HStack(alignment: .center, spacing: 12) {
                    Button {
                        updateSubscription()
                    } label: {
                        Text("Query")
                            .frame(minWidth: 72)
                    }
                    .buttonStyle(.borderedProminent)

                    Spacer(minLength: 0)

                    Picker("Kind", selection: $selectedKind) {
                        ForEach(kindOptions.keys.sorted(), id: \.self) { number in
                            if let name = kindOptions[number] {
                                Text(kindLabel(for: number, name: name))
                            } else {
                                Text("\(String(number))")
                            }
                        }
                    }
                    .pickerStyle(.menu)
                    .simultaneousGesture(TapGesture().onEnded {
                        refreshKindCounts()
                    })
                }
            }
            .padding(.horizontal)
            .padding(.top, 4)

            Divider()

            List {
                if !events.isEmpty {
                    if !currentAuthorPubkey.isEmpty {
                        Text("Showing events for \(currentAuthorPubkey)")
                            .font(.footnote)
                            .foregroundColor(.primary)
                            .listRowBackground(Color.clear)
                            .listRowInsets(EdgeInsets(top: 4, leading: 16, bottom: 0, trailing: 16))
                            .listRowSeparator(.hidden)
                    }

                    ForEach(events, id: \.id) { event in
                        queryEventRow(for: event)
                            .listRowInsets(EdgeInsets(top: 4, leading: 8, bottom: 4, trailing: 8))
                            .listRowBackground(Color.clear)
                            .listRowSeparator(.hidden)
                    }
                }
            }
            .listStyle(.plain)
            .scrollContentBackground(.hidden)
        }
        .onAppear {
            sanitizeSelectedAuthors()
            primeSeenAuthors()
            updateSubscription()
            updateMetadataSubscription()
            repoEventByRepoIDAndKind = appPrimeStore.repositoryEventByRepoIDAndKind
        }
        .onReceive(appPrimeStore.$repositoryEventByRepoIDAndKind) { repositoryEventByRepoIDAndKind in
            self.repoEventByRepoIDAndKind = repositoryEventByRepoIDAndKind
        }
        .onChange(of: identityStore.followedPubkeys) { _ in
            sanitizeSelectedAuthors()
        }
        .onChange(of: identityStore.publicKeyHex) { _ in
            updateSubscription()
            updateMetadataSubscription()
        }
        .onChange(of: selectedKind) { _ in
            resetQueryState()
            updateSubscription()
            updateMetadataSubscription()
        }
        .onDisappear {
            if let subscriptionId {
                relayPool.closeSubscription(with: subscriptionId)
            }
            if let seenPrimeSubscriptionId {
                relayPool.closeSubscription(with: seenPrimeSubscriptionId)
            }
            if let metadataSubscriptionId {
                relayPool.closeSubscription(with: metadataSubscriptionId)
            }
        }
        .safeAreaInset(edge: .top, spacing: 0) {
            ContextAwareHeaderView(
                title: "NIP-0034 Viewer",
                subtitle: "Browse repository announcements and related events.",
                systemImage: "list.bullet.rectangle.portrait",
                bannerHeight: 180
            )
            .padding(.horizontal)
            .padding(.top, 8)
        }
    }

    private var currentFilter: Filter {
        let authors: [String]?
        if currentAuthorPubkey.isEmpty {
            authors = nil
        } else {
            authors = [currentAuthorPubkey]
        }
        return Filter(authors: authors, kinds: [selectedKind])!
    }

    private var currentAuthorPubkey: String {
        switch selectedAuthorSource {
        case .followed:
            return selectedFollowedAuthorPubkey
        case .seen:
            return selectedSeenAuthorPubkey
        case .selfPubkey:
            return identityStore.publicKeyHex ?? ""
        }
    }

    private var seenAuthorPubkeys: [String] {
        seenAuthorEventIDsByPubkey.keys.sorted { lhs, rhs in
            let lhsCount = seenAuthorCounts(for: lhs).reduce(0, +)
            let rhsCount = seenAuthorCounts(for: rhs).reduce(0, +)
            if lhsCount != rhsCount { return lhsCount > rhsCount }
            return lhs < rhs
        }
    }

    private func seenAuthorLabel(for pubkey: String) -> String {
        let counts = seenAuthorCounts(for: pubkey).map(String.init).joined(separator: ",")
        return "\(pubkey) (\(counts))"
    }

    private func seenAuthorCounts(for pubkey: String) -> [Int] {
        kindOptions.keys.sorted().map { kind in
            seenAuthorEventIDsByPubkey[pubkey]?[kind]?.count ?? 0
        }
    }

    private func kindLabel(for kind: Int, name: String) -> String {
        let count: Int
        switch selectedAuthorSource {
        case .followed:
            count = seenAuthorEventIDsByPubkey[selectedFollowedAuthorPubkey]?[kind]?.count ?? 0
        case .seen:
            count = seenAuthorEventIDsByPubkey[selectedSeenAuthorPubkey]?[kind]?.count ?? 0
        case .selfPubkey:
            count = seenAuthorEventIDsByPubkey.values.reduce(0) { partial, kindsByPubkey in
                partial + (kindsByPubkey[kind]?.count ?? 0)
            }
        }
        return "\(name) (\(kind)) (\(count))"
    }

    private func sanitizeSelectedAuthors() {
        if identityStore.followedPubkeys.isEmpty == false,
           selectedFollowedAuthorPubkey.isEmpty == false,
           identityStore.followedPubkeys.contains(selectedFollowedAuthorPubkey) == false {
            selectedFollowedAuthorPubkey = ""
            if selectedAuthorSource == .followed {
                selectedAuthorSource = .selfPubkey
            }
        }

        if selectedSeenAuthorPubkey.isEmpty == false,
           seenAuthorEventIDsByPubkey[selectedSeenAuthorPubkey] == nil {
            selectedSeenAuthorPubkey = ""
            if selectedAuthorSource == .seen {
                selectedAuthorSource = .selfPubkey
            }
        }
    }

    private func recordSeenEvent(_ event: NostrEvent) {
        var kindsByPubkey = seenAuthorEventIDsByPubkey[event.pubkey] ?? [:]
        var eventIDs = kindsByPubkey[event.kind.rawValue] ?? []
        eventIDs.insert(event.id)
        kindsByPubkey[event.kind.rawValue] = eventIDs
        seenAuthorEventIDsByPubkey[event.pubkey] = kindsByPubkey
        recordSeenRelays(from: event)

        guard let repoID = repoID(for: event) else { return }
        var eventsByKind = repoEventByRepoIDAndKind[repoID] ?? [:]
        if eventsByKind[event.kind.rawValue]?.createdAt ?? 0 <= event.createdAt {
            eventsByKind[event.kind.rawValue] = event
            repoEventByRepoIDAndKind[repoID] = eventsByKind
            appPrimeStore.record(event: event)
        }
    }

    private func recordSeenRelays(from event: NostrEvent) {
        var relayURLs: [URL] = []
        for relayString in event.allValues(forTagName: .webURL) {
            guard let relayURL = normalizedRelayURL(from: relayString) else {
                continue
            }
            relayURLs.append(relayURL)
        }
        relayDirectory.record(seen: relayURLs)
    }

    private func normalizedRelayURL(from relayString: String) -> URL? {
        guard let url = URL(string: relayString),
              let components = URLComponents(url: url, resolvingAgainstBaseURL: false),
              components.scheme == "ws" || components.scheme == "wss" else {
            return nil
        }
        return url
    }

    private func referencedRepositoryAnnouncement(for event: NostrEvent) -> NostrEvent? {
        guard let repoID = repoID(for: event) else { return nil }
        switch event.kind.rawValue {
        case 1631:
            return repoEventByRepoIDAndKind[repoID]?[1617]
        case 30618:
            return repoEventByRepoIDAndKind[repoID]?[30617]
        default:
            return nil
        }
    }

    private func repoID(for event: NostrEvent) -> String? {
        event.tags.first(where: { $0.name == "d" })?.value
    }

    private func resetQueryState() {
        events = []
        eventIDs.removeAll()
        eventPubkeys.removeAll()
        eventsByID.removeAll()
        eventsByCoordinate.removeAll()
    }

    private func cacheQueryEvent(_ event: NostrEvent) {
        eventPubkeys.insert(event.pubkey)
        eventsByID[event.id] = event

        if let replaceableEvent = event as? ReplaceableEvent,
           let coordinates = replaceableEvent.replaceableEventCoordinates(relayURL: nil) {
            eventsByCoordinate[coordinates.tag.value] = event
        }
    }

    private func updateSubscription() {
        let filter = currentFilter
        let previousSubscriptionId = subscriptionId
        let activeSubscriptionId = UUID().uuidString

        subscriptionId = activeSubscriptionId

        eventsCancellable = relayPool.events
            .receive(on: DispatchQueue.main)
            .sink { relayEvent in
                guard relayEvent.subscriptionId == activeSubscriptionId || relayEvent.subscriptionId == seenPrimeSubscriptionId else {
                    return
                }

                let event = relayEvent.event

                if relayEvent.subscriptionId == seenPrimeSubscriptionId {
                    recordSeenEvent(event)
                    return
                }

                if let metadataEvent = event as? MetadataEvent {
                    print("[QueryRelayDemo] metadata event pubkey=\(metadataEvent.pubkey) createdAt=\(metadataEvent.createdAt) displayName=\(metadataEvent.displayName ?? "nil") name=\(metadataEvent.name ?? "nil") pictureURL=\(metadataEvent.pictureURL?.absoluteString ?? "nil") bannerURL=\(metadataEvent.bannerPictureURL?.absoluteString ?? "nil")")
                    if metadataByPubkey[metadataEvent.pubkey]?.createdAt ?? 0 <= metadataEvent.createdAt {
                        metadataByPubkey[metadataEvent.pubkey] = metadataEvent
                        print("[QueryRelayDemo] cached metadata pubkey=\(metadataEvent.pubkey)")
                        RemoteImagePrefetcher.shared.prefetch(url: metadataEvent.pictureURL)
                        RemoteImagePrefetcher.shared.prefetch(url: metadataEvent.bannerPictureURL)
                    }
                    recordSeenRelays(from: metadataEvent)
                    return
                }

                recordSeenEvent(event)
                if eventIDs.insert(event.id).inserted {
                    events.insert(event, at: 0)
                    cacheQueryEvent(event)
                }
                updateMetadataSubscription()
            }
        subscriptionWorkQueue.async { [relayPool] in
            if let previousSubscriptionId {
                relayPool.closeSubscription(with: previousSubscriptionId)
            }

            relayPool.subscribe(with: filter, subscriptionId: activeSubscriptionId)
        }
    }

    private func primeSeenAuthors() {
        guard let filter = Filter(kinds: Array(kindOptions.keys.sorted())) else {
            if let seenPrimeSubscriptionId {
                subscriptionWorkQueue.async { [relayPool] in
                    relayPool.closeSubscription(with: seenPrimeSubscriptionId)
                }
            }
            seenPrimeSubscriptionId = nil
            return
        }

        let previousSubscriptionId = seenPrimeSubscriptionId
        let activeSubscriptionId = UUID().uuidString
        seenPrimeSubscriptionId = activeSubscriptionId

        subscriptionWorkQueue.async { [relayPool] in
            if let previousSubscriptionId {
                relayPool.closeSubscription(with: previousSubscriptionId)
            }

            relayPool.subscribe(with: filter, subscriptionId: activeSubscriptionId)
        }
    }

    private func refreshKindCounts() {
        primeSeenAuthors()
        updateSubscription()
        updateMetadataSubscription()
    }

    private func updateMetadataSubscription() {
        var pubkeys = eventPubkeys
        if currentAuthorPubkey.isEmpty == false {
            pubkeys.insert(currentAuthorPubkey)
        }

        guard pubkeys != trackedMetadataPubkeys else {
            return
        }

        trackedMetadataPubkeys = pubkeys

        let previousSubscriptionId = metadataSubscriptionId

        guard pubkeys.isEmpty == false, let filter = Filter(authors: Array(pubkeys), kinds: [EventKind.metadata.rawValue]) else {
            metadataSubscriptionId = nil
            if let previousSubscriptionId {
                subscriptionWorkQueue.async { [relayPool] in
                    relayPool.closeSubscription(with: previousSubscriptionId)
                }
            }
            return
        }

        let activeSubscriptionId = UUID().uuidString
        metadataSubscriptionId = activeSubscriptionId

        subscriptionWorkQueue.async { [relayPool] in
            if let previousSubscriptionId {
                relayPool.closeSubscription(with: previousSubscriptionId)
            }

            relayPool.subscribe(with: filter, subscriptionId: activeSubscriptionId)
        }
    }

    @ViewBuilder
    private func queryEventRow(for event: NostrEvent) -> some View {
        NavigationLink(destination: EventDetailView(event: event,
                                                    metadata: metadataByPubkey[event.pubkey],
                                                    eventByID: eventsByID,
                                                    eventByCoordinate: eventsByCoordinate,
                                                    repoEventByRepoIDAndKind: repoEventByRepoIDAndKind,
                                                    referencedRepositoryAnnouncement: referencedRepositoryAnnouncement(for: event))) {
            EventCardView(event: event,
                          metadata: metadataByPubkey[event.pubkey],
                          eventByID: eventsByID,
                          eventByCoordinate: eventsByCoordinate,
                          repoEventByRepoIDAndKind: repoEventByRepoIDAndKind)
        }
        .buttonStyle(.plain)
    }
}

struct QueryRelayView_Previews: PreviewProvider {
    static var previews: some View {
        NavigationView {
            QueryRelayDemoView()
        }
        .environmentObject(RelayPool(relays: []))
        .environmentObject(DemoIdentityStore())
        .environmentObject(RelayDirectoryStore())
        .environmentObject(DemoAppPrimeStore())
        .environmentObject(DemoRepositoryHostStore())
    }
}
