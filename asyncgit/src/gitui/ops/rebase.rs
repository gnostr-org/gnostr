use super::{selected_rev, Action, OpTrait};
use crate::{
    gitui::items::TargetData,
    gitui::menu::arg::Arg,
    gitui::state::{PromptParams, State},
    gitui::term::Term,
    gitui::Res,
};
use std::{
    ffi::{OsStr, OsString},
    process::Command,
    rc::Rc,
};

pub(crate) fn init_args() -> Vec<Arg> {
    vec![
        Arg::new_flag("--keep-empty", "Keep empty commits", false),
        Arg::new_flag("--preserve-merges", "Preserve merges", false),
        Arg::new_flag(
            "--committer-date-is-author-date",
            "Lie about committer date",
            false,
        ),
        Arg::new_flag("--autosquash", "Autosquash", false),
        Arg::new_flag("--autostash", "Autostash", true),
        Arg::new_flag("--interactive", "Interactive", false),
        Arg::new_flag("--no-verify", "Disable hooks", false),
    ]
}

pub(crate) struct RebaseContinue;
impl OpTrait for RebaseContinue {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["rebase", "--continue"]);

            state.close_menu();
            state.run_cmd_interactive(term, cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "continue".into()
    }
}

pub(crate) struct RebaseAbort;
impl OpTrait for RebaseAbort {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(|state: &mut State, term: &mut Term| {
            let mut cmd = Command::new("git");
            cmd.args(["rebase", "--abort"]);

            state.close_menu();
            state.run_cmd(term, &[], cmd)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "abort".into()
    }
}

pub(crate) struct RebaseElsewhere;
impl OpTrait for RebaseElsewhere {
    fn get_action(&self, _target: Option<&TargetData>) -> Option<Action> {
        Some(Rc::new(move |state: &mut State, term: &mut Term| {
            let rev = state.prompt(
                term,
                &PromptParams {
                    prompt: "Rebase onto",
                    create_default_value: Box::new(selected_rev),
                    ..Default::default()
                },
            )?;

            rebase_elsewhere(state, term, &rev)?;
            Ok(())
        }))
    }

    fn display(&self, _state: &State) -> String {
        "onto elsewhere".into()
    }
}

fn rebase_elsewhere(state: &mut State, term: &mut Term, rev: &str) -> Res<()> {
    let mut cmd = Command::new("git");
    cmd.arg("rebase");
    cmd.args(state.pending_menu.as_ref().unwrap().args());
    cmd.arg(rev);

    state.close_menu();
    state.run_cmd_interactive(term, cmd)?;
    Ok(())
}

pub(crate) struct RebaseInteractive;
impl OpTrait for RebaseInteractive {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                let rev = OsString::from(r);
                Rc::new(move |state: &mut State, term: &mut Term| {
                    let args = state.pending_menu.as_ref().unwrap().args();
                    state.close_menu();
                    state.run_cmd_interactive(term, rebase_interactive_cmd(&args, &rev))
                })
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "interactively".into()
    }
}

fn rebase_interactive_cmd(args: &[OsString], rev: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["rebase", "-i"]);
    cmd.args(args);
    cmd.arg(parent(rev));
    cmd
}

fn parent(reference: &OsStr) -> OsString {
    let mut parent = reference.to_os_string();
    parent.push("^");
    parent
}

pub(crate) struct RebaseAutosquash;
impl OpTrait for RebaseAutosquash {
    fn get_action(&self, target: Option<&TargetData>) -> Option<Action> {
        let action = match target {
            Some(TargetData::Commit(r) | TargetData::Branch(r)) => {
                let rev = OsString::from(r);
                Rc::new(move |state: &mut State, term: &mut Term| {
                    let args = state.pending_menu.as_ref().unwrap().args();
                    state.close_menu();
                    state.run_cmd_interactive(term, rebase_autosquash_cmd(&args, &rev))
                })
            }
            _ => return None,
        };

        Some(action)
    }
    fn is_target_op(&self) -> bool {
        true
    }

    fn display(&self, _state: &State) -> String {
        "autosquash".into()
    }
}

fn rebase_autosquash_cmd(args: &[OsString], rev: &OsStr) -> Command {
    let mut cmd = Command::new("git");
    cmd.args(["rebase", "-i", "--autosquash", "--keep-empty"]);
    cmd.args(args);
    cmd.arg(rev);
    cmd
}
