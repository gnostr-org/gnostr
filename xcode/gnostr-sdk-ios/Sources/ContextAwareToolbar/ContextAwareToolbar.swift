import Foundation
import SwiftUI

public enum ContextAwareSortOrder: String, CaseIterable, Identifiable {
    case urlAscending
    case urlDescending

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .urlAscending:
            return "URL A-Z"
        case .urlDescending:
            return "URL Z-A"
        }
    }

    public var toggled: Self {
        switch self {
        case .urlAscending:
            return .urlDescending
        case .urlDescending:
            return .urlAscending
        }
    }

    public var toggleTitle: String {
        switch self {
        case .urlAscending:
            return "A-Z"
        case .urlDescending:
            return "Z-A"
        }
    }

    public func sort(urls: [URL]) -> [URL] {
        switch self {
        case .urlAscending:
            return urls.sorted { $0.absoluteString < $1.absoluteString }
        case .urlDescending:
            return urls.sorted { $0.absoluteString > $1.absoluteString }
        }
    }
}

public struct ContextAwareSortChipButton: View {
    private let title: String
    private let isSelected: Bool
    private let action: () -> Void

    public init(title: String, isSelected: Bool, action: @escaping () -> Void) {
        self.title = title
        self.isSelected = isSelected
        self.action = action
    }

    public var body: some View {
        Button(action: action) {
            Text(title)
                .font(.caption.weight(.semibold))
                .padding(.horizontal, 10)
                .padding(.vertical, 6)
                .background(
                    RoundedRectangle(cornerRadius: 999, style: .continuous)
                        .fill(isSelected ? Color.accentColor.opacity(0.16) : Color(.tertiarySystemFill))
                )
        }
        .buttonStyle(.plain)
    }
}

public struct ContextAwareActionChipButton: View {
    private let title: String
    private let systemImage: String
    private let role: ButtonRole?
    private let isEnabled: Bool
    private let action: () -> Void

    public init(title: String,
                systemImage: String,
                role: ButtonRole? = nil,
                isEnabled: Bool = true,
                action: @escaping () -> Void) {
        self.title = title
        self.systemImage = systemImage
        self.role = role
        self.isEnabled = isEnabled
        self.action = action
    }

    public var body: some View {
        Button(role: role, action: action) {
            Label {
                Text(title)
                    .font(.caption.weight(.semibold))
            } icon: {
                Image(systemName: systemImage)
            }
            .padding(.horizontal, 10)
            .padding(.vertical, 6)
            .foregroundStyle(role == .destructive ? Color.red : Color.primary)
            .background(
                RoundedRectangle(cornerRadius: 999, style: .continuous)
                    .fill(isEnabled ? Color(.tertiarySystemFill) : Color(.tertiarySystemFill).opacity(0.5))
            )
        }
        .buttonStyle(.plain)
        .disabled(isEnabled == false)
    }
}

public struct ContextAwareListToolbar<Content: View, Trailing: View>: View {
    private let content: Content
    private let trailing: Trailing
    private let horizontalPadding: CGFloat

    public init(@ViewBuilder content: () -> Content,
                @ViewBuilder trailing: () -> Trailing,
                horizontalPadding: CGFloat = 0) {
        self.content = content()
        self.trailing = trailing()
        self.horizontalPadding = horizontalPadding
    }

    public var body: some View {
        HStack {
            content
            Spacer(minLength: 12)
            trailing
        }
        .padding(.horizontal, horizontalPadding)
        .padding(.vertical, 8)
    }
}

public extension ContextAwareListToolbar where Trailing == EmptyView {
    init(horizontalPadding: CGFloat = 0,
         @ViewBuilder content: () -> Content) {
        self.init(content: content, trailing: { EmptyView() }, horizontalPadding: horizontalPadding)
    }
}

public struct ContextAwareSortToggleChip<Selection: Equatable>: View {
    @Binding private var selection: Selection
    private let ascending: Selection
    private let descending: Selection
    private let ascendingTitle: String
    private let descendingTitle: String

    public init(selection: Binding<Selection>, ascending: Selection, descending: Selection, ascendingTitle: String, descendingTitle: String) {
        _selection = selection
        self.ascending = ascending
        self.descending = descending
        self.ascendingTitle = ascendingTitle
        self.descendingTitle = descendingTitle
    }

    private var currentTitle: String {
        selection == descending ? descendingTitle : ascendingTitle
    }

    private var isSelected: Bool {
        selection == ascending || selection == descending
    }

    public var body: some View {
        ContextAwareSortChipButton(title: currentTitle, isSelected: isSelected) {
            selection = selection == ascending ? descending : ascending
        }
    }
}
