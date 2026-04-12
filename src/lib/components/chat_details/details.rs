use std::{borrow::Cow, cell::Cell};

use anyhow::Result;
use crossterm::event::Event;
use gnostr_asyncgit::sync::{
    self, commit_files::OldNew, CommitDetails, CommitId, CommitMessage, RepoPathRef, Tag,
};
use log::debug;
use nostr_sdk_0_34_0::prelude::*;
use nostr_sqlite_0_34_0::{Error, SQLiteDatabase};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span, Text},
    Frame,
};
use sync::CommitTags;

use super::style::Detail;
use crate::{
    app::Environment,
    components::{
        chat_details::style::style_detail,
        dialog_paragraph,
        utils::{scroll_vertical::VerticalScroll, time_to_string},
        CommandBlocking, CommandInfo, Component, DrawableComponent, EventState, ScrollType,
    },
    keys::{key_match, SharedKeyConfig},
    strings::{self, order},
    ui::style::SharedTheme,
};

pub struct DetailsComponent {
    repo: RepoPathRef,
    data: Option<CommitDetails>,
    tags: Vec<Tag>,
    theme: SharedTheme,
    focused: bool,
    current_width: Cell<u16>,
    scroll: VerticalScroll,
    scroll_to_bottom_next_draw: Cell<bool>,
    key_config: SharedKeyConfig,
}

///type WrappedCommitMessage<'a>
type WrappedCommitMessage<'a> = (Vec<Cow<'a, str>>, Vec<Cow<'a, str>>);

impl DetailsComponent {
    pub fn new(env: &Environment, focused: bool) -> Self {
        Self {
            repo: env.repo.clone(),
            data: None,
            tags: Vec::new(),
            theme: env.theme.clone(),
            focused,
            scroll_to_bottom_next_draw: Cell::new(false),
            current_width: Cell::new(0),
            scroll: VerticalScroll::new(),
            key_config: env.key_config.clone(),
        }
    }

    pub fn set_commit(&mut self, id: Option<CommitId>, tags: Option<CommitTags>) {
        self.tags.clear();

        self.data = id.and_then(|id| sync::get_commit_details(&self.repo.borrow(), id).ok());

        self.scroll.reset();

        if let Some(tags) = tags {
            self.tags.extend(tags);
        }
    }

    fn wrap_commit_details(message: &CommitMessage, width: usize) -> WrappedCommitMessage<'_> {
        let width = width.max(1);
        //bwrap
        //bwrap
        //bwrap
        let wrapped_title = bwrap::wrap!(&message.subject, width)
            .lines()
            .map(String::from)
            .map(Cow::from)
            .collect();

