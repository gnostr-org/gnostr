use super::{Action, OpTrait};
use crate::{gitui::gitui_error::Error, gitui::items::TargetData, gitui::state::State};
use std::rc::Rc;

pub(crate) struct CopyHash;
impl OpTrait for CopyHash {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        match target {
            Some(TargetData::Commit(r)) => copy_hash(r.clone()),
            _ => None,
        }
    }
    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "Copy hash".into()
    }
}

fn copy_hash(r: String) -> Option<Action> {
    Some(Rc::new(move |state, _term| {
        state.close_menu();
        match &mut state.clipboard {
            Some(cb) => {
                cb.set_text(r.clone()).map_err(Error::Clipboard)?;
                state.display_info("Commit hash copied to clipboard");
            }
            None => state.display_error("Clipboard not available"),
        }
        Ok(())
    }))
}
