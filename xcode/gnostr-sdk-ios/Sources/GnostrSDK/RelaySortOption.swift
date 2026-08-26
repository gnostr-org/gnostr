import Foundation

public enum RelaySortOption: String, CaseIterable, Identifiable {
    case urlAscending
    case urlDescending
    case pingAscending
    case pingDescending

    public var id: String { rawValue }

    public var title: String {
        switch self {
        case .urlAscending:
            return "URL A-Z"
        case .urlDescending:
            return "URL Z-A"
        case .pingAscending:
            return "Ping ↑"
        case .pingDescending:
            return "Ping ↓"
        }
    }

    public var shortTitle: String {
        switch self {
        case .urlAscending:
            return "URL A-Z"
        case .urlDescending:
            return "URL Z-A"
        case .pingAscending:
            return "Ping ↑"
        case .pingDescending:
            return "Ping ↓"
        }
    }

    public func sort(relays: [Relay]) -> [Relay] {
        relays.sorted { lhs, rhs in
            let leftRank = stateRank(for: lhs)
            let rightRank = stateRank(for: rhs)
            if leftRank != rightRank {
                return leftRank < rightRank
            }

            switch self {
            case .urlAscending:
                return lhs.url.absoluteString < rhs.url.absoluteString
            case .urlDescending:
                return lhs.url.absoluteString > rhs.url.absoluteString
            case .pingAscending:
                return pingSortComparison(lhs, rhs, ascending: true)
            case .pingDescending:
                return pingSortComparison(lhs, rhs, ascending: false)
            }
        }
    }

    private func pingSortComparison(_ lhs: Relay, _ rhs: Relay, ascending: Bool) -> Bool {
        let leftPing = lhs.connectionLatency
        let rightPing = rhs.connectionLatency
        switch (leftPing, rightPing) {
        case let (left?, right?):
            if left != right {
                return ascending ? left < right : left > right
            }
        case (.some, .none):
            return true
        case (.none, .some):
            return false
        case (.none, .none):
            break
        }

        if ascending {
            return lhs.url.absoluteString < rhs.url.absoluteString
        } else {
            return lhs.url.absoluteString > rhs.url.absoluteString
        }
    }

    private func stateRank(for relay: Relay) -> Int {
        switch relay.state {
        case .connected:
            return 0
        case .connecting:
            return 1
        case .notConnected, .error:
            return 2
        }
    }
}
