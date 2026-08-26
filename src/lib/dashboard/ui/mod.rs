use super::app::*;
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use std::{
    error::Error,
    time::Duration,
};
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




use crate::dashboard::handlers::event::{Config, Event as AppEvent, Events, Key};

pub async fn run<B: Backend>(
    terminal: &mut Terminal<B>,
    mut app: App,
) -> Result<(), Box<dyn Error>> {
    let mut events = Events::with_config(Config {
        exit_key: Key::Null,
        tick_rate: Duration::from_millis(100),
    })
    .await;

    loop {
        terminal.draw(|f| render_default::<B>(f, &mut app))?;

        tokio::select! {
            Some(event) = events.next() => {
                if handle_input(&mut app, event)? {
                    return Ok(());
                }
            }
            Some(p2p_event) = app.p2p_events_rx.recv() => {
                match p2p_event {
                    P2pEvent::PeerConnected(peer_id) => {
                        app.peers.push(peer_id);
                    }
                    P2pEvent::PeerDisconnected(peer_id) => {
                        app.peers.retain(|p| p != &peer_id);
                    }
                }
            }
        }
    }
}

fn handle_input(app: &mut App, event: AppEvent<Key>) -> Result<bool, Box<dyn Error>> {
    if let AppEvent::Input(key) = event {
        match key {
            Key::Char('q') => return Ok(true),
            Key::Left => app.mv(Dir::Left),
            Key::Down => app.mv(Dir::Down),
            Key::Up => app.mv(Dir::Up),
            Key::Right => app.mv(Dir::Right),
            Key::Enter => app.route.enter(),
            _ => {}
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
                Constraint::Length(3),       //tools
                Constraint::Percentage(100), //peers
            ]
            .as_ref(),
        )
        .split(chunks[0]);

    let right_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage(100), //main
            ]
            .as_ref(),
        )
        .split(chunks[1]);

    render_tools::<B>(f, app, left_chunks[0]);
    render_peers::<B>(f, app, left_chunks[1]);

    render_main::<B>(f, app, right_chunks[0]);
}

fn render_peers<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let border_style = app.get_border_style_from_id(ActiveBlock::Peers);
    let items: Vec<ListItem> = app
        .peers
        .iter()
        .map(|p| ListItem::new(p.to_string()))
        .collect();
    let list = List::new(items)
        .block(
            Block::default()
                .title(" Peers ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .style(Style::default().fg(Color::White));

    f.render_widget(list, layout_chunk);
}

//fn render_help<B: Backend>(f: &mut Frame<B>) {


fn render_tools<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
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

    f.render_stateful_widget(list, layout_chunk, &mut app.tools_state);
}



fn render_main<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    match app.active_tool {
        Tool::Gnostr => render_gnostr_chat::<B>(f, app, layout_chunk),
        Tool::Relay => render_relay::<B>(f, app, layout_chunk),
        Tool::Commit => render_commit_diff::<B>(f, app, layout_chunk),
    }
}

fn render_gnostr_chat<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let border_style = app.get_border_style_from_id(ActiveBlock::Main);
    let paragraph = Paragraph::new("Gnostr")
        .block(
            Block::default()
                .title(" gnostr ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, layout_chunk);
}

fn render_commit_diff<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let border_style = app.get_border_style_from_id(ActiveBlock::Main);
    let paragraph = Paragraph::new("")
        .block(
            Block::default()
                .title(" Commit Diff ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, layout_chunk);
}

fn render_relay<B: Backend>(f: &mut Frame, app: &mut App, layout_chunk: Rect) {
    let border_style = app.get_border_style_from_id(ActiveBlock::Main);
    let paragraph = Paragraph::new("Relay Info Placeholder")
        .block(
            Block::default()
                .title(" Relay ")
                .borders(Borders::ALL)
                .border_style(border_style),
        )
        .alignment(Alignment::Left);
    f.render_widget(paragraph, layout_chunk);
}
