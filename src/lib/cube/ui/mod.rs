use super::app::*;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use ratatui::{
    backend::Backend,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    symbols,
    text::Span,
    widgets::{
        Axis, Block, Borders, Cell, Chart, Dataset, GraphType, List, ListItem, Paragraph, Row,
        Table, Wrap,
    },
    Frame, Terminal,
};
use std::{
    env,
    error::Error,
    path::Path,
    time::{Duration, Instant},
};

const HELP_TEXT: &'static str = include_str!("../text/help.txt");
const WELCOME_TEXT: &'static str = include_str!("../text/welcome.txt");
const CUBE_TEXT: &'static str = include_str!("../text/cube.txt");

pub fn run<B: Backend>(terminal: &mut Terminal<B>) -> Result<(), Box<dyn Error>> {
    // Create app and load times
    let pathstr = env::var("HOME")? + "/.local/share/cube-tui/times";
    let path = Path::new(&pathstr);
    let mut app = App::new(Duration::from_millis(1000), path)?;
    app.load_times()?;

    // Main loop and tick logic
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|f| match app.route.screen {
            Screen::Default => render_default::<B>(f, &mut app),
            Screen::Help => render_help::<B>(f),
        })?;

        // Non-blocking key detection
        let timeout = app
            .tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));
        if event::poll(timeout)? {
            if handle_input(&mut app)? {
                return Ok(());
            }
        }
        if last_tick.elapsed() >= app.tick_rate {
            app.on_tick();
            last_tick = Instant::now();
        }
    }
}

fn handle_input(app: &mut App) -> Result<bool, Box<dyn Error>> {
    if let Event::Key(key) = event::read()? {
        match key.modifiers {
            KeyModifiers::NONE => match key.code {
                KeyCode::Char('q') => {
                    app.write_times()?;
                    return Ok(true);
                }
                KeyCode::Char(' ') => match app.timer.space_press() {
                    Some(mut t) => {
                        t.gen_stats(&app.times.times);
                        app.times.insert(t);
                        app.tick_rate = Duration::from_millis(1000);
                        app.new_scramble();
                    }
                    None => app.tick_rate = Duration::from_millis(100),
                },
                KeyCode::Esc => app.esc(),
                KeyCode::Enter => app.route.enter(),
                KeyCode::Char('h') | KeyCode::Left => app.mv(Dir::Left),
                KeyCode::Char('j') | KeyCode::Down => app.mv(Dir::Down),
                KeyCode::Char('k') | KeyCode::Up => app.mv(Dir::Up),
                KeyCode::Char('l') | KeyCode::Right => app.mv(Dir::Right),
                KeyCode::Char('d') => app.del(),
                KeyCode::Char('?') => app.help(),
                KeyCode::Char('<') => app.help(),
                KeyCode::Char('>') => app.esc(),
                _ => (),
            },
            KeyModifiers::CONTROL => match key.code {
                KeyCode::Char('w') => {
                    app.write_times()?;
                    app.load_times()?;
                }
                KeyCode::Char('c') => app.esc(),
                KeyCode::Char('q') => {
                    app.write_times()?;
                    return Ok(true);
                }
                _ => (),
            },
            _ => (),
        }
    }
    Ok(false)
}

//fn render_default<B: Backend>(f: &mut Frame<B>, app: &mut App) {
fn render_default<B: Backend>(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .split(f.area());

    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),       //help and tools height
                Constraint::Length(3),       //timer
                Constraint::Percentage(100), //table
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3),       //topic
                Constraint::Length(10),      //squares
                Constraint::Percentage(100), //tools view
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    render_help_and_tools::<B>(f, app, left_chunks[0]);
    render_timer::<B>(f, app, left_chunks[1]);
    render_times::<B>(f, app, left_chunks[2]);

    render_bests::<B>(f, app, right_chunks[0]);
    render_topic::<B>(f, app, right_chunks[1]);
    render_main::<B>(f, app, right_chunks[2]);
}

