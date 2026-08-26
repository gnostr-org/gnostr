import Foundation

public struct DynamicCodingKey: CodingKey {
    public var stringValue: String
    public var intValue: Int?

    public init?(stringValue: String) {
        self.stringValue = stringValue
        self.intValue = nil
    }

    public init?(intValue: Int) {
        self.stringValue = String(intValue)
        self.intValue = intValue
    }
}

public enum JSONValue: Codable, Hashable, Sendable {
    case string(String)
    case int(Int64)
    case double(Double)
    case bool(Bool)
    case array([JSONValue])
    case object([String: JSONValue])
    case null

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        if container.decodeNil() {
            self = .null
        } else if let value = try? container.decode(Bool.self) {
            self = .bool(value)
        } else if let value = try? container.decode(Int64.self) {
            self = .int(value)
        } else if let value = try? container.decode(Double.self) {
            self = .double(value)
        } else if let value = try? container.decode(String.self) {
            self = .string(value)
        } else if let value = try? decoder.container(keyedBy: DynamicCodingKey.self) {
            var object: [String: JSONValue] = [:]
            for key in value.allKeys {
                object[key.stringValue] = try value.decode(JSONValue.self, forKey: key)
            }
            self = .object(object)
        } else {
            var arrayContainer = try decoder.unkeyedContainer()
            var values: [JSONValue] = []
            while !arrayContainer.isAtEnd {
                values.append(try arrayContainer.decode(JSONValue.self))
            }
            self = .array(values)
        }
    }

    public func encode(to encoder: any Encoder) throws {
        switch self {
        case .string(let value):
            var container = encoder.singleValueContainer()
            try container.encode(value)
        case .int(let value):
            var container = encoder.singleValueContainer()
            try container.encode(value)
        case .double(let value):
            var container = encoder.singleValueContainer()
            try container.encode(value)
        case .bool(let value):
            var container = encoder.singleValueContainer()
            try container.encode(value)
        case .array(let values):
            var container = encoder.unkeyedContainer()
            for value in values {
                try container.encode(value)
            }
        case .object(let values):
            var container = encoder.container(keyedBy: DynamicCodingKey.self)
            for (key, value) in values {
                guard let codingKey = DynamicCodingKey(stringValue: key) else { continue }
                try container.encode(value, forKey: codingKey)
            }
        case .null:
            var container = encoder.singleValueContainer()
            try container.encodeNil()
        }
    }
}

public struct UncheckedUrl: Codable, Hashable, Sendable, CustomStringConvertible {
    public var string: String

    public init(_ string: String) {
        self.string = string
    }

    public var description: String { self.string }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        self.string = try container.decode(String.self)
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(self.string)
    }
}

public typealias RelayUrl = UncheckedUrl
public typealias Url = UncheckedUrl
public typealias NostrUrl = UncheckedUrl

public typealias IdHex = Id
public typealias PublicKeyHex = PublicKey
public typealias SignatureHex = Signature

public struct SubscriptionId: Codable, Hashable, Sendable, CustomStringConvertible {
    public var value: String

    public init(_ value: String) {
        self.value = value
    }

    public var description: String { self.value }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        self.value = try container.decode(String.self)
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(self.value)
    }
}

public struct MilliSatoshi: Codable, Hashable, Sendable, CustomStringConvertible, Comparable {
    public var value: UInt64

    public init(_ value: UInt64) {
        self.value = value
    }

    public var description: String { String(self.value) }

    public static func < (lhs: MilliSatoshi, rhs: MilliSatoshi) -> Bool {
        lhs.value < rhs.value
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.singleValueContainer()
        self.value = try container.decode(UInt64.self)
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.singleValueContainer()
        try container.encode(self.value)
    }
}

public struct ImageDimensions: Codable, Hashable, Sendable {
    public var width: UInt64
    public var height: UInt64

    public init(width: UInt64, height: UInt64) {
        self.width = width
        self.height = height
    }
}

public struct EventKindOrRange: Codable, Hashable, Sendable {
    public enum Value: Codable, Hashable, Sendable {
        case kind(EventKind)
        case range([EventKind])

