use std::{
    collections::HashMap,
    io,
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    time::{Duration, Instant, SystemTime},
};

use crossterm::{
    event::{self, Event as CEvent, KeyCode, KeyEventKind},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Wrap, Clear},
};
use time::{OffsetDateTime, macros::format_description};

use crate::ActivePeerState;

#[rustfmt::skip]
const BITCOIN_LOGO: [&str; 15] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⣀⣤⣴⣶⣾⣿⣿⣿⣿⣷⣶⣦⣤⣀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⣠⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⣄⠀⠀⠀⠀⠀",
    "⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠀⠀",
    "⠀⠀⣴⣿⣿⣿⣿⣿⣿⣿⠟⠿⠿⡿⠀⢰⣿⠁⢈⣿⣿⣿⣿⣿⣿⣿⣿⣦⠀⠀",
    "⠀⣼⣿⣿⣿⣿⣿⣿⣿⣿⣤⣄⠀⠀⠀⠈⠉⠀⠸⠿⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀",
    "⢰⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⢠⣶⣶⣤⡀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⡆",
    "⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃⠀⠀⠼⣿⣿⡿⠃⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣷",
    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡟⠀⠀⢀⣀⣀⠀⠀⠀⠀⢴⣿⣿⣿⣿⣿⣿⣿⣿⣿",
    "⢿⣿⣿⣿⣿⣿⣿⣿⢿⣿⠁⠀⠀⣼⣿⣿⣿⣦⠀⠀⠈⢻⣿⣿⣿⣿⣿⣿⣿⡿",
    "⠸⣿⣿⣿⣿⣿⣿⣏⠀⠀⠀⠀⠀⠛⠛⠿⠟⠋⠀⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⠇",
    "⠀⢻⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⣤⡄⠀⣀⣀⣀⣀⣠⣾⣿⣿⣿⣿⣿⣿⣿⡟⠀",
    "⠀⠀⠻⣿⣿⣿⣿⣿⣿⣿⣄⣰⣿⠁⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠀⠀",
    "⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠙⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠋⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠻⠿⢿⣿⣿⣿⣿⡿⠿⠟⠛⠉⠀⠀⠀⠀⠀⠀⠀⠀",
];

const _LOGO_HEIGHT: u16 = 15;
const _LOGO_WIDTH: u16 = 80;

#[rustfmt::skip]
const _BITCOIN_ICON: [&str; 9] = [
   "⠀⠀⠀⠀⣿⡇⠀⢸⣿⡇⠀⠀⠀⠀",
   "⠸⠿⣿⣿⣿⡿⠿⠿⣿⣿⣿⣶⣄⠀",
   "⠀⠀⢸⣿⣿⡇⠀⠀⠀⠈⣿⣿⣿⠀",
   "⠀⠀⢸⣿⣿⡇⠀⠀⢀⣠⣿⣿⠟⠀",
   "⠀⠀⢸⣿⣿⡿⠿⠿⠿⣿⣿⣥⣄⠀",
   "⠀⠀⢸⣿⣿⡇⠀⠀⠀⠀⢻⣿⣿⣧",
   "⠀⠀⢸⣿⣿⡇⠀⠀⠀⠀⣼⣿⣿⣿",
   "⢰⣶⣿⣿⣿⣷⣶⣶⣾⣿⣿⠿⠛⠁",
   "⠀⠀⠀⠀⣿⡇⠀⢸⣿⡇⠀⠀⠀⠀",
];

