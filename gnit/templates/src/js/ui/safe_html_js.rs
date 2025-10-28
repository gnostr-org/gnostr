struct UISafeHtml {
    safe_html_js: &'static [u8],
}

impl UISafeHtml {
    fn new() -> Self {
        let safe_html_js_bytes: &'static [u8] = include_bytes!("safe-html.js");
        UISafeHtml {
            safe_html_js: safe_html_js_bytes,
        }
    }

    fn safe_html_js(&self) -> &'static [u8] {
        self.safe_html_js
    }

    fn to_string(&self) -> String {
        if let Ok(safe_html_js_string) = String::from_utf8(self.safe_html_js.to_vec()) {
            safe_html_js_string
        } else {
            String::from("js/ui/safe-html.js is not valid UTF-8.")
        }
    }
}
