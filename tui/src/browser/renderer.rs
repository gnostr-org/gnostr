/*
 * Copyright (c) 2026 Geekspeaker Inc. All Rights Reserved.
 *
 * This software is "Source Available".
 * You may use and modify it for personal use.
 * Redistribution of modified versions is prohibited.
 *
 * See the LICENSE file for more details.
 */

use regex::{Captures, Regex};

/// Represents a form field extracted from HTML
#[derive(Debug, Clone)]
pub struct FormField {
    pub field_type: String,   // "text", "password", "hidden", "submit", "textarea"
    pub name: String,         // Field name attribute
    pub value: String,        // Current/default value
    pub placeholder: String,  // Placeholder text
    pub display_index: usize, // Index for display [F1], [F2], etc.
}

/// Result of rendering HTML - includes text, links, AND form fields
pub struct RenderResult {
    pub lines: Vec<String>,
    pub links: Vec<String>,
    pub form_fields: Vec<FormField>,
    pub form_action: Option<String>,
    pub form_method: String,
}

pub fn render_html_to_text(html: &str, width: usize) -> RenderResult {
    let effective_width = if width < 10 { 80 } else { width };

    // 0. Pre-process: Convert <img> tags to [IMG: alt] or [IMG]
    let img_re = Regex::new(r#"(?i)<img\s+[^>]*alt=[\"']([^\"']*)[\"'][^>]*/?>"#).unwrap();
    let html_with_img_alt = img_re.replace_all(html, |caps: &Captures| {
        let alt = caps[1].to_string();
        if alt.is_empty() {
            String::new()
        } else {
            alt
        }
    });
    // Fallback for images without alt attribute
    let img_no_alt_re = Regex::new(r#"(?i)<img\s+[^>]*/?>"#).unwrap();
    let html_with_all_imgs = img_no_alt_re.replace_all(&html_with_img_alt, "");

    // 0b. Extract and preserve <noscript> content (since we can't run JS)
    let noscript_re = Regex::new(r#"(?is)<noscript[^>]*>(.*?)</noscript>"#).unwrap();
    let html_with_noscript = noscript_re.replace_all(&html_with_all_imgs, |caps: &Captures| {
        format!("\n[NOSCRIPT: {}]\n", &caps[1])
    });

    // 1. Extract form information
    let mut form_fields: Vec<FormField> = Vec::new();
    let mut form_action: Option<String> = None;
    let mut form_method = "GET".to_string();

    // Extract form action and method
    // Try quoted first, then unquoted
    let form_re_quoted = Regex::new(r#"(?is)<form[^>]*action=["']([^"']+)["'][^>]*>"#).unwrap();
    let form_re_unquoted = Regex::new(r#"(?is)<form[^>]*action=([^\s>]+)[^>]*>"#).unwrap();

    if let Some(caps) = form_re_quoted.captures(&html_with_noscript) {
        let raw = caps[1].to_string();
        // Decode HTML entities and strip any remaining quotes
        let decoded = html_escape::decode_html_entities(&raw).to_string();
        let clean = decoded
            .trim_matches('"')
            .trim_matches('\'')
            .replace("\"", "");
        form_action = Some(clean);
    } else if let Some(caps) = form_re_unquoted.captures(&html_with_noscript) {
        let raw = caps[1].to_string();
        let decoded = html_escape::decode_html_entities(&raw).to_string();
        let clean = decoded
            .trim_matches('"')
            .trim_matches('\'')
            .replace("\"", "");
        form_action = Some(clean);
    }
    let method_re = Regex::new(r#"(?is)<form[^>]*method=["']([^"']+)["'][^>]*>"#).unwrap();
    if let Some(caps) = method_re.captures(&html_with_noscript) {
        form_method = caps[1].to_uppercase();
    }

    // Extract input fields
    let input_re = Regex::new(r#"(?i)<input\s+([^>]*)/?>"#).unwrap();
    let mut field_index = 0;

    let html_with_inputs = input_re.replace_all(&html_with_noscript, |caps: &Captures| {
        let attrs = &caps[1];

        // Extract type
        let type_re = Regex::new(r#"(?i)type=["']([^"']+)["']"#).unwrap();
        let field_type = type_re
            .captures(attrs)
            .map(|c| c[1].to_lowercase())
            .unwrap_or_else(|| "text".to_string());

        // Skip hidden and submit for now in the visual display
        if field_type == "hidden" {
            // Still track hidden fields for form submission
            let name_re = Regex::new(r#"(?i)name=["']([^"']+)["']"#).unwrap();
            let value_re = Regex::new(r#"(?i)value=["']([^"']+)["']"#).unwrap();
            let name = name_re
                .captures(attrs)
                .map(|c| c[1].to_string())
                .unwrap_or_default();
            let value = value_re
                .captures(attrs)
                .map(|c| html_escape::decode_html_entities(&c[1]).to_string())
                .unwrap_or_default();

            form_fields.push(FormField {
                field_type: "hidden".to_string(),
                name,
                value,
                placeholder: String::new(),
                display_index: 0,
            });
            return String::new(); // Don't render hidden fields
        }

        if field_type == "submit" || field_type == "button" {
            let value_re = Regex::new(r#"(?i)value=["']([^"']+)["']"#).unwrap();
            let label = value_re
                .captures(attrs)
                .map(|c| html_escape::decode_html_entities(&c[1]).to_string())
                .unwrap_or_else(|| "Submit".to_string());

            field_index += 1;
            form_fields.push(FormField {
                field_type: "submit".to_string(),
                name: String::new(),
                value: label.clone(),
                placeholder: String::new(),
                display_index: field_index,
            });
            return format!("[F{}: {}]", field_index, label);
        }

        // Regular input field
        field_index += 1;
        let name_re = Regex::new(r#"(?i)name=["']([^"']+)["']"#).unwrap();
        let placeholder_re = Regex::new(r#"(?i)placeholder=["']([^"']+)["']"#).unwrap();
        let value_re = Regex::new(r#"(?i)value=["']([^"']+)["']"#).unwrap();

        let name = name_re
            .captures(attrs)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        let placeholder = placeholder_re
            .captures(attrs)
            .map(|c| html_escape::decode_html_entities(&c[1]).to_string())
            .unwrap_or_else(|| name.clone());
        let value = value_re
            .captures(attrs)
            .map(|c| html_escape::decode_html_entities(&c[1]).to_string())
            .unwrap_or_default();

        // Determine display label BEFORE moving values
        let display_label = if !placeholder.is_empty() {
            placeholder.clone()
        } else if !name.is_empty() {
            name.clone()
        } else {
            "input".to_string()
        };

        form_fields.push(FormField {
            field_type,
            name,
            value,
            placeholder,
            display_index: field_index,
        });

        format!("[F{}: {}________]", field_index, display_label)
    });

    // Extract textarea fields
    let textarea_re = Regex::new(r#"(?is)<textarea\s+([^>]*)>(.*?)</textarea>"#).unwrap();
    let html_with_textareas = textarea_re.replace_all(&html_with_inputs, |caps: &Captures| {
        field_index += 1;
        let attrs = &caps[1];
        let inner = &caps[2];

        let name_re = Regex::new(r#"(?i)name=["']([^"']+)["']"#).unwrap();
        let placeholder_re = Regex::new(r#"(?i)placeholder=["']([^"']+)["']"#).unwrap();

        let name = name_re
            .captures(attrs)
            .map(|c| c[1].to_string())
            .unwrap_or_default();
        let placeholder = placeholder_re
            .captures(attrs)
            .map(|c| html_escape::decode_html_entities(&c[1]).to_string())
            .unwrap_or_else(|| name.clone());
        let value = html_escape::decode_html_entities(inner).trim().to_string();

        // Determine display label BEFORE moving values
        let display_label = if !placeholder.is_empty() {
            placeholder.clone()
        } else if !name.is_empty() {
            name.clone()
        } else {
            "text".to_string()
        };

        form_fields.push(FormField {
            field_type: "textarea".to_string(),
            name,
            value,
            placeholder,
            display_index: field_index,
        });

        format!("[F{}: {}________]", field_index, display_label)
    });

    // 2. Pre-process HTML to inject link indices AFTER link text
    let mut links = Vec::new();
    let re = Regex::new(r#"(?is)<a\s+[^>]*href=["']([^"']+)["'][^>]*>(.*?)</a>"#).unwrap();

    let injected_html = re.replace_all(&html_with_textareas, |caps: &Captures| {
        let url_raw = caps[1].to_string();
        let url_decoded = html_escape::decode_html_entities(&url_raw).to_string();
        // Strip any quotes that might have been captured
        let url_clean = url_decoded
            .trim()
            .trim_matches('"')
            .trim_matches('\'')
            .replace("\"", "")
            .to_string();
        links.push(url_clean);
        let index = links.len();
        let inner_text = &caps[2];
        format!("{}[{}]", inner_text, index)
    });

    // 3. Render text using standard html2text
    let text = html2text::from_read(injected_html.as_bytes(), effective_width);

    RenderResult {
        lines: text.lines().map(|s| s.to_string()).collect(),
        links,
        form_fields,
        form_action,
        form_method,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_simple_html() {
        let html = r#"
            <html>
                <body>
                    <h1>Hello World</h1>
                    <p>This is a <a href="https://example.com">link</a>.</p>
                </body>
            </html>
        "#;
        let result = render_html_to_text(html, 80);
        assert!(result.lines.iter().any(|line| line.contains("Hello World")));
        assert!(result.lines.iter().any(|line| line.contains("link[1]")));
        assert_eq!(result.links.len(), 1);
        assert_eq!(result.links[0], "https://example.com");
    }

    #[test]
    fn test_render_form() {
        let html = r#"
            <form action="/search" method="GET">
                <input type="text" name="q" placeholder="Search...">
                <input type="submit" value="Go">
            </form>
        "#;
        let result = render_html_to_text(html, 80);
        assert_eq!(result.form_fields.len(), 2);
        assert_eq!(result.form_fields[0].name, "q");
        assert_eq!(result.form_fields[0].field_type, "text");
        assert_eq!(result.form_fields[1].field_type, "submit");
        assert!(result
            .lines
            .iter()
            .any(|line| line.contains("[F1: Search...________]")));
        assert!(result.lines.iter().any(|line| line.contains("[F2: Go]")));
    }
}
