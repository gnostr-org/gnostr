import XCTest
@testable import LibP2PAppTemplateCore

@MainActor
final class DualPeerInteropTests: XCTestCase {
    func testDualP2PServiceInitialization() {
        let dual = DualP2PService()
        XCTAssertNotEqual(dual.primary.peerIDString, dual.secondary.peerIDString)
        XCTAssertNotEqual(dual.primary.listenPort, dual.secondary.listenPort)
        XCTAssertTrue(dual.secondary.listenPort > dual.primary.listenPort)
    }

    func testPeerIDDeterministicVsRandom() {
        let dual1 = DualP2PService()
        let dual2 = DualP2PService()
        // Primary peers should be deterministic (same platform = same ID)
        XCTAssertEqual(dual1.primary.peerIDString, dual2.primary.peerIDString)
        // Secondary peers should be random (different each time)
        XCTAssertNotEqual(dual1.secondary.peerIDString, dual2.secondary.peerIDString)
    }

    func testChatMessageMerging() {
        let dual = DualP2PService()
        dual.primary.chatMessages = [
            P2PService.ChatEntry(
                id: "msg-1",
                topic: "test",
                kind: "Chat",
                author: "alice",
                text: "hello",
                isLocal: true,
                timestamp: Date(),
                pingDeltaMs: nil
            )
        ]
        dual.secondary.chatMessages = [
            P2PService.ChatEntry(
                id: "msg-1",
                topic: "test",
                kind: "Chat",
                author: "alice",
                text: "hello",
                isLocal: false,
                timestamp: Date(),
                pingDeltaMs: nil
            ),
            P2PService.ChatEntry(
                id: "msg-2",
                topic: "test",
                kind: "Chat",
                author: "bob",
                text: "world",
                isLocal: false,
                timestamp: Date(),
                pingDeltaMs: nil
            )
        ]

        let merged = dual.chatMessages
        XCTAssertEqual(merged.count, 2, "Should deduplicate by ID")
        XCTAssertTrue(merged.contains(where: { $0.id == "msg-1" }))
        XCTAssertTrue(merged.contains(where: { $0.id == "msg-2" }))
    }

    func testStateAggregation() {
        let dual = DualP2PService()
        dual.primary.state = .running
        dual.secondary.state = .running
        XCTAssertEqual(dual.state, "running")

        dual.primary.state = .stopped
        XCTAssertEqual(dual.state, "starting/stopping")

        dual.secondary.state = .stopped
        XCTAssertEqual(dual.state, "stopped")
    }

    func testSendChatMessageClearsDraft() {
        let dual = DualP2PService()
        dual.chatDraftMessage = "test message"
        dual.sendChatMessage()
        XCTAssertEqual(dual.chatDraftMessage, "")
    }
}
