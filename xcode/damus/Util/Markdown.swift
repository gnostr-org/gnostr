//
//  Markdown.swift
//  damus
//
//  Created by Lionello Lunesu on 2022-12-28.
//

import Foundation

public struct Markdown {
    private let detector = try! NSDataDetector(types: NSTextCheckingResult.CheckingType.link.rawValue)

    /// Ensure the specified URL has a scheme by prepending "https://" if it's absent.
    static func withScheme(_ url: any StringProtocol) -> any StringProtocol {
        return url.contains("://") ? url : "https://" + url
    }

    public static func parseToAttributedString(content: String) -> AttributedString {
        // Similar to the parsing in NoteContentView
        let md_opts: AttributedString.MarkdownParsingOptions =
            .init(interpretedSyntax: .inlineOnlyPreservingWhitespace)

        if let txt = try? AttributedString(markdown: content, options: md_opts) {
            return txt
        } else {
            return AttributedString(stringLiteral: content)
        }
    }

    /// Process the input text and add markdown for any embedded URLs.
    public func process(_ input: String) -> AttributedString {
        let matches = detector.matches(in: input, options: [], range: NSRange(location: 0, length: input.utf16.count))
        var output = input
        // Start with the last match, because replacing the first would invalidate all subsequent indices
        for match in matches.reversed() {
            guard let range = Range(match.range, in: input) else { continue }
            let url = input[range]
            output.replaceSubrange(range, with: "[\(url)](\(Markdown.withScheme(url)))")
        }
        // TODO: escape unintentional markdown
        return Markdown.parseToAttributedString(content: output)
    }

    public static func parseToHTML(content: String) -> String {
        var html = content
        
        // Headers
        html = html.replacingOccurrences(of: "^###### (.*)$", with: "<h6>$1</h6>", options: .regularExpression, range: nil)
        html = html.replacingOccurrences(of: "^##### (.*)$", with: "<h5>$1</h5>", options: .regularExpression, range: nil)
        html = html.replacingOccurrences(of: "^#### (.*)$", with: "<h4>$1</h4>", options: .regularExpression, range: nil)
        html = html.replacingOccurrences(of: "^### (.*)$", with: "<h3>$1</h3>", options: .regularExpression, range: nil)
        html = html.replacingOccurrences(of: "^## (.*)$", with: "<h2>$1</h2>", options: .regularExpression, range: nil)
        html = html.replacingOccurrences(of: "^# (.*)$", with: "<h1>$1</h1>", options: .regularExpression, range: nil)

        // Bold
        html = html.replacingOccurrences(of: "\\*\\*(.*?)\\*\\*", with: "<strong>$1</strong>", options: .regularExpression, range: nil)
        html = html.replacingOccurrences(of: "__(.*?)__", with: "<strong>$1</strong>", options: .regularExpression, range: nil)

        // Italic
        html = html.replacingOccurrences(of: "\\*(.*?)\\*", with: "<em>$1</em>", options: .regularExpression, range: nil)
        html = html.replacingOccurrences(of: "_(.*?)_", with: "<em>$1</em>", options: .regularExpression, range: nil)

        // Links
        html = html.replacingOccurrences(of: "\\[(.*?)\\]\\((.*?)\\)", with: "<a href=\"$2\">$1</a>", options: .regularExpression, range: nil)

        // Code blocks (simple, assuming triple backticks on their own line)
        html = html.replacingOccurrences(of: "```\\n([\\s\\S]*?)\\n```", with: "<pre><code>$1</code></pre>", options: .regularExpression, range: nil)
        // Inline code
        html = html.replacingOccurrences(of: "`([^`]+)`", with: "<code>$1</code>", options: .regularExpression, range: nil)
        
        // Convert newlines to <br> for basic paragraph breaks, but only outside of pre/code blocks
        html = html.replacingOccurrences(of: "([^\n])\n([^\n])", with: "$1<br>$2", options: .regularExpression, range: nil)

        // Simple lists (start with -, *, or + followed by a space)
        html = html.replacingOccurrences(of: "^[\\-+\\*] (.*)$", with: "<li>$1</li>", options: .regularExpression, range: nil)
        if html.contains("<li>") {
            html = "<ul>\n" + html + "\n</ul>"
            html = html.replacingOccurrences(of: "</ul>\n<li>", with: "<li>", options: .literal, range: nil) // Remove extra ul around list items
            html = html.replacingOccurrences(of: "</li>\n<ul>", with: "</li>", options: .literal, range: nil)
        }

        // Wrap in a basic HTML structure with some minimal styling
        return """
        <!DOCTYPE html>
        <html>
        <head>
            <meta name="viewport" content="width=device-width, initial-scale=1">
            <style>
                body { font-family: -apple-system, sans-serif; margin: 15px; background-color: #f0f0f0; color: #333; }
                h1, h2, h3, h4, h5, h6 { color: #000; }
                pre { background-color: #eee; padding: 10px; border-radius: 5px; overflow-x: auto; }
                code { background-color: #eee; padding: 2px 4px; border-radius: 3px; }
                a { color: #007bff; text-decoration: none; }
                a:hover { text-decoration: underline; }
                ul { list-style-type: disc; margin-left: 20px; }
                li { margin-bottom: 5px; }
            </style>
        </head>
        <body>
            \(html)
        </body>
        </html>
        """
    }
}
