//
//  TagsView.swift
//  damus
//
//  Created by William Casarin on 2022-05-04.
//

import SwiftUI

struct TagRowPreview {
    let visible_indices: Set<Int>
    let height: CGFloat
    let is_foldable: Bool

    init(frames: [Int: CGRect]) {
        let rowStarts = frames.values.map(\.minY).sorted().reduce(into: [CGFloat]()) { rows, y in
            if let previous = rows.last, abs(previous - y) < 0.5 { return }
            rows.append(y)
        }
        is_foldable = rowStarts.count > 2
        let cutoff = rowStarts.prefix(2).last ?? 0
        let visible = frames.filter { $0.value.minY <= cutoff + 0.5 }
        visible_indices = Set(visible.keys)
        height = visible.values.map(\.maxY).max() ?? 0
    }
}

private struct TagFramesPreferenceKey: PreferenceKey {
    static var defaultValue: [Int: CGRect] = [:]

    static func reduce(value: inout [Int: CGRect], nextValue: () -> [Int: CGRect]) {
        value.merge(nextValue(), uniquingKeysWith: { _, new in new })
    }
}

struct TagsView: View {
    let tags: [[String]]
    var onCloneTapped: ((URL) -> Void)?
    var onWebTapped: ((URL) -> Void)?
    var onRelaysTapped: ((String) -> Void)? // Add this
    var onDTapped: ((String) -> Void)?
    var onSearchTapped: ((String) -> Void)? // Add this line

    @State private var totalHeight
          = CGFloat.zero
    @State private var tag_frames: [Int: CGRect] = [:]
    @State private var expanded = false

    private var visible_tags: [[String]] {
        tags.filter { !$0.isEmpty }
    }

    var body: some View {
        let preview = TagRowPreview(frames: tag_frames)
        VStack(alignment: .leading, spacing: 8) {
            GeometryReader { geometry in
                self.generateContent(in: geometry)
            }
            .frame(height: expanded || !preview.is_foldable ? totalHeight : preview.height)
            .clipped()
            if preview.is_foldable {
                Button {
                    expanded.toggle()
                } label: {
                    Label(expanded ? "Show fewer tags" : "Show all tags",
                          systemImage: expanded ? "chevron.up" : "chevron.down")
                        .font(.caption)
                }
                .buttonStyle(.borderless)
            }
        }
        .onPreferenceChange(TagFramesPreferenceKey.self) { frames in
            tag_frames = frames
        }
        .onChange(of: tags) { _ in
            expanded = false
        }
    }

    private func generateContent(in g: GeometryProxy) -> some View {
        var width = CGFloat.zero
        var height = CGFloat.zero
        let preview = TagRowPreview(frames: tag_frames)
        let tags = visible_tags

        return ZStack(alignment: .topLeading) {
            ForEach(Array(tags.enumerated()), id: \.offset) { index, tag in
                self.item(for: tag)
                    .padding([.horizontal, .vertical], 4)
                    .background {
                        GeometryReader { geometry in
                            Color.clear.preference(key: TagFramesPreferenceKey.self,
                                                   value: [index: geometry.frame(in: .named("tag-flow"))])
                        }
                    }
                    .opacity(expanded || !preview.is_foldable || preview.visible_indices.contains(index) ? 1 : 0)
                    .allowsHitTesting(expanded || !preview.is_foldable || preview.visible_indices.contains(index))
                    .accessibilityHidden(!expanded && preview.is_foldable && !preview.visible_indices.contains(index))
                    .alignmentGuide(.leading, computeValue: { d in
                        if (abs(width - d.width) > g.size.width)
                        {
                            width = 0
                            height -= d.height
                        }
                        let result = width
                        if index == tags.count - 1 {
                            width = 0 //last item
                        } else {
                            width -= d.width
                        }
                        return result
                    })
                    .alignmentGuide(.top, computeValue: {d in
                        let result = height
                        if index == tags.count - 1 {
                            height = 0 // last item
                        }
                        return result
                    })
            }
        }
        .coordinateSpace(name: "tag-flow")
        .background(viewHeightReader($totalHeight))
    }

    @ViewBuilder
    private func item(for tag: [String]) -> some View {
        let tagText = (tag.count > 1) ? "\(tag[0]): \(tag[1])" : tag[0]
        
        let tagContent = Text(tagText)
            .fixedSize(horizontal: false, vertical: true)
            // Determine font based on tag type
            .font((tag.first == "web" || tag.first == "source" || tag.first == "relays") ? .body : .footnote)
            .padding(.horizontal, 8)
            .padding(.vertical, 4)
            .background(Color.gray.opacity(0.2))
            .cornerRadius(8)

        if tag.first == "clone", tag.count > 1, let url = URL(string: tag[1]), let onCloneTapped = onCloneTapped {
            Button(action: {
                onCloneTapped(url)
            }) {
                tagContent
            }
            .buttonStyle(.plain)
        } else if (tag.first == "web" || tag.first == "source"), tag.count > 1, let url = URL(string: tag[1]), let onWebTapped = onWebTapped {
            Button(action: {
                onWebTapped(url)
            }) {
                tagContent
            }
            .buttonStyle(.plain)
        } else if tag.first == "relays", tag.count > 1, let onRelaysTapped = onRelaysTapped {
            Button(action: {
                onRelaysTapped(tag[1])
            }) {
                tagContent
            }
            .buttonStyle(.plain)
        } else if tag.first == "d", tag.count > 1, let onDTapped = onDTapped {
            Button(action: {
                onDTapped(tag[1])
            }) {
                tagContent
            }
            .buttonStyle(.plain)
        } else if tag.first == "a", tag.count > 1, let onSearchTapped = onSearchTapped {
            // Check if the "a" tag value contains ":30617:" or ":30618:"
            let a_tag_value = tag[1]
            let components = a_tag_value.components(separatedBy: ":")
            if (components.count == 3 && (components[0] == "30617" || components[0] == "30618")) {
                let searchString = components[2]
                Button(action: {
                    onSearchTapped(searchString)
                }) {
                    tagContent
                }
                .buttonStyle(.plain)
            } else {
                tagContent
            }
        } else {
            tagContent
        }
    }

    private func viewHeightReader(_ binding: Binding<CGFloat>) -> some View {
        return GeometryReader { geometry -> Color in
            let rect = geometry.frame(in: .local)
            DispatchQueue.main.async {
                binding.wrappedValue = rect.size.height
            }
            return .clear
        }
    }
}
