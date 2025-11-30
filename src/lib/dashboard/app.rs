
//use super::global_rt::global_rt;

use ratatui::{
    style::{Color, Modifier, Style},
    widgets::ListState,
};
use std::{
    error::Error,
    fmt::{self, Formatter},
    time::{Duration},
};

pub struct Route {
    pub screen: Screen,
    pub selected_block: ActiveBlock,
    pub active_block: ActiveBlock,
}

impl Route {
    fn default() -> Self {
        Self {
            screen: Screen::Default,
            selected_block: ActiveBlock::Tools,
            //this plus new pos (line 329-ish) determine initial navigation move
            active_block: ActiveBlock::Tools,
        }
    }

    pub fn enter(&mut self) {
        self.active_block = self.selected_block;
    }
}

pub enum Screen {
    Default,
    Help,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum ActiveBlock {
    Tools,
    Peers,
    Network,
    Stats,
    Home,
    Main,
}



#[derive(Copy, Clone)]
pub enum Tool {
    Gnostr,
    Relay,
    Commit,
}

impl fmt::Display for Tool {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), fmt::Error> {
        let text = match self {
            Tool::Gnostr => "Gnostr",
            Tool::Relay => "Relay",
            Tool::Commit => "Commit",
        };
        write!(f, "{}", text)?;
        Ok(())
    }
}

pub enum Dir {
    Up,
    Down,
    Left,
    Right,
}

pub struct App {
    pub tick_rate: Duration,
    pub route: Route,
    pub pos: (usize, usize),
    pub tools_state: ListState,
    layout: Vec<Vec<ActiveBlock>>,
    pub tools: Vec<Tool>,
    pub active_tool: Tool,
    pub p2p_events_rx: tokio::sync::mpsc::Receiver<P2pEvent>,
    pub peers: Vec<String>,
}

pub enum P2pEvent {
    PeerConnected(String),
    PeerDisconnected(String),
}

impl App {
    pub fn new(
        tick_rate: Duration,
        p2p_events_rx: tokio::sync::mpsc::Receiver<P2pEvent>,
    ) -> Result<Self, Box<dyn Error>> {
        // Setup state
        let mut tools_state = ListState::default();
        tools_state.select(Some(0));

        // Construct app
        Ok(App {
            tick_rate,
            route: Route::default(),
            tools_state,
            pos: (0, 0),
            layout: vec![
                vec![ActiveBlock::Tools, ActiveBlock::Peers, ActiveBlock::Network],
                vec![ActiveBlock::Stats, ActiveBlock::Main, ActiveBlock::Main],
            ],
            tools: vec![Tool::Gnostr, Tool::Relay, Tool::Commit],
            active_tool: Tool::Gnostr,
            p2p_events_rx,
            peers: Vec::new(),
        })
    }


    pub fn help(&mut self) {
        self.route.screen = Screen::Help;
    }

    pub fn get_border_style_from_id(&self, id: ActiveBlock) -> Style {
        let style = Style::default();
        if id == self.route.active_block {
            style
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
        } else if id == self.route.selected_block {
            style.fg(Color::Magenta) //.add_modifier(Modifier::BOLD);
        } else {
            style.fg(Color::Gray)
        }
    }

    pub fn get_highlight_style_from_id(&self, id: ActiveBlock) -> Style {
        let style = Style::default();
        if id == self.route.active_block {
            style.fg(Color::Magenta)
        } else if id == self.route.selected_block {
            style
                .fg(Color::Magenta)
                .add_modifier(Modifier::BOLD | Modifier::ITALIC)
        } else {
            style.fg(Color::Magenta)
        }
    }

    pub fn mv(&mut self, dir: Dir) {
        match self.route.active_block {
            ActiveBlock::Tools => match dir {
                Dir::Up => self.next_tool(),
                Dir::Down => self.previous_tool(),
                Dir::Right => {
                    self.route.active_block = ActiveBlock::Home;
                    self.mv(Dir::Right);
                }
                Dir::Left => {
                    self.route.active_block = ActiveBlock::Home;
                    self.mv(Dir::Left);
                }
            },
            _ => {
                match dir {
                    Dir::Up => self.mv_up(),
                    Dir::Down => self.mv_down(),
                    Dir::Right => self.mv_right(),
                    Dir::Left => self.mv_left(),
                }
                self.route.active_block = ActiveBlock::Home;
                self.route.selected_block = self.layout[self.pos.0][self.pos.1];
            }
        }
    }

    fn mv_up(&mut self) {
        if (self.pos.1) as i32 > 0 {
            self.pos.1 -= 1;
        }
    }

    fn mv_down(&mut self) {
        if self.pos.1 + 1 < self.layout[self.pos.0].len() {
            self.pos.1 += 1;
        }
    }

    fn mv_right(&mut self) {
        if self.layout.len() > self.pos.0 + 1 {
            let max = self.layout[self.pos.0 + 1].len() - 1;
            if self.pos.1 + 1 > max {
                self.pos.1 = max;
            }
            self.pos.0 += 1;
        }
    }

    pub fn mv_left(&mut self) {
        if (self.pos.0) as i32 > 0 {
            self.pos.0 -= 1;
        }
    }
    pub fn next_tool(&mut self) {
        let i = match self.tools_state.selected() {
            Some(i) => {
                if i >= self.tools.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.tools_state.select(Some(i));
        self.active_tool = self.tools[self.tools_state.selected().unwrap_or(0)];
    }

    fn previous_tool(&mut self) {
        let i = match self.tools_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.tools.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.tools_state.select(Some(i));
        self.active_tool = self.tools[self.tools_state.selected().unwrap_or(0)];
    }

    pub fn on_tick(&self) {

    }
}
