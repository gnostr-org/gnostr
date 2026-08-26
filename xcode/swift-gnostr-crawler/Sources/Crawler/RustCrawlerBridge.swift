import Foundation
#if canImport(Darwin)
import Darwin
#endif
import GnostrTypes

private struct RustEnvelope<T: Decodable>: Decodable {
    let ok: Bool
    let data: T?
    let error: String?
}

private typealias RustStringFn = @convention(c) (UnsafePointer<CChar>) -> UnsafeMutablePointer<CChar>?
private typealias RustFreeFn = @convention(c) (UnsafeMutablePointer<CChar>) -> Void
private typealias RustLogFn = @convention(c) (UnsafePointer<CChar>?) -> Void
private typealias RustSetLogCallbackFn = @convention(c) (RustLogFn?) -> Void

private struct RuntimeRequest: Codable {
    let port: UInt16?
}

private struct CrawlRequest: Codable {}

public final class RustCrawlerBridge: @unchecked Sendable {
    public static let shared = RustCrawlerBridge()

    private let handle: UnsafeMutableRawPointer?
    private let buildQueryFn: RustStringFn?
    private let websocketHttpURLFn: RustStringFn?
    private let roundtripRelayMetadataFn: RustStringFn?
    private let generateGitNoteEventFn: RustStringFn?
    private let generateTextNoteEventFn: RustStringFn?
    private let publishTextNoteEventFn: RustStringFn?
    private let publishGitNoteEventFn: RustStringFn?
    private let runtimeStartFn: RustStringFn?
    private let runtimeStopFn: RustStringFn?
    private let runtimeStatusFn: RustStringFn?
    private let crawlStartFn: RustStringFn?
    private let crawlStopFn: RustStringFn?
    private let crawlStatusFn: RustStringFn?
    private let setLogCallbackFn: RustSetLogCallbackFn?
    private let freeFn: RustFreeFn?
    public var onLogLine: ((String) -> Void)?

    private init() {
        self.handle = Self.openLibrary()
        if let handle {
            NSLog("Crawler FFI: loaded library")
            self.buildQueryFn = Self.loadSymbol("crawler_build_gnostr_query_json", from: handle)
            self.websocketHttpURLFn = Self.loadSymbol("crawler_websocket_http_url_json", from: handle)
            self.roundtripRelayMetadataFn = Self.loadSymbol("crawler_roundtrip_relay_metadata_json", from: handle)
            self.generateGitNoteEventFn = Self.loadSymbol("crawler_generate_git_note_event_json", from: handle)
            self.generateTextNoteEventFn = Self.loadSymbol("crawler_generate_text_note_event_json", from: handle)
            self.publishTextNoteEventFn = Self.loadSymbol("crawler_publish_text_note_json", from: handle)
            self.publishGitNoteEventFn = Self.loadSymbol("crawler_publish_git_note_json", from: handle)
            self.runtimeStartFn = Self.loadSymbol("crawler_runtime_start_json", from: handle)
            self.runtimeStopFn = Self.loadSymbol("crawler_runtime_stop_json", from: handle)
            self.runtimeStatusFn = Self.loadSymbol("crawler_runtime_status_json", from: handle)
            self.crawlStartFn = Self.loadSymbol("crawler_crawl_start_json", from: handle)
            self.crawlStopFn = Self.loadSymbol("crawler_crawl_stop_json", from: handle)
            self.crawlStatusFn = Self.loadSymbol("crawler_crawl_status_json", from: handle)
            self.setLogCallbackFn = Self.loadSymbol("crawler_set_log_callback", from: handle)
            self.freeFn = Self.loadSymbol("crawler_string_free", from: handle)
            let missing = [
                self.buildQueryFn == nil ? "crawler_build_gnostr_query_json" : nil,
                self.websocketHttpURLFn == nil ? "crawler_websocket_http_url_json" : nil,
                self.roundtripRelayMetadataFn == nil ? "crawler_roundtrip_relay_metadata_json" : nil,
                self.generateGitNoteEventFn == nil ? "crawler_generate_git_note_event_json" : nil,
                self.generateTextNoteEventFn == nil ? "crawler_generate_text_note_event_json" : nil,
                self.publishTextNoteEventFn == nil ? "crawler_publish_text_note_json" : nil,
                self.publishGitNoteEventFn == nil ? "crawler_publish_git_note_json" : nil,
                self.runtimeStartFn == nil ? "crawler_runtime_start_json" : nil,
                self.runtimeStopFn == nil ? "crawler_runtime_stop_json" : nil,
                self.runtimeStatusFn == nil ? "crawler_runtime_status_json" : nil,
                self.crawlStartFn == nil ? "crawler_crawl_start_json" : nil,
                self.crawlStopFn == nil ? "crawler_crawl_stop_json" : nil,
                self.crawlStatusFn == nil ? "crawler_crawl_status_json" : nil,
                self.setLogCallbackFn == nil ? "crawler_set_log_callback (optional)" : nil,
                self.freeFn == nil ? "crawler_string_free" : nil,
            ].compactMap { $0 }
            NSLog("Crawler FFI: missing symbols=%@", missing.isEmpty ? "none" : missing.joined(separator: ", "))
            self.registerLogCallback()
        } else {
            NSLog("Crawler FFI: library not found")
            self.buildQueryFn = nil
            self.websocketHttpURLFn = nil
            self.roundtripRelayMetadataFn = nil
            self.generateGitNoteEventFn = nil
            self.generateTextNoteEventFn = nil
            self.publishTextNoteEventFn = nil
            self.publishGitNoteEventFn = nil
            self.runtimeStartFn = nil
            self.runtimeStopFn = nil
            self.runtimeStatusFn = nil
            self.crawlStartFn = nil
            self.crawlStopFn = nil
            self.crawlStatusFn = nil
            self.setLogCallbackFn = nil
            self.freeFn = nil
        }
    }

