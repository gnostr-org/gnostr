struct JSCore {
    core_js: &'static [u8],
}

impl JSCore {
    fn new() -> Self {
        let core_js_bytes: &'static [u8] = include_bytes!("core.js");
        JSCore {
            core_js: core_js_bytes,
        }
    }

    fn core_js(&self) -> &'static [u8] {
        self.core_js
    }

    fn to_string(&self) -> String {
        if let Ok(core_js_string) = String::from_utf8(self.core_js.to_vec()) {
            core_js_string
        } else {
            String::from("js/core.js is not valid UTF-8.")
        }
    }
}
