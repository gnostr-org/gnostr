struct JSNostr {
    nostr_js: &'static [u8],
}

impl JSNostr {
    fn _new() -> Self {
        let nostr_js_bytes: &'static [u8] = include_bytes!("nostr.js");
        JSNostr {
            nostr_js: nostr_js_bytes,
        }
    }

    fn _nostr_js(&self) -> &'static [u8] {
        self.nostr_js
    }

    fn _to_string(&self) -> String {
        if let Ok(nostr_js_string) = String::from_utf8(self.nostr_js.to_vec()) {
            nostr_js_string
        } else {
            String::from("js/nostr.js is not valid UTF-8.")
        }
    }
}
