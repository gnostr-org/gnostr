//! A 3×3 grid widget that reports the dimensions of neighboring panels.
//!
//! Designed to be used inside the main area of [`MultiPanel`](crate::multi_panel::MultiPanel).

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::Text,
    widgets::{Block, Borders, Paragraph, StatefulWidget, Widget},
};

use crate::multi_panel::MultiPanelState;

/// Vertical alignment for positioning value text inside a grid cell.
#[derive(Clone, Copy)]
enum VerticalAlign {
    Top,
    Center,
    Bottom,
}

/// Renders a 3×3 grid of sub-widgets, each reporting a neighbor's size.
#[derive(Debug, Clone)]
pub struct NeighborReportGrid {
    left_fraction: f64,
    right_fraction: f64,
    header_height: u16,
    footer_height: u16,
    center_content: Option<String>,
    cell_color: Option<Color>,
}

impl NeighborReportGrid {
    /// Create a report grid with the same layout fractions used by the outer
    /// [`MultiPanel`](crate::multi_panel::MultiPanel).
    pub fn new(
        left_fraction: f64,
        right_fraction: f64,
        header_height: u16,
        footer_height: u16,
    ) -> Self {
        Self {
            left_fraction,
            right_fraction,
            header_height,
            footer_height,
            center_content: None,
            cell_color: None,
        }
    }

    /// Override the center cell's content. If not set, the center cell shows
    /// the main area dimensions.
    pub fn center_content(mut self, content: impl Into<String>) -> Self {
        self.center_content = Some(content.into());
        self
    }

    /// Set a background color for every cell in the grid.
    pub fn cell_color(mut self, color: Color) -> Self {
        self.cell_color = Some(color);
        self
    }
}

impl StatefulWidget for NeighborReportGrid {
    type State = MultiPanelState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        let rows = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
            ])
            .split(area);

        let cols_top = split_three(rows[0]);
        let cols_mid = split_three(rows[1]);
        let cols_bot = split_three(rows[2]);
        let cell_color = self.cell_color;

        subwidget(
            Some("Top-Left"),
            format!("{}x{}", cols_top[0].width, cols_top[0].height),
            Alignment::Left,
            VerticalAlign::Top,
            cols_top[0],
            buf,
            cell_color,
        );
        subwidget(
            Some("Top"),
            format!("{} rows", state.header_height(self.header_height)),
            Alignment::Center,
            VerticalAlign::Top,
            cols_top[1],
            buf,
            cell_color,
        );
        subwidget(
            Some("Top-Right"),
            format!("{}x{}", cols_top[2].width, cols_top[2].height),
            Alignment::Right,
            VerticalAlign::Top,
            cols_top[2],
            buf,
            cell_color,
        );

        subwidget(
            Some("Left"),
            format!(
                "{} cols",
                state.left_sidebar_width(area.width, self.left_fraction)
            ),
            Alignment::Left,
            VerticalAlign::Center,
            cols_mid[0],
            buf,
            cell_color,
        );
        subwidget(
            None,
            self.center_content
                .clone()
                .unwrap_or_else(|| format!("{}x{}", area.width, area.height)),
            Alignment::Center,
            VerticalAlign::Center,
            cols_mid[1],
            buf,
            cell_color,
        );
        subwidget(
            Some("Right"),
            format!(
                "{} cols",
                state.right_sidebar_width(area.width, self.right_fraction)
            ),
            Alignment::Right,
            VerticalAlign::Center,
            cols_mid[2],
            buf,
            cell_color,
        );

        subwidget(
            Some("Bot-Left"),
            format!("{}x{}", cols_bot[0].width, cols_bot[0].height),
            Alignment::Left,
            VerticalAlign::Bottom,
            cols_bot[0],
            buf,
            cell_color,
        );
        subwidget(
            Some("Bottom"),
            format!("{} rows", state.footer_height(self.footer_height)),
            Alignment::Center,
            VerticalAlign::Bottom,
            cols_bot[1],
            buf,
            cell_color,
        );
        subwidget(
            Some("Bot-Right"),
            format!("{}x{}", cols_bot[2].width, cols_bot[2].height),
            Alignment::Right,
            VerticalAlign::Bottom,
            cols_bot[2],
            buf,
            cell_color,
        );
    }
}

fn split_three(area: Rect) -> std::rc::Rc<[Rect]> {
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
            Constraint::Ratio(1, 3),
        ])
        .split(area)
}

fn value_area(inner: Rect, value: &str, valign: VerticalAlign) -> Rect {
    let lines = value.lines().count() as u16;
    let height = lines.min(inner.height);
    let y = match valign {
        VerticalAlign::Top => inner.y,
        VerticalAlign::Center => inner.y + (inner.height.saturating_sub(height)) / 2,
        VerticalAlign::Bottom => inner.y + inner.height.saturating_sub(height),
    };
    Rect {
        x: inner.x,
        y,
        width: inner.width,
        height,
    }
}

fn subwidget(
    title: Option<&str>,
    value: String,
    align: Alignment,
    valign: VerticalAlign,
    area: Rect,
    buf: &mut Buffer,
    cell_color: Option<Color>,
) {
    if area.width == 0 || area.height == 0 {
        return;
    }

    let style = cell_color.map_or_else(Style::default, |c| Style::default().bg(c));

    // Only draw borders when the cell is large enough; otherwise fall back to
    // plain text to avoid border artifacts in tiny areas.
    const MIN_BORDER_WIDTH: u16 = 4;
    const MIN_BORDER_HEIGHT: u16 = 3;
    if area.width >= MIN_BORDER_WIDTH && area.height >= MIN_BORDER_HEIGHT {
        let block = {
            let b = Block::default().borders(Borders::ALL).style(style);
            match title {
                Some(t) => b.title(t),
                None => b,
            }
        };
        let inner = block.inner(area);
        Widget::render(block, area, buf);
        if inner.width > 0 && inner.height > 0 {
            let value_rect = value_area(inner, &value, valign);
            let p = Paragraph::new(Text::from(value))
                .alignment(align)
                .style(style);
            Widget::render(p, value_rect, buf);
        }
    } else {
        let text = match title {
            Some(t) => Text::from(format!("{}\n{}", t, value)),
            None => Text::from(value),
        };
        let p = Paragraph::new(text).alignment(align).style(style);
        Widget::render(p, area, buf);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn report_grid_renders_without_panic() {
        let mut state = MultiPanelState::open(4.0);
        let widget = NeighborReportGrid::new(0.25, 0.25, 5, 5);
        let mut buf = Buffer::empty(Rect::new(0, 0, 80, 24));
        StatefulWidget::render(widget, buf.area, &mut buf, &mut state);
    }
}
