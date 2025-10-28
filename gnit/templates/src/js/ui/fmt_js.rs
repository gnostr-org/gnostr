struct UIFmt {
    fmt_js: &'static [u8],
}

impl UIFmt {
    fn new() -> Self {
        let fmt_js_bytes: &'static [u8] = include_bytes!("fmt.js");
        UIFmt {
            fmt_js: fmt_js_bytes,
        }
    }

    fn fmt_js(&self) -> &'static [u8] {
        self.fmt_js
    }

    fn to_string(&self) -> String {
        if let Ok(fmt_js_string) = String::from_utf8(self.fmt_js.to_vec()) {
            fmt_js_string
        } else {
            String::from("js/ui/fmt.js is not valid UTF-8.")
        }
    }
}
