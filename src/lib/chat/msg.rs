use crate::{get_blockheight, get_weeble, get_wobble, VERSION};
use std::fmt::Display;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};

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
    GitCommitDiff,
    Topic,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Msg {
    pub from: String,
    pub content: Vec<String>,
    pub kind: MsgKind,
    pub weeble: String,
    pub blockheight: String,
    pub wobble: String,
    pub chat_version: String,
}

impl Default for Msg {
    fn default() -> Self {
        Self {
            from: USER_NAME.clone(),
            content: vec![
                "0".to_string(), // 0
                "1".to_string(), // 1
                "2".to_string(), // 2
                "3".to_string(), // 3
                "4".to_string(), // 4
                "5".to_string(), // 5
                "6".to_string(), // 6
            ],
            kind: MsgKind::Chat,
            weeble: "0".to_string(),
            blockheight: "0".to_string(),
            wobble: "0".to_string(),
            chat_version: VERSION.to_string(),
        }
    }
}

impl Msg {
    pub fn set_kind(mut self, kind: MsgKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn set_content(mut self, content: String, index: usize) -> Self {
        self.content[index] = content;
        self
    }
    pub fn wrap_text(mut self, text: Msg, max_width: usize) -> Self {
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
            //          Join | Leave | System => Line::from(Span::styled(
            //              m.to_string(),
            //              Style::default()
            //                  .fg(Color::DarkGray)
            //                  .add_modifier(Modifier::ITALIC),
            //          )),
            Join | Leave | System => Line::default().right_aligned().spans(
                vec![
                    Span::styled(
                        format!(
                            "{{\"commit\": \"v{}:{} joined!/\"}}",
                            m.chat_version.clone(),
                            m.from.clone(),
                        ),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[0].clone().into(),
                    m.content[1].clone().into(),
                    //m.content[2].clone().into(),
                    //m.content[3].clone().into(),
                    //m.content[4].clone().into(),
                    //m.content[5].clone().into(),
                    //m.content[6].clone().into(),
                    Span::styled(
                        format!("{{\"version\": \"{}/\"}}", m.chat_version.clone(),),
                        Style::default()
                            .fg(gen_color_by_hash(&m.chat_version))
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::ITALIC),
                    ),
                ]
                .iter()
                .map(|i| format!("---{}---", i)),
            ),
            //m.to_string(),
            //Style::default()
            //    .fg(Color::DarkGray)
            //    .add_modifier(Modifier::ITALIC),
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
            GitCommitDiff => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"diff\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[1].clone().into(),
                ]
                .iter()
                .map(|i| format!("{}", i)),
            ),
            Topic => Line::default().spans(
                vec![
                    Span::styled(
                        format!("{{\"topic\": \"{}\"}}", m.content[0].clone()),
                        Style::default()
                            .fg(gen_color_by_hash(&m.from))
                            .add_modifier(Modifier::ITALIC),
                    ),
                    m.content[0].clone().into(),
                    m.content[1].clone().into(),
                    m.content[2].clone().into(),
                    m.content[3].clone().into(),
                    m.content[4].clone().into(),
                    m.content[5].clone().into(),
                    m.content[6].clone().into(),
                    m.chat_version.clone().into(),
                ]
                .iter()
                .map(|i| format!("topic:{}", i)),
            ),
        }
    }
}

impl Display for Msg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            MsgKind::Join => write!(
                f,
                "{} joined:{:?}/{:?}/{:?}",
                self.from,
                get_weeble(),
                get_blockheight(),
                get_wobble()
            ),
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
            MsgKind::GitCommitDiff => {
                write!(f, "[GitCommitDiff] {}:{}", self.from, self.content[0])
            }
            MsgKind::Topic => {
                write!(
                    f,
                    //           0   1   2   3   4   5   6
                    "[Topic] {}:{}\n{}\n{}\n{}\n{}\n{}\n{}",
                    self.from,
                    self.content[0],
                    self.content[1],
                    self.content[2],
                    self.content[3],
                    self.content[4],
                    self.content[5],
                    self.content[6],
                )
            }
        }
    }
}