        if let Some(ref body) = message.body {
            //bwrap
            //bwrap
            //bwrap
            let wrapped_message: Vec<Cow<'_, str>> = bwrap::wrap!(body, width)
                .lines()
                .map(String::from)
                .map(Cow::from)
                .collect();

            (wrapped_title, wrapped_message)
        } else {
            (wrapped_title, vec![])
        }
    }

    fn get_wrapped_lines(data: &Option<CommitDetails>, width: usize) -> WrappedCommitMessage<'_> {
        if let Some(ref data) = data {
            if let Some(ref message) = data.message {
                return Self::wrap_commit_details(message, width);
            }
        }

        (vec![], vec![])
    }

    fn get_number_of_lines(details: &Option<CommitDetails>, width: usize) -> usize {
        let (wrapped_title, wrapped_message) = Self::get_wrapped_lines(details, width);

        wrapped_title.len() + wrapped_message.len()
    }

    fn get_theme_for_line(&self, bold: bool) -> Style {
        if bold {
            self.theme.text(true, false).add_modifier(Modifier::BOLD)
        } else {
            self.theme.text(true, false)
        }
    }

    fn get_wrapped_text_message(&self, width: usize, height: usize) -> Vec<Line<'_>> {
        let (wrapped_title, wrapped_message) = Self::get_wrapped_lines(&self.data, width);

        [&wrapped_title[..], &wrapped_message[..]]
            .concat()
            .iter()
            .enumerate()
            .skip(self.scroll.get_top())
            .take(height)
            .map(|(i, line)| {
                Line::from(vec![Span::styled(
                    line.clone(),
                    self.get_theme_for_line(i < wrapped_title.len()),
                )])
            })
            .collect()
    }

    #[allow(unstable_name_collisions, clippy::too_many_lines)]
    fn get_text_info(&self) -> Vec<Line<'_>> {
        self.data.as_ref().map_or_else(Vec::new, |data| {
            let mut res = vec![
                //
                //commit formatting
                //insert commit here
                //adhere to git log formatting

                //EXAMPLE
                //commit 77aa531796ba0de324f928abd05c1f5314df9f74 (HEAD ->
                // WEEBLE/BLOCKHEIGHT/WOBBLE/PARENT/CHILD-additonal_string)
                //
                //Author: randymcmillan <randymcmillan@protonmail.com>
                //Date:   Sun Apr 20 21:37:43 2025 -0400
                //
                //    src/lib/components/topiclist.rs:commit keys
                //
                //    apply cargo fmt
                Line::from(vec![
                    Span::styled(
                        Cow::from(strings::commit::details_sha().to_string()),
                        self.theme.text(false, false),
                    ),
                    Span::styled(Cow::from(data.hash.clone()), self.theme.text(true, false)),
                ]),
                //
                Line::from(vec![
                    style_detail(&self.theme, &Detail::Author),
                    Span::styled(
                        Cow::from(format!(
                            //"174:chat_details/details.rs {} <{}>",
                            "{} <{}>",
                            //author.name/email
                            data.author.name,
                            data.author.email
                        )),
                        self.theme.text(true, false),
                    ),
                ]),
                Line::from(vec![
                    style_detail(&self.theme, &Detail::Date),
                    Span::styled(
                        Cow::from(time_to_string(data.author.time, false)),
                        self.theme.text(true, false),
                    ),
                ]),
            ];

            if let Some(ref committer) = data.committer {
                res.extend(vec![
                    Line::from(vec![
                        style_detail(&self.theme, &Detail::Date),
                        Span::styled(
                            Cow::from(time_to_string(committer.time, false).to_string()),
                            self.theme.text(true, false),
                        ),
                    ]),
                    Line::from(vec![
                        style_detail(&self.theme, &Detail::Committer),
                        Span::styled(
                            Cow::from(format!(
                                //"207:chat_details/details.rs:get_text_info {} <{}>",
                                //"207:{} <{}>",
                                "{} <{}>",
                                committer.name, committer.email
                            )),
                            self.theme.text(true, false),
                        ),
                    ]),
                ]);
            }

            //if !self.tags.is_empty() {
            //    res.push(Line::from(style_detail(&self.theme, &Detail::Sha)));

            //    res.push(Line::from(
            //        itertools::Itertools::intersperse(
            //            self.tags.iter().map(|tag| {
            //                Span::styled(Cow::from(&tag.name), self.theme.text(true,
            // false))            }),
            //            Span::styled(Cow::from(","), self.theme.text(true, false)),
            //        )
            //        .collect::<Vec<Span>>(),
            //    ));
            //}

            res
        })
    }

    //fn get_short_pubkey(&self) {}
    #[allow(clippy::needless_pass_by_ref_mut)]
    fn move_scroll_top(&mut self, move_type: ScrollType) -> bool {
        if self.data.is_some() {
            self.scroll.move_top(move_type)
        } else {
            false
        }
    }
}

impl DrawableComponent for DetailsComponent {
    //context notes
    //a pubkey has been selected from the topiclist
    //the detail popup has been presented
    //and now we are scrolling through the topiclist
    //viewing each detail while the topiclist is still visible
    fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
        const CANSCROLL_STRING: &str = "[\u{2026}]";
        const EMPTY_STRING: &str = "";

