struct UIRender {
    render_js: &'static [u8],
}

impl UIRender {
    fn new() -> Self {
        let render_js_bytes: &'static [u8] = include_bytes!("render.js");
        UIRender {
            render_js: render_js_bytes,
        }
    }

    fn render_js(&self) -> &'static [u8] {
        self.render_js
    }

    fn to_string(&self) -> String {
        if let Ok(render_js_string) = String::from_utf8(self.render_js.to_vec()) {
            render_js_string
        } else {
            String::from("js/ui/render.js is not valid UTF-8.")
        }
    }
}
