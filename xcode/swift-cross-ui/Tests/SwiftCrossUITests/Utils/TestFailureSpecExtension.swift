import SwiftSyntaxMacrosGenericTestSupport
import Testing

extension TestFailureSpec {
    public var issueComment: Comment {
        "\(message) \nat: \(location.filePath) \(location.line):\(location.column)"
    }
}