//fn render_help<B: Backend>(f: &mut Frame<B>) {
fn render_help<B: Backend>(f: &mut Frame) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(100)].as_ref())
        .split(f.area());

    let paragraph = Paragraph::new(HELP_TEXT)
        .block(Block::default().title(" > ").borders(Borders::ALL))
        .alignment(Alignment::Left);
    f.render_widget(paragraph, chunks[0]);
}

fn render_help_and_tools<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref())
        .split(layout_chunk);

    let border_style = app.get_border_style_from_id(ActiveBlock::Help);
    let paragraph = Paragraph::new("<?".to_string())
        .block(
            Block::default()
                .title("")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, chunks[0]);

    let border_style = app.get_border_style_from_id(ActiveBlock::Tools);
    let selected_style = app.get_highlight_style_from_id(ActiveBlock::Tools);
    let items = [
        ListItem::new(Tool::Gnostr.to_string()),
        ListItem::new(Tool::Relay.to_string()),
        ListItem::new(Tool::Commit.to_string()),
    ];
    let list = List::new(items)
        .block(
            Block::default()
                .title(" Tools ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White))
        .highlight_style(selected_style);

    f.render_stateful_widget(list, chunks[1], &mut app.tools_state);
}

fn render_timer<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let text = format!("{}", app.timer.text());
    let borderstyle = app.get_border_style_from_id(ActiveBlock::Timer);
    let mut paragraphstyle = Style::default();
    paragraphstyle = match app.timer.on {
        false => match app.timer.lasttime {
            Some(_) => paragraphstyle.fg(Color::White),
            None => paragraphstyle.fg(Color::Gray),
        },
        true => paragraphstyle.fg(Color::Magenta),
    };
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title(" Start/Stop ")
                .borders(Borders::ALL)
                .border_style(borderstyle),
        )
        .style(paragraphstyle)
        .alignment(Alignment::Left)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, layout_chunk);
}

fn render_times<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let selected_style = app.get_highlight_style_from_id(ActiveBlock::Times);
    let normal_style = Style::default().fg(Color::Gray);
    let header_cells = ["i", "time", "ao5", "ao12"].iter().map(|h| Cell::from(*h));
    let header = Row::new(header_cells)
        .style(normal_style)
        .height(1)
        .bottom_margin(1);
    let numrows = app.times.times.len();
    let rows = app.times.times.iter().rev().enumerate().map(|(i, t)| {
        let ao5 = match t.ao5 {
            Some(v) => format!("{:.2}", v),
            None => "-".to_string(),
        };
        let ao12 = match t.ao12 {
            Some(v) => format!("{:.2}", v),
            None => "-".to_string(),
        };
        let cells = vec![
            (numrows - i).to_string(),
            format!("{:.2}", t.time),
            format!("{}", ao5),
            format!("{}", ao12),
        ];
        Row::new(cells)
    });
    let border_style = app.get_border_style_from_id(ActiveBlock::Times);
    let table = Table::new(rows, &[])
        .header(header)
        .block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Stream ")
                .border_style(border_style),
        )
        .highlight_style(selected_style)
        .widths(&[
            Constraint::Ratio(1, 10),
            Constraint::Ratio(3, 10),
            Constraint::Ratio(3, 10),
            Constraint::Ratio(3, 10),
        ]);
    f.render_stateful_widget(table, layout_chunk, &mut app.times_state);
}

fn render_topic<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let border_style = app.get_border_style_from_id(ActiveBlock::Scramble);
    let paragraph = Paragraph::new(format!("\n{}", app.scramble.clone()))
        .block(
            Block::default()
                .title(" Meta/Header/Topic ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, layout_chunk);
}

fn render_bests<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                Constraint::Ratio(1, 3),
                //Constraint::Ratio(1, 6),
                //Constraint::Ratio(1, 6),
                //Constraint::Ratio(1, 6),
            ]
            .as_ref(),
        )
        .split(layout_chunk);

    render_stat::<B>(f, app, "Stats 1", app.times.pbsingle, chunks[0]);
    render_stat::<B>(f, app, "Stats 1", app.times.pbsingle, chunks[0]);
    render_stat::<B>(f, app, "Stats 2", app.times.pbao5, chunks[1]);
    render_stat::<B>(f, app, "Stats 3", app.times.pbao12, chunks[2]);
    //render_stat(f, app, "Stats 4", app.times.ao100, chunks[3]);
    //render_stat(f, app, "Stats 5", app.times.ao1k, chunks[4]);
    //render_stat(f, app, "Stats 6", app.times.rollingavg, chunks[5]);
}

