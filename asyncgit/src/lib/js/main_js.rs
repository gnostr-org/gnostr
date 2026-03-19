pub struct JSMain {
    pub main_js: &'static [u8],
}

impl JSMain {
    pub fn new() -> Self {
        let main_js_bytes: &'static [u8] = include_bytes!("main.js");
        JSMain {
            main_js: main_js_bytes,
        }
    }

    pub fn js_main(&self) -> &'static [u8] {
        self.main_js
    }

    pub fn to_string(&self) -> String {
        if let Ok(main_js_string) = String::from_utf8(self.main_js.to_vec()) {
            main_js_string
        } else {
            String::from("js/main.js is not valid UTF-8.")
        }
    }
}
