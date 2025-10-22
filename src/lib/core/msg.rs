use std::fmt::Display;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use crate::ui::solarized_light;

pub(crate) static USER_NAME: Lazy<String> = Lazy::new(|| {
    format!(
        "{}",
        std::env::var("USER")
            .unwrap_or_else(|_| hostname::get().unwrap().to_string_lossy().to_string()),
    )
});
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Default)]
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
}

impl Default for Msg {
    fn default() -> Self {
        Self {
            from: USER_NAME.clone(),
            content: vec!["".to_string(), "".to_string()],
            kind: MsgKind::Chat,
        }
    }
}

impl Msg {
    pub fn set_kind(mut self, kind: MsgKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn set_content(mut self, content: String) -> Self {
        self.content[0] = content;
        self
    }
    pub fn wrap_text(mut self, text: &Msg, max_width: usize) -> Self {
            for line in text.content.bytes() {

            line
                .flat_map(|line| {
                    line.chars()
                        .collect::<Vec<char>>()
                        .chunks(max_width)
                        .map(|chunk| chunk.iter().collect::<String>())
                        .collect::<Vec<String>>()
                })
                .collect()
        }

            text.content = line;
            text
    }
}

impl<'a> From<&'a Msg> for ratatui::text::Line<'a> {
    fn from(m: &'a Msg) -> Self {
        use ratatui::style::{Color, Modifier, Style};
        use ratatui::text::{Line, Span};
        use MsgKind::*;

        pub const YELLOW: Color = Color::Rgb(181, 137, 0);
        pub const ORANGE: Color = Color::Rgb(203, 75, 22);
        pub const RED: Color = Color::Rgb(220, 50, 47);
        pub const MAGENTA: Color = Color::Rgb(211, 54, 130);
        pub const VIOLET: Color = Color::Rgb(108, 113, 196);
        pub const BLUE: Color = Color::Rgb(38, 139, 210);
        pub const CYAN: Color = Color::Rgb(42, 161, 152);
        pub const GREEN: Color = Color::Rgb(133, 153, 0);


        fn gen_color_by_hash(s: &str) -> Color {
            static LIGHT_COLORS: [Color; 8] = [
            YELLOW,
            ORANGE,
            RED,
            MAGENTA,
            VIOLET,
            BLUE,
            CYAN,
            GREEN,
            ];
            let h = s.bytes().fold(0, |acc, b| acc ^ b as usize);
            LIGHT_COLORS[h % LIGHT_COLORS.len()]
        }

        match m.kind {
            Join | Leave | System => Line::from(Span::styled(
                m.to_string(),
                Style::default()
                    .fg(/*gen_color_by_hash(&m.kind)*/YELLOW)
                    .bg(/*gen_color_by_hash(&m.kind)*/ORANGE)
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
            MsgKind::Join => write!(f, "{} join", self.from),
            MsgKind::Leave => write!(f, "{} left", self.from),
            MsgKind::Chat => write!(f, "{}: {}", self.from, self.content[0]),
            MsgKind::System => write!(f, "[System] {}", self.content[0]),
            MsgKind::Raw => write!(f, "{}", self.content[0]),
            MsgKind::Command => write!(f, "[Command] {}:{}", self.from, self.content[0]),
            MsgKind::GitCommitId => {
                write!(f, "[GitCommitId] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitTree => {
                write!(f, "[GitCommitTree] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitParent => {
                write!(f, "[GitCommitParent] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitHeader => {
                write!(f, "[GitCommitHeader] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitAuthor => {
                write!(f, "[GitCommitAuthor] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitEmail => {
                write!(f, "[GitCommitEmail] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitName => {
                write!(f, "[GitCommitName] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitBody => {
                write!(f, "[GitCommitBody] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitMessagePart => {
                write!(f, "[GitCommitBody] {}:{}", self.from, self.content[0])
            }
            MsgKind::GitCommitTime => {
                write!(f, "[GitCommitTime] {}:{}", self.from, self.content[0])
            }
        }
    }
}
