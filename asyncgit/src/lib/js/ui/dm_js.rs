struct UIDM {
    dm_js: &'static [u8],
}

impl UIDM {
    fn _new() -> Self {
        let dm_js_bytes: &'static [u8] = include_bytes!("dm.js");
        UIDM { dm_js: dm_js_bytes }
    }

    fn _dm_js(&self) -> &'static [u8] {
        self.dm_js
    }

    fn _to_string(&self) -> String {
        if let Ok(dm_js_string) = String::from_utf8(self.dm_js.to_vec()) {
            dm_js_string
        } else {
            String::from("js/ui/dm.js is not valid UTF-8.")
        }
    }
}
