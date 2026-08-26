import Foundation
import SwiftUI
import GnostrTypes

public enum CrawlerQueryError: Error, Equatable, Sendable {
    case invalidGenericTag
    case invalidKind(String)
}

public struct RelayProcessState: Codable, Hashable, Sendable {
    public var running: Bool
    public var pid: UInt32?
    public var message: String
    public var diskUsageBytes: UInt64?

    public init(running: Bool, pid: UInt32? = nil, message: String, diskUsageBytes: UInt64? = nil) {
        self.running = running
        self.pid = pid
        self.message = message
        self.diskUsageBytes = diskUsageBytes
    }

    private enum CodingKeys: String, CodingKey {
        case running
        case pid
        case message
        case diskUsageBytes = "disk_usage_bytes"
    }
}

public struct RelayMetadata: Codable, Hashable, Sendable {
    public var contact: String?
    public var description: String?
    public var name: String?
    public var pingMs: UInt64?
    public var software: String?
    public var supportedNips: [Int]?
    public var supportedNipExtensions: [String]?
    public var version: String?

    public init(
        contact: String? = nil,
        description: String? = nil,
        name: String? = nil,
        pingMs: UInt64? = nil,
        software: String? = nil,
        supportedNips: [Int]? = nil,
        supportedNipExtensions: [String]? = nil,
        version: String? = nil
    ) {
        self.contact = contact
        self.description = description
        self.name = name
        self.pingMs = pingMs
        self.software = software
        self.supportedNips = supportedNips
        self.supportedNipExtensions = supportedNipExtensions
        self.version = version
    }

    private enum CodingKeys: String, CodingKey {
        case contact
        case description
        case name
        case pingMs = "ping_ms"
        case software
        case supportedNips = "supported_nips"
        case supportedNipExtensions = "supported_nip_extensions"
        case version
    }
}

public struct RelayDiscoveryEntry: Codable, Hashable, Sendable {
    public var url: String
    public var contact: String?
    public var description: String?
    public var name: String?
    public var pingMs: UInt64?
    public var software: String?
    public var version: String?
    public var supportedNips: [Int]
    public var supportedNipExtensions: [String]
    public var sourceNips: [Int]

    public init(
        url: String,
        contact: String? = nil,
        description: String? = nil,
        name: String? = nil,
        pingMs: UInt64? = nil,
        software: String? = nil,
        version: String? = nil,
        supportedNips: [Int] = [],
        supportedNipExtensions: [String] = [],
        sourceNips: [Int] = []
    ) {
        self.url = url
        self.contact = contact
        self.description = description
        self.name = name
        self.pingMs = pingMs
        self.software = software
        self.version = version
        self.supportedNips = supportedNips
        self.supportedNipExtensions = supportedNipExtensions
        self.sourceNips = sourceNips
    }

    private enum CodingKeys: String, CodingKey {
        case url
        case contact
        case description
        case name
        case pingMs = "ping_ms"
        case software
        case version
        case supportedNips = "supported_nips"
        case supportedNipExtensions = "supported_nip_extensions"
        case sourceNips = "source_nips"
    }
}

public struct CrawlerQueryParameters: Codable, Hashable, Sendable {
    public var relay: String?
    public var authors: String?
    public var ids: String?
    public var limit: Int?
    public var genericTag: String?
    public var genericValue: String?
    public var hashtag: String?
    public var mentions: String?
    public var references: String?
    public var kinds: String?
    public var search: String?

    public init(
        relay: String? = nil,
        authors: String? = nil,
        ids: String? = nil,
        limit: Int? = nil,
        genericTag: String? = nil,
        genericValue: String? = nil,
        hashtag: String? = nil,
        mentions: String? = nil,
        references: String? = nil,
        kinds: String? = nil,
        search: String? = nil
    ) {
        self.relay = relay
        self.authors = authors
        self.ids = ids
        self.limit = limit
        self.genericTag = genericTag
        self.genericValue = genericValue
        self.hashtag = hashtag
        self.mentions = mentions
        self.references = references
        self.kinds = kinds
        self.search = search
    }

    public func queryItems() -> [URLQueryItem] {
        var items: [URLQueryItem] = []
        if let relay { items.append(URLQueryItem(name: "relay", value: relay)) }
        if let authors { items.append(URLQueryItem(name: "authors", value: authors)) }
        if let ids { items.append(URLQueryItem(name: "ids", value: ids)) }
        if let limit { items.append(URLQueryItem(name: "limit", value: String(limit))) }
        if let genericTag { items.append(URLQueryItem(name: "generic_tag", value: genericTag)) }
        if let genericValue { items.append(URLQueryItem(name: "generic_value", value: genericValue)) }
        if let hashtag { items.append(URLQueryItem(name: "hashtag", value: hashtag)) }
        if let mentions { items.append(URLQueryItem(name: "mentions", value: mentions)) }
        if let references { items.append(URLQueryItem(name: "references", value: references)) }
        if let kinds { items.append(URLQueryItem(name: "kinds", value: kinds)) }
        if let search { items.append(URLQueryItem(name: "search", value: search)) }
        return items
    }

