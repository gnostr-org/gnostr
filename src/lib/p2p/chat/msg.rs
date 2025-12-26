use git2::Oid;
use gnostr_asyncgit::sync::CommitId;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use textwrap::{fill, Options};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

const CHAT_MSG_MAX_WIDTH: usize = 80;

fn gen_color_by_hash(s: &str) -> Color {
    static LIGHT_COLORS: [Color; 5] = [
        Color::LightMagenta,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightCyan,
        // Color::White,
    ];
    let h = s.bytes().fold(0, |acc, b| acc ^ b as usize);
    LIGHT_COLORS[h % LIGHT_COLORS.len()]
}


pub(crate) static USER_NAME: Lazy<String> = Lazy::new(|| {
    std::env::var("USER")
        .unwrap_or_else(|_| hostname::get().unwrap().to_string_lossy().to_string())
        .to_string()
});
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default, PartialEq)]
pub enum MsgKind {
    #[default]
    Chat,
    Join,
    Leave,
    System,
    Raw,
    Command,
    GitCommitId,
    GitCommitParent,
    GitCommitTree,
    GitCommitAuthor,
    GitCommitName,
    GitCommitEmail,
    GitCommitMessagePart,
    GitCommitHeader,
    GitCommitBody,
    GitCommitTime,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Msg {
    pub from: String,
    pub content: Vec<String>,
    pub kind: MsgKind,
    pub commit_id: CommitId,
}

impl Default for Msg {
    fn default() -> Self {
        Self {
            from: USER_NAME.clone(),
            content: vec!["".to_string(), "".to_string()],
            kind: MsgKind::Chat,
            commit_id: CommitId::new(Oid::zero()),
        }
    }
}

impl Msg {
    pub fn set_kind(mut self, kind: MsgKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn set_content(mut self, content: String, _int: usize) -> Self {
        self.content[0] = content;
        self
    }

    pub fn set_commit_id(mut self, commit_id: CommitId) -> Self {
        self.commit_id = commit_id;
        self
    }

    pub fn wrap_text(self, _text: Msg, _max_width: usize) -> Self {
        //	for line in text.content.bytes() {

        //    line
        //        .flat_map(|line| {
        //            line.chars()
        //                .collect::<Vec<char>>()
        //                .chunks(max_width)
        //                .map(|chunk| chunk.iter().collect::<String>())
        //                .collect::<Vec<String>>()
        //        })
        //        .collect()
        //}
        //	//return line

        self
    }

    pub fn to_lines(&self) -> Vec<Line<'_>> {
        use MsgKind::*;

        match self.kind {
            //System
            System => {
                vec![Line::from(Span::styled(
                    self.to_string(),
                    Style::default()
                        .fg(Color::Red)
                        .add_modifier(Modifier::ITALIC),
                ))]
            }
            //Join
            Join => {
                vec![Line::from(Span::styled(
                    self.to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::ITALIC),
                ))]
            }
            //Leave
            Leave => {
                vec![Line::from(Span::styled(
                    self.to_string(),
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::ITALIC),
                ))]
            }
            Chat => {
                let wrapped_content = fill(&self.content[0], Options::new(CHAT_MSG_MAX_WIDTH));
                let mut lines = Vec::new();

                if self.from == *USER_NAME {
                    // For outgoing messages, align left, sender then content
                    for (i, line) in wrapped_content.lines().enumerate() {
                        if i == 0 {
                            // First line includes the sender
                            lines.push(
                                Line::default().left_aligned().spans(vec![
                                    Span::styled(
                                        format!("{}{} ", &self.from, ">"),
                                        Style::default().fg(gen_color_by_hash(&self.from)),
                                    ),
                                    line.to_string().into(),
                                ]),
                            );
                        } else {
                            // Subsequent lines are just wrapped content, indented
                            lines.push(Line::default().left_aligned().spans(vec![
                                Span::raw(" ".repeat(self.from.len() + 2)), // Indent to align with first line's content
                                line.to_string().into(),
                            ]));
                        }
                    }
                } else {
                    // For incoming messages, align right, content then sender
                    for (i, line) in wrapped_content.lines().enumerate() {
                        if i == 0 {
                            // First line includes the sender
                            lines.push(
                                Line::default().right_aligned().spans(vec![
                                    line.to_string().into(),
                                    Span::styled(
                                        format!(" {}{}", "<", &self.from),
                                        Style::default().fg(gen_color_by_hash(&self.from)),
                                    ),
                                ]),
                            );
                        } else {
                            // Subsequent lines are just wrapped content, indented
                            lines.push(Line::default().right_aligned().spans(vec![
                                line.to_string().into(),
                                Span::raw(" ".repeat(self.from.len() + 2)), // Indent to align with first line's content
                            ]));
                        }
                    }
                }
                lines
            }
            Raw => vec![Line::from(self.content[0].clone())],
            Command => {
                vec![Line::default().spans(vec![
                    Span::styled(
                        format!("Command: {}{} ", &self.from, ">"),
                        Style::default()
                            .fg(gen_color_by_hash(&self.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    self.content[0].clone().into(),
                ])]
            }
            GitCommitId
            | GitCommitTree
            | GitCommitAuthor
            | GitCommitParent
            | GitCommitMessagePart
            | GitCommitName
            | GitCommitEmail
            | GitCommitTime
            | GitCommitHeader
            | GitCommitBody => {
                vec![Line::default().spans(vec![
                    Span::styled(
                        self.to_string(),
                        Style::default()
                            .fg(gen_color_by_hash(&self.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                ])]
            }
        }
    }
}

impl Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            MsgKind::Join => write!(f, "{} joined!", self.from),
            MsgKind::Leave => write!(f, "{} left!", self.from),
            MsgKind::Chat => {
                let wrapped_content = fill(&self.content[0], Options::new(CHAT_MSG_MAX_WIDTH));
                write!(f, "{}: {}", self.from, wrapped_content)
            }
            MsgKind::System => write!(f, "[System] {}", self.content[0]),
            MsgKind::Raw => write!(f, "{}", self.content[0]),
            MsgKind::Command => write!(f, "[Command] {}:{}", self.from, self.content[0]),
            MsgKind::GitCommitId => {
                write!(
                    f,
                    "{{\"commit\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitTree => {
                write!(
                    f,
                    "{{\"tree\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitParent => {
                write!(
                    f,
                    "{{\"parent\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitHeader => {
                write!(
                    f,
                    "{{\"header\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitAuthor => {
                write!(
                    f,
                    "{{\"Author\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitEmail => {
                write!(
                    f,
                    "{{\"email\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitName => {
                write!(
                    f,
                    "{{\"name\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitBody => {
                write!(
                    f,
                    "{{\"body\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitMessagePart => {
                write!(
                    f,
                    "{{\"msg\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
            MsgKind::GitCommitTime => {
                write!(
                    f,
                    "{{\"time\": \"{}\"}} {}",
                    self.content[0], self.content[1]
                )
            }
        }
    }
}
