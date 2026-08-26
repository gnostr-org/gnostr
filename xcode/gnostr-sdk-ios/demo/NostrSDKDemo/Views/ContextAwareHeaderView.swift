//
//  ContextAwareHeaderView.swift
//  NostrSDKDemo
//
//  Created by Copilot on 6/9/26.
//

import SwiftUI

struct ContextAwareHeaderView<Hero: View, Accessory: View>: View {
    let title: String
    let subtitle: String?
    let systemImage: String
    let bannerURL: URL?
    let fallbackBannerImageName: String
    let bannerHeight: CGFloat
    let hero: Hero
    let accessory: Accessory

    init(title: String,
         subtitle: String? = nil,
         systemImage: String,
         bannerURL: URL? = nil,
         fallbackBannerImageName: String = "DefaultBanner",
         bannerHeight: CGFloat = 180,
         @ViewBuilder hero: () -> Hero = { EmptyView() },
         @ViewBuilder accessory: () -> Accessory = { EmptyView() }) {
        self.title = title
        self.subtitle = subtitle
        self.systemImage = systemImage
        self.bannerURL = bannerURL
        self.fallbackBannerImageName = fallbackBannerImageName
        self.bannerHeight = bannerHeight
        self.hero = hero()
        self.accessory = accessory()
    }

    var body: some View {
        VStack(alignment: .leading, spacing: 12) {
            banner
            hero

            HStack(alignment: .top, spacing: 12) {
                ZStack {
                    RoundedRectangle(cornerRadius: 14, style: .continuous)
                        .fill(Color(.secondarySystemBackground))

                    Image(systemName: systemImage)
                        .font(.title3.weight(.semibold))
                        .foregroundStyle(.primary)
                }
                .frame(width: 44, height: 44)

                VStack(alignment: .leading, spacing: 4) {
                    Text(title)
                        .font(.headline)

                    if let subtitle, subtitle.isEmpty == false {
                        Text(subtitle)
                            .font(.caption)
                            .foregroundColor(.secondary)
                    }
                }

                Spacer(minLength: 0)

                accessory
            }
        }
        .padding(.bottom, 8)
    }

    @ViewBuilder
    private var banner: some View {
        if let bannerURL {
            bannerFrame {
                AsyncImage(url: bannerURL) { phase in
                    switch phase {
                    case .empty:
                        fallbackBanner
                    case .success(let image):
                        image
                            .resizable()
                            .scaledToFill()
                    case .failure:
                        fallbackBanner
                    @unknown default:
                        fallbackBanner
                    }
                }
            }
        } else {
            bannerFrame {
                fallbackBanner
            }
        }
    }

    private var fallbackBanner: some View {
        Image(fallbackBannerImageName)
            .resizable()
            .scaledToFill()
    }

    private func bannerFrame<Content: View>(@ViewBuilder content: () -> Content) -> some View {
        ZStack {
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .fill(Color(.systemBackground))

            content()
                .frame(maxWidth: .infinity)
                .frame(height: bannerHeight)
                .clipShape(RoundedRectangle(cornerRadius: 16, style: .continuous))
        }
        .frame(height: bannerHeight)
        .overlay(
            RoundedRectangle(cornerRadius: 16, style: .continuous)
                .stroke(Color(.separator).opacity(0.18), lineWidth: 1)
        )
    }
}

struct HeaderMetricPill: View {
    let value: String
    let label: String

    var body: some View {
        VStack(alignment: .trailing, spacing: 2) {
            Text(value)
                .font(.caption.weight(.semibold))
            Text(label)
                .font(.caption2)
                .foregroundColor(.secondary)
        }
        .padding(.horizontal, 10)
        .padding(.vertical, 6)
        .background(
            RoundedRectangle(cornerRadius: 10, style: .continuous)
                .fill(Color(.secondarySystemBackground))
        )
    }
}