    public func queryURL(baseURL: URL, nip: Int? = nil) -> URL {
        var components = URLComponents(url: baseURL, resolvingAgainstBaseURL: false) ?? URLComponents()
        let path = nip.map { "/\($0)/query" } ?? "/query"
        components.path = normalizedPath(baseURL.path, path)
        components.queryItems = self.queryItems()
        guard let url = components.url else {
            preconditionFailure("failed to build crawler query URL")
        }
        return url
    }

    public func makeFilter() throws -> Filter {
        var filter = Filter()
        filter.limit = self.limit
        filter.authors = splitCSV(self.authors).map { PublicKey(hex: $0) }
        filter.ids = splitCSV(self.ids).map { Id(hex: $0) }
        filter.kinds = try splitCSV(self.kinds).map(parseEventKind)

        if let genericTag {
            guard let tag = genericTag.trimmingCharacters(in: .whitespacesAndNewlines).first else {
                throw CrawlerQueryError.invalidGenericTag
            }
            if let genericValue {
                filter.tags["#\(tag)"] = splitCSV(genericValue)
            }
        }

        if let hashtag {
            filter.tags["#t"] = splitCSV(hashtag)
        }
        if let mentions {
            filter.tags["#p"] = splitCSV(mentions)
        }
        if let references {
            filter.tags["#e"] = splitCSV(references)
        }

        return filter
    }

    public func buildWireQuery(subscriptionID: String = "gnostr-query") throws -> String {
        if let rustWire = try RustCrawlerBridge.shared.buildGnostrQuery(self) {
            return rustWire
        }
        let filter = try self.makeFilter()
        let message = ClientMessage.req(SubscriptionId(subscriptionID), [filter])
        let encoder = JSONEncoder()
        encoder.outputFormatting = [.sortedKeys]
        return String(data: try encoder.encode(message), encoding: .utf8) ?? "[]"
    }
}

public struct CrawlerBucketRefreshState: Codable, Hashable, Sendable {
    public var ok: Bool
    public var message: String

    public init(ok: Bool, message: String) {
        self.ok = ok
        self.message = message
    }
}

public enum CrawlerQueryBuilder {
    public static func buildGnostrQuery(
        authors: String? = nil,
        ids: String? = nil,
        limit: Int? = nil,
        generic: (tag: String, value: String)? = nil,
        hashtag: String? = nil,
        mentions: String? = nil,
        references: String? = nil,
        kinds: String? = nil,
        search: (element: String, value: String)? = nil
    ) throws -> String {
        let parameters = CrawlerQueryParameters(
            authors: authors,
            ids: ids,
            limit: limit,
            genericTag: generic?.tag,
            genericValue: generic?.value,
            hashtag: hashtag,
            mentions: mentions,
            references: references,
            kinds: kinds,
            search: search.map { "\($0.element)=\($0.value)" }
        )
        return try parameters.buildWireQuery()
    }
}

public final class CrawlerLogStore: ObservableObject {
    @Published public private(set) var lines: [String]

    private let maxLines: Int
    private let sink = CrawlerLogSink()

    public init(maxLines: Int = 500, bindImmediately: Bool = true) {
        self.lines = []
        self.maxLines = maxLines
        if bindImmediately {
            bind()
        }
    }

    public func bind() {
        self.sink.store = self
        RustCrawlerBridge.shared.onLogLine = { [sink] line in
            sink.handle(line)
        }
    }

    public func clear() {
        self.lines.removeAll()
    }

    fileprivate func append(_ line: String) {
        self.lines.insert(line, at: 0)
        if self.lines.count > self.maxLines {
            self.lines.removeLast(self.lines.count - self.maxLines)
        }
    }
}

private final class CrawlerLogSink: @unchecked Sendable {
    weak var store: CrawlerLogStore?

    func handle(_ line: String) {
        DispatchQueue.main.async { [weak store] in
            store?.append(line)
        }
    }
}

public final class CrawlerClient: @unchecked Sendable {
    public let baseURL: URL
    private let session: URLSession
    private let decoder: JSONDecoder

    public init(baseURL: URL = URL(string: "http://127.0.0.1:3030")!, session: URLSession = .shared) {
        self.baseURL = baseURL
        self.session = session
        self.decoder = JSONDecoder()
    }

    public func relayDiscovery() async throws -> [RelayDiscoveryEntry] {
        try await self.getJSON("api/relay/discovery")
    }

    public func relayStatus() async throws -> RelayProcessState {
        try await self.getJSON("api/relay/status")
    }

    public func startRelay() async throws -> RelayProcessState {
        try await self.postJSON("api/relay/start")
    }

    public func stopRelay() async throws -> RelayProcessState {
        try await self.postJSON("api/relay/stop")
    }

    public func relaysJSON(nip: Int? = nil) async throws -> [String] {
        try await self.getJSON(path(for: nip, suffix: "relays.json"))
    }

    public func relaysYAML(nip: Int? = nil) async throws -> String {
        try await self.getText(path(for: nip, suffix: "relays.yaml"))
    }

    public func relaysTXT(nip: Int? = nil) async throws -> String {
        try await self.getText(path(for: nip, suffix: "relays.txt"))
    }