        public init(from decoder: any Decoder) throws {
            let container = try decoder.singleValueContainer()
            if let kind = try? container.decode(EventKind.self) {
                self = .kind(kind)
            } else {
                self = .range(try container.decode([EventKind].self))
            }
        }

        public func encode(to encoder: any Encoder) throws {
            switch self {
            case .kind(let kind):
                var container = encoder.singleValueContainer()
                try container.encode(kind)
            case .range(let kinds):
                var container = encoder.singleValueContainer()
                try container.encode(kinds)
            }
        }
    }

    public var value: Value

    public init(_ value: Value) {
        self.value = value
    }

    public init(from decoder: any Decoder) throws {
        self.value = try Value(from: decoder)
    }

    public func encode(to encoder: any Encoder) throws {
        try self.value.encode(to: encoder)
    }
}

public struct RelayLimitation: Codable, Hashable, Sendable {
    public var maxMessageLength: Int?
    public var maxSubscriptions: Int?
    public var maxFilters: Int?
    public var maxLimit: Int?
    public var maxSubidLength: Int?
    public var maxEventTags: Int?
    public var maxContentLength: Int?
    public var minPowDifficulty: Int?
    public var authRequired: Bool?
    public var paymentRequired: Bool?
    public var restrictedWrites: Bool?
    public var createdAtLowerLimit: UInt64?
    public var createdAtUpperLimit: UInt64?
}

public struct RelayRetention: Codable, Hashable, Sendable {
    public var kinds: [EventKindOrRange]
    public var time: Int?
    public var count: Int?

    public init(kinds: [EventKindOrRange] = [], time: Int? = nil, count: Int? = nil) {
        self.kinds = kinds
        self.time = time
        self.count = count
    }
}

public struct Fee: Codable, Hashable, Sendable {
    public var amount: Int
    public var unit: String
    public var kinds: [EventKindOrRange]
    public var period: Int?

    public init(amount: Int, unit: String, kinds: [EventKindOrRange] = [], period: Int? = nil) {
        self.amount = amount
        self.unit = unit
        self.kinds = kinds
        self.period = period
    }
}

public struct RelayFees: Codable, Hashable, Sendable {
    public var admission: [Fee]
    public var subscription: [Fee]
    public var publication: [Fee]

    public init(admission: [Fee] = [], subscription: [Fee] = [], publication: [Fee] = []) {
        self.admission = admission
        self.subscription = subscription
        self.publication = publication
    }
}

public struct RelayInformationDocument: Codable, Hashable, Sendable {
    public var name: String?
    public var description: String?
    public var banner: Url?
    public var icon: Url?
    public var pubkey: PublicKeyHex?
    public var selfPubkey: PublicKeyHex?
    public var contact: String?
    public var supportedNips: [UInt32]
    public var software: String?
    public var version: String?
    public var limitation: RelayLimitation?
    public var retention: [RelayRetention]
    public var relayCountries: [String]
    public var languageTags: [String]
    public var tags: [String]
    public var postingPolicy: Url?
    public var paymentsUrl: Url?
    public var fees: RelayFees?
    public var other: [String: JSONValue]

    public init(
        name: String? = nil,
        description: String? = nil,
        banner: Url? = nil,
        icon: Url? = nil,
        pubkey: PublicKeyHex? = nil,
        selfPubkey: PublicKeyHex? = nil,
        contact: String? = nil,
        supportedNips: [UInt32] = [],
        software: String? = nil,
        version: String? = nil,
        limitation: RelayLimitation? = nil,
        retention: [RelayRetention] = [],
        relayCountries: [String] = [],
        languageTags: [String] = [],
        tags: [String] = [],
        postingPolicy: Url? = nil,
        paymentsUrl: Url? = nil,
        fees: RelayFees? = nil,
        other: [String: JSONValue] = [:]
    ) {
        self.name = name
        self.description = description
        self.banner = banner
        self.icon = icon
        self.pubkey = pubkey
        self.selfPubkey = selfPubkey
        self.contact = contact
        self.supportedNips = supportedNips
        self.software = software
        self.version = version
        self.limitation = limitation
        self.retention = retention
        self.relayCountries = relayCountries
        self.languageTags = languageTags
        self.tags = tags
        self.postingPolicy = postingPolicy
        self.paymentsUrl = paymentsUrl
        self.fees = fees
        self.other = other
    }
}

