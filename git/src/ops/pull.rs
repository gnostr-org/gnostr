use std::process::Command;
use std::rc::Rc;

use derive_more::Display;

use super::{Action, OpTrait};
use crate::items::TargetData;
use crate::state::State;
use crate::term::Term;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Pull")]
pub(crate) struct Pull;
impl OpTrait for Pull {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["pull"]);

            state.run_external_cmd(term, &[], cmd)?;
            Ok(())
        }))
    }
}
