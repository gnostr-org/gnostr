struct UISettings {
    settings_js: &'static [u8],
}

impl UISettings {
    fn _new() -> Self {
        let settings_js_bytes: &'static [u8] = include_bytes!("settings.js");
        UISettings {
            settings_js: settings_js_bytes,
        }
    }

    fn _settings_js(&self) -> &'static [u8] {
        self.settings_js
    }

    fn _to_string(&self) -> String {
        if let Ok(settings_js_string) = String::from_utf8(self.settings_js.to_vec()) {
            settings_js_string
        } else {
            String::from("js/ui/settings.js is not valid UTF-8.")
        }
    }
}
