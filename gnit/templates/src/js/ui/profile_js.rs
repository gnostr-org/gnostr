struct UIProfile {
    profile_js: &'static [u8],
}

impl UIProfile {
    fn new() -> Self {
        let profile_js_bytes: &'static [u8] = include_bytes!("profile.js");
        UIProfile {
            profile_js: profile_js_bytes,
        }
    }

    fn profile_js(&self) -> &'static [u8] {
        self.profile_js
    }

    fn to_string(&self) -> String {
        if let Ok(profile_js_string) = String::from_utf8(self.profile_js.to_vec()) {
            profile_js_string
        } else {
            String::from("js/ui/profile.js is not valid UTF-8.")
        }
    }
}
