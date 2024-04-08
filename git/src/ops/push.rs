use std::process::Command;
use std::rc::Rc;

use derive_more::Display;

use super::{Action, OpTrait};
use crate::items::TargetData;
use crate::state::State;
use crate::term::Term;

#[derive(Default, Clone, Copy, PartialEq, Eq, Debug, Display)]
#[display(fmt = "Push")]
pub(crate) struct Push;
impl OpTrait for Push {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["push"]);

            state.run_external_cmd(term, &[], cmd)?;
            Ok(())
        }))
    }
}