    public var isAvailable: Bool {
        NSLog("Crawler FFI: isAvailable=%@", self.handle != nil ? "yes" : "no")
        return self.handle != nil
            && self.buildQueryFn != nil
            && self.websocketHttpURLFn != nil
            && self.roundtripRelayMetadataFn != nil
            && self.generateGitNoteEventFn != nil
            && self.generateTextNoteEventFn != nil
            && self.publishTextNoteEventFn != nil
            && self.publishGitNoteEventFn != nil
            && self.runtimeStartFn != nil
            && self.runtimeStopFn != nil
            && self.runtimeStatusFn != nil
            && self.crawlStartFn != nil
            && self.crawlStopFn != nil
            && self.crawlStatusFn != nil
            && self.freeFn != nil
    }

    public func registerLogCallback() {
        guard let setLogCallbackFn else {
            NSLog("Crawler FFI: registerLogCallback skipped (missing symbol)")
            return
        }
        NSLog("Crawler FFI: registering Rust log callback")
        setLogCallbackFn(Self.rustLogCallback)
    }

    public func buildGnostrQuery(_ parameters: CrawlerQueryParameters) throws -> String? {
        NSLog("Crawler FFI: buildGnostrQuery")
        guard let buildQueryFn else { return nil }
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        guard let data = try? encoder.encode(parameters), let json = String(data: data, encoding: .utf8) else {
            throw CocoaError(.coderInvalidValue)
        }
        return self.callString(buildQueryFn, input: json)
    }

    public func websocketHTTPURL(_ url: String) -> String? {
        NSLog("Crawler FFI: websocketHTTPURL %@", url)
        return self.callString(self.websocketHttpURLFn, input: url)
    }

    public func normalize(_ relay: RelayMetadata) throws -> RelayMetadata {
        NSLog("Crawler FFI: normalize relay %@", String(describing: relay))
        guard self.isAvailable, let normalized: RelayMetadata = self.callRoundTrip(self.roundtripRelayMetadataFn, request: relay) else {
            return relay
        }
        return normalized
    }

    public func generateGitNoteEvent(note: GitNote, privateKeyHex: String, powTargetBits: UInt8 = 0) throws -> Event? {
        NSLog("Crawler FFI: generateGitNoteEvent")
        let request = GitNoteEventRequest(note: note, privateKeyHex: privateKeyHex, powTargetBits: powTargetBits)
        return try self.callRoundTrip(self.generateGitNoteEventFn, request: request)
    }

    public func generateTextNoteEvent(content: String, privateKeyHex: String, powTargetBits: UInt8 = 0) throws -> Event? {
        NSLog("Crawler FFI: generateTextNoteEvent")
        let request = TextNoteEventRequest(content: content, privateKeyHex: privateKeyHex, powTargetBits: powTargetBits)
        return try self.callRoundTrip(self.generateTextNoteEventFn, request: request)
    }

    func publishTextNoteEvent(
        content: String,
        privateKeyHex: String,
        relays: [URL],
        powTargetBits: UInt8 = 0
    ) throws -> PublishedTextNoteResult? {
        NSLog("Crawler FFI: publishTextNoteEvent")
        let request = PublishTextNoteRequest(
            relays: relays.map(\.absoluteString),
            content: content,
            privateKeyHex: privateKeyHex,
            powTargetBits: powTargetBits
        )
        return try self.callRoundTrip(self.publishTextNoteEventFn, request: request)
    }

