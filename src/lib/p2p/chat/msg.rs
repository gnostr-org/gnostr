use git2::Oid;
use gnostr_asyncgit::sync::CommitId;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

pub(crate) static USER_NAME: Lazy<String> = Lazy::new(|| {
    format!(
        "{}",
        std::env::var("USER")
            .unwrap_or_else(|_| hostname::get().unwrap().to_string_lossy().to_string()),
    )
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
}

impl<'a> From<&'a Msg> for ratatui::text::Line<'a> {
    fn from(m: &'a Msg) -> Self {
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use MsgKind::*;

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
                if m.from == *USER_NAME {
                    Line::default().left_aligned().spans(vec![
                        Span::styled(
                            format!("{}{} ", &m.from, ">"),
                            Style::default().fg(gen_color_by_hash(&m.from)),
                        ),
                        m.content[0].clone().into(),
                    ])
                } else {
                    Line::default().right_aligned().spans(vec![
                        m.content[0].clone().into(),
                        Span::styled(
                            format!(" {}{}", "<", &m.from),
                            Style::default().fg(gen_color_by_hash(&m.from)),
                        ),
                    ])
                }
            }
            Raw => m.content[0].clone().into(),
            Command => Line::default().spans(vec![
                Span::styled(
                    format!("Command: {}{} ", &m.from, ">"),
                    Style::default()
                        .fg(gen_color_by_hash(&m.from))
                        .add_modifier(Modifier::ITALIC),
                ),
                m.content[0].clone().into(),
            ]),
            //Git => Line::default().spans(
            //    vec![
            //        Span::styled(
            //            format!("{}", m.content[0].clone()),
            //            Style::default()
            //                .fg(gen_color_by_hash(&m.from))
            //                .add_modifier(Modifier::ITALIC),
            //        ),
            //        //m.content[1].clone().into(),
            //    ]
            //    .iter()
            //    .map(|i| format!("{}", i)),
            //),
            GitCommitId => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"commit\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitTree => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"tree\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitAuthor => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"Author\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitParent => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"parent\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitMessagePart => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"msg\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitName => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"name\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitEmail => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"email\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitTime => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"time\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitHeader => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"header\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            GitCommitBody => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"body\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
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
                write!(f, "{{\"commit\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitTree => {
                write!(f, "{{\"tree\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitParent => {
                write!(f, "{{\"parent\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitHeader => {
                write!(f, "{{\"header\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitAuthor => {
                write!(f, "{{\"Author\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitEmail => {
                write!(f, "{{\"email\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitName => {
                write!(f, "{{\"name\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitBody => {
                write!(f, "{{\"body\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitMessagePart => {
                write!(f, "{{\"msg\": \"{}\"}} {}", self.content[0], self.content[1])
            }
            MsgKind::GitCommitTime => {
                write!(f, "{{\"time\": \"{}\"}} {}", self.content[0], self.content[1])
            }
        }
    }
}
