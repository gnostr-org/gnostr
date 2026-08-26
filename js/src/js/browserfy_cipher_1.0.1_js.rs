struct JSBrowserifyCipher {
    browserify_cipher_js: &'static [u8],
}

impl JSBrowserifyCipher {
    fn _new() -> Self {
        let browserify_cipher_js_bytes: &'static [u8] = include_bytes!("browserify-cipher@1.0.1.js");
        JSBrowserifyCipher {
            browserify_cipher_js: browserify_cipher_js_bytes,
        }
    }

    fn _browserify_cipher_js(&self) -> &'static [u8] {
        self.browserify_cipher_js
    }

    fn _to_string(&self) -> String {
        if let Ok(browserify_cipher_js_string) = String::from_utf8(self.browserify_cipher_js.to_vec()) {
            browserify_cipher_js_string
        } else {
            String::from("js/browserify-cipher@1.0.1.js is not valid UTF-8.")
        }
    }
}
