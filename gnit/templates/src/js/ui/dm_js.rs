struct UIDM {
    dm_js: &'static [u8],
}

impl UIDM {
    fn new() -> Self {
        let dm_js_bytes: &'static [u8] = include_bytes!("dm.js");
        UIDM { dm_js: dm_js_bytes }
    }

    fn dm_js(&self) -> &'static [u8] {
        self.dm_js
    }

    fn to_string(&self) -> String {
        if let Ok(dm_js_string) = String::from_utf8(self.dm_js.to_vec()) {
            dm_js_string
        } else {
            String::from("js/ui/dm.js is not valid UTF-8.")
        }
    }
}
