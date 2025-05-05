use crate::{
    app::Environment,
    components::{
        chat_details::style::{style_detail, Detail},
        dialog_paragraph,
        utils::time_to_string,
        CommandBlocking, CommandInfo, Component, DrawableComponent, EventState,
    },
    strings::{self},
    ui::style::SharedTheme,
};
use anyhow::Result;
use crossterm::event::Event;
use gnostr_asyncgit::sync::{self, commit_files::OldNew, CommitDetails, CommitId, RepoPathRef};
use nostr_sdk::prelude::*;
use nostr_sqlite::SQLiteDatabase;
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    text::{Line, Span, Text},
    Frame,
};
use std::borrow::Cow;
use tracing::debug;

pub struct CompareDetailsComponent {
    repo: RepoPathRef,
    data: Option<OldNew<CommitDetails>>,
    db: SQLiteDatabase,
    theme: SharedTheme,
    focused: bool,
}

impl CompareDetailsComponent {
    ///
    pub async fn new(env: &Environment, focused: bool) -> Self {
        Self {
            data: None,
            db: Self::connect()
                .await
                .expect("Failed to connect to database"),
            theme: env.theme.clone(),
            focused,
            repo: env.repo.clone(),
        }
    }

    pub async fn create_directory_if_not_exists() -> Result<(), Box<dyn std::error::Error>> {
        //use async_std::fs;
        //use async_std::fs::File;
        use async_std::io::ErrorKind;
        use async_std::path::{Path, PathBuf};
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
                Ok(()) // Return Ok(()) to indicate success
            }
            Err(error) => {
                // Handle the error.  We're interested in the "already exists" case.
                if error.kind() == ErrorKind::AlreadyExists {
                    println!("Directory already exists: {}", full_path.display());
                    Ok(())
                } else {
                    use git2::{Repository, RepositoryInitOptions};
                    let _ = match Repository::discover(&".") {
                        Ok(repo) => {
                            println!("Found existing git repository in {:?}", base_path);
                            repo
                        }
                        Err(_) => {
                            let mut opts = RepositoryInitOptions::new();
                            opts.initial_head("gnostr"); // Set the initial branch name
                            let repo = Repository::init_opts(&base_path, &opts)?;
                            println!("Initialized new git repository in {:?}", base_path);
                            repo
                        }
                    };
                    Ok(())
                }
            }
        }
    }

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
    fn get_commit_text(&self, data: &CommitDetails) -> Vec<Line> {
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
                        &Keys::parse(&data.old.padded_hash())
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
                        &Keys::parse(&data.new.padded_hash())
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