#[rustfmt::skip]
const BITCOIN_LOGO_LARGE: [&str; 30] = [
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣀⣠⣤⣴⣶⣶⣿⣿⣿⣿⣿⣿⣿⣿⣶⣶⣶⣤⣄⣀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣠⣴⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣦⣄⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⢀⣤⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣤⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠉⠛⠛⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣄⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⣴⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⢰⣿⣿⠇⠀⠉⠉⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣦⠀⠀⠀⠀⠀",
    "⠀⠀⠀⢀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠉⠉⠛⠛⠿⠿⡏⠀⠀⠀⣾⣿⡿⠀⠀⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡀⠀⠀⠀",
    "⠀⠀⢀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⠛⠃⠀⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⡀⠀⠀",
    "⠀⠀⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⣆⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠛⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⠀⠀",
    "⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⠀⠀⠀⢠⣶⣦⣤⣀⡀⠀⠀⠀⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣇⠀",
    "⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀⠀⣼⣿⣿⣿⣿⣿⣷⡄⠀⠀⠀⠀⠀⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡄",
    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⡿⠀⠀⠀⠀⠀⠀⣸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇",
    "⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀⠀⠘⠿⠿⢿⣿⣿⡿⠟⠁⠀⠀⠀⠀⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
    "⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⠀⠀⠀⣴⣶⣤⣤⣀⡀⠀⠀⠀⠀⠀⠀⠐⠿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
    "⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣷⣦⡀⠀⠀⠀⠀⠀⠘⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿",
    "⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠿⠿⢿⡿⠁⠀⠀⠀⠀⠀⣼⣿⣿⣿⣿⣿⣿⣿⣿⣧⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡇",
    "⠈⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠏⠀⠀⠀⠀⠀⠀⠀⠀⠀⠠⣿⣿⣿⣿⣿⣿⣿⣿⣿⠇⠀⠀⠀⠀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠃",
    "⠀⢹⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣯⣄⣀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠉⠉⠉⠉⠉⠀⠀⠀⠀⠀⠀⠀⢠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀",
    "⠀⠀⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣶⠀⠀⠀⢀⡀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⣠⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠀⠀",
    "⠀⠀⠈⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡏⠀⠀⠀⣾⣿⣿⠀⠀⠀⢠⣤⣄⣀⣀⣀⣀⣤⣴⣾⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠁⠀⠀",
    "⠀⠀⠀⠈⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠁⠀⠀⢰⣿⣿⡇⠀⠀⢀⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠁⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠻⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣶⣦⣾⣿⣿⡀⠀⠀⢸⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⠟⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣷⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠙⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠋⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠛⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠛⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠈⠙⠻⢿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⣿⡿⠟⠋⠁⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
    "⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠉⠙⠛⠻⠿⠿⢿⣿⣿⣿⣿⣿⣿⡿⠿⠿⠟⠛⠋⠉⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀⠀",
];
const _LOGO_LARGE_HEIGHT: u16 = 30;

pub enum Event<I> {
    Input(I),
    Tick,
}

#[derive(PartialEq, Copy, Clone)]
pub enum FocusedWidget {
    BlockHeight,
    Instructions,
    PeerList,
    Log,
    Input,
    Modal,
}

pub struct App {
    pub messages: Arc<Mutex<Vec<(String, SystemTime)>>>,
    pub scroll_state: u16,
    pub running: Arc<AtomicBool>,
    pub block_height: Arc<Mutex<i32>>,
    pub block_hash: Arc<Mutex<String>>,
    pub peer_list: Arc<Mutex<HashMap<String, ActivePeerState>>>,
    pub focused_widget: FocusedWidget,
    last_user_input_time: Instant,
    auto_scroll_enabled: bool,
    current_scroll_y: f32,       // Animated scroll position
    scroll_animation_speed: f32, // Controls animation speed
    log_widget_height: u16,
    pub log_visible: bool,
    peer_list_width_percentage: u16,
    pub splash_screen_shown: bool,
    pub input_text: String,
    pub cursor_position: usize,
    pub available_commands: Vec<String>,
    pub current_suggestions: Vec<String>,
    pub selected_suggestion_index: Option<usize>,
    pub show_send_tx_modal: bool,
    pub send_tx_modal_messages: Arc<Mutex<Vec<String>>>,
}

