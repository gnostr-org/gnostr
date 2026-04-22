struct JSBip39 {
    bip39_js: &'static [u8],
}

impl JSBip39 {
    fn _new() -> Self {
        let bip39_js_bytes: &'static [u8] = include_bytes!("bip39@3.0.4.js");
        JSBip39 {
            bip39_js: bip39_js_bytes,
        }
    }

    fn _bip39_js(&self) -> &'static [u8] {
        self.bip39_js
    }

    fn _to_string(&self) -> String {
        if let Ok(bip39_js_string) = String::from_utf8(self.bip39_js.to_vec()) {
            bip39_js_string
        } else {
            String::from("js/bip39@3.0.4.js is not valid UTF-8.")
        }
    }
}
