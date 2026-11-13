import XCTest
@testable import Kubo

final class KuboTests: XCTestCase {
    func testVersion() {
        XCTAssertFalse(Kubo.version().isEmpty)
    }

    func testInitAndRun() throws {
        let dir = NSTemporaryDirectory() + "kubo-swift-test"
        try? FileManager.default.removeItem(atPath: dir)
        try Kubo.initRepo(dir)
        let k = Kubo(repoPath: dir)
        let out = try k.run("version")
        XCTAssertTrue(out.contains("ipfs version"))
    }
}