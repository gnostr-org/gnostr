struct JSBip32 {
    bip32_js: &'static [u8],
}

impl JSBip32 {
    fn _new() -> Self {
        let bip32_js_bytes: &'static [u8] = include_bytes!("bip32@2.0.6.js");
        JSBip32 {
            bip32_js: bip32_js_bytes,
        }
    }

    fn _bip32_js(&self) -> &'static [u8] {
        self.bip32_js
    }

    fn _to_string(&self) -> String {
        if let Ok(bip32_js_string) = String::from_utf8(self.bip32_js.to_vec()) {
            bip32_js_string
        } else {
            String::from("js/bip32@2.0.6.js is not valid UTF-8.")
        }
    }
}