        let chunks = Layout::default()
            .direction(Direction::Vertical)
            //first in                            //second in
            .constraints([Constraint::Length(10), Constraint::Min(0)].as_ref())
            .split(rect);

        //TODO more nip-0034 git stuff
        //first in
        f.render_widget(
            dialog_paragraph(
                &strings::commit::details_info_title(&self.key_config),
                Text::from(self.get_text_info()),
                &self.theme,
                false,
            ),
            chunks[0],
        );

        // We have to take the border into account which is one
        // character on each side.
        let border_width: u16 = 2;

        let width = chunks[1].width.saturating_sub(border_width);
        let height = chunks[1].height.saturating_sub(border_width);

        self.current_width.set(width);

        let number_of_lines = Self::get_number_of_lines(&self.data, usize::from(width));

        self.scroll
            .update_no_selection(number_of_lines, usize::from(height));

        if self.scroll_to_bottom_next_draw.get() {
            self.scroll.move_top(ScrollType::End);
            self.scroll_to_bottom_next_draw.set(false);
        }

        let can_scroll = usize::from(height) < number_of_lines;

        f.render_widget(
            dialog_paragraph(
                &format!(
                    //"289:chat_details/details.rs:strings:commit:details_message_title:{} {}",
                    "305:{} {}",
                    //"{} {}",
                    strings::commit::details_message_title(&self.key_config,),
                    if !self.focused && can_scroll {
                        CANSCROLL_STRING
                    } else {
                        EMPTY_STRING
                    }
                ),
                Text::from(self.get_wrapped_text_message(width as usize, height as usize)),
                &self.theme,
                self.focused,
            ),
            chunks[1],
        );

        //self.focused
        if self.focused {
            self.scroll.draw(f, chunks[1], &self.theme);
        }

        Ok(())
    }
    //Files below this (in layout)
}

impl Component for DetailsComponent {
    fn commands(&self, out: &mut Vec<CommandInfo>, force_all: bool) -> CommandBlocking {
        let width = usize::from(self.current_width.get());
        let number_of_lines = Self::get_number_of_lines(&self.data, width);

        out.push(
            CommandInfo::new(
                strings::commands::navigate_commit_message(&self.key_config),
                number_of_lines > 0,
                self.focused || force_all,
            )
            .order(order::NAV),
        );

        CommandBlocking::PassingOn
    }

    fn event(&mut self, event: &Event) -> Result<EventState> {
        if self.focused {
            if let Event::Key(e) = event {
                return Ok(if key_match(e, self.key_config.keys.move_up) {
                    self.move_scroll_top(ScrollType::Up).into()
                } else if key_match(e, self.key_config.keys.move_down) {
                    self.move_scroll_top(ScrollType::Down).into()
                } else if key_match(e, self.key_config.keys.home)
                    || key_match(e, self.key_config.keys.shift_up)
                {
                    self.move_scroll_top(ScrollType::Home).into()
                } else if key_match(e, self.key_config.keys.end)
                    || key_match(e, self.key_config.keys.shift_down)
                {
                    self.move_scroll_top(ScrollType::End).into()
                } else {
                    EventState::NotConsumed
                });
            }
        }

        Ok(EventState::NotConsumed)
    }

    fn focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self, focus: bool) {
        if focus {
            self.scroll_to_bottom_next_draw.set(true);
        } else {
            self.scroll.reset();
        }

        self.focused = focus;
    }
}

pub struct CompareDetailsComponent {
    repo: RepoPathRef,
    data: Option<OldNew<CommitDetails>>,
    //_db: SQLiteDatabase,
    theme: SharedTheme,
    focused: bool,
}

impl CompareDetailsComponent {
    pub async fn new(env: &Environment, focused: bool) -> Self {
        Self {
            data: None,
            // The _db field is marked as unused, so we do not initialize it here.
            // The initialization code was removed.
            theme: env.theme.clone(),
            focused,
            repo: env.repo.clone(),
        }
    }

