import MacroToolkit
import SwiftSyntax
import SwiftSyntaxMacroExpansion
import SwiftSyntaxMacros

extension Type {
    var isOptional: Bool {
        let description = _syntax.trimmedDescription

        return description.hasSuffix("?")
            || (description.hasPrefix("Optional<") && description.hasSuffix(">"))
    }
}