impl App {
        pub fn new(
            messages: Arc<Mutex<Vec<(String, SystemTime)>>>,
            running: Arc<AtomicBool>,
            block_height: Arc<Mutex<i32>>,
            block_hash: Arc<Mutex<String>>,
            peer_list: Arc<Mutex<HashMap<String, ActivePeerState>>>,
        ) -> App {
            App {
                messages,
                scroll_state: 0,
                running,
                block_height,
                block_hash,
                peer_list,
                focused_widget: FocusedWidget::Log,
                last_user_input_time: Instant::now(),
                auto_scroll_enabled: true,
                current_scroll_y: 0.0,       // Initialize animated scroll position
                scroll_animation_speed: 0.1, // Initialize animation speed
                log_widget_height: 0,
                log_visible: true,
                peer_list_width_percentage: 50,
                splash_screen_shown: false,
                input_text: String::new(),
                cursor_position: 0,
                available_commands: vec![
                    "sendrawtransaction".to_string(),
                    "getblockcount".to_string(),
                    "getbestblockhash".to_string(),
                    "getpeerinfo".to_string(),
                    "help".to_string(),
                ],
                current_suggestions: Vec::new(),
                            selected_suggestion_index: None,
                            show_send_tx_modal: false,
                            send_tx_modal_messages: Arc::new(Mutex::new(Vec::new())),
                        }        }
    
        fn update_suggestions(&mut self) {
            let input_lower = self.input_text.to_lowercase();
            self.current_suggestions = self.available_commands
                .iter()
                .filter(|cmd| cmd.to_lowercase().starts_with(&input_lower))
                .cloned()
                .collect();
            self.selected_suggestion_index = if self.current_suggestions.is_empty() { None } else { Some(0) };
        }

