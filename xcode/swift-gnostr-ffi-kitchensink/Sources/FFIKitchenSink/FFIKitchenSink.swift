import Foundation
@_exported import AsyncGit
@_exported import Crawler
@_exported import Relay
@_exported import GnostrTypes

public enum FFIKitchenSink {
    public static let crawlerBridge = RustCrawlerBridge.shared
    public static let gnostrTypesBridge = RustGnostrTypesBridge.shared
    public static let relayBridge = RustRelayBridge.shared

    public static func asyncGitEventKinds() -> [AsyncGitEventKind] {
        [.repoAnnouncement, .repoState, .patches, .gitStatusOpen, .gitStatusApplied, .gitStatusClosed, .gitStatusDraft]
    }

    public static func crawlerClient(baseURL: URL = URL(string: "http://127.0.0.1:3030")!, session: URLSession = .shared) -> CrawlerClient {
        CrawlerClient(baseURL: baseURL, session: session)
    }

    public static func relayClient(baseURL: URL = URL(string: "http://127.0.0.1:3030")!, session: URLSession = .shared) -> RelayClient {
        RelayClient(baseURL: baseURL, session: session)
    }
}
