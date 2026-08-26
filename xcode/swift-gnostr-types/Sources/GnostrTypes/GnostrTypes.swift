import Foundation

public enum EventKind: UInt32, Codable, Sendable {
    case metadata = 0
    case textNote = 1
    case recommendRelay = 2
    case contactList = 3
    case encryptedDirectMessage = 4
    case eventDeletion = 5
    case repost = 6
    case reaction = 7
    case badgeAward = 8
    case groupChatMessage = 9
    case groupChatThreadedReply = 10
    case groupChatThread = 11
    case groupChatReply = 12
    case seal = 13
    case dmChat = 14
    case genericRepost = 16
    case channelCreation = 40
    case channelMetadata = 41
    case channelMessage = 42
    case channelHideMessage = 43
    case channelMuteUser = 44
    case bid = 1021
    case bidConfirmation = 1022
    case timestamp = 1040
    case giftWrap = 1059
    case fileMetadata = 1063
    case liveChatMessage = 1311
    case patches = 1617
    case gitIssue = 1621
    case gitReply = 1622
    case gitStatusOpen = 1630
    case gitStatusApplied = 1631
    case gitStatusClosed = 1632
    case gitStatusDraft = 1633
    case followSets = 30000
    case genericSets = 30001
    case relaySets = 30002
    case bookmarkSets = 30003
    case curationSets = 30004
    case profileBadges = 30008
    case badgeDefinition = 30009
    case interestSets = 30015
    case createOrUpdateStall = 30017
    case createOrUpdateProduct = 30018
    case marketplaceUi = 30019
    case productSoldAuction = 30020
    case longFormContent = 30023
    case draftLongFormContent = 30024
    case emojiSets = 30030
    case releaseArtifactSets = 30063
    case appSpecificData = 30078
    case liveEvent = 30311
    case userStatus = 30315
    case classifiedListing = 30402
    case draftClassifiedListing = 30403
    case repoAnnouncement = 30617
    case gitRepoAnnouncement = 30618
    case wikiArticle = 30818
    case dateBasedCalendarEvent = 31922
    case timeBasedCalendarEvent = 31923
    case calendar = 31924
    case calendarEventRsvp = 31925
    case handlerRecommendation = 31989
    case handlerInformation = 31990
    case communityDefinition = 34550
    case other = 0xffff_fffe
}

public struct Id: Codable, Hashable, Sendable, CustomStringConvertible {
    public let hex: String

    public init(hex: String) {
        self.hex = hex.lowercased()
    }

    public var description: String { self.hex }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        self.hex = try container.decode(String.self).lowercased()
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(self.hex)
    }
}

public struct PublicKey: Codable, Hashable, Sendable, CustomStringConvertible {
    public let hex: String

    public init(hex: String) {
        self.hex = hex.lowercased()
    }

    public var description: String { self.hex }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        self.hex = try container.decode(String.self).lowercased()
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(self.hex)
    }
}

public struct Signature: Codable, Hashable, Sendable, CustomStringConvertible {
    public let hex: String

    public init(hex: String) {
        self.hex = hex.lowercased()
    }

    public var description: String { self.hex }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        self.hex = try container.decode(String.self).lowercased()
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(self.hex)
    }
}

public struct Unixtime: Codable, Hashable, Sendable, CustomStringConvertible {
    public let value: Int64

    public init(_ value: Int64) {
        self.value = value
    }

    public var description: String { String(self.value) }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        self.value = try container.decode(Int64.self)
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(self.value)
    }
}

public struct Tag: Codable, Hashable, Sendable {
    public var fields: [String]

    public init(_ fields: [String]) {
        self.fields = fields
    }

    public static func event(_ id: Id, marker: String? = nil) -> Tag {
        var fields = ["e", id.hex]
        if let marker {
            fields.append(marker)
        }
        return Tag(fields)
    }

    public static func pubkey(_ key: PublicKey, relay: String? = nil, marker: String? = nil) -> Tag {
        var fields = ["p", key.hex]
        if let relay {
            fields.append(relay)
        }
        if let marker {
            fields.append(marker)
        }
        return Tag(fields)
    }

    public func tagName() -> String { self.fields.first ?? "" }
    public func value() -> String { self.fields.dropFirst().first ?? "" }

    public init(from decoder: any Decoder) throws {
        var container = try decoder.unkeyedContainer()
        var fields: [String] = []
        while !container.isAtEnd {
            fields.append(try container.decode(String.self))
        }
        self.fields = fields
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.unkeyedContainer()
        for field in self.fields {
            try container.encode(field)
        }
    }
}

public struct PreEvent: Codable, Hashable, Sendable {
    public var pubkey: PublicKey
    public var createdAt: Unixtime
    public var kind: EventKind
    public var tags: [Tag]
    public var content: String

    public init(pubkey: PublicKey, createdAt: Unixtime, kind: EventKind, tags: [Tag], content: String) {
        self.pubkey = pubkey
        self.createdAt = createdAt
        self.kind = kind
        self.tags = tags
        self.content = content
    }

