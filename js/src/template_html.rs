use std::collections::HashMap;

#[derive(Clone, Copy, Debug, Default)]
pub struct TemplateHtml {
}

impl TemplateHtml {
    pub fn new() -> Self {
        TemplateHtml {}
    }

    pub fn bytes(&self) -> &'static [u8] {
        include_bytes!("template.html")
    }

    pub fn as_str(&self) -> &'static str {
        include_str!("template.html")
    }

    pub fn to_string(&self) -> String {
        self.as_str()
            .replace("__BUILD_NAME__", env!("GITUI_BUILD_NAME"))
    }
}

pub fn get_template_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert("template.html".to_string(), include_bytes!("template.html") as &'static [u8]);
    assets
}
