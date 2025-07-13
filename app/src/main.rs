//use gnostr::get_blockheight;
//use gnostr::gnostr::*;
use gpui::*;

struct GnostrApp {
    text: SharedString,
}

impl GnostrApp {
    fn get_blockheight(&self) -> String {
        gnostr::get_blockheight().expect("REASON")
    }
}
impl Render for GnostrApp {
    fn render(&mut self, _window: &mut Window, _cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .bg(rgb(0x2e7d32))
            .size_full()
            .justify_center()
            .items_center()
            .text_xl()
            .text_color(rgb(0xffffff))
            .child(format!(
                "{}:{}!",
                &self.text,
                GnostrApp::get_blockheight(&self)
            ))
    }
}

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.open_window(WindowOptions::default(), |_, cx| {
            cx.new(|_cx| GnostrApp {
                text: "blockheight".into(),
            })
        })
        .unwrap();
    });
}