    private enum CodingKeys: String, CodingKey {
        case pubkey
        case createdAt = "created_at"
        case kind
        case tags
        case content
    }
}

public struct Event: Codable, Hashable, Sendable {
    public var id: Id
    public var pubkey: PublicKey
    public var createdAt: Unixtime
    public var kind: EventKind
    public var sig: Signature
    public var content: String
    public var tags: [Tag]

    public init(id: Id, pubkey: PublicKey, createdAt: Unixtime, kind: EventKind, sig: Signature, content: String, tags: [Tag]) {
        self.id = id
        self.pubkey = pubkey
        self.createdAt = createdAt
        self.kind = kind
        self.sig = sig
        self.content = content
        self.tags = tags
    }

    private enum CodingKeys: String, CodingKey {
        case id
        case pubkey
        case createdAt = "created_at"
        case kind
        case sig
        case content
        case tags
    }
}

public struct GitNote: Codable, Hashable, Sendable {
    public let noteID: String
    public let annotatedID: String
    public let notesRef: String?
    public let message: String
    public let author: String
    public let committer: String
    public let committerTime: Int64

    public init(
        noteID: String,
        annotatedID: String,
        notesRef: String?,
        message: String,
        author: String,
        committer: String,
        committerTime: Int64
    ) {
        self.noteID = noteID.lowercased()
        self.annotatedID = annotatedID.lowercased()
        self.notesRef = notesRef
        self.message = message
        self.author = author
        self.committer = committer
        self.committerTime = committerTime
    }

    private enum CodingKeys: String, CodingKey {
        case noteID = "note_id"
        case annotatedID = "annotated_id"
        case notesRef = "notes_ref"
        case message
        case author
        case committer
        case committerTime = "committer_time"
    }
}

public enum RepoState: String, Codable, Sendable {
    case clean
    case merge
    case rebase
    case revert
    case other
}

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

    private enum CodingKeys: String, CodingKey {
        case name
        case description
        case identifier
        case rootCommit = "root_commit"
        case gitServer = "git_server"
        case web
        case relays
        case blossoms
        case hashtags
        case maintainers
        case trustedMaintainer = "trusted_maintainer"
        case maintainersWithoutAnnouncement = "maintainers_without_announcement"
    }
}

public struct NAddr: Codable, Hashable, Sendable {
    public var d: String
    public var relays: [String]
    public var kind: EventKind
    public var author: PublicKey

    public init(d: String, relays: [String], kind: EventKind, author: PublicKey) {
        self.d = d
        self.relays = relays
        self.kind = kind
        self.author = author
    }
}

public enum NIP34 {
    public static let repoAnnouncementKind: UInt32 = 30617
    public static let repoStateKind: UInt32 = 30618
    public static let patchesKind: UInt32 = 1617
    public static let gitStatusOpenKind: UInt32 = 1630
    public static let gitStatusAppliedKind: UInt32 = 1631
    public static let gitStatusClosedKind: UInt32 = 1632
    public static let gitStatusDraftKind: UInt32 = 1633

    public static var rustBridgeAvailable: Bool {
        RustGnostrTypesBridge.shared.isAvailable
    }

    public static func rustGitNoteEventID(commitID: String) -> String? {
        RustGnostrTypesBridge.shared.gitNoteEventID(commitID: commitID)
    }

    public static func rustGitNoteTags(note: GitNote) throws -> [Tag] {
        try RustGnostrTypesBridge.shared.gitNoteTags(note: note)
    }

    public static func rustGenerateGitNoteEvent(
        note: GitNote,
        privateKeyHex: String,
        powTargetBits: UInt8 = 0
    ) throws -> Event {
        try RustGnostrTypesBridge.shared.generateGitNoteEvent(
            note: note,
            privateKeyHex: privateKeyHex,
            powTargetBits: powTargetBits
        )
    }

    public static func rustNormalizeEvent(_ event: Event) throws -> Event {
        try RustGnostrTypesBridge.shared.normalize(event)
    }

    public static func rustNormalizePreEvent(_ preEvent: PreEvent) throws -> PreEvent {
        try RustGnostrTypesBridge.shared.normalize(preEvent)
    }

    public static func rustNormalizeTag(_ tag: Tag) throws -> Tag {
        try RustGnostrTypesBridge.shared.normalize(tag)
    }

    public static func rustNormalizeNAddr(_ naddr: NAddr) throws -> NAddr {
        try RustGnostrTypesBridge.shared.normalize(naddr)
    }
}

public func getLeadingZeroBits(_ bytes: [UInt8]) -> UInt8 {
    var count: UInt8 = 0
    for byte in bytes {
        if byte == 0 {
            count += 8
            continue
        }
        count += UInt8(byte.leadingZeroBitCount)
        break
    }
    return count
}

public func statusKinds() -> [EventKind] {
    [.gitStatusOpen, .gitStatusApplied, .gitStatusClosed, .gitStatusDraft]
}
