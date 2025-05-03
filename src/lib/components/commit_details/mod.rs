mod compare_details;
mod details;
mod style;

use anyhow::Result;
use gnostr_asyncgit::{
    sync::{commit_files::OldNew, CommitTags},
    AsyncCommitFiles, CommitFilesParams,
};
use compare_details::CompareDetailsComponent;
use crossterm::event::Event;
use details::DetailsComponent;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    Frame,
};

use super::{
    command_pump, event_pump, CommandBlocking, CommandInfo, Component, DrawableComponent,
    EventState, StatusTreeComponent,
};
use crate::{
    accessors,
    app::Environment,
    keys::{key_match, SharedKeyConfig},
    strings,
    ui::style::SharedTheme,
};

use crate::components::dialog_paragraph;
use crate::components::Text;

pub struct CommitDetailsComponent {
    commit: Option<CommitFilesParams>,
    single_details: DetailsComponent,
    compare_details: CompareDetailsComponent,
    file_tree: StatusTreeComponent,
    chat: StatusTreeComponent,
    git_commit_files: AsyncCommitFiles,
    visible: bool,
    key_config: SharedKeyConfig,
    theme: SharedTheme,
    focused: bool,
}

impl CommitDetailsComponent {
    accessors!(self, [single_details, compare_details, file_tree]);

    ///
    pub fn new(env: &Environment, focused: bool) -> Self {
        Self {
            single_details: DetailsComponent::new(env, false),
            compare_details: CompareDetailsComponent::new(env, false),
            git_commit_files: AsyncCommitFiles::new(env.repo.borrow().clone(), &env.sender_git),
            file_tree: StatusTreeComponent::new(env, "", false),
            chat: StatusTreeComponent::new(env, "", false),
            visible: false,
            commit: None,
            key_config: env.key_config.clone(),
            theme: env.theme.clone(),
            focused,
        }
    }

    fn get_files_title(&self) -> String {
        let files_count = self.file_tree.file_count();

        format!(
            "69:commit_details/mod.rs CommitDetailsComponent {} {}",
            strings::commit::details_files_title(&self.key_config),
            files_count
        )
    }

    ///
    pub fn set_commits(
        &mut self,
        params: Option<CommitFilesParams>,
        tags: &Option<CommitTags>,
    ) -> Result<()> {
        if params.is_none() {
            self.single_details.set_commit(None, None);
            self.compare_details.set_commits(None);
        }

        self.commit = params;

        if let Some(id) = params {
            self.file_tree.set_commit(Some(id.id));

            if let Some(other) = id.other {
                self.compare_details.set_commits(Some(OldNew {
                    new: id.id,
                    old: other,
                }));
            } else {
                self.single_details.set_commit(Some(id.id), tags.clone());
            }

            if let Some((fetched_id, res)) = self.git_commit_files.current()? {
                if fetched_id == id {
                    self.file_tree.update(res.as_slice())?;
                    self.file_tree.set_title(self.get_files_title());

                    return Ok(());
                }
            }

            self.file_tree.clear()?;
            self.git_commit_files.fetch(id)?;
        }

        self.file_tree.set_title(self.get_files_title());

        Ok(())
    }

    ///
    pub fn any_work_pending(&self) -> bool {
        self.git_commit_files.is_pending()
    }

    ///
    pub const fn files(&self) -> &StatusTreeComponent {
        &self.file_tree
    }

    fn details_focused(&self) -> bool {
        self.single_details.focused() || self.compare_details.focused()
    }

    fn set_details_focus(&mut self, focus: bool) {
        if self.is_compare() {
            self.compare_details.focus(focus);
        } else {
            self.single_details.focus(focus);
        }
    }

    fn is_compare(&self) -> bool {
        self.commit.is_some_and(|p| p.other.is_some())
    }
}

impl DrawableComponent for CommitDetailsComponent {
    fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
        if !self.visible {
            return Ok(());
        }

        let vertical_constraints = if self.is_compare() {
            [
                Constraint::Length(10),
                Constraint::Min(0),
                Constraint::Min(0),
            ]
        } else {
            let details_focused = self.details_focused();
            let percentages = if self.file_tree.focused() {
                //file_tree refers to a File: widget that indicated
                //which files are part of the commit
                //once arrow right from topiclist or revlog
                //
                //Info
                //Message (scrollable) TODO fix toggle
                //
                //filetree
                //
                (30, 60, 20) //commit Info should remain visible
            } else if details_focused {
                //topiclist or revlog split
                (80, 10, 10) //commit Info and Message visible
                             //filetree obfuscated
            } else {
                //topiclist or revlog toggle split
                //Info AND
                //Message 50%
                //
                //filetree 50%
                (80, 10, 10)
            };

            [
                Constraint::Percentage(percentages.0),
                Constraint::Percentage(percentages.1),
                Constraint::Percentage(percentages.2),
            ]
        };

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints.as_ref())
            .split(rect);

        //notice the diff isnt rendered here
        //this renders the left side of the
        //commit inspection
        if self.is_compare() {
            self.compare_details.draw(f, chunks[0])?;
        } else {
            self.single_details.draw(f, chunks[0])?;
        }
        self.file_tree.draw(f, chunks[1])?;

        //this is rendered below the file_tree
        f.render_widget(
            dialog_paragraph(
                &format!(
                    "209:commit_details/mod.rs:chat widget!!!!! {} {} w:{},h:{}",
                    strings::commit::details_message_title(&self.key_config,),
                    strings::commit::details_message_title(&self.key_config,),
                    chunks[2].width,
                    chunks[2].height,
                    //if !self.focused && can_scroll {
                    //    CANSCROLL_STRING
                    //} else {
                    //    EMPTY_STRING
                    //}
                ),
                //Text::from(self.get_wrapped_text_message(width as usize, height as usize)),
                Text::from(""),
                &self.theme,
                self.focused,
            ),
            chunks[2],
        );

        //render p2p chat
        Ok(())
    }
}

impl Component for CommitDetailsComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        if self.visible || force_all {
            command_pump(out, force_all, self.components().as_slice());
        }

        CommandBlocking::PassingOn
    }

    fn event(&mut self, ev: &Event) -> Result<EventState> {
        if event_pump(ev, self.components_mut().as_mut_slice())?.is_consumed() {
            if !self.file_tree.is_visible() {
                self.hide();
            }

            return Ok(EventState::Consumed);
        }

        if self.focused() {
            if let Event::Key(e) = ev {
                return if key_match(e, self.key_config.keys.move_down) && self.details_focused() {
                    self.set_details_focus(false);
                    self.file_tree.focus(true);
                    Ok(EventState::Consumed)
                } else if key_match(e, self.key_config.keys.move_up)
                    && self.file_tree.focused()
                    && !self.is_compare()
                {
                    self.file_tree.focus(false);
                    self.set_details_focus(true);
                    Ok(EventState::Consumed)
                } else {
                    Ok(EventState::NotConsumed)
                };
            }
        }

        Ok(EventState::NotConsumed)
    }

    fn is_visible(&self) -> bool {
        self.visible
    }
    fn hide(&mut self) {
        self.visible = false;
    }
    fn show(&mut self) -> Result<()> {
        self.visible = true;
        self.file_tree.show()?;
        Ok(())
    }

    fn focused(&self) -> bool {
        self.details_focused() || self.file_tree.focused()
    }

    fn focus(&mut self, focus: bool) {
        self.single_details.focus(false);
        self.compare_details.focus(false);
        self.file_tree.focus(focus);
        self.file_tree.show_selection(true);
    }
}