public struct Metadata: Codable, Hashable, Sendable {
    public var name: String?
    public var about: String?
    public var picture: String?
    public var nip05: String?
    public var other: [String: JSONValue]

    public init(name: String? = nil, about: String? = nil, picture: String? = nil, nip05: String? = nil, other: [String: JSONValue] = [:]) {
        self.name = name
        self.about = about
        self.picture = picture
        self.nip05 = nip05
        self.other = other
    }
}

public struct Profile: Codable, Hashable, Sendable {
    public var pubkey: PublicKey
    public var relays: [RelayUrl]

    public init(pubkey: PublicKey, relays: [RelayUrl] = []) {
        self.pubkey = pubkey
        self.relays = relays
    }
}

public struct NEvent: Codable, Hashable, Sendable {
    public var id: Id
    public var relays: [RelayUrl]
    public var kind: EventKind?
    public var author: PublicKey?

    public init(id: Id, relays: [RelayUrl] = [], kind: EventKind? = nil, author: PublicKey? = nil) {
        self.id = id
        self.relays = relays
        self.kind = kind
        self.author = author
    }
}

public struct NProfile: Codable, Hashable, Sendable {
    public var pubkey: PublicKey
    public var relays: [RelayUrl]

    public init(pubkey: PublicKey, relays: [RelayUrl] = []) {
        self.pubkey = pubkey
        self.relays = relays
    }
}

public struct NRelay: Codable, Hashable, Sendable {
    public var relay: RelayUrl

    public init(relay: RelayUrl) {
        self.relay = relay
    }
}

public struct SimpleRelayUsage: Codable, Hashable, Sendable {
    public var write: Bool
    public var read: Bool

    public init(write: Bool = false, read: Bool = true) {
        self.write = write
        self.read = read
    }
}

public struct SimpleRelayList: Codable, Hashable, Sendable {
    public var relays: [UncheckedUrl: SimpleRelayUsage]

    public init(_ relays: [UncheckedUrl: SimpleRelayUsage] = [:]) {
        self.relays = relays
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: DynamicCodingKey.self)
        var relays: [UncheckedUrl: SimpleRelayUsage] = [:]
        for key in container.allKeys {
            relays[UncheckedUrl(key.stringValue)] = try container.decode(SimpleRelayUsage.self, forKey: key)
        }
        self.relays = relays
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        for (relay, usage) in self.relays {
            try container.encode(usage, forKey: DynamicCodingKey(stringValue: relay.string)!)
        }
    }
}

public enum RelayListUsage: String, Codable, Hashable, Sendable {
    case inbox = "read"
    case outbox = "write"
    case both = "both"
}

public struct RelayList: Codable, Hashable, Sendable {
    public var relays: [RelayUrl: RelayListUsage]

    public init(_ relays: [RelayUrl: RelayListUsage] = [:]) {
        self.relays = relays
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: DynamicCodingKey.self)
        var relays: [RelayUrl: RelayListUsage] = [:]
        for key in container.allKeys {
            relays[RelayUrl(key.stringValue)] = try container.decode(RelayListUsage.self, forKey: key)
        }
        self.relays = relays
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        for (relay, usage) in self.relays {
            try container.encode(usage, forKey: DynamicCodingKey(stringValue: relay.string)!)
        }
    }
}

public struct Filter: Codable, Hashable, Sendable {
    public var ids: [IdHex]
    public var authors: [PublicKeyHex]
    public var kinds: [EventKind]
    public var tags: [String: [String]]
    public var since: Unixtime?
    public var until: Unixtime?
    public var limit: Int?

