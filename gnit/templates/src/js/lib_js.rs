struct JSLib {
    lib_js: &'static [u8],
}

impl JSLib {
    fn new() -> Self {
        let lib_js_bytes: &'static [u8] = include_bytes!("lib.js");
        JSLib {
            lib_js: lib_js_bytes,
        }
    }

    fn lib_js(&self) -> &'static [u8] {
        self.lib_js
    }

    fn to_string(&self) -> String {
        if let Ok(lib_js_string) = String::from_utf8(self.lib_js.to_vec()) {
            lib_js_string
        } else {
            String::from("js/lib.js is not valid UTF-8.")
        }
    }
}
