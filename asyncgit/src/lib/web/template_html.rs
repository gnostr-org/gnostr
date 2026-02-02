#[allow(unused)]
use handlebars::Handlebars;
use std::collections::HashMap;

#[derive(Clone)]
pub struct TemplateHtml {
    pub template_html: &'static [u8],
}

impl TemplateHtml {
    pub fn new() -> Self {
        let template_html_bytes: &'static [u8] = include_bytes!("../../template.html");
        TemplateHtml {
            template_html: template_html_bytes,
        }
    }

    pub fn template_html(&self) -> &'static [u8] {
        self.template_html
    }

    pub fn to_string(&self) -> String {
        if let Ok(template_html_string) = String::from_utf8(self.template_html.to_vec()) {
            template_html_string
        } else {
            String::from("../../template.html is not valid UTF-8.")
        }
    }
    pub fn get_template_assets() -> HashMap<String, &'static [u8]> {
        let mut assets = HashMap::new();
        assets.insert("template.html".to_string(), include_bytes!("../../template.html") as &'static [u8]);
        assets
    }
}

pub fn get_template_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert("template.html".to_string(), include_bytes!("../../template.html") as &'static [u8]);
    assets
}
