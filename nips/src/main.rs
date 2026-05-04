use std::{
    error::Error,
    fs,
    io::{stdout},
    path::PathBuf,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph, Wrap},
};

struct App {
    nips: Vec<PathBuf>,
    selected_nip_index: usize,
    content_scroll: u16,
    show_nip_list: bool,
}

impl App {
    fn new(nips: Vec<PathBuf>) -> App {
        App {
            nips,
            selected_nip_index: 0,
            content_scroll: 0,
            show_nip_list: true,
        }
    }

    fn next_nip(&mut self) {
        if !self.nips.is_empty() {
            self.selected_nip_index = (self.selected_nip_index + 1) % self.nips.len();
            self.content_scroll = 0; // Reset scroll on new selection
        }
    }

    fn previous_nip(&mut self) {
        if !self.nips.is_empty() {
            self.selected_nip_index = (self.selected_nip_index + self.nips.len() - 1) % self.nips.len();
            self.content_scroll = 0; // Reset scroll on new selection
        }
    }

    fn scroll_content_up(&mut self) {
        self.content_scroll = self.content_scroll.saturating_sub(1);
    }

    fn scroll_content_down(&mut self) {
        self.content_scroll = self.content_scroll.saturating_add(1);
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, mut app: App) -> Result<(), Box<dyn Error>> {
    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if let Event::Key(key) = event::read()? {
            if key.kind == KeyEventKind::Press {
                match key.code {
                    KeyCode::Char('q') => return Ok(()),
                    KeyCode::Esc => {
                        app.show_nip_list = !app.show_nip_list;
                    }
                    KeyCode::Enter => {
                        app.show_nip_list = !app.show_nip_list;
                    }
                    KeyCode::Left => {
                        app.previous_nip();
                        app.show_nip_list = false;
                    }
                    KeyCode::Right => {
                        app.next_nip();
                        app.show_nip_list = false;
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.next_nip(),
                    KeyCode::Up | KeyCode::Char('k') => app.previous_nip(),
                    KeyCode::PageDown => {
                        app.content_scroll = app.content_scroll.saturating_add(10);
                    }
                    KeyCode::PageUp => {
                        app.content_scroll = app.content_scroll.saturating_sub(10);
                    }
                    _ => {}
                }
            }
        }
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let main_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // For the header
            Constraint::Min(0),    // For the main content
        ])
        .split(frame.size());

    // Render the main title
    let title = Paragraph::new("Nostr NIPs Browser")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD));
    frame.render_widget(title, main_chunks[0]);

    let main_layout_constraints = if app.show_nip_list {
        vec![Constraint::Percentage(30), Constraint::Percentage(70)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let main_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(main_layout_constraints)
        .split(main_chunks[1]);

    let content_frame_index = if app.show_nip_list {
        1
    } else {
        0
    };

    if app.show_nip_list {
        // NIP List
        let items: Vec<ListItem> = app.nips
            .iter()
            .enumerate()
            .map(|(i, path)| {
                let content = path.file_name().unwrap().to_string_lossy().into_owned();
                ListItem::new(content).style(if i == app.selected_nip_index {
                    Style::default().fg(Color::Black).bg(Color::LightGreen)
                } else {
                    Style::default()
                })
            })
            .collect();

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("NIPs"))
            .highlight_style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_symbol(">> ");

        frame.render_stateful_widget(list, main_layout[0], &mut ListState::default().with_selected(Some(app.selected_nip_index)));
    }

    // NIP Content
    let mut content_text = String::new();
    let mut title_text = "Content".to_string();
    if let Some(selected_nip_path) = app.nips.get(app.selected_nip_index) {
        title_text = selected_nip_path.file_name().unwrap().to_string_lossy().into_owned();
        if let Ok(content) = fs::read_to_string(selected_nip_path) {
            content_text = content;
        } else {
            content_text = format!("Error reading file: {}", selected_nip_path.display());
        }
    } else {
        content_text = "No NIP selected.".to_string();
    }

    let paragraph = Paragraph::new(content_text)
        .block(Block::default().borders(Borders::ALL).title(title_text))
        .wrap(Wrap { trim: false })
        .scroll((app.content_scroll, 0));

    frame.render_widget(paragraph, main_layout[content_frame_index]);
}

fn main() -> Result<(), Box<dyn Error>> {
    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut nips: Vec<PathBuf> = fs::read_dir(".")?
        .filter_map(|entry| {
            let path = entry.ok()?.path();
            if path.is_file() && path.extension().map_or(false, |ext| ext == "md") {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    nips.sort_by(|a, b| {
        let a_name = a.file_name().unwrap().to_string_lossy();
        let b_name = b.file_name().unwrap().to_string_lossy();
        a_name.cmp(&b_name)
    });


    let app = App::new(nips);
    let res = run_app(&mut terminal, app);

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        println!("{err:?}");
    }

    Ok(())
}