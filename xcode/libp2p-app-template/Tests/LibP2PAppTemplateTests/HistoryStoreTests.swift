import XCTest
@testable import LibP2PAppTemplateCore

final class HistoryStoreTests: XCTestCase {
    func testStableIDConsistency() {
        let id1 = HistoryStore.stableID(author: "alice", text: "hello", timestamp: 1000)
        let id2 = HistoryStore.stableID(author: "alice", text: "hello", timestamp: 2000)
        XCTAssertEqual(id1, id2, "stableID should be content-based, not time-based")
    }

    func testStableIDUniqueness() {
        let id1 = HistoryStore.stableID(author: "alice", text: "hello", timestamp: 1000)
        let id2 = HistoryStore.stableID(author: "alice", text: "world", timestamp: 1000)
        let id3 = HistoryStore.stableID(author: "bob", text: "hello", timestamp: 1000)
        XCTAssertNotEqual(id1, id2)
        XCTAssertNotEqual(id1, id3)
        XCTAssertNotEqual(id2, id3)
    }

    func testAddAndRetrieve() async {
        let store = HistoryStore()
        let msg = await store.add(topic: "test", kind: "Chat", author: "alice", text: "hello")
        XCTAssertEqual(msg.author, "alice")
        XCTAssertEqual(msg.text, "hello")
        XCTAssertEqual(msg.topic, "test")

        let history = await store.history(for: "test", limit: 10)
        XCTAssertEqual(history.count, 1)
        XCTAssertEqual(history.first?.text, "hello")
    }

    func testDeduplication() async {
        let store = HistoryStore()
        let msg1 = await store.add(topic: "test", kind: "Chat", author: "alice", text: "hello")
        let msg2 = await store.add(topic: "test", kind: "Chat", author: "alice", text: "hello")
        // Same content should produce same ID and be deduplicated
        XCTAssertEqual(msg1.id, msg2.id)

        let history = await store.allMessages(for: "test")
        XCTAssertEqual(history.count, 1)
    }

    func testMerge() async {
        let store = HistoryStore()
        let msg1 = await store.add(topic: "test", kind: "Chat", author: "alice", text: "hello")

        let remoteMessages = [
            HistoryMessage(id: msg1.id, topic: "test", kind: "Chat", author: "alice", text: "hello", timestamp: 999),
            HistoryMessage(id: "new-id", topic: "test", kind: "Chat", author: "bob", text: "world", timestamp: 999)
        ]

        let merged = await store.merge(remoteMessages)
        XCTAssertEqual(merged.count, 1, "Only one new message should be merged")
        XCTAssertEqual(merged.first?.author, "bob")

        let all = await store.allMessages(for: "test")
        XCTAssertEqual(all.count, 2)
    }

    func testTrim() async {
        let store = HistoryStore()
        for i in 0..<250 {
            _ = await store.add(topic: "test", kind: "Chat", author: "alice", text: "msg-\(i)")
        }
        let all = await store.allMessages(for: "test")
        XCTAssertEqual(all.count, 200, "Should trim to maxMessagesPerTopic")
    }

    func testHistoryLimit() async {
        let store = HistoryStore()
        for i in 0..<50 {
            _ = await store.add(topic: "test", kind: "Chat", author: "alice", text: "msg-\(i)")
        }
        let history = await store.history(for: "test", limit: 10)
        XCTAssertEqual(history.count, 10)
        XCTAssertEqual(history.first?.text, "msg-40")
        XCTAssertEqual(history.last?.text, "msg-49")
    }
}