fn render_stat<B: Backend>(
    f: &mut Frame,
    app: &mut App,
    title: &str,
    stat: Option<f64>,
    layout_chunk: Rect,
) {
    let border_style = app.get_border_style_from_id(ActiveBlock::Stats);
    let text = match stat {
        Some(v) => format!("{:.2}", v),
        None => "n/a".to_string(),
    };
    let paragraph = Paragraph::new(text)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true });
    f.render_widget(paragraph, layout_chunk);
}

fn render_main<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    match app.active_tool {
        Tool::Gnostr => render_gnostr_chat::<B>(f, app, layout_chunk),
        Tool::Relay => render_relay::<B>(f, app, layout_chunk),
        Tool::Commit => render_cube::<B>(f, app, layout_chunk),
    }
}

fn render_gnostr_chat<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let border_style = app.get_border_style_from_id(ActiveBlock::Main);
    let paragraph = Paragraph::new(WELCOME_TEXT)
        .block(
            Block::default()
                .title(" gnostr ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, layout_chunk);
}

fn render_cube<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let border_style = app.get_border_style_from_id(ActiveBlock::Main);
    let paragraph = Paragraph::new(CUBE_TEXT)
        .block(
            Block::default()
                .title(" Commit ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, layout_chunk);
}

fn render_relay<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let singles = app
        .times
        .times
        .iter()
        .enumerate()
        .map(|(i, v)| (i as f64, v.time))
        .collect::<Vec<(f64, f64)>>();
    let ao5s = &app
        .times
        .iter()
        .enumerate()
        .filter_map(|(i, v)| match v.ao5 {
            Some(a) => Some((i as f64, a)),
            None => None,
        })
        .collect::<Vec<(f64, f64)>>();
    let ao12s = &app
        .times
        .iter()
        .enumerate()
        .filter_map(|(i, v)| match v.ao12 {
            Some(a) => Some((i as f64, a)),
            None => None,
        })
        .collect::<Vec<(f64, f64)>>();

    let border_style = app.get_border_style_from_id(ActiveBlock::Main);
    let datasets = vec![
        Dataset::default()
            .name("single")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Cyan))
            .data(&singles),
        Dataset::default()
            .name("ao5")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::LightGreen))
            .data(&ao5s),
        Dataset::default()
            .name("ao12")
            .marker(symbols::Marker::Dot)
            .graph_type(GraphType::Line)
            .style(Style::default().fg(Color::Magenta))
            .data(&ao12s),
    ];

    let xmid = app.times.times.len() / 2;
    let xmax = app.times.times.len();
    let xmid_str = xmid.to_string();
    let xmax_str = xmax.to_string();

    let ymin = app.times.pbsingle.unwrap_or(0.0);
    let ymax = app.times.worst;
    let ymid = app.times.rollingavg.unwrap_or(0.0);
    let ymin_str = format!("{:.1}", ymin);
    let ymid_str = format!("{:.1}", ymid);
    let ymax_str = format!("{:.1}", ymax);

    let relay = Chart::new(datasets) //relay io
        .block(
            Block::default()
                .title(" Relay ")
                .border_style(border_style)
                .borders(Borders::ALL),
        )
        .x_axis(
            Axis::default()
                .title(Span::styled("n", Style::default()))
                .style(Style::default().fg(Color::White))
                .bounds([0.0, app.times.times.len() as f64])
                .labels(
                    ["0", &xmid_str, &xmax_str]
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect::<Vec<_>>(),
                ),
        )
        .y_axis(
            Axis::default()
                .title(Span::styled("Time", Style::default()))
                .style(Style::default().fg(Color::White))
                .bounds([ymin, ymax])
                .labels(
                    [ymin_str, ymid_str, ymax_str]
                        .iter()
                        .cloned()
                        .map(Span::from)
                        .collect::<Vec<_>>(),
                ),
        );
    f.render_widget(relay, layout_chunk);
}
