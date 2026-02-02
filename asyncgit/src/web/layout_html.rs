#[allow(unused)]
use handlebars::Handlebars;
use std::collections::HashMap;

#[derive(Clone)]
pub struct LayoutHtml {
    pub layout_html: &'static [u8],
}

impl LayoutHtml {
    pub fn new() -> Self {
        let layout_html_bytes: &'static [u8] = include_bytes!("layout.html");
        LayoutHtml {
            layout_html: layout_html_bytes,
        }
    }

    pub fn layout_html(&self) -> &'static [u8] {
        self.layout_html
    }

    pub fn to_string(&self) -> String {
        if let Ok(layout_html_string) = String::from_utf8(self.layout_html.to_vec()) {
            layout_html_string
        } else {
            String::from("layout.html is not valid UTF-8.")
        }
    }
    pub fn get_layout_assets() -> HashMap<String, &'static [u8]> {
        let mut assets = HashMap::new();
        assets.insert("layout.html".to_string(), include_bytes!("layout.html") as &'static [u8]);
        assets
    }
}

pub fn get_layout_assets() -> HashMap<String, &'static [u8]> {
    let mut assets = HashMap::new();
    assets.insert("layout.html".to_string(), include_bytes!("layout.html") as &'static [u8]);
    assets
}
