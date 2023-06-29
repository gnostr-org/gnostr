use std::sync::mpsc;
use std::time::{Duration, Instant};
use std::{fs, io, process, thread};

use chrono::prelude::*;
use crossterm::event::{self, Event as CEvent, KeyCode};
use crossterm::terminal::{disable_raw_mode, enable_raw_mode};
use homedir::get_my_home;
use rand::distributions::Alphanumeric;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tui::backend::CrosstermBackend;
use tui::layout::{Alignment, Constraint, Direction, Layout};
use tui::style::{Color, Modifier, Style};
use tui::text::{Span, Spans};
use tui::widgets::{
    Block, BorderType, Borders, Cell, List, ListItem, ListState, Paragraph, Row, Table, Tabs,
};
use tui::Terminal;
const APP_NAME: &str = env!("CARGO_PKG_NAME");
const VERSION: &str = env!("CARGO_PKG_VERSION");

const ICON_FONT_SIZE: u16 = 12;
const DB_PATH: &str = "./data/db.json";

const INDIGO: Color = Color::Rgb(182, 46, 209);

#[derive(Error, Debug)]
pub enum Error {
    #[error("error reading the DB file: {0}")]
    ReadDBError(#[from] io::Error),
    #[error("error parsing the DB file: {0}")]
    ParseDBError(#[from] serde_json::Error),
}

enum Event<I> {
    Input(I),
    Tick,
}

#[derive(Serialize, Deserialize, Clone)]
struct Pet {
    id: usize,
    name: String,
    category: String,
    age: usize,
    created_at: DateTime<Utc>,
}

#[derive(Copy, Clone, Debug)]
enum MenuItem {
    Home,
    Pets,
}

impl From<MenuItem> for usize {
    fn from(input: MenuItem) -> usize {
        match input {
            MenuItem::Home => 0,
            MenuItem::Pets => 1,
        }
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode().expect("can run in raw mode");

    let db_path: &str = &format!(
        "{:}/.gnostr/data/db.json",
        get_my_home().unwrap().unwrap().display()
    );
    let _ = add_random_pet_to_db(db_path);
    let (tx, rx) = mpsc::channel();
    let tick_rate = Duration::from_millis(200);
    thread::spawn(move || {
        let mut last_tick = Instant::now();
        loop {
            let timeout = tick_rate
                .checked_sub(last_tick.elapsed())
                .unwrap_or_else(|| Duration::from_secs(0));

            if event::poll(timeout).expect("poll works") {
                if let CEvent::Key(key) = event::read().expect("can read events") {
                    tx.send(Event::Input(key)).expect("can send events");
                }
            }

            if last_tick.elapsed() >= tick_rate {
                if let Ok(_) = tx.send(Event::Tick) {
                    last_tick = Instant::now();
                }
            }
        }
    });

    let stdout = io::stdout();
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    terminal.clear()?;

    //MENU TITLES

    let menu_titles = vec!["Home", "Relays", "Add", "Delete", "Quit"];
    let mut active_menu_item = MenuItem::Home;
    let mut pet_list_state = ListState::default();
    pet_list_state.select(Some(0));

    //LOOP
    loop {
        terminal.draw(|rect| {
            let size = rect.size();
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(3),
                        Constraint::Min(2),
                        Constraint::Length(3),
                    ]
                    .as_ref(),
                )
                .split(size);

            let copyright = Paragraph::new(format!(" {} FOOTER", APP_NAME))
                .style(Style::default().fg(Color::LightCyan))
                .alignment(Alignment::Left)
                .block(
                    Block::default()
                        .borders(Borders::ALL)
                        .style(Style::default().fg(Color::White))
                        .title(format!(" {} v{}", APP_NAME, VERSION))
                        .border_type(BorderType::Plain),
                );

            let menu = menu_titles
                .iter()
                .map(|t| {
                    let (first, rest) = t.split_at(1);
                    Spans::from(vec![
                        Span::styled(
                            first,
                            Style::default()
                                .fg(Color::Yellow)
                                .add_modifier(Modifier::UNDERLINED),
                        ),
                        Span::styled(rest, Style::default().fg(Color::White)),
                    ])
                })
                .collect();

            let tabs = Tabs::new(menu)
                .select(active_menu_item.into())
                .block(Block::default().title("Menu").borders(Borders::ALL))
                .style(Style::default().fg(Color::White))
                .highlight_style(Style::default().fg(Color::Yellow))
                .divider(Span::raw("|"));

            rect.render_widget(tabs, chunks[0]);
            match active_menu_item {
                MenuItem::Home => rect.render_widget(render_home(), chunks[1]),
                MenuItem::Pets => {
                    let pets_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(20), Constraint::Percentage(80)].as_ref(),
                        )
                        .split(chunks[1]);
                    let (left, right) = render_pets(db_path, &pet_list_state);
                    rect.render_stateful_widget(left, pets_chunks[0], &mut pet_list_state);
                    rect.render_widget(right, pets_chunks[1]);

                    //footer not persist after quit here
                    rect.render_widget(copyright.clone(), chunks[2]);
                }
            }
        })?;

