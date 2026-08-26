import Foundation
import GnostrTypes

public struct AsyncGitRepository: Sendable {
    public let url: URL

    public init(url: URL) {
        self.url = url
    }

    public func repoState() -> RepoState {
        if let rustState = RustAsyncGitBridge.shared.repoState(at: self.url.path) {
            return rustState
        }

        let gitDir = self.url.appendingPathComponent(".git", isDirectory: true)

        if FileManager.default.fileExists(atPath: gitDir.appendingPathComponent("MERGE_HEAD").path) {
            return .merge
        }
        if FileManager.default.fileExists(atPath: gitDir.appendingPathComponent("rebase-merge").path)
            || FileManager.default.fileExists(atPath: gitDir.appendingPathComponent("rebase-apply").path)
        {
            return .rebase
        }
        if FileManager.default.fileExists(atPath: gitDir.appendingPathComponent("REVERT_HEAD").path) {
            return .revert
        }
        if FileManager.default.fileExists(atPath: gitDir.path) {
            return .clean
        }
        return .other
    }

    public func defaultNotesRef() -> String {
        RustAsyncGitBridge.shared.defaultNotesRef(at: self.url.path) ?? "refs/notes/commits"
    }

    public func gitNoteEventID(commitID: String) throws -> String {
        if let eventID = RustAsyncGitBridge.shared.gitNoteEventID(commitID: commitID) {
            return eventID
        }
        throw CocoaError(.fileReadUnknown)
    }

    public func generateGitNoteEvent(
        note: GitNote,
        privateKeyHex: String,
        powTargetBits: UInt8 = 0
    ) throws -> Event {
        try NIP34.rustGenerateGitNoteEvent(note: note, privateKeyHex: privateKeyHex, powTargetBits: powTargetBits)
    }

    public func gitNote(
        noteID: String,
        annotatedID: String,
        message: String,
        author: String,
        committer: String,
        committerTime: Int64,
        notesRef: String? = nil
    ) -> GitNote {
        GitNote(
            noteID: noteID,
            annotatedID: annotatedID,
            notesRef: notesRef,
            message: message,
            author: author,
            committer: committer,
            committerTime: committerTime
        )
    }
}