    #[allow(dead_code)]
    pub async fn create_directory_if_not_exists() -> Result<(), Box<dyn std::error::Error>> {
        //use async_std::fs;
        //use async_std::fs::File;
        use async_std::{
            io::ErrorKind,
            path::{Path, PathBuf},
        };
        let base_path = async_std::path::Path::new(".git");
        let filename = async_std::path::Path::new("nostr-cache.sqlite");
        let full_path: PathBuf = base_path.join(filename);
        println!("The full path is: {}", full_path.display());
        if Path::new(&full_path).is_file().await {
            println!("The directory '{}' exists.", full_path.display());
        } else {
            println!("The directory '{}' does not exist.", full_path.display());
        }
        match async_std::fs::File::create(full_path.clone()).await {
            Ok(_) => {
                println!("Successfully created directory: {}", full_path.display());
                Ok(())
            }
            Err(error) => {
                // Handle the error.  We're interested in the "already exists" case.
                if error.kind() == ErrorKind::AlreadyExists {
                    println!("Directory already exists: {}", full_path.display());
                    Ok(())
                } else {
                    use git2::{Repository, RepositoryInitOptions};
                    let _ = match Repository::discover(".") {
                        Ok(repo) => {
                            println!("Found existing git repository in {:?}", base_path);
                            repo
                        }
                        Err(_) => {
                            let mut opts = RepositoryInitOptions::new();
                            opts.initial_head("gnostr"); // Set the initial branch name
                            let repo = Repository::init_opts(base_path, &opts)?;
                            println!("Initialized new git repository in {:?}", base_path);
                            repo
                        }
                    };
                    Ok(())
                }
            }
        }
    }

    #[allow(dead_code)]
    pub async fn connect() -> Result<SQLiteDatabase, Error> {
        use async_std::path::{Path, PathBuf};
        let directory_to_check = ".git";
        if Path::new(directory_to_check).is_dir().await {
            debug!("The directory '{}' exists.", directory_to_check);
        } else {
            debug!("The directory '{}' does not exist.", directory_to_check);
            let _ = Self::create_directory_if_not_exists().await;
        }
        let base_path = async_std::path::Path::new(".git");
        let filename = async_std::path::Path::new("nostr-cache.sqlite");
        let full_path: PathBuf = base_path.join(filename);
        if Path::new(&full_path).is_file().await {
            debug!("The file '{}' exists.", full_path.display());
        } else {
            debug!("The file '{}' does not exist.", full_path.display());
        }
        debug!("The full path is: {}", full_path.display());
        let db = SQLiteDatabase::open(".git/nostr-cache.sqlite")
            .await
            .expect("");
        Ok(db)
    }

    pub fn set_commits(&mut self, ids: Option<OldNew<CommitId>>) {
        self.data = ids.and_then(|ids| {
            let old = sync::get_commit_details(&self.repo.borrow(), ids.old).ok()?;
            let new = sync::get_commit_details(&self.repo.borrow(), ids.new).ok()?;

            Some(OldNew { old, new })
        });
    }

    //commit header Info
    #[allow(unstable_name_collisions)]
    fn get_commit_text(&self, data: &CommitDetails) -> Vec<Line<'_>> {
        //TODO git commit log formatting
        let mut res = vec![
            Line::from(vec![
                style_detail(&self.theme, &Detail::Author),
                Span::styled(
                    Cow::from(format!(
                        "data.author.name {} data.author.email <{}>",
                        data.author.name, data.author.email
                    )),
                    self.theme.text(true, false),
                ),
            ]),
            Line::from(vec![
                style_detail(&self.theme, &Detail::Date),
                Span::styled(
                    Cow::from(time_to_string(data.author.time, false)),
                    self.theme.text(true, false),
                ),
            ]),
        ];

        //commit message box
        res.push(Line::from(vec![
            style_detail(&self.theme, &Detail::Message),
            Span::styled(
                Cow::from(format!(
                    "data.message {}",
                    data.message
                        .as_ref()
                        .map(|msg| msg.subject.clone())
                        .unwrap_or_default(),
                )),
                self.theme.text(true, false),
            ),
        ]));

