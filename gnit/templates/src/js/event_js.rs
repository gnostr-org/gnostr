struct JSEvent {
    event_js: &'static [u8],
}

impl JSEvent {
    fn new() -> Self {
        let event_js_bytes: &'static [u8] = include_bytes!("event.js");
        JSEvent {
            event_js: event_js_bytes,
        }
    }

    fn event_js(&self) -> &'static [u8] {
        self.event_js
    }

    fn to_string(&self) -> String {
        if let Ok(event_js_string) = String::from_utf8(self.event_js.to_vec()) {
            event_js_string
        } else {
            String::from("js/event.js is not valid UTF-8.")
        }
    }
}