    public init(
        ids: [IdHex] = [],
        authors: [PublicKeyHex] = [],
        kinds: [EventKind] = [],
        tags: [String: [String]] = [:],
        since: Unixtime? = nil,
        until: Unixtime? = nil,
        limit: Int? = nil
    ) {
        self.ids = ids
        self.authors = authors
        self.kinds = kinds
        self.tags = tags
        self.since = since
        self.until = until
        self.limit = limit
    }
}

public struct PayRequestData: Codable, Hashable, Sendable {
    public var callback: UncheckedUrl
    public struct MetadataItem: Codable, Hashable, Sendable {
        public var key: String
        public var value: String

        public init(key: String, value: String) {
            self.key = key
            self.value = value
        }
    }

    public var metadata: [MetadataItem]
    public var allowsNostr: Bool?
    public var nostrPubkey: PublicKeyHex?
    public var other: [String: JSONValue]

    public init(
        callback: UncheckedUrl,
        metadata: [MetadataItem] = [],
        allowsNostr: Bool? = nil,
        nostrPubkey: PublicKeyHex? = nil,
        other: [String: JSONValue] = [:]
    ) {
        self.callback = callback
        self.metadata = metadata
        self.allowsNostr = allowsNostr
        self.nostrPubkey = nostrPubkey
        self.other = other
    }
}

public enum ClientMessage: Codable, Hashable, Sendable {
    case event(Event)
    case req(SubscriptionId, [Filter])
    case close(SubscriptionId)
    case auth(Event)
}

public enum Why: String, Codable, Hashable, Sendable {
    case authRequired
    case blocked
    case duplicate
    case error
    case invalid
    case pow
    case rateLimited
    case restricted
}

public enum RelayMessage: Codable, Hashable, Sendable {
    case auth(String)
    case closed(SubscriptionId, String)
    case eose(SubscriptionId)
    case event(SubscriptionId, Event)
    case notice(String)
    case notify(String)
    case ok(Id, Bool, String)
}

public extension Tag {
    static func relay(_ relay: RelayUrl, marker: String? = nil) -> Tag {
        var fields = ["r", relay.string]
        if let marker {
            fields.append(marker)
        }
        return Tag(fields)
    }

    static func hashtag(_ value: String) -> Tag {
        Tag(["t", value])
    }

    static func subject(_ value: String) -> Tag {
        Tag(["subject", value])
    }

    static func newRelay(_ relay: RelayUrl, marker: String? = nil) -> Tag {
        var fields = ["r", relay.string]
        if let marker {
            fields.append(marker)
        }
        return Tag(fields)
    }

    static func newHashtag(_ value: String) -> Tag {
        hashtag(value)
    }

    static func newSubject(_ value: String) -> Tag {
        subject(value)
    }

    func parseRelay() -> (UncheckedUrl, String?)? {
        guard self.tagName() == "r", !self.fields.isEmpty else { return nil }
        let relay = UncheckedUrl(self.value())
        return (relay, self.fields.dropFirst(2).first)
    }
}

extension Metadata {
    private enum CodingKeys: String, CodingKey {
        case name
        case about
        case picture
        case nip05
    }

    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: DynamicCodingKey.self)
        var other: [String: JSONValue] = [:]
        for key in container.allKeys {
            other[key.stringValue] = try container.decode(JSONValue.self, forKey: key)
        }
        self.name = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "name")!)
        self.about = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "about")!)
        self.picture = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "picture")!)
        self.nip05 = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "nip05")!)
        other.removeValue(forKey: "name")
        other.removeValue(forKey: "about")
        other.removeValue(forKey: "picture")
        other.removeValue(forKey: "nip05")
        self.other = other
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        try container.encode(self.name.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "name")!)
        try container.encode(self.about.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "about")!)
        try container.encode(self.picture.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "picture")!)
        try container.encode(self.nip05.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "nip05")!)
        for (key, value) in self.other {
            try container.encode(value, forKey: DynamicCodingKey(stringValue: key)!)
        }
    }
}

