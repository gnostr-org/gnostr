use crate::types::Event;
use git2::Oid;
use gnostr_asyncgit::sync::CommitId;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use textwrap::{Options, wrap}; // Add this line

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
    NostrEvent,
    GitDiff,
    OneShot,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Msg {
    pub from: String,
    pub content: Vec<String>,
    pub kind: MsgKind,
    pub commit_id: CommitId,
    pub nostr_event: Option<Event>,
    pub message_id: Option<String>,    // New field
    pub sequence_num: Option<usize>,   // New field
    pub total_chunks: Option<usize>,   // New field
}

impl Default for Msg {
    fn default() -> Self {
        Self {
            from: USER_NAME.clone(),
            content: vec!["".to_string(), "".to_string()],
            kind: MsgKind::Chat,
            commit_id: CommitId::new(Oid::zero()),
            nostr_event: None,
            message_id: None,
            sequence_num: None,
            total_chunks: None,
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

    pub fn set_nostr_event(mut self, event: Event) -> Self {
        self.kind = MsgKind::NostrEvent;
        self.content = vec![serde_json::to_string(&event).unwrap_or_default()];
        self.nostr_event = Some(event);
        self
    }

    // New setter for message_id
    pub fn set_message_id(mut self, message_id: String) -> Self {
        self.message_id = Some(message_id);
        self
    }

    // New setter for sequence_num
    pub fn set_sequence_num(mut self, sequence_num: usize) -> Self {
        self.sequence_num = Some(sequence_num);
        self
    }

    // New setter for total_chunks
    pub fn set_total_chunks(mut self, total_chunks: usize) -> Self {
        self.total_chunks = Some(total_chunks);
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
}

impl<'a> From<&'a Msg> for ratatui::text::Line<'a> {
    fn from(m: &'a Msg) -> Self {
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use MsgKind::*;

        match m.kind {
            //System
            System => Line::from(Span::styled(
                m.to_string(),
                Style::default()
                    .fg(Color::Red)
                    .add_modifier(Modifier::ITALIC),
            )),
            //Join
            Join => Line::from(Span::styled(
                m.to_string(),
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::ITALIC),
            )),
            //Leave
            Leave => Line::from(Span::styled(
                m.to_string(),
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::ITALIC),
            )),
            Chat => {
                Line::from(Span::raw(m.content[0].clone()))
            }
            Raw => m.content[0].clone().into(),
            Command => Line::default().spans(vec![
                Span::styled(
                    format!("Command: {}{} ", &m.from, ">"),
                    Style::default()
                        .add_modifier(Modifier::ITALIC),
                ),
                m.content[0].clone().into(),
            ]),
            //Git => Line::default().spans(
            //    vec![
            //        Span::styled(
            //            format!("{}", m.content[0].clone()),
            //            Style::default()
            //                .add_modifier(Modifier::ITALIC),
            //        ),
            //        //m.content[1].clone().into(),
            //    ]
            //    .iter()
            //    .map(|i| format!("{}", i)),
            //),
            GitCommitId => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"commit\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitTree => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"tree\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitAuthor => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"Author\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitParent => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"parent\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitMessagePart => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"msg\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitName => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"name\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitEmail => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"email\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitTime => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"time\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitHeader => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"header\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitBody => Line::default().spans(
                [
                    Span::styled(
                        format!("{{\"body\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            NostrEvent => Line::default().spans(vec![
                Span::styled(
                    "[Nostr Event]".to_string(),
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                m.content[0].clone().into(),
            ]),
            GitDiff => {
                let mut spans = Vec::new();
                for line in m.content.iter() {
                    let style = if line.starts_with('+') {
                        Style::default().fg(Color::Green)
                    } else if line.starts_with('-') {
                        Style::default().fg(Color::Red)
                    } else if line.starts_with('@') || line.starts_with('\\') {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    spans.push(Span::styled(line.clone(), style));
                }
                Line::from(spans)
            }
            OneShot => Line::from(Span::styled(
                format!("[ONESHOT] {}: {}", m.from, m.content[0]),
                Style::default().fg(Color::Rgb(255, 165, 0)), // Orange color for OneShot messages
            )),
        }
    }
}

impl Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            MsgKind::Join => write!(f, "{} joined!", self.from),
            MsgKind::Leave => write!(f, "{} left!", self.from),
            MsgKind::Chat => write!(f, "{}: {}", self.from, self.content[0]),
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
            MsgKind::NostrEvent => {
                write!(f, "[Nostr Event] {}", self.content[0])
            }
            MsgKind::GitDiff => {
                write!(f, "[Git Diff]")
            }
            MsgKind::OneShot => {
                write!(f, "[ONESHOT] {}: {}", self.from, self.content[0])
            }
        }
    }
}
