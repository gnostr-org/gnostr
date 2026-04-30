use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct TemplateHtml;

impl TemplateHtml {
    pub fn new() -> Self {
        Self
    }

    pub fn bytes(&self) -> &'static [u8] {
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../js/src/template.html"))
    }

    pub fn as_str(&self) -> &'static str {
        include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../js/src/template.html"))
    }

    pub fn to_string(&self) -> String {
        self.as_str().to_string()
    }
}

pub fn get_template_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert(
        "template.html".to_string(),
        include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/../js/src/template.html"))
            as &'static [u8],
    );
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
