use super::Res;
use crate::gitui::gitui_error::Error;
use ratatui::{backend::Backend, Terminal};
use std::borrow::Cow;
use tui_prompts::{State as _, TextState};

pub struct PromptData {
    pub prompt_text: Cow<'static, str>,
}

pub struct Prompt {
    pub data: Option<PromptData>,
    pub state: TextState<'static>,
}

impl Prompt {
    pub(crate) fn new() -> Self {
        Prompt {
            data: None,
            state: TextState::new(),
        }
    }

    pub(crate) fn set(&mut self, data: PromptData) {
        self.data = Some(data);
        self.state.focus();
    }

    pub(crate) fn reset<B: Backend>(&mut self, terminal: &mut Terminal<B>) -> Res<()> {
        self.data = None;
        self.state = TextState::new();
        terminal.hide_cursor().map_err(Error::Term)?;
        Ok(())
    }
}