        match rx.recv()? {
            Event::Input(event) => match event.code {
                KeyCode::Char('q') => {
                    disable_raw_mode()?;
                    //terminal.clear()?;
                    render_home();
                    terminal.show_cursor()?;
                    break;
                }
                KeyCode::Char('h') => active_menu_item = MenuItem::Home,
                KeyCode::Char('r') => active_menu_item = MenuItem::Pets,
                KeyCode::Char('a') => {
                    add_random_pet_to_db(db_path).expect("can add new random pet");
                }
                KeyCode::Char('d') => {
                    remove_pet_at_index(db_path, &mut pet_list_state).expect("can remove pet");
                }
                KeyCode::Down => {
                    if let Some(selected) = pet_list_state.selected() {
                        let amount_pets = read_db(db_path).expect("can fetch pet list").len();
                        if selected >= amount_pets - 1 {
                            pet_list_state.select(Some(0));
                        } else {
                            pet_list_state.select(Some(selected + 1));
                        }
                    }
                }
                KeyCode::Up => {
                    if let Some(selected) = pet_list_state.selected() {
                        let amount_pets = read_db(db_path).expect("can fetch pet list").len();
                        if selected > 0 {
                            pet_list_state.select(Some(selected - 1));
                        } else {
                            pet_list_state.select(Some(amount_pets - 1));
                        }
                    }
                }
                _ => {}
            },
            Event::Tick => {}
        }
    }

    clearscreen::clear().expect("failed to clear screen");
    Ok(())
}

fn render_home<'a>() -> Paragraph<'a> {
    let home = Paragraph::new(vec![
        //REF: Unicode Character “█” (U+2588)

        //center line
        //Spans::from(vec![Span::raw("
        //███████████████████████████████████████•███████████████████████████████████████
        //")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw("")]),
        Spans::from(vec![Span::raw(
            "
 █•█ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 ███•███ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 █████•█████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 ███████•███████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 █████████•█████████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
   ██████████•███████████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
█    ████████•█████████████",
        )]),
        Spans::from(vec![Span::raw(
            "
 ████     ██████•███████████████",
        )]),
        Spans::from(vec![Span::raw(
            "
 ████████      ███•█████████████████",
        )]),
        Spans::from(vec![Span::raw(
            "
 ████████████      █•███████████████████",
        )]),
        Spans::from(vec![Span::raw(
            "
  ████████████████         ██████████████████  ",
        )]),
        Spans::from(vec![Span::raw(
            "
██████████████████           ██████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
███████████████████             ███████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
█████████████████████             █████████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
████████████████████████           ████████████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
 ████████████████████████████           ████████████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
███████████████████████████████     █      ████████████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
 ██████████████████████████████████     ███        ██████████████████████  ",
        )]),
        Spans::from(vec![Span::raw(
            "
████████████████████████████████████     █████          ████████████████████",
        )]),
        Spans::from(vec![Span::raw(
            "
█████████████████████████████████████     ███████           ██████████████████
",
        )]),
        //vim command to find center
        //:exe 'normal '.(virtcol('$')/2).'|'
        // █
        // ▉ ▊ ▋ ▌ ▍ ▎ ▏ ▐ ▔ ▕ ▀ ▁ ▂ ▃ ▄ ▅ ▆ ▇ █ ▉ ▊ ▋ ▌ ▍ ▎ ▏ ▐ ▔ ▕
        // █
        // █
        //FULL BLOCK
        //Unicode: U+2588, UTF-8: E2 96 88

        //center line
        Spans::from(vec![Span::raw(
            "
█████████████████████████████████████  •  ███████            █████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
████████████████████████████████████     ████████           ████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
██████████████████████████████████     ██████████        ███████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
████████████████████████████████     ███████████████████████████████",
        )]),
        Spans::from(vec![Span::raw(
            "
███████████████████████████████     ██████████████████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
█████████████████████████████     ████████████████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
██████████████████████████       █████████████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
██████████████████████           █████████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
███████████████████             ██████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
█████████████████             ████████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
████████████████           ███████████████
",
        )]),
        Spans::from(vec![Span::raw(
            "
████████████████       ███████████████",
        )]),
        Spans::from(vec![Span::raw(
            "
 ████████████████•████████████████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 ██████████████•██████████████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 ████████████•████████████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 █████████•█████████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 ███████•███████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 █████•█████ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 ███•███ ",
        )]),
        Spans::from(vec![Span::raw(
            "
 █•█ ",
        )]),
        //center line
        //Spans::from(vec![Span::raw("
        //███████████████████████████████████████•███████████████████████████████████████
        //")]),
        Spans::from(vec![Span::styled(
            "    ",
            Style::default().fg(Color::LightBlue),
        )]),
        Spans::from(vec![Span::raw("")]),
        //Spans::from(vec![Span::raw("Press 'p' to access pets, 'a' to add random new pets and 'd'
        // to delete the currently selected pet.")]),
    ])
    .alignment(Alignment::Center)
    .block(
        Block::default()
            .borders(Borders::ALL)
            //.style(Style::default().fg(Color::Magenta))
            //.style(Style::default().fg(Color::Black))
            .style(Style::default().fg(Color::White))
            //.style(Style::default().fg(Color::Rgb(100,1,1)))
            //.style(Style::default().fg(Color::Rgb(255,1,1)))
            //TODO git repo
            .title("  gnostr  ")
            .border_type(BorderType::Plain),
    );
    home
}

