use super::cube::gen_scramble;
use super::global_rt::global_rt;
use ordered_float::*;
use std::{
    error::Error,
    fmt::{self, Formatter},
    fs,
    path::Path,
    time::{Duration, Instant},
};
use ratatui::{
    style::{Color, Modifier, Style},
    widgets::{ListState, TableState},
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

    pub fn esc(&mut self) {
        if self.active_block != ActiveBlock::Home {
            self.active_block = ActiveBlock::Home;
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
    Help,
    Timer,
    Times,
    Scramble,
    Stats,
    Home,
    Main,
}

#[derive(Clone, Copy)]
pub struct Time {
    pub time: f64,
    pub ao5: Option<f64>,
    pub ao12: Option<f64>,
}

impl Time {
    pub fn from(time: f64) -> Self {
        Self {
            time,
            ao5: None,
            ao12: None,
        }
    }

    pub fn gen_stats(&mut self, times: &Vec<Time>) {
        let mut tr = times.clone();
        tr.push(*self);
        tr.reverse();

        self.ao5 = if tr.len() >= 5 {
            let set = &tr[0..5];
            Some(Times::calc_aon(set))
        } else {
            None
        };
        self.ao12 = if tr.len() >= 12 {
            let set = &tr[0..12];
            Some(Times::calc_aon(set))
        } else {
            None
        };
    }
}

impl std::fmt::Display for Time {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        f.write_str(&self.time.to_string())?;
        Ok(())
    }
}

pub struct Times {
    pub times: Vec<Time>,
    pub pbsingle: Option<f64>,
    pub pbao5: Option<f64>,
    pub pbao12: Option<f64>,
    pub ao100: Option<f64>,
    pub ao1k: Option<f64>,
    pub rollingavg: Option<f64>,
    pub sum: f64,
    pub worst: f64,
}

impl Times {
    pub fn new() -> Self {
        Self {
            times: vec![],
            pbsingle: None,
            pbao5: None,
            pbao12: None,
            ao100: None,
            ao1k: None,
            rollingavg: None,
            sum: 0.0,
            worst: 0.0,
        }
    }

    pub fn insert(&mut self, time: Time) {
        self.times.push(time);
        Times::update_best(&mut self.pbsingle, Some(time.time));
        Times::update_best(&mut self.pbao5, time.ao5);
        Times::update_best(&mut self.pbao12, time.ao12);

        if self.times.len() >= 100 {
            let mut tr = self.times.clone();
            tr.reverse();
            self.ao100 = Some(Times::calc_aon(&tr[0..100]));
            if self.times.len() >= 1000 {
                self.ao1k = Some(Times::calc_aon(&tr[0..1000]));
            }
        }

        self.sum += time.time;

        self.rollingavg = match self.rollingavg {
            Some(_) => Some(self.sum / self.times.len() as f64),
            None => Some(time.time),
        };
        if time.time > self.worst {
            self.worst = time.time;
        }
    }

    fn update_best(curr: &mut Option<f64>, t: Option<f64>) {
        let new = match t {
            Some(x) => x,
            None => return,
        };

        match curr {
            Some(v) => {
                if new < *v {
                    *curr = Some(new);
                }
            }
            None => *curr = Some(new),
        }
    }

    fn calc_aon(set: &[Time]) -> f64 {
        let mut t = set
            .iter()
            .take(set.len())
            .map(|v| OrderedFloat(v.time))
            .collect::<Vec<OrderedFloat<f64>>>();
        // Remove best and worst time
        t.sort();
        t.pop();
        t.remove(0);

        let mut sum = OrderedFloat(0.0);
        let _ = t.iter().map(|v| sum += v).collect::<Vec<()>>();
        sum.into_inner() / t.len() as f64
    }

    pub fn iter(&self) -> TimesIterator {
        TimesIterator {
            curr: 0,
            times: &self.times,
        }
        .into_iter()
    }
}

pub struct TimesIterator<'a> {
    curr: usize,
    times: &'a Vec<Time>,
}

impl<'a> Iterator for TimesIterator<'a> {
    type Item = Time;
    fn next(&mut self) -> Option<Self::Item> {
        self.curr += 1;
        self.times.get(self.curr).copied()
    }
}

#[derive(Debug)]
pub struct CubeTimer {
    pub starttime: Option<Instant>,
    pub on: bool,
    pub lasttime: Option<Duration>,
}

impl CubeTimer {
    pub fn default() -> Self {
        Self {
            starttime: None,
            on: false,
            lasttime: None,
        }
    }

    pub fn space_press(&mut self) -> Option<Time> {
        match self.on {
            false => {
                self.timer_on();
                None
            }
            true => Some(self.timer_off()),
        }
    }

    fn timer_on(&mut self) {
        self.on = true;
        self.starttime = Some(Instant::now());
    }

    fn timer_off(&mut self) -> Time {
        self.on = false;
        self.lasttime = Some(self.elapsed());
        self.starttime = None;
        Time::from(
            self.lasttime
                .unwrap_or(Duration::from_secs(0))
                .as_secs_f64(),
        )
    }

    fn elapsed(&self) -> Duration {
        match self.starttime {
            Some(v) => v.elapsed(),
            None => Duration::new(0, 0),
        }
    }

