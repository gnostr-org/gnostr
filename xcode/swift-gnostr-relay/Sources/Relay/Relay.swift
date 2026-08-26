import Foundation
@_exported import Crawler
import GnostrTypes

public struct RelayConfiguration: Codable, Hashable, Sendable {
    public var logging: String
    public var configFilePath: String

    public init(logging: String = "info", configFilePath: String = ".gnostr/relay.toml") {
        self.logging = logging
        self.configFilePath = configFilePath
    }

    private enum CodingKeys: String, CodingKey {
        case logging
        case configFilePath = "config_file_path"
    }

    public static func rustDefault() -> RelayConfiguration? {
        RustRelayBridge.shared.defaultConfiguration()
    }
}

public final class RelayClient: @unchecked Sendable {
    private let crawlerClient: CrawlerClient

    public init(baseURL: URL = URL(string: "http://127.0.0.1:3030")!, session: URLSession = .shared) {
        self.crawlerClient = CrawlerClient(baseURL: baseURL, session: session)
    }

    public func status() async throws -> RelayProcessState {
        try await self.crawlerClient.relayStatus()
    }

    public func start() async throws -> RelayProcessState {
        try await self.crawlerClient.startRelay()
    }

    public func stop() async throws -> RelayProcessState {
        try await self.crawlerClient.stopRelay()
    }

    public func discovery() async throws -> [RelayDiscoveryEntry] {
        try await self.crawlerClient.relayDiscovery()
    }
}

public enum RelayEndpoints {
    public static func listenEndpoint(host: String, port: UInt16) -> String {
        if let bridged = RustRelayBridge.shared.listenEndpoint(host: host, port: port) {
            return bridged
        }
        return "ws://\(host):\(port)"
    }
}
