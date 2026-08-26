import Foundation
import Testing
@testable import AsyncGit

@Test func repoStateDetectsMergeAndRebaseMarkers() throws {
    let tempRoot = FileManager.default.temporaryDirectory.appendingPathComponent(UUID().uuidString)
    let gitDir = tempRoot.appendingPathComponent(".git", isDirectory: true)
    try FileManager.default.createDirectory(at: gitDir, withIntermediateDirectories: true)
    defer { try? FileManager.default.removeItem(at: tempRoot) }

    let repo = AsyncGitRepository(url: tempRoot)
    #expect(repo.repoState() == .clean)

    FileManager.default.createFile(
        atPath: gitDir.appendingPathComponent("MERGE_HEAD").path,
        contents: Data(),
        attributes: nil
    )
    #expect(repo.repoState() == .merge)

    try? FileManager.default.removeItem(at: gitDir.appendingPathComponent("MERGE_HEAD"))
    try FileManager.default.createDirectory(
        at: gitDir.appendingPathComponent("rebase-merge", isDirectory: true),
        withIntermediateDirectories: true
    )
    #expect(repo.repoState() == .rebase)
}

@Test func gitNoteCodableRoundTrips() throws {
    let note = GitNote(
        noteID: "deadbeef",
        annotatedID: "cafebabe",
        notesRef: "refs/notes/commits",
        message: "hello",
        author: "alice",
        committer: "bob",
        committerTime: 1234
    )

    let data = try JSONEncoder().encode(note)
    let decoded = try JSONDecoder().decode(GitNote.self, from: data)
    #expect(decoded == note)
}

@Test func eventKindsMatchRustConstants() {
    #expect(AsyncGitEventKind.repoAnnouncement.rawValue == 30617)
    #expect(AsyncGitEventKind.repoState.rawValue == 30618)
    #expect(AsyncGitEventKind.patches.rawValue == 1617)
}
