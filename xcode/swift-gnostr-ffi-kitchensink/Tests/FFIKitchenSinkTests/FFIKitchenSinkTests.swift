import Foundation
import Testing
@testable import FFIKitchenSink

@Test func packageReexportsCoreTypes() {
    let note = GitNote(
        noteID: "deadbeef",
        annotatedID: "cafebabe",
        notesRef: "refs/notes/commits",
        message: "hello",
        author: "alice",
        committer: "bob",
        committerTime: 1234
    )
    #expect(note.noteID == "deadbeef")
    #expect(FFIKitchenSink.asyncGitEventKinds().contains(.patches))
    _ = FFIKitchenSink.crawlerClient()
    _ = FFIKitchenSink.relayClient()
}

@Test func bridgeNamespaceIsReachable() {
    _ = FFIKitchenSink.crawlerBridge
    _ = FFIKitchenSink.gnostrTypesBridge
    _ = FFIKitchenSink.relayBridge
}

@Test func crawlerClientFactoryPreservesBaseURL() {
    let baseURL = URL(string: "https://crawler.example/api")!
    let client = FFIKitchenSink.crawlerClient(baseURL: baseURL)
    #expect(client.baseURL == baseURL)
}
