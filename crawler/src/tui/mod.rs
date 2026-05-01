use ratatui::layout::{Constraint, Direction, Layout, Rect};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PaneFocus {
    Types,
    Events,
    Detail,
}

impl PaneFocus {
    pub fn next(self) -> Self {
        match self {
            Self::Types => Self::Events,
            Self::Events => Self::Detail,
            Self::Detail => Self::Types,
        }
    }

    pub fn prev(self) -> Self {
        match self {
            Self::Types => Self::Detail,
            Self::Events => Self::Types,
            Self::Detail => Self::Events,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Types => "types",
            Self::Events => "events",
            Self::Detail => "detail",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MasterDetailLayout {
    pub header: Rect,
    pub body: Rect,
    pub footer: Rect,
    pub left: Rect,
    pub left_detail: Rect,
    pub middle: Rect,
    pub right: Rect,
}

impl MasterDetailLayout {
    pub fn new(area: Rect) -> Self {
        let root = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0), Constraint::Length(2)])
            .split(area);
        let panes = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(32),
                Constraint::Percentage(34),
                Constraint::Percentage(34),
            ])
            .split(root[1]);
        let left = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Min(10), Constraint::Length(9)])
            .split(panes[0]);
        Self {
            header: root[0],
            body: root[1],
            footer: root[2],
            left: left[0],
            left_detail: left[1],
            middle: panes[1],
            right: panes[2],
        }
    }
}

pub fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(vertical[1])[1]
}

pub fn clamp_index(current: usize, len: usize, delta: i32) -> usize {
    if len == 0 {
        return 0;
    }
    let next = if delta.is_negative() {
        current.saturating_sub(delta.unsigned_abs() as usize)
    } else {
        current.saturating_add(delta as usize)
    };
    next.min(len - 1)
}

pub fn preview_text(value: &str, max: usize) -> String {
    let value = value.trim();
    let mut chars = value.chars();
    let prefix = chars.by_ref().take(max).collect::<String>();
    if chars.next().is_some() {
        format!("{}…", prefix)
    } else {
        prefix
    }
}

pub fn title_case(value: &str) -> String {
    let mut chars = value.chars();
    match chars.next() {
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
        None => String::new(),
    }
}
