import Foundation
import SwiftSyntax
import SwiftParser

/// The Visitor class that defines our linting rules
class LinterVisitor: SyntaxVisitor {
    let url: URL
    var violations: Int = 0

    init(url: URL) {
        self.url = url
        // We use .foldAll to ensure we see all nested nodes
        super.init(viewMode: .all)
    }

    // RULE 1: Class names must be Uppercase
    override func visit(_ node: ClassDeclSyntax) -> SyntaxVisitorContinueKind {
        let className = node.name.text
        if let firstChar = className.first, !firstChar.isUppercase {
            printViolation(
                line: node.name.positionAfterSkippingLeadingTrivia.utf8Offset, 
                message: "Style Violation: Class '\(className)' should start with an uppercase letter."
            )
        }
        return .visitChildren
    }

    // RULE 2: Detect TODOs in comments
    override func visit(_ node: TokenSyntax) -> SyntaxVisitorContinueKind {
        for piece in node.leadingTrivia {
            if case .lineComment(let text) = piece, text.contains("TODO") {
                printViolation(
                    line: node.position.utf8Offset,
                    message: "Warning: Found a TODO comment."
                )
            }
        }
        return .visitChildren
    }

    private func printViolation(line: Int, message: String) {
        violations += 1
        print("\(url.lastPathComponent):\(line): \(message)")
    }
}

// --- Main Execution Logic ---

let arguments = CommandLine.arguments

guard arguments.count > 1 else {
    print("Usage: MySwiftLinter <path-to-swift-file>")
    exit(1)
}

let filePath = arguments[1]
let fileURL = URL(fileURLWithPath: filePath)

do {
    let sourceCode = try String(contentsOf: fileURL, encoding: .utf8)
    let sourceFile = Parser.parse(source: sourceCode)
    
    let visitor = LinterVisitor(url: fileURL)
    visitor.walk(sourceFile)
    
    print("\nLinting complete. Found \(visitor.violations) violation(s).")
    
    if visitor.violations > 0 {
        exit(1)
    }
} catch {
    print("Error: Could not read file at \(filePath)")
    exit(1)
}
