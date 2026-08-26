@_exported import GnostrTypes
@_exported import GnostrTypes
import Foundation

public enum AsyncGitEventKind: UInt32, Sendable {
    case repoAnnouncement = 30617
    case repoState = 30618
    case patches = 1617
    case gitStatusOpen = 1630
    case gitStatusApplied = 1631
    case gitStatusClosed = 1632
    case gitStatusDraft = 1633
}

public typealias NoteInfo = GitNote
