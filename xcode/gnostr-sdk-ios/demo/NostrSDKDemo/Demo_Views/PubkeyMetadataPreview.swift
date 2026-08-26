//
//  PubkeyMetadataPreview.swift
//  NostrSDKDemo
//
//  Created by Copilot on 2026-06-06.
//

import SwiftUI
import Combine
import GnostrSDK
import UIKit

@MainActor
final class PubkeyMetadataLoader: ObservableObject {
    @Published var metadata: MetadataEvent?

    private var relayPool: RelayPool?
    private var subscriptionId: String?
    private var cancellable: AnyCancellable?
    private var trackedPubkey: String?

    func attach(relayPool: RelayPool) {
        self.relayPool = relayPool
    }

    init() {
    }

    func update(publicKeyInput: String, isValid: Bool) {
        guard let relayPool else {
            return
        }

        guard isValid, let pubkey = normalizedHexPubkey(from: publicKeyInput) else {
            stop()
            metadata = nil
            trackedPubkey = nil
            return
        }

        guard trackedPubkey != pubkey else {
            return
        }

        trackedPubkey = pubkey
        metadata = nil

        if let subscriptionId {
            relayPool.closeSubscription(with: subscriptionId)
        }

        cancellable = relayPool.events
            .receive(on: DispatchQueue.main)
            .map { $0.event }
            .sink { [weak self] event in
                guard let self else { return }
                guard let metadataEvent = event as? MetadataEvent, metadataEvent.pubkey == pubkey else {
                    return
                }

                if self.metadata?.createdAt ?? 0 <= metadataEvent.createdAt {
                    self.metadata = metadataEvent
                    RemoteImagePrefetcher.shared.prefetch(url: metadataEvent.pictureURL)
                }
            }

        guard let filter = Filter(authors: [pubkey], kinds: [EventKind.metadata.rawValue]) else {
            subscriptionId = nil
            return
        }

        subscriptionId = relayPool.subscribe(with: filter)
    }

    func stop() {
        guard let relayPool else {
            return
        }

        if let subscriptionId {
            relayPool.closeSubscription(with: subscriptionId)
        }
        subscriptionId = nil
        cancellable?.cancel()
        cancellable = nil
    }

    private func normalizedHexPubkey(from value: String) -> String? {
        if value.contains("npub") {
            return PublicKey(npub: value)?.hex
        }
        return PublicKey(hex: value)?.hex
    }
}

private final class RemoteImagePrefetcher {
    static let shared = RemoteImagePrefetcher()

    private var taskURLs: Set<URL> = []

    func prefetch(url: URL?) {
        guard let url else { return }
        guard taskURLs.insert(url).inserted else { return }

        // Prefetch remote images off the main actor so metadata previews stay responsive.
        Task.detached(priority: .background) { [url] in
            print("[NIP44Metadata] image prefetch attempt url=\(url.absoluteString)")
            defer { Task { @MainActor in RemoteImagePrefetcher.shared.taskURLs.remove(url) } }

            do {
                let (data, _) = try await URLSession.shared.data(from: url)
                guard let image = UIImage(data: data) else {
                    print("[NIP44Metadata] image decode failed url=\(url.absoluteString)")
                    return
                }
                await MainActor.run {
                    RemoteImageLoader.cache.setObject(image, forKey: url as NSURL)
                }
            } catch {
                print("[NIP44Metadata] image prefetch failed url=\(url.absoluteString) error=\(error)")
            }
        }
    }
}

private final class RemoteImageLoader {
    static let cache = NSCache<NSURL, UIImage>()
}

struct PubkeyMetadataPreviewView: View {
    let metadata: MetadataEvent?
    @Environment(\.horizontalSizeClass) private var horizontalSizeClass

    private var avatarSize: CGFloat {
        horizontalSizeClass == .compact ? 48 : 56
    }

    private var titleFont: Font {
        horizontalSizeClass == .compact ? .headline : .title3
    }

    private var aboutLineLimit: Int {
        horizontalSizeClass == .compact ? 2 : 3
    }

    var body: some View {
        GeometryReader { proxy in
            let bannerHeight = max(84, min(proxy.size.width * 0.99, horizontalSizeClass == .compact ? 88 : 96))

            VStack(alignment: .leading, spacing: 0) {
                ZStack(alignment: .bottomLeading) {
                    RoundedRectangle(cornerRadius: 16, style: .continuous)
                        .fill(Color(.tertiarySystemFill))
                        .frame(height: bannerHeight)

                    if let bannerPictureURL = metadata?.bannerPictureURL {
                        CachedRemoteImageView(url: bannerPictureURL)
                            .frame(width: proxy.size.width, height: bannerHeight)
                            .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
                    }

                    ZStack {
                        Circle()
                            .fill(Color(.secondarySystemBackground))

                        if let pictureURL = metadata?.pictureURL {
                            CachedRemoteImageView(url: pictureURL)
                        } else {
                            Image("GnostrIcon")
                                .resizable()
                                .scaledToFit()
                                .padding(0)
                        }
                    }
                    .frame(width: avatarSize, height: avatarSize)
                    .clipShape(Circle())
                    .overlay(Circle().stroke(Color(.separator).opacity(0.2), lineWidth: 1))
                    .padding(0)
                }

                VStack(alignment: .leading, spacing: 4) {
                    if let metadata {
                        if let title = metadata.displayName ?? metadata.name ?? metadata.nostrAddress {
                            Text(title)
                                .font(titleFont)
                        }

                        if let about = metadata.about, !about.isEmpty {
                            Text(about)
                                .font(.caption)
                                .foregroundColor(.primary)
                                .lineLimit(aboutLineLimit)
                        }
                    }
                }
                .padding(.horizontal, 0)
            }
            .frame(width: proxy.size.width, height: proxy.size.height, alignment: .leading)
            .onAppear {
                logDimensions(size: proxy.size, bannerHeight: bannerHeight)
            }
            .onChange(of: proxy.size.width) { _ in
                logDimensions(size: proxy.size, bannerHeight: bannerHeight)
            }
            .onChange(of: proxy.size.height) { _ in
                logDimensions(size: proxy.size, bannerHeight: bannerHeight)
            }
        }
    }

    private func logDimensions(size: CGSize, bannerHeight: CGFloat) {
        print("[PubkeyMetadataPreview] size=\(size) bannerHeight=\(bannerHeight)")
    }
}

private struct CachedRemoteImageView: UIViewRepresentable {
    let url: URL?

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

        func load(url: URL?, into imageView: UIImageView) {
            guard let url else {
                imageView.image = nil
                return
            }

            currentURL = url
            print("[NIP44Metadata] image load attempt url=\(url.absoluteString)")

            if let cachedImage = RemoteImageLoader.cache.object(forKey: url as NSURL) {
                print("[NIP44Metadata] image load cache hit url=\(url.absoluteString)")
                imageView.image = cachedImage
                return
            }

            task?.cancel()
            // Load metadata preview images in the background and apply them on the main actor only.
            task = Task.detached(priority: .background) { [weak imageView] in
                do {
                    let (data, _) = try await URLSession.shared.data(from: url)
                    guard let image = UIImage(data: data) else {
                        print("[NIP44Metadata] image decode failed url=\(url.absoluteString)")
                        return
                    }

                    RemoteImageLoader.cache.setObject(image, forKey: url as NSURL)

                    await MainActor.run {
                        guard let imageView, self.currentURL == url else { return }
                        imageView.image = image
                    }
                } catch {
                    print("[NIP44Metadata] image load failed url=\(url.absoluteString) error=\(error)")
                }
            }
        }
    }
}
