import Foundation
import Testing
@testable import Crawler

private func writeText(_ text: String, to url: URL) throws {
    try FileManager.default.createDirectory(at: url.deletingLastPathComponent(), withIntermediateDirectories: true)
    try text.write(to: url, atomically: true, encoding: .utf8)
}

@Test func loadsCrawlerAndP2PBuckets() throws {
    let root = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString, isDirectory: true)
    let crawlerRoot = root.appendingPathComponent("crawler", isDirectory: true)
    let p2pRoot = root.appendingPathComponent("p2p", isDirectory: true)

    defer {
        try? FileManager.default.removeItem(at: root)
    }

    try writeText(
        """
        - wss://relay.example/
        - relay.example
        """,
        to: crawlerRoot.appendingPathComponent("relays.yaml")
    )
    try writeText(
        "wss://relay-two/\n",
        to: crawlerRoot.appendingPathComponent("34/relays.txt")
    )

    try writeText(
        #"["wss://root.example/"]"#,
        to: p2pRoot.appendingPathComponent("relays.json")
    )
    try writeText(
        "{}",
        to: p2pRoot.appendingPathComponent("23/relay.example.json")
    )

    let crawlerBuckets = CrawlerNetworkFileSystem.loadRelayBuckets(in: crawlerRoot, source: "crawler")
    let p2pBuckets = CrawlerNetworkFileSystem.loadRelayBuckets(in: p2pRoot, source: "p2p")

    #expect(crawlerBuckets.contains(where: { $0.bucket == "relays" && $0.relays == ["wss://relay.example", "wss://relay.example/"] }))
    #expect(crawlerBuckets.contains(where: { $0.bucket == "34" && $0.relays == ["wss://relay-two/"] }))
    #expect(p2pBuckets.contains(where: { $0.bucket == "relays" && $0.relays == ["wss://root.example/"] }))
    #expect(p2pBuckets.contains(where: { $0.bucket == "23" && $0.relays == ["https://relay.example"] }))
}
