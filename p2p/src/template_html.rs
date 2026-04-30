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