    func publishGitNoteEvent(
        note: GitNote,
        privateKeyHex: String,
        relays: [URL],
        powTargetBits: UInt8 = 0
    ) throws -> PublishedGitNoteResult? {
        NSLog("Crawler FFI: publishGitNoteEvent")
        let publishRequest = PublishGitNoteRequest(
            relays: relays.map(\.absoluteString),
            note: note,
            privateKeyHex: privateKeyHex,
            powTargetBits: powTargetBits
        )
        return try self.callRoundTrip(self.publishGitNoteEventFn, request: publishRequest)
    }

    public func startCrawlerRuntime(port: UInt16 = 3030) -> RelayProcessState? {
        NSLog("Crawler FFI: start requested on port %d", port)
        let state: RelayProcessState? = self.callRoundTrip(self.runtimeStartFn, request: RuntimeRequest(port: port))
        NSLog("Crawler FFI: start result=%@", String(describing: state))
        return state
    }

    public func stopCrawlerRuntime() -> RelayProcessState? {
        NSLog("Crawler FFI: stop requested")
        let state: RelayProcessState? = self.callRoundTrip(self.runtimeStopFn, request: RuntimeRequest(port: nil))
        NSLog("Crawler FFI: stop result=%@", String(describing: state))
        return state
    }

    public func crawlerRuntimeStatus() -> RelayProcessState? {
        NSLog("Crawler FFI: status requested")
        let state: RelayProcessState? = self.callRoundTrip(self.runtimeStatusFn, request: RuntimeRequest(port: nil))
        NSLog("Crawler FFI: status result=%@", String(describing: state))
        return state
    }

    public func startCrawlerCrawl() -> RelayProcessState? {
        NSLog("Crawler FFI: crawl start requested")
        let state: RelayProcessState? = self.callRoundTrip(self.crawlStartFn, request: CrawlRequest())
        NSLog("Crawler FFI: crawl start result=%@", String(describing: state))
        return state
    }

    public func stopCrawlerCrawl() -> RelayProcessState? {
        NSLog("Crawler FFI: crawl stop requested")
        let state: RelayProcessState? = self.callRoundTrip(self.crawlStopFn, request: CrawlRequest())
        NSLog("Crawler FFI: crawl stop result=%@", String(describing: state))
        return state
    }

    public func crawlerCrawlStatus() -> RelayProcessState? {
        NSLog("Crawler FFI: crawl status requested")
        let state: RelayProcessState? = self.callRoundTrip(self.crawlStatusFn, request: CrawlRequest())
        NSLog("Crawler FFI: crawl status result=%@", String(describing: state))
        return state
    }

