struct JSState {
    state_js: &'static [u8],
}

impl JSState {
    fn new() -> Self {
        let state_js_bytes: &'static [u8] = include_bytes!("state.js");
        JSState {
            state_js: state_js_bytes,
        }
    }

    fn state_js(&self) -> &'static [u8] {
        self.state_js
    }

    fn to_string(&self) -> String {
        if let Ok(state_js_string) = String::from_utf8(self.state_js.to_vec()) {
            state_js_string
        } else {
            String::from("js/state.js is not valid UTF-8.")
        }
    }
}
