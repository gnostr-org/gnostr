use std::io;

use crate::p2p::chat::msg;

pub struct App {
    pub topic: String,
    pub on_submit: Option<Box<dyn FnMut(msg::Msg) + Send>>,
}

impl Default for App {
    fn default() -> Self {
        Self {
            topic: String::from("gnostr"),
            on_submit: None,
        }
    }
}

impl App {
    pub fn on_submit<F: FnMut(msg::Msg) + Send + 'static>(&mut self, hook: F) {
        self.on_submit = Some(Box::new(hook));
    }

    pub fn add_msg_fn(&self) -> Box<dyn FnMut(msg::Msg) + Send + 'static> {
        Box::new(|_| {})
    }

    pub fn run(&mut self) -> io::Result<()> {
        Ok(())
    }
}