        res
    }
}

impl DrawableComponent for CompareDetailsComponent {
    fn draw(&self, f: &mut Frame, rect: Rect) -> Result<()> {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(5), Constraint::Length(5)].as_ref())
            .split(rect);

        if let Some(data) = &self.data {
            f.render_widget(
                dialog_paragraph(
                    &strings::commit::compare_details_info_title(
                        true,
                        //Nostr PublicKey
                        &Keys::parse(data.old.padded_hash())
                            .unwrap()
                            .public_key()
                            //.to_bech32()?,
                            .to_string(),
                    ),
                    Text::from(format!(
                        "self.get_commit_text {:?}",
                        self.get_commit_text(&data.old)
                    )),
                    &self.theme,
                    false,
                ),
                chunks[0],
            );

            f.render_widget(
                dialog_paragraph(
                    &strings::commit::compare_details_info_title(
                        false,
                        &Keys::parse(data.new.padded_hash())
                            .unwrap()
                            .public_key()
                            //.to_bech32()?,
                            .to_string(),
                    ),
                    Text::from(format!(
                        "self.get_commit_text {:?}",
                        self.get_commit_text(&data.new)
                    )),
                    &self.theme,
                    false,
                ),
                chunks[1],
            );
        }

        Ok(())
    }
}

impl Component for CompareDetailsComponent {
    fn commands(&self, _out: &mut Vec<CommandInfo>, _force_all: bool) -> CommandBlocking {
        CommandBlocking::PassingOn
    }

    fn event(&mut self, _event: &Event) -> Result<EventState> {
        Ok(EventState::NotConsumed)
    }

    fn focused(&self) -> bool {
        self.focused
    }

    fn focus(&mut self, focus: bool) {
        self.focused = focus;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_wrapped_lines(message: &CommitMessage, width: usize) -> Vec<Cow<'_, str>> {
        let (wrapped_title, wrapped_message) =
            DetailsComponent::wrap_commit_details(message, width);

        [&wrapped_title[..], &wrapped_message[..]].concat()
    }

    #[test]
    fn test_textwrap() {
        let message = CommitMessage::from("Commit message");

        assert_eq!(get_wrapped_lines(&message, 7), vec!["Commit", "message"]);
        assert_eq!(get_wrapped_lines(&message, 14), vec!["Commit message"]);
        assert_eq!(get_wrapped_lines(&message, 0), vec!["Commit", "message"]);

        let message_with_newline = CommitMessage::from("Commit message\n");

        assert_eq!(
            get_wrapped_lines(&message_with_newline, 7),
            vec!["Commit", "message"]
        );
        assert_eq!(
            get_wrapped_lines(&message_with_newline, 14),
            vec!["Commit message"]
        );
        assert_eq!(get_wrapped_lines(&message, 0), vec!["Commit", "message"]);

        let message_with_body = CommitMessage::from("Commit message\nFirst line\nSecond line");

        assert_eq!(
            get_wrapped_lines(&message_with_body, 7),
            vec!["Commit", "message", "First", "line", "Second", "line"]
        );
        assert_eq!(
            get_wrapped_lines(&message_with_body, 14),
            vec!["Commit message", "First line", "Second line"]
        );
        assert_eq!(
            get_wrapped_lines(&message_with_body, 7),
            vec!["Commit", "message", "First", "line", "Second", "line"]
        );
    }
}

#[cfg(test)]
mod test_line_count {
    use super::*;

    #[test]
    fn test_smoke() {
        let commit = CommitDetails {
            message: Some(CommitMessage {
                subject: String::from("subject line"),
                body: Some(String::from("body lone")),
            }),
            ..CommitDetails::default()
        };
        let lines = DetailsComponent::get_number_of_lines(&Some(commit.clone()), 50);
        assert_eq!(lines, 2);

        let lines = DetailsComponent::get_number_of_lines(&Some(commit), 8);
        assert_eq!(lines, 4);
    }
}
