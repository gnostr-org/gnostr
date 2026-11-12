//! Mouse-interactive widgets and helpers.
//!
//! Inspired by the Ratatui `custom-widget` example, this module provides a
//! reusable, themeable button that reacts to mouse movement and clicks.

use std::io::{self, stdout};

use crossterm::{
    event::{DisableMouseCapture, EnableMouseCapture, MouseButton, MouseEvent, MouseEventKind},
    execute,
};
use ratatui::{
    buffer::Buffer,
    layout::Rect,
    style::{Color, Style},
    text::Line,
    widgets::StatefulWidget,
};

/// Visual state of a [`Button`].
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum VisualState {
    #[default]
    Normal,
    Selected,
    Active,
}

/// Theme colors for a [`Button`].
#[derive(Debug, Clone, Copy)]
pub struct ButtonTheme {
    pub text: Color,
    pub background: Color,
    pub highlight: Color,
    pub shadow: Color,
}

impl ButtonTheme {
    /// A blue color scheme.
    pub const BLUE: Self = Self {
        text: Color::Rgb(16, 24, 48),
        background: Color::Rgb(48, 72, 144),
        highlight: Color::Rgb(64, 96, 192),
        shadow: Color::Rgb(32, 48, 96),
    };

    /// A red color scheme.
    pub const RED: Self = Self {
        text: Color::Rgb(48, 16, 16),
        background: Color::Rgb(144, 48, 48),
        highlight: Color::Rgb(192, 64, 64),
        shadow: Color::Rgb(96, 32, 32),
    };

    /// A green color scheme.
    pub const GREEN: Self = Self {
        text: Color::Rgb(16, 48, 16),
        background: Color::Rgb(48, 144, 48),
        highlight: Color::Rgb(64, 192, 64),
        shadow: Color::Rgb(32, 96, 32),
    };
}

impl Default for ButtonTheme {
    fn default() -> Self {
        Self::BLUE
    }
}

/// A themeable button widget that responds to mouse events.
#[derive(Debug, Clone)]
pub struct Button<'a> {
    label: Line<'a>,
    theme: ButtonTheme,
}

impl<'a> Button<'a> {
    /// Create a new button with the given label.
    pub fn new<T: Into<Line<'a>>>(label: T) -> Self {
        Self {
            label: label.into(),
            theme: ButtonTheme::default(),
        }
    }

    /// Set the button theme.
    pub const fn theme(mut self, theme: ButtonTheme) -> Self {
        self.theme = theme;
        self
    }

    fn colors(&self, state: VisualState) -> (Color, Color, Color, Color) {
        let theme = self.theme;
        match state {
            VisualState::Normal => (theme.background, theme.text, theme.shadow, theme.highlight),
            VisualState::Selected => (theme.highlight, theme.text, theme.shadow, theme.highlight),
            VisualState::Active => (theme.background, theme.text, theme.highlight, theme.shadow),
        }
    }
}

/// Mutable state for a [`Button`], including its current visual state and the
/// last area it was rendered into.
#[derive(Debug, Default, Clone, Copy)]
pub struct ButtonState {
    pub state: VisualState,
    pub last_area: Option<Rect>,
}

impl ButtonState {
    /// Update the button state from a mouse event.
    ///
    /// Returns `true` if the event was consumed by this button.
    pub fn handle_mouse(&mut self, event: MouseEvent) -> bool {
        let Some(area) = self.last_area else {
            return false;
        };

        let over = is_mouse_over(area, event.column, event.row);

        match event.kind {
            MouseEventKind::Moved => {
                if over && self.state != VisualState::Active {
                    self.state = VisualState::Selected;
                    true
                } else if !over && self.state == VisualState::Selected {
                    self.state = VisualState::Normal;
                    true
                } else {
                    false
                }
            }
            MouseEventKind::Down(MouseButton::Left) if over => {
                self.state = if self.state == VisualState::Active {
                    VisualState::Normal
                } else {
                    VisualState::Active
                };
                true
            }
            _ => false,
        }
    }
}

impl StatefulWidget for Button<'_> {
    type State = ButtonState;

    fn render(self, area: Rect, buf: &mut Buffer, state: &mut Self::State) {
        state.last_area = Some(area);

        let (background, text, shadow, highlight) = self.colors(state.state);
        buf.set_style(area, Style::new().bg(background).fg(text));

        if area.height > 2 {
            let top = "▔".repeat(area.width as usize);
            buf.set_string(
                area.x,
                area.y,
                &top,
                Style::new().fg(highlight).bg(background),
            );
        }

        if area.height > 1 {
            let bottom = "▁".repeat(area.width as usize);
            buf.set_string(
                area.x,
                area.y + area.height - 1,
                &bottom,
                Style::new().fg(shadow).bg(background),
            );
        }

        let label_width = self.label.width() as u16;
        let x = area.x + (area.width.saturating_sub(label_width)) / 2;
        let y = area.y + (area.height.saturating_sub(1)) / 2;
        buf.set_line(x, y, &self.label, area.width);
    }
}

/// Check whether a mouse position is inside a rectangle.
pub fn is_mouse_over(area: Rect, column: u16, row: u16) -> bool {
    area.x <= column && column < area.x + area.width && area.y <= row && row < area.y + area.height
}

/// Enable crossterm mouse capture.
pub fn enable_mouse_capture() -> io::Result<()> {
    execute!(stdout(), EnableMouseCapture)
}

/// Disable crossterm mouse capture.
pub fn disable_mouse_capture() -> io::Result<()> {
    execute!(stdout(), DisableMouseCapture)
}
