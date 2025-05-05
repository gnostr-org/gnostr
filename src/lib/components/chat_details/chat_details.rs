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

    pub async fn create_directory_if_not_exists(
        dir_path_str: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Convert the directory path string to a Path.  Crucial for cross-platform compatibility
        use async_std::path::Path;
        let dir_path = Path::new(dir_path_str);

        // Attempt to create the directory.
        use async_std::fs;
        use async_std::io::ErrorKind;
        match fs::create_dir_all(dir_path).await {
            Ok(_) => {
                println!("Successfully created directory: {}", dir_path.display());
                Ok(()) // Return Ok(()) to indicate success
            }
            Err(error) => {
                // Handle the error.  We're interested in the "already exists" case.
                use tokio::fs;
                if error.kind() == ErrorKind::AlreadyExists {
                    println!("Directory already exists: {}", dir_path.display());
                    Ok(()) // Treat "already exists" as success.
                } else {
                    // For other errors, return the error.
                    Err(Box::new(error)) // Box the error for easier handling
                }
            }
        }
    }

    pub async fn connect() -> Result<SQLiteDatabase, Error> {
        let directory_to_create = ".git/nostr-cache.sqlite";

        // Call the function to create the directory
        if let Err(e) = Self::create_directory_if_not_exists(directory_to_create).await {
            eprintln!(
                "Failed to create directory: {} - {}",
                directory_to_create, e
            );
            //  IMPORTANT:  Consider how your application should handle this error.
            //  Should it panic?  Should it try a different directory?
            //  For this example, we just print an error and exit.  You might
            //  choose to continue, depending on your application's logic.
            std::process::exit(1); // Exit with a non-zero exit code to signal failure
        }

        println!("Directory creation process completed.");
        //The directory now exists
        let directory_to_check = ".git";
        use async_std::path::Path;
        if Path::new(directory_to_check).is_dir().await {
            println!("The directory '{}' exists.", directory_to_check);
        } else {
            println!("The directory '{}' does not exist.", directory_to_check);
        }

        let db_future = SQLiteDatabase::open(".git/nostr-cache.sqlite");
        let db: SQLiteDatabase = db_future.await.expect("");
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
