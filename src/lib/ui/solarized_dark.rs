use ratatui::style::{Color, Style};
use ratatui::widgets::Paragraph;

pub struct SolarizedDark;

impl SolarizedDark {
    // Base colors
    pub const BASE03: Color = Color::Rgb(0, 43, 54);
    pub const BASE02: Color = Color::Rgb(7, 54, 66);
    pub const BASE01: Color = Color::Rgb(88, 110, 117);
    pub const BASE00: Color = Color::Rgb(101, 123, 131);
    pub const BASE0: Color = Color::Rgb(131, 148, 150);
    pub const BASE1: Color = Color::Rgb(147, 161, 161);
    pub const BASE2: Color = Color::Rgb(238, 232, 213);
    pub const BASE3: Color = Color::Rgb(253, 246, 227);

    // Accent colors
    pub const YELLOW: Color = Color::Rgb(181, 137, 0);
    pub const ORANGE: Color = Color::Rgb(203, 75, 22);
    pub const RED: Color = Color::Rgb(220, 50, 47);
    pub const MAGENTA: Color = Color::Rgb(211, 54, 130);
    pub const VIOLET: Color = Color::Rgb(108, 113, 196);
    pub const BLUE: Color = Color::Rgb(38, 139, 210);
    pub const CYAN: Color = Color::Rgb(42, 161, 152);
    pub const GREEN: Color = Color::Rgb(133, 153, 0);
}

fn _test_solarized_dark() -> Paragraph<'static> {
    let my_paragraph = Paragraph::new("Hello, Solarized World!").style(
        Style::default()
            .fg(SolarizedDark::BASE2)
            .bg(SolarizedDark::BASE03),
    );
    my_paragraph
}