    private func callString(_ fn: RustStringFn?, input: String) -> String? {
        NSLog("Crawler FFI: callString input=%@", input)
        guard let fn else {
            NSLog("Crawler FFI: callString skipped (missing fn)")
            return nil
        }
        let inputCString = input.cString(using: .utf8)!
        return inputCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base) else { return nil }
            defer { self.freeFn?(rawResult) }
            let decoded = Self.decodeEnvelopeString(rawResult)
            NSLog("Crawler FFI: callString decoded=%@", decoded ?? "nil")
            return decoded
        }
    }

    private func callRoundTrip<Request: Codable, Response: Codable>(_ fn: RustStringFn?, request: Request) -> Response? {
        NSLog("Crawler FFI: callRoundTrip request=%@", String(describing: request))
        guard let fn else {
            NSLog("Crawler FFI: callRoundTrip skipped (missing fn)")
            return nil
        }
        let encoder = JSONEncoder()
        encoder.keyEncodingStrategy = .convertToSnakeCase
        guard let data = try? encoder.encode(request), let json = String(data: data, encoding: .utf8) else {
            return nil
        }
        let inputCString = json.cString(using: .utf8)!
        return inputCString.withUnsafeBufferPointer { buffer in
            guard let base = buffer.baseAddress else { return nil }
            guard let rawResult = fn(base) else { return nil }
            defer { self.freeFn?(rawResult) }
            let responseJSON = String(cString: rawResult)
            guard let data = responseJSON.data(using: .utf8) else { return nil }
            let envelope = try? JSONDecoder().decode(RustEnvelope<String>.self, from: data)
            guard let envelope, envelope.ok, let payload = envelope.data else { return nil }
            guard let payloadData = payload.data(using: .utf8) else { return nil }
            let decoded = try? JSONDecoder().decode(Response.self, from: payloadData)
            NSLog("Crawler FFI: callRoundTrip decoded=%@", decoded.map { String(describing: $0) } ?? "nil")
            return decoded
        }
    }

    private static func decodeEnvelopeString(_ rawResult: UnsafeMutablePointer<CChar>) -> String? {
        let json = String(cString: rawResult)
        NSLog("Crawler FFI: envelope=%@", json)
        guard let data = json.data(using: .utf8) else { return nil }
        let envelope = try? JSONDecoder().decode(RustEnvelope<String>.self, from: data)
        guard let envelope, envelope.ok, let value = envelope.data else { return nil }
        return value
    }

    private static func openLibrary() -> UnsafeMutableRawPointer? {
        let env = ProcessInfo.processInfo.environment
        let bundle = Bundle.main
        NSLog("Crawler FFI: bundleURL=%@", bundle.bundleURL.path)
        NSLog("Crawler FFI: executableURL=%@", bundle.executableURL?.path ?? "nil")
        var candidates: [String] = []
        if let explicit = env["GNOSTR_CRAWLER_FFI_LIBRARY"], !explicit.isEmpty {
            NSLog("Crawler FFI: explicit library=%@", explicit)
            candidates.append(explicit)
        } else {
            let cached = Self.homeDirectoryURL()
                .appendingPathComponent(".cache/cargo/debug/deps/libcrawler_ffi.dylib")
                .path
            candidates.append(contentsOf: [
                cached,
                bundle.privateFrameworksURL?.appendingPathComponent("libcrawler_ffi.dylib").path,
                bundle.bundleURL.appendingPathComponent("Frameworks/libcrawler_ffi.dylib").path,
                bundle.executableURL?.deletingLastPathComponent().appendingPathComponent("Frameworks/libcrawler_ffi.dylib").path,
                bundle.bundleURL.appendingPathComponent("libcrawler_ffi.dylib").path,
                "Rust/crawler-ffi/target/debug/libcrawler_ffi.dylib",
                "Rust/crawler-ffi/target/release/libcrawler_ffi.dylib",
            ].compactMap { $0 })
        }

        for candidate in candidates {
            NSLog("Crawler FFI: probing %@", candidate)
            dlerror()
            if let handle = dlopen(candidate, RTLD_NOW | RTLD_LOCAL) {
                NSLog("Crawler FFI: opened %@", candidate)
                return handle
            }
            if let error = dlerror() {
                NSLog("Crawler FFI: failed to open %@: %@", candidate, String(cString: error))
            } else {
                NSLog("Crawler FFI: failed to open %@: unknown dlerror", candidate)
            }
        }

        return nil
    }

    private static func homeDirectoryURL() -> URL {
        URL(fileURLWithPath: NSHomeDirectory(), isDirectory: true)
    }

    private static func loadSymbol<T>(_ name: String, from handle: UnsafeMutableRawPointer) -> T? {
        NSLog("Crawler FFI: loading symbol %@", name)
        guard let symbol = dlsym(handle, name) else { return nil }
        return unsafeBitCast(symbol, to: T.self)
    }

    private static let rustLogCallback: RustLogFn = { cString in
        guard let cString else { return }
        let line = String(cString: cString)
        NSLog("Crawler Rust: %@", line)
        RustCrawlerBridge.shared.onLogLine?(line)
    }
}

private struct GitNoteEventRequest: Codable {
    let note: GitNote
    let privateKeyHex: String
    let powTargetBits: UInt8

    private enum CodingKeys: String, CodingKey {
        case note
        case privateKeyHex = "private_key_hex"
        case powTargetBits = "pow_target_bits"
    }
}

private struct TextNoteEventRequest: Codable {
    let content: String
    let privateKeyHex: String
    let powTargetBits: UInt8

    private enum CodingKeys: String, CodingKey {
        case content
        case privateKeyHex = "private_key_hex"
        case powTargetBits = "pow_target_bits"
    }
}

private struct PublishTextNoteRequest: Codable {
    let relays: [String]
    let content: String
    let privateKeyHex: String
    let powTargetBits: UInt8

    private enum CodingKeys: String, CodingKey {
        case relays
        case content
        case privateKeyHex = "private_key_hex"
        case powTargetBits = "pow_target_bits"
    }
}

struct PublishedTextNoteResult: Codable, Sendable {
    let relayURLs: [URL]
    let event: Event

    private enum CodingKeys: String, CodingKey {
        case relayURLs = "relay_urls"
        case event
    }
}

private struct PublishGitNoteRequest: Codable {
    let relays: [String]
    let note: GitNote
    let privateKeyHex: String
    let powTargetBits: UInt8

    private enum CodingKeys: String, CodingKey {
        case relays
        case note
        case privateKeyHex = "private_key_hex"
        case powTargetBits = "pow_target_bits"
    }
}

typealias PublishedGitNoteResult = PublishedTextNoteResult
