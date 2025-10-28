struct UIUtil {
    util_js: &'static [u8],
}

impl UIUtil {
    fn new() -> Self {
        let util_js_bytes: &'static [u8] = include_bytes!("util.js");
        UIUtil {
            util_js: util_js_bytes,
        }
    }

    fn util_js(&self) -> &'static [u8] {
        self.util_js
    }

    fn to_string(&self) -> String {
        if let Ok(util_js_string) = String::from_utf8(self.util_js.to_vec()) {
            util_js_string
        } else {
            String::from("js/ui/util.js is not valid UTF-8.")
        }
    }
}
