struct JSNobleSecp256k1 {
    noble_secp256k1_js: &'static [u8],
}

impl JSNobleSecp256k1 {
    fn _new() -> Self {
        let noble_secp256k1_js_bytes: &'static [u8] = include_bytes!("noble-secp256k1@1.2.14.js");
        JSNobleSecp256k1 {
            noble_secp256k1_js: noble_secp256k1_js_bytes,
        }
    }

    fn _noble_secp256k1_js(&self) -> &'static [u8] {
        self.noble_secp256k1_js
    }

    fn _to_string(&self) -> String {
        if let Ok(noble_secp256k1_js_string) = String::from_utf8(self.noble_secp256k1_js.to_vec()) {
            noble_secp256k1_js_string
        } else {
            String::from("js/noble-secp256k1@1.2.14.js is not valid UTF-8.")
        }
    }
}
