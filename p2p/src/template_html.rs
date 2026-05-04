use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct TemplateHtml;

impl TemplateHtml {
    pub fn new() -> Self {
        Self
    }

    pub fn bytes(&self) -> &'static [u8] {
        gnostr_js::TEMPLATE_HTML_BYTES
    }

    pub fn as_str(&self) -> &'static str {
        gnostr_js::TEMPLATE_HTML
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

pub fn get_template_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert("template.html".to_string(), gnostr_js::TEMPLATE_HTML_BYTES);
    assets
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn template_html_exposes_expected_bytes() {
        let template = TemplateHtml::new();
        assert!(template.bytes().starts_with(b"<!DOCTYPE html>"));
        assert!(template.as_str().contains(r#"<script defer src="/js/main.js?v=1"></script>"#));
    }

    #[test]
    fn template_asset_map_includes_template_html() {
        let assets = get_template_assets();
        let template = assets.get("template.html").expect("template asset");
        assert!(template.starts_with(b"<!DOCTYPE html>"));
    }
}