    pub fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<io::Stdout>>,
    ) -> anyhow::Result<()> {
        let (tx, rx) = mpsc::channel();
        let tick_rate = Duration::from_millis(250);
        let running_clone = self.running.clone();
        std::thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                if !running_clone.load(Ordering::SeqCst) {
                    break;
                }
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));

                if event::poll(timeout).unwrap_or(false) {
                    if let Ok(CEvent::Key(key)) = event::read() {
                        tx.send(Event::Input(key)).ok();
                    }
                }

                if last_tick.elapsed() >= tick_rate {
                    tx.send(Event::Tick).ok();
                    last_tick = Instant::now();
                }
            }
        });

        while self.running.load(Ordering::SeqCst) {
            terminal.draw(|f| {
                // ... (rendering logic)
                // Check if it's the first start and no peers are connected, and splash screen
                // hasn't been shown yet
                if !self.splash_screen_shown && self.peer_list.lock().unwrap().is_empty() {
                    // Render splash screen with BITCOIN_LOGO_LARGE to fill the screen
                    let logo_area = f.size(); // Use the full screen for the splash screen

                    let logo_lines: Vec<Line> = BITCOIN_LOGO_LARGE
                        .iter()
                        .map(|line| Line::from(Span::raw(*line)))
                        .collect();
                    let logo_widget = Paragraph::new(logo_lines)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Bitcoin P2P Client")
                                .border_style(Style::default().fg(Color::Yellow)),
                        ) // Neutral border style
                        .style(Style::default().fg(Color::Yellow)); // Neutral text style

                    // Render the logo widget to fill the entire area
                    f.render_widget(logo_widget, logo_area);

                    // Set splash_screen_shown to true after rendering it once
                    self.splash_screen_shown = true;
                } else {
                    // Existing UI rendering logic
                    let size = f.size();
                    let chunks = Layout::default()
                        .direction(Direction::Vertical)
                        .constraints(
                            [
                                Constraint::Length(3),
                                Constraint::Length(3),
                                Constraint::Min(0),
                                Constraint::Length(1), // For suggestions
                                Constraint::Length(3),
                            ]
                            .as_ref(),
                        )
                        .split(size);

                    let block_height_value = *self.block_height.lock().unwrap();
                    let block_hash_value = self.block_hash.lock().unwrap();
                    let user_agent = format!("UA: /Gnostr:{}/", env!("CARGO_PKG_VERSION"));
                    
                    let header_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                        .split(chunks[0]);

                    let block_height_widget = Paragraph::new("")
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(format!(
                                    "Block Height: {} Hash: {}",
                                    block_height_value, block_hash_value
                                ))
                                .border_style(match self.focused_widget {
                                    FocusedWidget::BlockHeight => {
                                        Style::default().fg(Color::Magenta)
                                    }
                                    _ => Style::default().fg(Color::White),
                                }),
                        )
                        .style(Style::default().fg(Color::Cyan));
                    f.render_widget(block_height_widget, header_chunks[0]);

                    let ua_widget = Paragraph::new(user_agent)
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("User Agent")
                                .border_style(Style::default().fg(Color::Green)),
                        )
                        .style(Style::default().fg(Color::Green));
                    f.render_widget(ua_widget, header_chunks[1]);

                    let instruction_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[1]);

                    let instruction_quit = Paragraph::new("Press 'q' to quit.")
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("Instructions")
                                .border_style(match self.focused_widget {
                                    FocusedWidget::Instructions => {
                                        Style::default().fg(Color::Magenta)
                                    }
                                    _ => Style::default().fg(Color::Yellow),
                                }),
                        )
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(instruction_quit, instruction_chunks[0]);

                    let instruction_scroll_up = Paragraph::new("Press 'Up' to scroll up.")
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title("")
                                .border_style(match self.focused_widget {
                                    FocusedWidget::Instructions => {
                                        Style::default().fg(Color::Magenta)
                                    }
                                    _ => Style::default().fg(Color::Yellow),
                                }),
                        )
                        .style(Style::default().fg(Color::Yellow));
                    f.render_widget(instruction_scroll_up, instruction_chunks[1]);

                    let bottom_half_chunks = Layout::default()
                        .direction(Direction::Horizontal)
                        .constraints(
                            [Constraint::Percentage(50), Constraint::Percentage(50)].as_ref(),
                        )
                        .split(chunks[2]);

                    if self.log_visible {
                        let messages = self.messages.lock().unwrap();
                        let num_messages = messages.len();
                        let log_area_height = bottom_half_chunks[0].height;
                        let log_height = {
                            let content_height = if num_messages == 0 { 1 } else { num_messages };
                            let desired_height = (content_height + 2) as u16; // +2 for borders
                            std::cmp::min(desired_height, log_area_height)
                        };
                        self.log_widget_height = log_height;

                        let formatted_messages: Vec<Line> = messages
                            .iter()
                            .map(|(msg, timestamp)| {
                                let offset_datetime: OffsetDateTime = (*timestamp).into();
                                let format = format_description!("[hour]:[minute]:[second]");
                                Line::from(Span::raw(format!(
                                    "[{}] {}",
                                    offset_datetime.format(&format).unwrap_or_default(),
                                    msg
                                )))
                            })
                            .collect();

                        let border_style = match self.focused_widget {
                            FocusedWidget::Log => Style::default().fg(Color::Magenta),
                            _ => Style::default().fg(Color::White),
                        };
                        let log_block = Block::default()
                            .borders(if self.peer_list_width_percentage == 0 {
                                Borders::NONE
                            } else {
                                Borders::ALL
                            })
                            .title("Bitcoin P2P Client Log")
                            .border_style(border_style);

                        let paragraph = Paragraph::new(formatted_messages)
                            .block(log_block)
                            .style(Style::default().fg(Color::White))
                            .wrap(Wrap { trim: true })
                            .scroll((self.current_scroll_y.round() as u16, 0));

                        f.render_widget(paragraph, bottom_half_chunks[0]);
                    } else {
                        // Log is hidden, display Bitcoin logo full-width
                        let logo_area = bottom_half_chunks[0]; // Use the area for the log panel, which is now full-width

                        let logo_lines: Vec<Line> = BITCOIN_LOGO
                            .iter()
                            .map(|line| Line::from(Span::raw(*line)))
                            .collect();
                        let logo_widget = Paragraph::new(logo_lines)
                            .block(
                                Block::default()
                                    .borders(if self.peer_list_width_percentage == 0 {
                                        Borders::NONE
                                    } else {
                                        Borders::ALL
                                    })
                                    .title("Bitcoin Logo")
                                    .border_style(Style::default().fg(Color::Yellow)),
                            ) // Neutral border style
                            .style(Style::default().fg(Color::Yellow)); // Neutral text style

                        // Render the logo widget to fill the available horizontal space
                        f.render_widget(logo_widget, logo_area);
                    }

                    // Render Peers in bottom_half_chunks[1] (conditionally)
                    if self.peer_list_width_percentage > 0 {
                        let peer_list_lock = self.peer_list.lock().unwrap(); // Lock the mutex once
                        if peer_list_lock.is_empty() {
                            // If the list is empty, show a "Connecting..." message
                            let connecting_message = vec![Line::from("Connecting to peers...")];
                            let connecting_widget = Paragraph::new(connecting_message)
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .title("Peers")
                                        .border_style(match self.focused_widget {
                                            FocusedWidget::PeerList => {
                                                Style::default().fg(Color::Magenta)
                                            }
                                            _ => Style::default().fg(Color::Yellow),
                                        }),
                                )
                                .style(Style::default().fg(Color::Yellow));
                            f.render_widget(connecting_widget, bottom_half_chunks[1]); // Assuming bottom_half_chunks[1] is correct for the peer list pane
                        } else {
                            // If the list is not empty, render the actual peer list
                            let mut sorted_peers: Vec<(&String, &ActivePeerState)> =
                                peer_list_lock.iter().collect();
                            sorted_peers.sort_by(|a, b| {
                                // Sort by inbound traffic (descending)
                                b.1.inbound_traffic
                                    .cmp(&a.1.inbound_traffic)
                                    // Then by connection time (ascending)
                                    .then_with(|| a.1.connection_time.cmp(&b.1.connection_time))
                            });

                            let peer_list_content: Vec<Line> = sorted_peers
                                .into_iter()
                                .map(|(peer_addr, state)| {
                                    Line::from(vec![
                                        Span::styled(format!("{}: ", peer_addr), Style::default().fg(Color::Cyan)),
                                        Span::raw(format!("V: {}, UA: {}, In: {} B, Out: {} B", 
                                            state.protocol_version, 
                                            state.user_agent,
                                            state.inbound_traffic, 
                                            state.outbound_traffic
                                        )),
                                    ])
                                })
                                .collect();

                            let peer_list_widget = Paragraph::new(peer_list_content)
                                .block(
                                    Block::default()
                                        .borders(Borders::ALL)
                                        .title("Peers")
                                        .border_style(match self.focused_widget {
                                            FocusedWidget::PeerList => {
                                                Style::default().fg(Color::Magenta)
                                            }
                                            _ => Style::default().fg(Color::Yellow),
                                        }),
                                )
                                .style(Style::default().fg(Color::Yellow));
                            f.render_widget(peer_list_widget, bottom_half_chunks[1]); // Assuming bottom_half_chunks[1] is correct for the peer list pane
                        }
                    } else {
                                                                                        // GEMINI we want to stretch the log widget
                                                                                        // to fill horizontally when
                                                                                        // self.peer_list_width_percentage = 0
                                                                                    }
                                                                
                                                                                                        // Render command suggestions
                                                                                                        if self.focused_widget == FocusedWidget::Input && !self.current_suggestions.is_empty() {
                                                                                                            let suggestions_text: Vec<Span> = self.current_suggestions.iter().enumerate().map(|(i, s)| {
                                                                                                                if Some(i) == self.selected_suggestion_index {
                                                                                                                    Span::styled(format!(" {} ", s), Style::default().fg(Color::Black).bg(Color::Gray))
                                                                                                                } else {
                                                                                                                    Span::raw(format!(" {} ", s))
                                                                                                                }
                                                                                                            }).collect();
                                                                                    
                                                                                                            let suggestions_widget = Paragraph::new(Line::from(suggestions_text))
                                                                                                                .block(Block::default().borders(Borders::NONE))
                                                                                                                .style(Style::default().fg(Color::White));
                                                                                                            f.render_widget(suggestions_widget, chunks[3]);
                                                                                                        }
                                                                                    
                                                                                                        // Render the input command widget
                                                                                                        let input_widget = Paragraph::new(self.input_text.as_str())
                                                                                                            .block(
                                                                                                                Block::default()
                                                                                                                    .borders(Borders::ALL)
                                                                                                                    .title("Command Input")
                                                                                                                    .border_style(match self.focused_widget {
                                                                                                                        FocusedWidget::Input => Style::default().fg(Color::Magenta),
                                                                                                                        _ => Style::default().fg(Color::White),
                                                                                                                    }),
                                                                                                            )
                                                                                                            .style(Style::default().fg(Color::White));
                                                                                                                            f.render_widget(input_widget, chunks[4]);
                                                                                                        
                                                                                                                            // Render the modal if active
                                                                                                                            if self.show_send_tx_modal {
                                                                                                                                let modal_area = Layout::default()
                                                                                                                                    .direction(Direction::Vertical)
                                                                                                                                    .constraints(
                                                                                                                                        [
                                                                                                                                            Constraint::Percentage(30),
                                                                                                                                            Constraint::Percentage(40),
                                                                                                                                            Constraint::Percentage(30),
                                                                                                                                        ]
                                                                                                                                        .as_ref(),
                                                                                                                                    )
                                                                                                                                    .split(f.size())[1]; // Center vertically
                                                                                                        
                                                                                                                                let modal_chunks = Layout::default()
                                                                                                                                    .direction(Direction::Horizontal)
                                                                                                                                    .constraints(
                                                                                                                                        [
                                                                                                                                            Constraint::Percentage(20),
                                                                                                                                            Constraint::Percentage(60),
                                                                                                                                            Constraint::Percentage(20),
                                                                                                                                        ]
                                                                                                                                        .as_ref(),
                                                                                                                                    )
                                                                                                                                    .split(modal_area);
                                                                                                        
                                                                                                                                let modal_block = Block::default()
                                                                                                                                    .borders(Borders::ALL)
                                                                                                                                    .title("Sending Raw Transaction")
                                                                                                                                    .border_style(Style::default().fg(Color::Red));
                                                                                                        
                                                                                                                                let modal_messages: Vec<Line> = self.send_tx_modal_messages.lock().unwrap()
                                                                                                                                    .iter()
                                                                                                                                    .map(|msg| Line::from(Span::raw(msg.clone())))
                                                                                                                                    .collect();
                                                                                                        
                                                                                                                                let modal_paragraph = Paragraph::new(modal_messages)
                                                                                                                                    .block(modal_block)
                                                                                                                                    .wrap(Wrap { trim: true });
                                                                                                        
                                                                                                                                f.render_widget(Clear, modal_chunks[1]); // Clear the area first
                                                                                                                                f.render_widget(modal_paragraph, modal_chunks[1]);
                                                                                                                            }
                                                                                                                        }
                                                                                                                    })?;
                                                                                                        
                                                                                                                    match rx.recv_timeout(tick_rate) {
                Ok(Event::Input(event)) => {
                    self.last_user_input_time = Instant::now(); // Update timer on any input
                    
                    if event.kind == KeyEventKind::Press {
                        match event.code {
                            KeyCode::Char('q') | KeyCode::Char('Q') if self.focused_widget != FocusedWidget::Input => {
                                self.running.store(false, Ordering::SeqCst);
                            }
                            KeyCode::Tab => {
                            if let FocusedWidget::Input = self.focused_widget {
                                if !self.current_suggestions.is_empty() {
                                    let next_index = match self.selected_suggestion_index {
                                        Some(i) => (i + 1) % self.current_suggestions.len(),
                                        None => 0,
                                    };
                                    self.selected_suggestion_index = Some(next_index);
                                } else {
                                    // If no suggestions, move focus out of input
                                    self.focused_widget = FocusedWidget::BlockHeight;
                                }
                            } else {
                                // Cycle through other widgets
                                self.focused_widget = match self.focused_widget {
                                    FocusedWidget::BlockHeight => FocusedWidget::Instructions,
                                    FocusedWidget::Instructions => FocusedWidget::Log,
                                    FocusedWidget::Log => FocusedWidget::PeerList,
                                    FocusedWidget::PeerList => FocusedWidget::Input,
                                    FocusedWidget::Input => FocusedWidget::Modal,
                                    FocusedWidget::Modal => FocusedWidget::BlockHeight,
                                };
                            }
                        }
                        KeyCode::Down => {
                            if let FocusedWidget::Log = self.focused_widget {
                                self.scroll_state = self.scroll_state.saturating_add(1);
                                let messages_count = self.messages.lock().unwrap().len();
                                let visible_lines = self.log_widget_height.saturating_sub(2);
                                let max_scroll = (messages_count as u16).saturating_sub(visible_lines);
                                if self.scroll_state >= max_scroll {
                                    self.scroll_state = max_scroll;
                                    self.auto_scroll_enabled = true;
                                } else {
                                    self.auto_scroll_enabled = false;
                                }
                            } else if let FocusedWidget::Input = self.focused_widget {
                                if !self.current_suggestions.is_empty() {
                                    let next_index = match self.selected_suggestion_index {
                                        Some(i) => (i + 1) % self.current_suggestions.len(),
                                        None => 0,
                                    };
                                    self.selected_suggestion_index = Some(next_index);
                                }
                            }
                        }
                        KeyCode::Up => {
                            if let FocusedWidget::Log = self.focused_widget {
                                self.scroll_state = self.scroll_state.saturating_sub(1);
                                self.auto_scroll_enabled = false; // User manually scrolled
                            } else if let FocusedWidget::Input = self.focused_widget {
                                if !self.current_suggestions.is_empty() {
                                    let prev_index = match self.selected_suggestion_index {
                                        Some(0) => self.current_suggestions.len() - 1,
                                        Some(i) => i - 1,
                                        None => self.current_suggestions.len() - 1,
                                    };
                                    self.selected_suggestion_index = Some(prev_index);
                                }
                            }
                        }
                        KeyCode::Left => {
                            self.focused_widget = match self.focused_widget {
                                FocusedWidget::Instructions => FocusedWidget::BlockHeight,
                                FocusedWidget::PeerList => FocusedWidget::Log,
                                FocusedWidget::Log => FocusedWidget::Instructions,
                                _ => self.focused_widget,
                            };
                        }
                        KeyCode::Right => {
                            self.focused_widget = match self.focused_widget {
                                FocusedWidget::BlockHeight => FocusedWidget::Instructions,
                                FocusedWidget::Instructions => FocusedWidget::Log,
                                FocusedWidget::Log => FocusedWidget::PeerList,
                                _ => self.focused_widget,
                            };
                        }
                        KeyCode::Esc => {
                            if self.show_send_tx_modal {
                                self.show_send_tx_modal = false;
                                self.send_tx_modal_messages.lock().unwrap().clear();
                            } else if self.focused_widget == FocusedWidget::Log
                                && !self.auto_scroll_enabled
                            {
                                self.auto_scroll_enabled = true;
                                self.last_user_input_time = Instant::now(); // Reset timer to allow auto-scroll after delay
                            }
                        }
                        KeyCode::Char('p') => {
                            self.peer_list_width_percentage =
                                if self.peer_list_width_percentage == 50 {
                                    0
                                } else {
                                    50
                                };
                            if self.peer_list_width_percentage == 0
                                && self.focused_widget == FocusedWidget::PeerList
                            {
                                self.focused_widget = FocusedWidget::Log; // Move focus if peer list is hidden
                            }
                        }
                        KeyCode::Char('l') => {
                            self.log_visible = !self.log_visible;
                            if !self.log_visible && self.focused_widget == FocusedWidget::Log {
                                self.focused_widget = FocusedWidget::BlockHeight;
                            }
                        }
                        KeyCode::Enter => {
                            if let FocusedWidget::Log = self.focused_widget {
                                let messages_count = self.messages.lock().unwrap().len();
                                if messages_count > 0 {
                                    let visible_lines = self.log_widget_height.saturating_sub(2);
                                    let max_scroll = (messages_count as u16).saturating_sub(visible_lines);
                                    self.scroll_state = max_scroll;
                                } else {
                                    self.scroll_state = 0;
                                }
                                self.auto_scroll_enabled = true;
                            } else if let FocusedWidget::Input = self.focused_widget {
                                if let Some(index) = self.selected_suggestion_index {
                                    if let Some(command) = self.current_suggestions.get(index) {
                                        self.input_text.clear();
                                        self.input_text.push_str(command);
                                        self.input_text.push(' '); // Add a space after the command
                                        self.cursor_position = self.input_text.len();
                                        self.current_suggestions.clear();
                                        self.selected_suggestion_index = None;
                                    }
                                } else {
                                    // Process the command
                                    let command_text = self.input_text.trim().to_string();
                                    if command_text.starts_with("sendrawtransaction") {
                                        self.show_send_tx_modal = true;
                                        self.send_tx_modal_messages.lock().unwrap().push(format!("Attempting to send raw transaction: {}", command_text));
                                        // In a real scenario, you'd parse the transaction hex and initiate sending here.
                                        // For now, just simulate some logging.
                                        self.send_tx_modal_messages.lock().unwrap().push("Connecting to peers...".to_string());
                                    } else {
                                        // Handle other commands or log unknown command
                                        self.messages.lock().unwrap().push((format!("Unknown command: {}", command_text), SystemTime::now()));
                                    }
                                    self.input_text.clear();
                                    self.cursor_position = 0;
                                }
                            }
                        }
                        KeyCode::Char(c) => {
                            if let FocusedWidget::Input = self.focused_widget {
                                self.input_text.push(c);
                                self.cursor_position += 1;
                                self.update_suggestions();
                            } else if c == 'c' {
                                self.focused_widget = FocusedWidget::Input;
                            }
                        }
                        KeyCode::Backspace => {
                            if let FocusedWidget::Input = self.focused_widget {
                                if self.cursor_position > 0 {
                                    self.input_text.pop();
                                    self.cursor_position -= 1;
                                    self.update_suggestions();
                                }
                            }
                        }
                        _ => {}
                    }
                    }
                }
                Ok(Event::Tick) => {
                    // --- Scrolling Animation Logic ---
                    let messages_count = self.messages.lock().unwrap().len();
                    let visible_lines = self.log_widget_height.saturating_sub(2);
                    let bottom_scroll_target = if (messages_count as u16) > visible_lines {
                        (messages_count as u16 - visible_lines) as f32
                    } else {
                        0.0
                    };

                    // Auto-scroll logic should apply if auto_scroll_enabled is true, regardless of
                    // focus
                    if self.auto_scroll_enabled {
                        // Auto-scroll towards the bottom
                        if self.current_scroll_y < bottom_scroll_target {
                            let diff = bottom_scroll_target - self.current_scroll_y;
                            self.current_scroll_y += diff * self.scroll_animation_speed;
                            // Snap to bottom if very close
                            if (bottom_scroll_target - self.current_scroll_y).abs() < 0.1 {
                                self.current_scroll_y = bottom_scroll_target;
                            }
                        } else if self.current_scroll_y > bottom_scroll_target {
                            // Ensure we don't scroll past the bottom
                            self.current_scroll_y = bottom_scroll_target;
                        }
                    }

                    // Manual scroll animation should only happen when the log is focused
                    if self.focused_widget == FocusedWidget::Log {
                        // Manual scroll animation towards target scroll_state
                        if self.current_scroll_y != self.scroll_state as f32 {
                            let diff = self.scroll_state as f32 - self.current_scroll_y;
                            self.current_scroll_y += diff * self.scroll_animation_speed;
                            // Snap to target if very close
                            if (self.scroll_state as f32 - self.current_scroll_y).abs() < 0.1 {
                                self.current_scroll_y = self.scroll_state as f32;
                            }
                        }
                    }
                    // --- End Scrolling Logic ---
                }
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    self.running.store(false, Ordering::SeqCst);
                }
            }
        }
        Ok(())
    }
}

pub fn init_tui() -> anyhow::Result<Terminal<CrosstermBackend<io::Stdout>>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

pub fn restore_tui(terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> anyhow::Result<()> {
    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;
    Ok(())
}