    public func indexHTML() async throws -> String {
        try await self.getText("")
    }

    public func primeBuckets() async throws -> CrawlerBucketRefreshState {
        try await self.postJSON("api/relays/prime")
    }

    public func queryPage(_ parameters: CrawlerQueryParameters = .init(), nip: Int? = nil) async throws -> String {
        try await self.getText(parameters.queryURL(baseURL: self.baseURL, nip: nip))
    }

    public func buildQueryWire(
        authors: String? = nil,
        ids: String? = nil,
        limit: Int? = nil,
        generic: (tag: String, value: String)? = nil,
        hashtag: String? = nil,
        mentions: String? = nil,
        references: String? = nil,
        kinds: String? = nil,
        search: (element: String, value: String)? = nil
    ) throws -> String {
        try CrawlerQueryBuilder.buildGnostrQuery(
            authors: authors,
            ids: ids,
            limit: limit,
            generic: generic,
            hashtag: hashtag,
            mentions: mentions,
            references: references,
            kinds: kinds,
            search: search
        )
    }

    private func getJSON<T: Decodable>(_ path: String) async throws -> T {
        let data = try await self.request(path: path, method: "GET")
        return try self.decoder.decode(T.self, from: data)
    }

    private func getJSON<T: Decodable>(path: URL) async throws -> T {
        let (data, response) = try await self.session.data(from: path)
        try Self.validate(response: response, url: path)
        return try self.decoder.decode(T.self, from: data)
    }

    private func postJSON<T: Decodable>(_ path: String) async throws -> T {
        let data = try await self.request(path: path, method: "POST")
        return try self.decoder.decode(T.self, from: data)
    }

    private func getText(_ path: String) async throws -> String {
        let data = try await self.request(path: path, method: "GET")
        guard let string = String(data: data, encoding: .utf8) else {
            throw CocoaError(.fileReadInapplicableStringEncoding)
        }
        return string
    }

    private func getText(_ url: URL) async throws -> String {
        let (data, response) = try await self.session.data(from: url)
        try Self.validate(response: response, url: url)
        guard let string = String(data: data, encoding: .utf8) else {
            throw CocoaError(.fileReadInapplicableStringEncoding)
        }
        return string
    }

    private func request(path: String, method: String) async throws -> Data {
        let url = self.url(path: path)
        var request = URLRequest(url: url)
        request.httpMethod = method
        let (data, response) = try await self.session.data(for: request)
        try Self.validate(response: response, url: url)
        return data
    }

    private func url(path: String) -> URL {
        self.url(path: path, queryItems: nil)
    }

    private func url(path: String, queryItems: [URLQueryItem]?) -> URL {
        var components = URLComponents(url: self.baseURL, resolvingAgainstBaseURL: false) ?? URLComponents()
        components.path = normalizedPath(self.baseURL.path, path)
        components.queryItems = queryItems
        guard let url = components.url else {
            preconditionFailure("failed to build crawler URL for path \(path)")
        }
        return url
    }

    private func path(for nip: Int?, suffix: String) -> String {
        if let nip {
            return "\(nip)/\(suffix)"
        }
        return suffix
    }

    private static func validate(response: URLResponse, url: URL) throws {
        guard let http = response as? HTTPURLResponse else {
            return
        }
        guard (200..<300).contains(http.statusCode) else {
            throw NSError(
                domain: "CrawlerClient",
                code: http.statusCode,
                userInfo: [NSLocalizedDescriptionKey: "crawler request failed for \(url.absoluteString) with status \(http.statusCode)"]
            )
        }
    }
}

private func normalizedPath(_ basePath: String, _ relativePath: String) -> String {
    let prefix = basePath.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    let suffix = relativePath.trimmingCharacters(in: CharacterSet(charactersIn: "/"))
    let combined = [prefix, suffix].filter { !$0.isEmpty }.joined(separator: "/")
    return "/" + combined
}

private func splitCSV(_ value: String?) -> [String] {
    guard let value else { return [] }
    return value
        .split(separator: ",")
        .map { $0.trimmingCharacters(in: .whitespacesAndNewlines) }
        .filter { !$0.isEmpty }
}

private func parseEventKind(_ input: String) throws -> EventKind {
    let trimmed = input.trimmingCharacters(in: .whitespacesAndNewlines)
    let lowered = trimmed.lowercased()
    let prefixes = [
        "nip:", "nip=", "nip/", "nip ",
        "nips:", "nips=", "nips/", "nips ",
        "kind:", "kind=", "kind/", "kind ",
        "kinds:", "kinds=", "kinds/", "kinds ",
    ]
    let normalized: String
    if let prefix = prefixes.first(where: { lowered.hasPrefix($0) }) {
        normalized = String(trimmed.dropFirst(prefix.count)).trimmingCharacters(in: .whitespacesAndNewlines)
    } else {
        normalized = trimmed
    }

    guard let value = UInt32(normalized) else {
        throw CrawlerQueryError.invalidKind(input)
    }
    guard let kind = EventKind(rawValue: value) else {
        throw CrawlerQueryError.invalidKind(input)
    }
    return kind
}
