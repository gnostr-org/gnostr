struct JSMempool {
    mempool_js: &'static [u8],
}

impl JSMempool {
    fn _new() -> Self {
        let mempool_js_bytes: &'static [u8] = include_bytes!("mempool.js");
        JSMempool {
            mempool_js: mempool_js_bytes,
        }
    }

    fn _mempool_js(&self) -> &'static [u8] {
        self.mempool_js
    }

    fn _to_string(&self) -> String {
        if let Ok(mempool_js_string) = String::from_utf8(self.mempool_js.to_vec()) {
            mempool_js_string
        } else {
            String::from("js/mempool.js is not valid UTF-8.")
        }
    }
}