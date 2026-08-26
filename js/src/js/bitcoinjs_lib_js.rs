struct JSBitcoinjsLib {
    bitcoinjs_lib_js: &'static [u8],
}

impl JSBitcoinjsLib {
    fn _new() -> Self {
        let bitcoinjs_lib_js_bytes: &'static [u8] = include_bytes!("bitcoinjs-lib.js");
        JSBitcoinjsLib {
            bitcoinjs_lib_js: bitcoinjs_lib_js_bytes,
        }
    }

    fn _bitcoinjs_lib_js(&self) -> &'static [u8] {
        self.bitcoinjs_lib_js
    }

    fn _to_string(&self) -> String {
        if let Ok(bitcoinjs_lib_js_string) = String::from_utf8(self.bitcoinjs_lib_js.to_vec()) {
            bitcoinjs_lib_js_string
        } else {
            String::from("js/bitcoinjs-lib.js is not valid UTF-8.")
        }
    }
}