fn render_pets<'a>(db_path: &str, pet_list_state: &ListState) -> (List<'a>, Table<'a>) {
    let pets = Block::default()
        .borders(Borders::ALL)
        .style(Style::default().fg(Color::White))
        .title("Relays")
        .border_type(BorderType::Plain);

    let pet_list = read_db(db_path).expect("can fetch pet list");
    let items: Vec<_> = pet_list
        .iter()
        .map(|pet| {
            ListItem::new(Spans::from(vec![Span::styled(
                pet.name.clone(),
                Style::default(),
            )]))
        })
        .collect();

    let selected_pet = pet_list
        .get(
            pet_list_state
                .selected()
                .expect("there is always a selected pet"),
        )
        .expect("exists")
        .clone();

    let list = List::new(items).block(pets).highlight_style(
        Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    );

    let pet_detail = Table::new(vec![Row::new(vec![
        Cell::from(Span::raw(selected_pet.id.to_string())),
        Cell::from(Span::raw(selected_pet.name)),
        Cell::from(Span::raw(selected_pet.category)),
        Cell::from(Span::raw(selected_pet.age.to_string())),
        Cell::from(Span::raw(selected_pet.created_at.to_string())),
    ])])
    .header(Row::new(vec![
        Cell::from(Span::styled(
            "ID",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Name",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Category",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Age",
            Style::default().add_modifier(Modifier::BOLD),
        )),
        Cell::from(Span::styled(
            "Created At",
            Style::default().add_modifier(Modifier::BOLD),
        )),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .style(Style::default().fg(Color::White))
            .title("Detail")
            .border_type(BorderType::Plain),
    )
    .widths(&[
        Constraint::Percentage(5),
        Constraint::Percentage(20),
        Constraint::Percentage(20),
        Constraint::Percentage(5),
        Constraint::Percentage(20),
    ]);

    (list, pet_detail)
}

fn read_db(db_path: &str) -> Result<Vec<Pet>, Error> {
    let db_content = fs::read_to_string(db_path)?;
    if db_content.len() < 3 {
        let _ = add_random_pet_to_db(db_path);
    }
    let parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
    Ok(parsed)
}

fn add_random_pet_to_db(db_path: &str) -> Result<Vec<Pet>, Error> {
    let mut rng = rand::thread_rng();
    let db_content = fs::read_to_string(db_path)?;
    let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
    let catsdogs = match rng.gen_range(0, 1) {
        0 => "cats",
        _ => "dogs",
    };

    let random_pet = Pet {
        id: rng.gen_range(0, 9999999),
        name: rng.sample_iter(Alphanumeric).take(10).collect(),
        category: catsdogs.to_owned(),
        age: rng.gen_range(1, 15),
        created_at: Utc::now(),
    };

    parsed.push(random_pet);
    fs::write(db_path, &serde_json::to_vec(&parsed)?)?;
    Ok(parsed)
}

fn remove_pet_at_index(db_path: &str, pet_list_state: &mut ListState) -> Result<(), Error> {
    if let Some(selected) = pet_list_state.selected() {
        let db_content = fs::read_to_string(db_path)?;
        let mut parsed: Vec<Pet> = serde_json::from_str(&db_content)?;
        parsed.remove(selected);
        fs::write(db_path, &serde_json::to_vec(&parsed)?)?;
        let amount_pets = read_db(db_path).expect("can fetch pet list").len();
        if selected > 0 {
            pet_list_state.select(Some(selected - 1));
        } else {
            pet_list_state.select(Some(0));
        }
    }
    Ok(())
}
