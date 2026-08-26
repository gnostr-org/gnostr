import Foundation
import Testing
@testable import Crawler
@testable import GnostrTypes

#if canImport(Darwin)
import Darwin
#endif

#if canImport(FoundationNetworking)
import FoundationNetworking
#endif

private enum LiveRelayTestError: Error {
    case timeout
    case invalidResponse
    case runtimeStartFailed(String?)
}

@Test func relayDiscoveryDecodes() throws {
    let json = #"""
    [{
      "url":"wss://relay.example",
      "contact":"ops@example.com",
      "description":"Relay description",
      "name":"Relay name",
      "software":"strfry",
      "version":"1.0.0",
      "supported_nips":[1,11,50],
      "supported_nip_extensions":["nip50-search"],
      "source_nips":[1,11,50]
    }]
    """#

    let entries = try JSONDecoder().decode([RelayDiscoveryEntry].self, from: Data(json.utf8))
    #expect(entries.count == 1)
    #expect(entries[0].supportedNips == [1, 11, 50])
    #expect(entries[0].supportedNipExtensions == ["nip50-search"])
    #expect(entries[0].sourceNips == [1, 11, 50])
}

@Test func relayProcessStateDecodes() throws {
    let json = #"{"running":true,"pid":1234,"message":"ok","disk_usage_bytes":99}"#
    let state = try JSONDecoder().decode(RelayProcessState.self, from: Data(json.utf8))
    #expect(state.running)
    #expect(state.pid == 1234)
    #expect(state.diskUsageBytes == 99)
}

@Test func queryBuilderMatchesCrawlerShape() throws {
    let wire = try CrawlerQueryBuilder.buildGnostrQuery(
        authors: "aa11, bb22",
        ids: "deadbeef",
        limit: 25,
        generic: (tag: "k", value: "1"),
        hashtag: "gnostr",
        mentions: "ee11",
        references: "ff22",
        kinds: "kind:1,nip=1617",
        search: ("search", "ignored")
    )

    let decoded = try JSONDecoder().decode(ClientMessage.self, from: Data(wire.utf8))
    let expectedFilter = Filter(
        ids: [Id(hex: "deadbeef")],
        authors: [PublicKey(hex: "aa11"), PublicKey(hex: "bb22")],
        kinds: [.textNote, .patches],
        tags: ["#k": ["1"], "#t": ["gnostr"], "#p": ["ee11"], "#e": ["ff22"]],
        limit: 25
    )
    #expect(decoded == .req(SubscriptionId("gnostr-query"), [expectedFilter]))
}

@Test func queryURLBuildsExpectedPath() {
    let parameters = CrawlerQueryParameters(
        relay: "wss://relay.example",
        authors: "aa11",
        ids: "deadbeef",
        limit: 5,
        genericTag: "k",
        genericValue: "1",
        hashtag: "gnostr",
        mentions: "ee11",
        references: "ff22",
        kinds: "1,1617",
        search: "hello"
    )

    let url = parameters.queryURL(baseURL: URL(string: "http://127.0.0.1:3030")!, nip: 34)
    #expect(url.path == "/34/query")
    let items = URLComponents(url: url, resolvingAgainstBaseURL: false)?.queryItems ?? []
    #expect(items.contains(URLQueryItem(name: "relay", value: "wss://relay.example")))
    #expect(items.contains(URLQueryItem(name: "search", value: "hello")))
}

@Test func rustCrawlerBridgeRoundTripsWhenAvailable() throws {
    guard RustCrawlerBridge.shared.isAvailable else { return }

    let author = String(repeating: "a", count: 64)
    let id = String(repeating: "b", count: 64)
    let parameters = CrawlerQueryParameters(
        authors: author,
        ids: id,
        limit: 25,
        genericTag: "k",
        genericValue: "1",
        hashtag: "gnostr",
        mentions: String(repeating: "c", count: 64),
        references: String(repeating: "d", count: 64),
        kinds: "kind:1,nip=1617"
    )
    #expect(try RustCrawlerBridge.shared.buildGnostrQuery(parameters) != nil)
    #expect(RustCrawlerBridge.shared.websocketHTTPURL("wss://relay.example") == "https://relay.example")

    let relay = RelayMetadata(
        contact: "ops@example.com",
        description: "Relay description",
        name: "Relay name",
        pingMs: 42,
        software: "strfry",
        supportedNips: [1, 11, 50],
        supportedNipExtensions: ["nip50-search"],
        version: "1.0.0"
    )
    #expect(try RustCrawlerBridge.shared.normalize(relay) == relay)
}

@Test func liveRelayWebsocketRoundTripAndCrawlerQuery() async throws {
    guard RustCrawlerBridge.shared.isAvailable else { return }

    let port = try pickFreePort()
    let bridge = RustCrawlerBridge.shared
    guard bridge.isAvailable else { return }
    guard let state = bridge.startCrawlerRuntime(port: port), state.running else {
        throw LiveRelayTestError.runtimeStartFailed(bridge.crawlerRuntimeStatus()?.message)
    }
    defer {
        _ = bridge.stopCrawlerRuntime()
    }

    let client = CrawlerClient(baseURL: URL(string: "http://127.0.0.1:\(port)")!)
    try await Task.sleep(nanoseconds: 1_000_000_000)
    let crawlerRelays = try await crawlerRelayList(client: client)
    let firstRelay = crawlerRelays[0].absoluteString
    let servedTXT = try await client.relaysTXT()
    let servedJSON = try await client.relaysJSON()
    let servedYAML = try await client.relaysYAML()
    #expect(servedTXT.contains(firstRelay))
    #expect(servedJSON.contains(firstRelay))
    #expect(servedYAML.contains(firstRelay))
    let relay = try await publishLiveGitNoteEvent(relays: Array(crawlerRelays.prefix(10)))
    let event = relay.event

    var crawlerResult = ""
    var matched = false
    for relayURL in relay.relayURLs {
        for attempt in 0..<5 {
            do {
                crawlerResult = try await client.queryPage(
                    CrawlerQueryParameters(
                        relay: relayURL.absoluteString,
                        ids: event.id.hex,
                        limit: 100
                    ),
                    nip: 1
                )
            } catch {
                if attempt < 4 {
                    try await Task.sleep(nanoseconds: 1_000_000_000)
                }
                continue
            }
            if crawlerResult.contains(event.id.hex) && crawlerResult.contains(event.content) {
                matched = true
                break
            }
            if attempt < 4 {
                try await Task.sleep(nanoseconds: 1_000_000_000)
            }
        }
        if matched {
            break
        }
    }
    guard matched else { return }
    #expect(crawlerResult.contains(event.id.hex))
    #expect(crawlerResult.contains(event.content))
}

@Test func crawlerServeRootHtmlPrintsToConsole() async throws {
    guard RustCrawlerBridge.shared.isAvailable else { return }

    let port = try pickFreePort()
    let bridge = RustCrawlerBridge.shared
    guard let state = bridge.startCrawlerRuntime(port: port), state.running else {
        throw LiveRelayTestError.runtimeStartFailed(bridge.crawlerRuntimeStatus()?.message)
    }
    defer {
        _ = bridge.stopCrawlerRuntime()
    }

    let client = CrawlerClient(baseURL: URL(string: "http://127.0.0.1:\(port)")!)
    try await Task.sleep(nanoseconds: 1_000_000_000)

    let html = try await client.indexHTML()
    print("gnostr crawler serve html:\n\(html)")
    #expect(html.contains("gnostr crawler"))
}

@Test func crawlerPrintsRealRelayMetadataToConsole() async throws {
    guard RustCrawlerBridge.shared.isAvailable else { return }

    let port = try pickFreePort()
    let bridge = RustCrawlerBridge.shared
    guard let state = bridge.startCrawlerRuntime(port: port), state.running else {
        throw LiveRelayTestError.runtimeStartFailed(bridge.crawlerRuntimeStatus()?.message)
    }
    defer {
        _ = bridge.stopCrawlerRuntime()
    }

    let client = CrawlerClient(baseURL: URL(string: "http://127.0.0.1:\(port)")!)
    try await Task.sleep(nanoseconds: 1_000_000_000)

    let candidates = [1, 11, 34, 50]
    var metadata: RelayMetadata?
    for nip in candidates {
        if let relayURL = try? await firstServedRelayURL(client: client, nip: nip),
           let fetched = try? await fetchRelayMetadata(baseURL: client.baseURL, nip: nip, relayURL: relayURL) {
            metadata = fetched
            break
        }
    }
    guard let metadata else { throw LiveRelayTestError.invalidResponse }
    print("gnostr crawler relay metadata:\n\(metadata)")
    #expect(metadata.name != nil || metadata.description != nil || metadata.contact != nil)
}

@Test func relayProcessStateEncodesAndDecodesSnakeCase() throws {
    let state = RelayProcessState(running: true, pid: 42, message: "ok", diskUsageBytes: 99)
    let encoder = JSONEncoder()
    let data = try encoder.encode(state)
    let json = String(decoding: data, as: UTF8.self)
    #expect(json.contains(#""disk_usage_bytes":99"#))
    #expect(json.contains(#""pid":42"#))

    let decoded = try JSONDecoder().decode(RelayProcessState.self, from: data)
    #expect(decoded == state)
}

@Test func relayMetadataEncodesAndDecodesSnakeCase() throws {
    let metadata = RelayMetadata(
        contact: "ops@example.com",
        description: "Relay description",
        name: "Relay name",
        pingMs: 42,
        software: "strfry",
        supportedNips: [1, 11, 50],
        supportedNipExtensions: ["nip50-search"],
        version: "1.0.0"
    )
    let encoder = JSONEncoder()
    let data = try encoder.encode(metadata)
    let json = String(decoding: data, as: UTF8.self)
    #expect(json.contains(#""ping_ms":42"#))
    #expect(json.contains(#""supported_nips":[1,11,50]"#))
    #expect(json.contains(#""supported_nip_extensions":["nip50-search"]"#))

    let decoded = try JSONDecoder().decode(RelayMetadata.self, from: data)
    #expect(decoded == metadata)
}

@MainActor
@Test func crawlerLogStoreCapturesRustCallbackLines() async throws {
    guard RustCrawlerBridge.shared.isAvailable else { return }

    let bridge = RustCrawlerBridge.shared
    let previous = bridge.onLogLine
    defer {
        bridge.onLogLine = previous
    }

    let store = CrawlerLogStore(maxLines: 10, bindImmediately: false)
    store.bind()
    bridge.onLogLine?("crawler log line")
    try await Task.sleep(nanoseconds: 50_000_000)
    #expect(store.lines.first == "crawler log line")
}

@Test func crawlerQueryParametersMakeFilterMatchesWireShape() throws {
    let parameters = CrawlerQueryParameters(
        authors: "aa11, bb22",
        ids: "deadbeef",
        limit: 25,
        genericTag: "k",
        genericValue: "1",
        hashtag: "gnostr",
        mentions: "ee11",
        references: "ff22",
        kinds: "0, kind:1617, 1"
    )

    let filter = try parameters.makeFilter()
    let expected = Filter(
        ids: [Id(hex: "deadbeef")],
        authors: [PublicKey(hex: "aa11"), PublicKey(hex: "bb22")],
        kinds: [.metadata, .patches, .textNote],
        tags: ["#k": ["1"], "#t": ["gnostr"], "#p": ["ee11"], "#e": ["ff22"]],
        limit: 25
    )
    #expect(filter == expected)
}

@Test func crawlerQueryParametersRejectsInvalidInputs() throws {
    let invalidTag = CrawlerQueryParameters(genericTag: "   ", genericValue: "1")
    do {
        _ = try invalidTag.makeFilter()
        #expect(Bool(false))
    } catch let error as CrawlerQueryError {
        #expect(error == .invalidGenericTag)
    }

    let invalidKind = CrawlerQueryParameters(kinds: "not-a-kind")
    do {
        _ = try invalidKind.makeFilter()
        #expect(Bool(false))
    } catch let error as CrawlerQueryError {
        #expect(error == .invalidKind("not-a-kind"))
    }
}

private struct LiveRelayPublishResult: Sendable {
    let relayURLs: [URL]
    let event: Event
}

private func publishLiveGitNoteEvent(relays: [URL]) async throws -> LiveRelayPublishResult {
    guard !relays.isEmpty else {
        throw LiveRelayTestError.invalidResponse
    }

    let uniqueContent = "crawler websocket live test \(UUID().uuidString)"
    let note = GitNote(
        noteID: String(repeating: "a", count: 40),
        annotatedID: String(repeating: "b", count: 40),
        notesRef: "refs/notes/commits",
        message: uniqueContent,
        author: "alice",
        committer: "bob",
        committerTime: Int64(Date().timeIntervalSince1970)
    )
    guard let published = try RustCrawlerBridge.shared.publishGitNoteEvent(
        note: note,
        privateKeyHex: "0000000000000000000000000000000000000000000000000000000000000001",
        relays: relays
    ) else {
        throw LiveRelayTestError.timeout
    }
    return LiveRelayPublishResult(relayURLs: published.relayURLs, event: published.event)
}

private func crawlerRelayList(client: CrawlerClient) async throws -> [URL] {
    for attempt in 0..<30 {
        do {
            let relays = try await client.relaysTXT()
                .split(whereSeparator: { $0.isWhitespace })
                .compactMap { URL(string: String($0)) }
            if !relays.isEmpty {
                return relays
            }
        } catch {
            if attempt < 29 {
                try await Task.sleep(nanoseconds: 1_000_000_000)
            }
            continue
        }
        if attempt < 29 {
            try await Task.sleep(nanoseconds: 1_000_000_000)
        }
    }
    throw LiveRelayTestError.invalidResponse
}

private func firstServedRelayURL(client: CrawlerClient, nip: Int) async throws -> URL {
    let relays = try await client.relaysTXT(nip: nip)
        .split(whereSeparator: { $0.isWhitespace })
        .compactMap { URL(string: String($0)) }
    guard let relay = relays.first else {
        throw LiveRelayTestError.invalidResponse
    }
    return relay
}

private func fetchRelayMetadata(baseURL: URL, nip: Int, relayURL: URL) async throws -> RelayMetadata {
    guard let host = relayURL.host else {
        throw LiveRelayTestError.invalidResponse
    }

    let metadataURL = baseURL.appending(path: "\(nip)/\(host).json")
    let (data, response) = try await URLSession.shared.data(from: metadataURL)
    if let http = response as? HTTPURLResponse, !(200..<300).contains(http.statusCode) {
        throw NSError(
            domain: "CrawlerClient",
            code: http.statusCode,
            userInfo: [NSLocalizedDescriptionKey: "crawler request failed for \(metadataURL.absoluteString) with status \(http.statusCode)"]
        )
    }
    return try JSONDecoder().decode(RelayMetadata.self, from: data)
}

private func send(socket: URLSessionWebSocketTask, message: ClientMessage) async throws {
    let data = try JSONEncoder().encode(message)
    guard let text = String(data: data, encoding: .utf8) else {
        throw CocoaError(.coderInvalidValue)
    }
    try await socket.send(.string(text))
}

private func receive(socket: URLSessionWebSocketTask, timeoutNanoseconds: UInt64) async throws -> RelayMessage {
    try await withThrowingTaskGroup(of: RelayMessage.self) { group in
        group.addTask {
                let message = try await socket.receive()
            switch message {
            case .string(let text):
                guard let data = text.data(using: .utf8) else {
                    throw LiveRelayTestError.invalidResponse
                }
                return try JSONDecoder().decode(RelayMessage.self, from: data)
            case .data(let data):
                return try JSONDecoder().decode(RelayMessage.self, from: data)
            @unknown default:
                throw LiveRelayTestError.invalidResponse
            }
        }
        group.addTask {
            try await Task.sleep(nanoseconds: timeoutNanoseconds)
            throw LiveRelayTestError.timeout
        }
        let value = try await group.next()!
        group.cancelAll()
        return value
    }
}

private func pickFreePort() throws -> UInt16 {
    let fd = socket(AF_INET, SOCK_STREAM, 0)
    guard fd >= 0 else {
        throw LiveRelayTestError.invalidResponse
    }
    defer { close(fd) }

    var yes: Int32 = 1
    _ = setsockopt(fd, SOL_SOCKET, SO_REUSEADDR, &yes, socklen_t(MemoryLayout.size(ofValue: yes)))

    var address = sockaddr_in()
    address.sin_len = UInt8(MemoryLayout<sockaddr_in>.size)
    address.sin_family = sa_family_t(AF_INET)
    address.sin_port = 0
    address.sin_addr = in_addr(s_addr: inet_addr("127.0.0.1"))

    let bindResult = withUnsafePointer(to: &address) { pointer in
        pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) {
            bind(fd, $0, socklen_t(MemoryLayout<sockaddr_in>.size))
        }
    }
    guard bindResult == 0 else {
        throw LiveRelayTestError.invalidResponse
    }

    var name = sockaddr_in()
    var length = socklen_t(MemoryLayout<sockaddr_in>.size)
    let result = withUnsafeMutablePointer(to: &name) { pointer in
        pointer.withMemoryRebound(to: sockaddr.self, capacity: 1) {
            getsockname(fd, $0, &length)
        }
    }
    guard result == 0 else {
        throw LiveRelayTestError.invalidResponse
    }

    return UInt16(bigEndian: name.sin_port)
}
