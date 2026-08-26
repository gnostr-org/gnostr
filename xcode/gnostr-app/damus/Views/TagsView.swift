//
//  TagsView.swift
//  damus
//
//  Created by William Casarin on 2022-05-04.
//

import SwiftUI

struct TagsView: View {
    let tags: [[String]]
    var onCloneTapped: ((URL) -> Void)?
    var onWebTapped: ((URL) -> Void)?
    var onRelaysTapped: ((String) -> Void)? // Add this
    var onDTapped: ((String) -> Void)?
    var onSearchTapped: ((String) -> Void)? // Add this line

    @State private var totalHeight
          = CGFloat.zero

    var body: some View {
        VStack {
            GeometryReader { geometry in
                self.generateContent(in: geometry)
            }
        }
        .frame(height: totalHeight)
    }

    private func generateContent(in g: GeometryProxy) -> some View {
        var width = CGFloat.zero
        var height = CGFloat.zero

        return ZStack(alignment: .topLeading) {
            ForEach(self.tags, id: \.self) { tag in
                self.item(for: tag)
                    .padding([.horizontal, .vertical], 4)
                    .alignmentGuide(.leading, computeValue: { d in
                        if (abs(width - d.width) > g.size.width)
                        {
                            width = 0
                            height -= d.height
                        }
                        let result = width
                        if tag == self.tags.last! {
                            width = 0 //last item
                        } else {
                            width -= d.width
                        }
                        return result
                    })
                    .alignmentGuide(.top, computeValue: {d in
                        let result = height
                        if tag == self.tags.last! {
                            height = 0 // last item
                        }
                        return result
                    })
            }
        }.background(viewHeightReader($totalHeight))
    }

    @ViewBuilder
    private func item(for tag: [String]) -> some View {
        let tagText = (tag.count > 1) ? "\(tag[0]): \(tag[1])" : tag[0]
        
        let tagContent = Text(tagText)
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
