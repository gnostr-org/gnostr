use ratatui::{
    buffer::Buffer,
    layout::Rect,
    prelude::Stylize,
    style::Style,
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, StatefulWidget, Widget},
};

pub struct ListPanel<'a> {
    pub title: &'a str,
    pub items: Vec<ListItem<'a>>,
    pub empty_message: &'a str,
    pub highlight_style: Style,
    pub highlight_symbol: &'a str,
}

impl<'a> StatefulWidget for ListPanel<'a> {
    type State = ListState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        if self.items.is_empty() {
            Paragraph::new(self.empty_message)
                .block(Block::default().borders(Borders::ALL).title(self.title))
                .render(area, buf);
            return;
        }

        StatefulWidget::render(
            List::new(self.items)
                .block(Block::default().borders(Borders::ALL).title(self.title))
                .highlight_style(self.highlight_style)
                .highlight_symbol(self.highlight_symbol),
            area,
            buf,
            state,
        );
    }
}

pub struct DetailPanel<'a> {
    pub title: &'a str,
    pub lines: Vec<Line<'a>>,
    pub empty_message: &'a str,
    pub scroll: (u16, u16),
}

impl<'a> Widget for DetailPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let content = if self.lines.is_empty() {
            vec![Line::from(self.empty_message)]
        } else {
            self.lines
        };

        Paragraph::new(content)
            .block(Block::default().borders(Borders::ALL).title(self.title))
            .render(area, buf);
    }
}

#[allow(dead_code)]
pub struct KeyHintPanel<'a> {
    pub title: &'a str,
    pub lines: Vec<Line<'a>>,
}

impl<'a> Widget for KeyHintPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Paragraph::new(self.lines)
            .block(Block::default().borders(Borders::ALL).title(self.title))
            .render(area, buf);
    }
}

#[allow(dead_code)]
pub fn tree_line<'a>(path: String, meta: &'a str, selected: bool) -> ListItem<'a> {
    let style = if selected {
        Style::default().reversed()
    } else {
        Style::default()
    };

    ListItem::new(Line::from(vec![
        Span::styled(path, style),
        Span::raw("  "),
        Span::styled(meta, Style::default().dim()),
    ]))
}