extension PayRequestData {
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: DynamicCodingKey.self)
        let callbackKey = DynamicCodingKey(stringValue: "callback")!
        self.callback = try container.decode(UncheckedUrl.self, forKey: callbackKey)

        if let metadataKey = DynamicCodingKey(stringValue: "metadata"),
           var nested = try? container.nestedUnkeyedContainer(forKey: metadataKey) {
            var items: [MetadataItem] = []
            while !nested.isAtEnd {
                var pair = try nested.nestedUnkeyedContainer()
                let key = try pair.decode(String.self)
                let value = try pair.decode(String.self)
                items.append(MetadataItem(key: key, value: value))
            }
            self.metadata = items
        } else {
            self.metadata = []
        }

        self.allowsNostr = try container.decodeIfPresent(Bool.self, forKey: DynamicCodingKey(stringValue: "allowsNostr")!)
        self.nostrPubkey = try container.decodeIfPresent(PublicKeyHex.self, forKey: DynamicCodingKey(stringValue: "nostrPubkey")!)

        var other: [String: JSONValue] = [:]
        for key in container.allKeys {
            let name = key.stringValue
            if name == "callback" || name == "metadata" || name == "allowsNostr" || name == "nostrPubkey" {
                continue
            }
            other[name] = try container.decode(JSONValue.self, forKey: key)
        }
        self.other = other
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        try container.encode(self.callback, forKey: DynamicCodingKey(stringValue: "callback")!)

        var metadata = container.nestedUnkeyedContainer(forKey: DynamicCodingKey(stringValue: "metadata")!)
        for item in self.metadata {
            var pair = metadata.nestedUnkeyedContainer()
            try pair.encode(item.key)
            try pair.encode(item.value)
        }

        try container.encode(self.allowsNostr.map(JSONValue.bool) ?? .null, forKey: DynamicCodingKey(stringValue: "allowsNostr")!)
        try container.encode(self.nostrPubkey.map { JSONValue.string($0.hex) } ?? .null, forKey: DynamicCodingKey(stringValue: "nostrPubkey")!)

        for (key, value) in self.other {
            try container.encode(value, forKey: DynamicCodingKey(stringValue: key)!)
        }
    }
}

