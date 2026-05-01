use serde_json::Value;
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Widget},
};

pub struct JsonPanel<'a> {
    pub title: &'a str,
    pub value: Option<&'a Value>,
    pub error: Option<&'a str>,
    pub empty_message: &'a str,
    pub scroll: (u16, u16),
}

impl<'a> Widget for JsonPanel<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let lines = if let Some(error) = self.error {
            vec![
                Line::from(vec![
                    Span::styled("json error: ", Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)),
                    Span::raw(error.to_string()),
                ]),
            ]
        } else if let Some(value) = self.value {
            value_to_lines(value)
        } else {
            vec![Line::from(self.empty_message)]
        };

        Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).title(self.title))
            .scroll(self.scroll)
            .render(area, buf);
    }
}

pub fn value_to_lines(value: &Value) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    push_value(&mut lines, value, 0, None);
    lines
}

fn push_value(lines: &mut Vec<Line<'static>>, value: &Value, indent: usize, label: Option<&str>) {
    match value {
        Value::Object(map) => {
            if let Some(label) = label {
                lines.push(Line::from(vec![
                    indent_span(indent),
                    Span::styled(format!("{}: {{", label), Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    indent_span(indent),
                    Span::styled("{", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
                ]));
            }
            for (key, value) in map {
                push_value(lines, value, indent + 1, Some(key));
            }
            lines.push(Line::from(vec![
                indent_span(indent),
                Span::styled("}", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
            ]));
        }
        Value::Array(items) => {
            if let Some(label) = label {
                lines.push(Line::from(vec![
                    indent_span(indent),
                    Span::styled(format!("{}: [", label), Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                ]));
            } else {
                lines.push(Line::from(vec![
                    indent_span(indent),
                    Span::styled("[", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
                ]));
            }
            for item in items {
                push_value(lines, item, indent + 1, None);
            }
            lines.push(Line::from(vec![
                indent_span(indent),
                Span::styled("]", Style::default().fg(Color::Magenta).add_modifier(Modifier::BOLD)),
            ]));
        }
        _ => {
            let rendered = match serde_json::to_string(value) {
                Ok(rendered) => rendered,
                Err(_) => String::from("null"),
            };
            let key = label.map(|label| format!("{}: ", label)).unwrap_or_default();
            let key_style = Style::default().fg(Color::Cyan);
            let value_style = match value {
                Value::String(_) => Style::default().fg(Color::Green),
                Value::Number(_) => Style::default().fg(Color::Yellow),
                Value::Bool(_) => Style::default().fg(Color::Magenta),
                Value::Null => Style::default().fg(Color::DarkGray),
                _ => Style::default(),
            };
            lines.push(Line::from(vec![
                indent_span(indent),
                Span::styled(key, key_style),
                Span::styled(rendered, value_style),
            ]));
        }
    }
}

fn indent_span(indent: usize) -> Span<'static> {
    Span::raw("  ".repeat(indent))
}
