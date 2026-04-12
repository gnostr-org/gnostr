struct JSBuffer {
    buffer_js: &'static [u8],
}

impl JSBuffer {
    fn _new() -> Self {
        let buffer_js_bytes: &'static [u8] = include_bytes!("buffer@6.0.3.js");
        JSBuffer {
            buffer_js: buffer_js_bytes,
        }
    }

    fn _buffer_js(&self) -> &'static [u8] {
        self.buffer_js
    }

    fn _to_string(&self) -> String {
        if let Ok(buffer_js_string) = String::from_utf8(self.buffer_js.to_vec()) {
            buffer_js_string
        } else {
            String::from("js/buffer@6.0.3.js is not valid UTF-8.")
        }
    }
}
