import Foundation

public struct RepoRef: Codable, Hashable, Sendable {
    public var name: String
    public var description: String
    public var identifier: String
    public var rootCommit: String
    public var gitServer: [String]
    public var web: [String]
    public var relays: [String]
    public var blossoms: [String]
    public var hashtags: [String]
    public var maintainers: [String]
    public var trustedMaintainer: String
    public var maintainersWithoutAnnouncement: [String]?

    public init(
        name: String,
        description: String,
        identifier: String,
        rootCommit: String,
        gitServer: [String] = [],
        web: [String] = [],
        relays: [String] = [],
        blossoms: [String] = [],
        hashtags: [String] = [],
        maintainers: [String] = [],
        trustedMaintainer: String,
        maintainersWithoutAnnouncement: [String]? = nil
    ) {
        self.name = name
        self.description = description
        self.identifier = identifier
        self.rootCommit = rootCommit
        self.gitServer = gitServer
        self.web = web
        self.relays = relays
        self.blossoms = blossoms
        self.hashtags = hashtags
        self.maintainers = maintainers
        self.trustedMaintainer = trustedMaintainer
        self.maintainersWithoutAnnouncement = maintainersWithoutAnnouncement
    }
}
