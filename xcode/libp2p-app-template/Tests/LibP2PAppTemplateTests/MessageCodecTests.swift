import XCTest
@testable import LibP2PAppTemplateCore

final class MessageCodecTests: XCTestCase {
    func testEncodeDecodeChatMessage() {
        let original = P2PService.RustChatMessage(from: "alice", content: "hello world", kind: "Chat")
        let data = try! JSONEncoder().encode(original)
        let decoded = try! JSONDecoder().decode(P2PService.RustChatMessage.self, from: data)

        XCTAssertEqual(decoded.from, "alice")
        XCTAssertEqual(decoded.content, ["hello world"])
        XCTAssertEqual(decoded.kind, "Chat")
        XCTAssertEqual(decoded.commitId, String(repeating: "0", count: 40))
    }

    func testEncodeDecodePingMessage() {
        let original = P2PService.RustChatMessage(from: "bob", content: "1693843200000", kind: "Ping")
        let data = try! JSONEncoder().encode(original)
        let decoded = try! JSONDecoder().decode(P2PService.RustChatMessage.self, from: data)

        XCTAssertEqual(decoded.from, "bob")
        XCTAssertEqual(decoded.content, ["1693843200000"])
        XCTAssertEqual(decoded.kind, "Ping")
    }

    func testCodingKeys() {
        let msg = P2PService.RustChatMessage(from: "test", content: "data")
        let data = try! JSONEncoder().encode(msg)
        let json = String(data: data, encoding: .utf8)!
        XCTAssertTrue(json.contains("\"from\""))
        XCTAssertTrue(json.contains("\"content\""))
        XCTAssertTrue(json.contains("\"kind\""))
        XCTAssertTrue(json.contains("\"commit_id\""))
        // Optional nil fields are omitted by synthesized Codable
        XCTAssertFalse(json.contains("\"message_id\""))
    }
}
