struct JSModel {
    model_js: &'static [u8],
}

impl JSModel {
    fn new() -> Self {
        let model_js_bytes: &'static [u8] = include_bytes!("model.js");
        JSModel {
            model_js: model_js_bytes,
        }
    }

    fn model_js(&self) -> &'static [u8] {
        self.model_js
    }

    fn to_string(&self) -> String {
        if let Ok(model_js_string) = String::from_utf8(self.model_js.to_vec()) {
            model_js_string
        } else {
            String::from("js/model.js is not valid UTF-8.")
        }
    }
}