extension RelayInformationDocument {
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: DynamicCodingKey.self)
        self.name = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "name")!)
        self.description = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "description")!)
        self.banner = try container.decodeIfPresent(Url.self, forKey: DynamicCodingKey(stringValue: "banner")!)
        self.icon = try container.decodeIfPresent(Url.self, forKey: DynamicCodingKey(stringValue: "icon")!)
        self.pubkey = try container.decodeIfPresent(PublicKeyHex.self, forKey: DynamicCodingKey(stringValue: "pubkey")!)
        self.selfPubkey = try container.decodeIfPresent(PublicKeyHex.self, forKey: DynamicCodingKey(stringValue: "self_pubkey")!)
        self.contact = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "contact")!)
        self.supportedNips = try container.decodeIfPresent([UInt32].self, forKey: DynamicCodingKey(stringValue: "supported_nips")!) ?? []
        self.software = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "software")!)
        self.version = try container.decodeIfPresent(String.self, forKey: DynamicCodingKey(stringValue: "version")!)
        self.limitation = try container.decodeIfPresent(RelayLimitation.self, forKey: DynamicCodingKey(stringValue: "limitation")!)
        self.retention = try container.decodeIfPresent([RelayRetention].self, forKey: DynamicCodingKey(stringValue: "retention")!) ?? []
        self.relayCountries = try container.decodeIfPresent([String].self, forKey: DynamicCodingKey(stringValue: "relay_countries")!) ?? []
        self.languageTags = try container.decodeIfPresent([String].self, forKey: DynamicCodingKey(stringValue: "language_tags")!) ?? []
        self.tags = try container.decodeIfPresent([String].self, forKey: DynamicCodingKey(stringValue: "tags")!) ?? []
        self.postingPolicy = try container.decodeIfPresent(Url.self, forKey: DynamicCodingKey(stringValue: "posting_policy")!)
        self.paymentsUrl = try container.decodeIfPresent(Url.self, forKey: DynamicCodingKey(stringValue: "payments_url")!)
        self.fees = try container.decodeIfPresent(RelayFees.self, forKey: DynamicCodingKey(stringValue: "fees")!)

        var other: [String: JSONValue] = [:]
        for key in container.allKeys {
            let name = key.stringValue
            if [
                "name", "description", "banner", "icon", "pubkey", "self_pubkey", "contact",
                "supported_nips", "software", "version", "limitation", "retention",
                "relay_countries", "language_tags", "tags", "posting_policy", "payments_url",
                "fees"
            ].contains(name) {
                continue
            }
            other[name] = try container.decode(JSONValue.self, forKey: key)
        }
        self.other = other
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        try container.encode(self.name.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "name")!)
        try container.encode(self.description.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "description")!)
        try container.encodeIfPresent(self.banner, forKey: DynamicCodingKey(stringValue: "banner")!)
        try container.encodeIfPresent(self.icon, forKey: DynamicCodingKey(stringValue: "icon")!)
        try container.encode(self.pubkey.map { JSONValue.string($0.hex) } ?? .null, forKey: DynamicCodingKey(stringValue: "pubkey")!)
        try container.encodeIfPresent(self.selfPubkey, forKey: DynamicCodingKey(stringValue: "self_pubkey")!)
        try container.encode(self.contact.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "contact")!)
        try container.encode(self.supportedNips, forKey: DynamicCodingKey(stringValue: "supported_nips")!)
        try container.encode(self.software.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "software")!)
        try container.encode(self.version.map(JSONValue.string) ?? .null, forKey: DynamicCodingKey(stringValue: "version")!)
        if let limitation = self.limitation {
            try container.encode(limitation, forKey: DynamicCodingKey(stringValue: "limitation")!)
        } else {
            try container.encodeNil(forKey: DynamicCodingKey(stringValue: "limitation")!)
        }
        if !self.retention.isEmpty {
            try container.encode(self.retention, forKey: DynamicCodingKey(stringValue: "retention")!)
        }
        if !self.relayCountries.isEmpty {
            try container.encode(self.relayCountries, forKey: DynamicCodingKey(stringValue: "relay_countries")!)
        }
        if !self.languageTags.isEmpty {
            try container.encode(self.languageTags, forKey: DynamicCodingKey(stringValue: "language_tags")!)
        }
        if !self.tags.isEmpty {
            try container.encode(self.tags, forKey: DynamicCodingKey(stringValue: "tags")!)
        }
        try container.encodeIfPresent(self.postingPolicy, forKey: DynamicCodingKey(stringValue: "posting_policy")!)
        try container.encodeIfPresent(self.paymentsUrl, forKey: DynamicCodingKey(stringValue: "payments_url")!)
        if let fees = self.fees {
            try container.encode(fees, forKey: DynamicCodingKey(stringValue: "fees")!)
        } else {
            try container.encodeNil(forKey: DynamicCodingKey(stringValue: "fees")!)
        }
        for (key, value) in self.other {
            try container.encode(value, forKey: DynamicCodingKey(stringValue: key)!)
        }
    }
}

extension Filter {
    public init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: DynamicCodingKey.self)
        self.ids = try container.decodeIfPresent([IdHex].self, forKey: DynamicCodingKey(stringValue: "ids")!) ?? []
        self.authors = try container.decodeIfPresent([PublicKeyHex].self, forKey: DynamicCodingKey(stringValue: "authors")!) ?? []
        self.kinds = try container.decodeIfPresent([EventKind].self, forKey: DynamicCodingKey(stringValue: "kinds")!) ?? []
        self.since = try container.decodeIfPresent(Unixtime.self, forKey: DynamicCodingKey(stringValue: "since")!)
        self.until = try container.decodeIfPresent(Unixtime.self, forKey: DynamicCodingKey(stringValue: "until")!)
        self.limit = try container.decodeIfPresent(Int.self, forKey: DynamicCodingKey(stringValue: "limit")!)

        var tags: [String: [String]] = [:]
        for key in container.allKeys {
            let name = key.stringValue
            guard name.hasPrefix("#"), name.count == 2 else { continue }
            tags[name] = try container.decode([String].self, forKey: key)
        }
        self.tags = tags
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.container(keyedBy: DynamicCodingKey.self)
        if !self.ids.isEmpty {
            try container.encode(self.ids, forKey: DynamicCodingKey(stringValue: "ids")!)
        }
        if !self.authors.isEmpty {
            try container.encode(self.authors, forKey: DynamicCodingKey(stringValue: "authors")!)
        }
        if !self.kinds.isEmpty {
            try container.encode(self.kinds, forKey: DynamicCodingKey(stringValue: "kinds")!)
        }
        try container.encodeIfPresent(self.since, forKey: DynamicCodingKey(stringValue: "since")!)
        try container.encodeIfPresent(self.until, forKey: DynamicCodingKey(stringValue: "until")!)
        try container.encodeIfPresent(self.limit, forKey: DynamicCodingKey(stringValue: "limit")!)
        for (key, value) in self.tags {
            try container.encode(value, forKey: DynamicCodingKey(stringValue: key)!)
        }
    }
}