    pub fn text(&self) -> String {
        match self.starttime {
            Some(v) => format!("{:.1}", v.elapsed().as_secs_f64()),
            None => format!(
                "{:.3}",
                self.lasttime
                    .unwrap_or(Duration::from_secs(0))
                    .as_secs_f64()
            ),
        }
    }
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

pub struct App<'a> {
    pub tick_rate: Duration,
    pub timer: CubeTimer,
    pub route: Route,
    pub path: &'a Path,
    pub pos: (usize, usize),
    pub times: Times,
    pub times_state: TableState,
    pub tools_state: ListState,
    layout: Vec<Vec<ActiveBlock>>,
    pub scramble: String,
    pub tools: Vec<Tool>,
    pub active_tool: Tool,
}

impl<'a> App<'a> {
    pub fn new(tick_rate: Duration, path: &'a Path) -> Result<Self, Box<dyn Error>> {
        // Setup state
        let mut tools_state = ListState::default();
        tools_state.select(Some(0));

        // Construct app
        Ok(App {
            tick_rate,
            timer: CubeTimer::default(),
            route: Route::default(),
            path,
            times: Times::new(),
            times_state: TableState::default(),
            tools_state,
            //this plus default active block determine initial navigation move
            pos: (0, 0),
            layout: vec![//effects nav order
                vec![ActiveBlock::Tools, ActiveBlock::Timer, ActiveBlock::Times],
                vec![ActiveBlock::Stats, ActiveBlock::Scramble, ActiveBlock::Main],
            ],
            scramble: gen_scramble(),
            tools: vec![Tool::Gnostr, Tool::Relay, Tool::Commit],
            active_tool: Tool::Gnostr,
        })
    }

    pub fn load_times(&mut self) -> Result<(), Box<dyn Error>> {
        let directory = self.path.with_file_name("");
        fs::create_dir_all(directory)?;

        // Create file if it doesn't exist
        match fs::File::open(&self.path) {
            Err(_) => _ = fs::File::create(&self.path)?,
            Ok(_) => (),
        };

        let mut times: Vec<Time> = fs::read_to_string(&self.path)?
            .lines()
            .filter_map(|v| v.parse::<f64>().ok())
            .map(|v| Time::from(v))
            .collect();

        self.times = Times::new();
        for time in &mut times {
            time.gen_stats(&self.times.times);
            self.times.insert(*time);
        }
        Ok(())
    }

    pub fn write_times(&self) -> Result<(), Box<dyn Error>> {
        let write_data: Vec<u8> = self
            .times
            .times
            .iter()
            .flat_map(|v| format!("{}\n", v.to_string()).bytes().collect::<Vec<u8>>())
            .collect();
        fs::write(&self.path, write_data)?;
        Ok(())
    }

    pub fn esc(&mut self) {
        match self.route.screen {
            Screen::Default => self.route.esc(),
            Screen::Help => self.route.screen = Screen::Default,
        }
    }

    pub fn help(&mut self) {
        self.route.screen = Screen::Help;
    }

    pub fn get_border_style_from_id(&self, id: ActiveBlock) -> Style {
        let style = Style::default();
        if id == self.route.active_block {
            return style.fg(Color::Magenta).add_modifier(Modifier::BOLD | Modifier::ITALIC);
        } else if id == self.route.selected_block {
            return style.fg(Color::Magenta);//.add_modifier(Modifier::BOLD);
        } else {
            return style.fg(Color::Gray);
        }
    }

    pub fn get_highlight_style_from_id(&self, id: ActiveBlock) -> Style {
        let style = Style::default();
        if id == self.route.active_block {
            return style.fg(Color::Magenta);
        } else if id == self.route.selected_block {
            return style.fg(Color::Magenta).add_modifier(Modifier::BOLD | Modifier::ITALIC)
        } else {
            return style.fg(Color::Magenta);
        }
    }

    pub fn del(&mut self) {
        match self.route.active_block {
            ActiveBlock::Times => self.del_time(),
            _ => (),
        }
    }

    pub fn mv(&mut self, dir: Dir) {
        match self.route.active_block {
            ActiveBlock::Times => match dir {
                Dir::Up => self.previous_time(),
                Dir::Down => self.next_time(),
                Dir::Right => {
                    self.route.active_block = ActiveBlock::Home;
                    self.mv(Dir::Right);
                }
                Dir::Left => {
                    self.route.active_block = ActiveBlock::Home;
                    self.mv(Dir::Left);
                }
            },
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
        if (self.pos.1) as i32 - 1 >= 0 {
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
        if (self.pos.0) as i32 - 1 >= 0 {
            self.pos.0 -= 1;
        }
    }

    pub fn next_time(&mut self) {
        let len = self.times.times.len();
        if len == 0 {
            return;
        }
        let i = match self.times_state.selected() {
            Some(i) => {
                if i >= self.times.times.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.times_state.select(Some(i));
    }

    fn previous_time(&mut self) {
        let len = self.times.times.len();
        if len == 0 {
            return;
        }
        let i = match self.times_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.times.times.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.times_state.select(Some(i));
    }

    fn del_time(&mut self) {
        match self.times_state.selected() {
            Some(v) => {
                // Edge cases (literally)
                let len = self.times.times.len();
                if len <= 0 || v >= len {
                    return;
                }
                self.times.times.remove(len - v - 1);
                // Go up one if selection fell off
                if v == self.times.times.len() {
                    self.previous_time();
                }
            }
            None => (),
        };
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

    pub fn new_scramble(&mut self) {
        self.scramble = gen_scramble();
    }

    pub fn on_tick(&self) {
        ()
    }
}