extension ClientMessage {
    public init(from decoder: any Decoder) throws {
        var container = try decoder.unkeyedContainer()
        let word = try container.decode(String.self)
        switch word {
        case "EVENT":
            self = .event(try container.decode(Event.self))
        case "REQ":
            let id = try container.decode(SubscriptionId.self)
            var filters: [Filter] = []
            while !container.isAtEnd {
                filters.append(try container.decode(Filter.self))
            }
            self = .req(id, filters)
        case "CLOSE":
            self = .close(try container.decode(SubscriptionId.self))
        case "AUTH":
            self = .auth(try container.decode(Event.self))
        default:
            throw DecodingError.dataCorrupted(.init(codingPath: container.codingPath, debugDescription: "Unknown message: \(word)"))
        }
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.unkeyedContainer()
        switch self {
        case .event(let event):
            try container.encode("EVENT")
            try container.encode(event)
        case .req(let id, let filters):
            try container.encode("REQ")
            try container.encode(id)
            for filter in filters {
                try container.encode(filter)
            }
        case .close(let id):
            try container.encode("CLOSE")
            try container.encode(id)
        case .auth(let event):
            try container.encode("AUTH")
            try container.encode(event)
        }
    }
}

extension RelayMessage {
    public init(from decoder: any Decoder) throws {
        var container = try decoder.unkeyedContainer()
        let word = try container.decode(String.self)
        switch word {
        case "AUTH":
            self = .auth(try container.decode(String.self))
        case "CLOSED":
            self = .closed(try container.decode(SubscriptionId.self), try container.decode(String.self))
        case "EOSE":
            self = .eose(try container.decode(SubscriptionId.self))
        case "EVENT":
            self = .event(try container.decode(SubscriptionId.self), try container.decode(Event.self))
        case "NOTICE":
            self = .notice(try container.decode(String.self))
        case "NOTIFY":
            self = .notify(try container.decode(String.self))
        case "OK":
            self = .ok(try container.decode(Id.self), try container.decode(Bool.self), try container.decode(String.self))
        default:
            throw DecodingError.dataCorrupted(.init(codingPath: container.codingPath, debugDescription: "Unknown relay message: \(word)"))
        }
    }

    public func encode(to encoder: any Encoder) throws {
        var container = encoder.unkeyedContainer()
        switch self {
        case .auth(let value):
            try container.encode("AUTH")
            try container.encode(value)
        case .closed(let id, let message):
            try container.encode("CLOSED")
            try container.encode(id)
            try container.encode(message)
        case .eose(let id):
            try container.encode("EOSE")
            try container.encode(id)
        case .event(let id, let event):
            try container.encode("EVENT")
            try container.encode(id)
            try container.encode(event)
        case .notice(let value):
            try container.encode("NOTICE")
            try container.encode(value)
        case .notify(let value):
            try container.encode("NOTIFY")
            try container.encode(value)
        case .ok(let id, let ok, let message):
            try container.encode("OK")
            try container.encode(id)
            try container.encode(ok)
            try container.encode(message)
        }
    }
}
