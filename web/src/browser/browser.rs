/*
 * Copyright (c) 2026 Geekspeaker Inc. All Rights Reserved.
 *
 * This software is "Source Available".
 * You may use and modify it for personal use.
 * Redistribution of modified versions is prohibited.
 *
 * See the LICENSE file for more details.
 */

use crate::browser::network::NetworkClient;
use crate::browser::renderer::{render_html_to_text, FormField};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{
        Block, BorderType, Borders, Clear, Paragraph, Scrollbar, ScrollbarOrientation,
        ScrollbarState, Wrap,
    },
    Frame,
};
use std::time::Duration;

#[derive(PartialEq)]
pub enum InputMode {
    Normal,
    EditingUrl,
    Searching,
    LinkFollow,
    FormInput, // New: editing a form field
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchEngine {
    Google,
    DuckDuckGo,
    Bing,
    Brave,
    Swisscows,
    Qwant,
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub enum Theme {
    Dark,  // Default: dark background, cyan accents
    Light, // Light background, dark text
    Retro, // Green/amber terminal style
    Ocean, // Blue shades
}

// Tab state for multi-tab support
#[derive(Clone)]
pub struct Tab {
    pub url: String,
    pub content: Vec<String>,
    pub links: Vec<String>,
    pub scroll_offset: u16,
    pub _title: String,
}

// Helper for Start Page Content with Ads
fn get_start_page(engine_name: &str) -> (Vec<String>, Vec<String>) {
    let content = vec![
        "╔══════════════════════════════════════════════════════════╗".to_string(),
        "║       Welcome to Methodwise - Text-First Browsing        ║".to_string(),
        "╚══════════════════════════════════════════════════════════╝".to_string(),
        "".to_string(),
        "Quick Start:".to_string(),
        "  [e/g] Edit URL or Search      [?] Toggle Help".to_string(),
        "  [j/k] Scroll Down/Up          [f] Follow Link".to_string(),
        "  [h/l] Back/Forward            [b/B] Bookmarks".to_string(),
        "  [H] History                   [F12] Debug Console".to_string(),
        "  [s] Switch Search Engine".to_string(),
        "".to_string(),
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string(),
        "🌟 SPONSORED (Support Methodwise):".to_string(),
        "     [1] digitalocean.com      - Simple Cloud Hosting ($200 Credit)".to_string(),
        "     [2] geekspeaker.com           - Advertise with us".to_string(),
        "".to_string(),
        "Recommended Sites (click or type number):".to_string(),
        "".to_string(),
        "  📰 News & Info:".to_string(),
        "     [3] text.npr.org          - NPR News (text mode)".to_string(),
        "     [4] lite.cnn.com          - CNN Lite".to_string(),
        "     [5] en.wikipedia.org      - Wikipedia".to_string(),
        "".to_string(),
        "  🔍 Search:".to_string(),
        "     [6] duckduckgo.com        - Privacy search".to_string(),
        "     [7] search.brave.com      - Brave Search".to_string(),
        "     [8] swisscows.com         - Anonymous Search".to_string(),
        "     [9] qwant.com             - EU Privacy Search".to_string(),
        "".to_string(),
        "  💻 Developer:".to_string(),
        "     [10] docs.rs              - Rust Documentation".to_string(),
        "     [11] news.ycombinator.com - Hacker News".to_string(),
        "     [12] lobste.rs            - Lobsters".to_string(),
        "     [13] hackerweb.app        - Readable HN".to_string(),
        "     [14] hckrnews.com         - HN Filter".to_string(),
        "     [15] skimfeed.com         - Tech News Aggregator".to_string(),
        "".to_string(),
        "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━".to_string(),
        format!(
            "Search Engine: {} | Press 'e' to start browsing!",
            engine_name
        ),
    ];

    let links = vec![
        "https://m.do.co/c/35078d16a2da".to_string(),
        "https://text.npr.org".to_string(),
        "https://lite.cnn.com".to_string(),
        "https://en.wikipedia.org".to_string(),
        "https://duckduckgo.com".to_string(),
        "https://search.brave.com".to_string(),
        "https://swisscows.com/en".to_string(),
        "https://www.qwant.com/".to_string(),
        "https://docs.rs".to_string(),
        "https://news.ycombinator.com".to_string(),
        "https://lobste.rs".to_string(),
        "https://hackerweb.app/".to_string(),
        "https://hckrnews.com/".to_string(),
        "https://skimfeed.com/long.html".to_string(),
    ];

    (content, links)
}

impl Tab {
    fn new(engine_name: &str) -> Self {
        let (content, links) = get_start_page(engine_name);
        Tab {
            url: String::new(),
            content,
            links,
            scroll_offset: 0,
            _title: "New Tab".to_string(),
        }
    }
}

pub struct BrowserApp {
    pub url_input: String,
    pub content: Vec<String>,
    pub links: Vec<String>,
    pub scroll_offset: u16,
    pub input_mode: InputMode,
    pub client: NetworkClient,
    pub status_message: String,
    pub history: Vec<String>,
    pub history_index: usize,
    pub search_query: String,
    pub link_input: String,
    pub show_help: bool,
    pub is_loading: bool,
    pub viewport_height: u16,
    pub viewport_width: u16,
    pub search_engine: SearchEngine,
    pub debug_log: Vec<String>,
    pub show_debug: bool,
    // New: Bookmarks & History UI
    pub bookmarks: Vec<String>,
    pub show_bookmarks: bool,
    pub show_history: bool,
    pub selected_index: usize, // For overlay selection
    // New: Form Support
    pub form_fields: Vec<FormField>,
    pub focused_field: usize,
    pub form_action: Option<String>,
    pub form_method: String,
    // Cursor position for text editing
    pub cursor_pos: usize,
    // Theme support
    pub theme: Theme,
    // Tab support
    pub tabs: Vec<Tab>,
    pub active_tab: usize,
}

impl BrowserApp {
    pub fn new(engine: SearchEngine) -> Self {
        let engine_name = match engine {
            SearchEngine::Google => "Google",
            SearchEngine::DuckDuckGo => "DuckDuckGo",
            SearchEngine::Bing => "Bing",
            SearchEngine::Brave => "Brave",
            SearchEngine::Swisscows => "Swisscows",
            SearchEngine::Qwant => "Qwant",
        };

        let (content, links) = get_start_page(engine_name);

        Self {
            url_input: String::new(),
            content: content.clone(),
            links: links.clone(),
            scroll_offset: 0,
            input_mode: InputMode::Normal,
            client: NetworkClient::new(),
            status_message: "Ready".to_string(),
            history: Vec::new(),
            history_index: 0,
            search_query: String::new(),
            link_input: String::new(),
            show_help: false,
            is_loading: false,
            viewport_height: 0,
            viewport_width: 80,
            search_engine: engine,
            debug_log: Vec::new(),
            show_debug: false,
            bookmarks: Vec::new(),
            show_bookmarks: false,
            show_history: false,
            selected_index: 0,
            form_fields: Vec::new(),
            focused_field: 0,
            form_action: None,
            form_method: "GET".to_string(),
            cursor_pos: 0,
            theme: Theme::Dark,
            tabs: vec![Tab::new(engine_name)],
            active_tab: 0,
        }
    }

    pub fn log(&mut self, msg: String) {
        if self.debug_log.len() >= 20 {
            self.debug_log.remove(0);
        }
        self.debug_log.push(format!(
            "[{}] {}",
            chrono::Local::now().format("%H:%M:%S"),
            msg
        ));
    }

    fn get_bookmarks_path() -> std::path::PathBuf {
        let home = std::env::var("USERPROFILE")
            .or_else(|_| std::env::var("HOME"))
            .unwrap_or_else(|_| ".".to_string());
        std::path::PathBuf::from(home)
            .join(".methodwise")
            .join("bookmarks.txt")
    }

    pub fn load_bookmarks(&mut self) {
        let path = Self::get_bookmarks_path();
        if path.exists() {
            if let Ok(content) = std::fs::read_to_string(&path) {
                self.bookmarks = content
                    .lines()
                    .filter(|l| !l.is_empty())
                    .map(|s| s.to_string())
                    .collect();
            }
        }
    }

    pub fn save_bookmarks(&self) {
        let path = Self::get_bookmarks_path();
        if let Some(parent) = path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let content = self.bookmarks.join("\n");
        let _ = std::fs::write(&path, content);
    }

    pub async fn go_home(&mut self) {
        // Navigate to the online homepage
        self.url_input = "https://methodwise.com/text.html".to_string();
        self.log("Action: Go Home".to_string());
        self.navigate().await;
    }

    pub async fn load_initial_url(&mut self, url: Option<String>, force_static: bool) {
        if let Some(u) = url {
            self.url_input = u;
            self.navigate().await;
        } else if !force_static {
            // Attempt to load dynamic homepage from methodwise.com
            // If it fails (e.g. offline or domain doesn't exist), we silently keep the static start page.
            self.status_message = "Checking for homepage updates...".to_string();

            // We clone the client to avoid borrow checker issues if we used self.navigate
            // But here we use self.client directly.
            match self
                .client
                .fetch_url("https://methodwise.com/text.html")
                .await
            {
                Ok((_final_url, body)) => {
                    self.url_input = "https://methodwise.com/text.html".to_string();
                    let result = render_html_to_text(&body, self.viewport_width as usize);
                    self.content = result.lines;
                    self.links = result.links;
                    self.form_fields = result.form_fields;
                    self.focused_field = 0;
                    self.scroll_offset = 0;
                    self.status_message = "Welcome to Methodwise".to_string();

                    // Update the active tab as well
                    if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                        tab.url = self.url_input.clone();
                        tab.content = self.content.clone();
                        tab.links = self.links.clone();
                    }
                }
                Err(_) => {
                    self.status_message = "Ready (Offline Mode)".to_string();
                }
            }
        }
    }

    pub async fn run_step(
        &mut self,
        terminal_width: u16,
        terminal_height: u16,
    ) -> Result<bool, anyhow::Error> {
        self.viewport_height = terminal_height.saturating_sub(4); // Approx spacing for bars
        self.viewport_width = terminal_width;

        if event::poll(Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) => return self.handle_key(key).await,
                Event::Mouse(mouse) => self.handle_mouse(mouse).await,
                _ => {}
            }
        }
        Ok(false)
    }

    async fn handle_mouse(&mut self, mouse: event::MouseEvent) {
        // Note: Verbose mouse logging removed - only log important actions

        // Layout constants matching ui() function:
        // vertical_chunks[0] = ASCII Art Area (8 lines)  → rows 0-7
        // vertical_chunks[1] = Navbar (3 lines)          → rows 8-10
        // vertical_chunks[2] = Tab Bar (1 line)          → row 11
        // vertical_chunks[3] = Content Area (variable)   → rows 12 to (height - 2)
        // vertical_chunks[4] = Status Bar (1 line)       → last row
        const ASCII_HEIGHT: u16 = 8;
        const NAVBAR_HEIGHT: u16 = 3;
        const TAB_BAR_ROW: u16 = ASCII_HEIGHT + NAVBAR_HEIGHT; // row 11
        const CONTENT_START: u16 = TAB_BAR_ROW + 1; // row 12

        // Handle Clicks & Drags
        if mouse.kind == event::MouseEventKind::Down(event::MouseButton::Left)
            || mouse.kind == event::MouseEventKind::Drag(event::MouseButton::Left)
        {
            let row = mouse.row;
            let col = mouse.column;

            // 1. Tab Bar Click (Row 11)
            if row == TAB_BAR_ROW {
                if mouse.kind == event::MouseEventKind::Down(event::MouseButton::Left) {
                    // Tab clicking logic - tabs are formatted as "[*1: title]" or "[1: title]"
                    // Each tab is roughly 20 chars wide
                    let tab_click_idx = col as usize / 20;
                    if tab_click_idx < self.tabs.len() && tab_click_idx != self.active_tab {
                        // Switch tab
                        // Save current tab state
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            tab.url = self.url_input.clone();
                            tab.content = self.content.clone();
                            tab.links = self.links.clone();
                            tab.scroll_offset = self.scroll_offset;
                        }
                        // Load target tab
                        self.active_tab = tab_click_idx;
                        if let Some(tab) = self.tabs.get(self.active_tab) {
                            self.url_input = tab.url.clone();
                            self.content = tab.content.clone();
                            self.links = tab.links.clone();
                            self.scroll_offset = tab.scroll_offset;
                        }
                        self.status_message = format!("Switched to Tab {}", self.active_tab + 1);
                    }
                }
            }
            // 2. Navigation & Address Bar (Rows 8-10)
            else if (ASCII_HEIGHT..TAB_BAR_ROW).contains(&row) {
                // Layout from ui(): Back(7) | Fwd(7) | Address(Min) | Home(7) | New Tab(7)
                let width = self.viewport_width;

                if mouse.kind == event::MouseEventKind::Down(event::MouseButton::Left) {
                    if col < 7 {
                        // Back Button
                        self.go_back().await;
                    } else if col < 14 {
                        // Forward Button
                        self.go_forward().await;
                    } else if col >= width.saturating_sub(7) {
                        // New Tab Button (rightmost)
                        self.create_new_tab();
                    } else if col >= width.saturating_sub(14) {
                        // Home Button
                        self.go_home().await;
                    } else {
                        // Address Bar (Middle)
                        self.input_mode = InputMode::EditingUrl;
                        self.status_message = "Editing URL...".to_string();
                    }
                }
            }
            // 3. Content Area (Row 12 onwards, with border offset)
            else if row >= CONTENT_START {
                // Content block has a 1-cell border, so actual content starts at row 13
                let content_y_start = CONTENT_START + 1; // Account for top border

                // Scrollbar Check: rightmost column
                if col >= self.viewport_width.saturating_sub(2) {
                    // Drag/Click Scrollbar
                    if row >= content_y_start {
                        let relative_row = (row - content_y_start) as f64;
                        let height = self.viewport_height as f64;
                        let total_content = self.content.len() as f64;

                        if total_content > height {
                            let ratio = relative_row / height;
                            let new_offset = (ratio * total_content) as u16;
                            self.scroll_offset =
                                new_offset.min((self.content.len() as u16).saturating_sub(1));
                        }
                    }
                } else if row >= content_y_start {
                    // Content Click (Only Down)
                    if mouse.kind == event::MouseEventKind::Down(event::MouseButton::Left) {
                        let relative_row = (row - content_y_start) as usize;
                        let content_idx = self.scroll_offset as usize + relative_row;

                        // Account for left border (1 cell)
                        let content_x_start = 1;
                        let mut found_link_id = None;

                        if content_idx < self.content.len() {
                            let line = self.content[content_idx].clone();
                            let click_idx_in_line = (col as usize).saturating_sub(content_x_start);

                            let re = regex::Regex::new(r"\[(\d+)\]").unwrap();
                            for cap in re.captures_iter(&line) {
                                if let Some(m) = cap.get(0) {
                                    let start = m.start();
                                    let end = m.end();
                                    if click_idx_in_line >= start.saturating_sub(2)
                                        && click_idx_in_line <= end + 2
                                    {
                                        if let Ok(id) = cap[1].parse::<usize>() {
                                            found_link_id = Some(id);
                                            break;
                                        }
                                    }
                                }
                            }

                            // Check for form field click: [F1:...], [F2:...], etc.
                            let form_re = regex::Regex::new(r"\[F(\d+)").unwrap();
                            for cap in form_re.captures_iter(&line) {
                                if let Some(m) = cap.get(0) {
                                    let start = m.start();
                                    let end = m.end() + 15; // Extend to cover the field
                                    if click_idx_in_line >= start && click_idx_in_line <= end {
                                        if let Ok(field_id) = cap[1].parse::<usize>() {
                                            // Find and focus the field
                                            let editable_fields: Vec<_> = self
                                                .form_fields
                                                .iter()
                                                .enumerate()
                                                .filter(|(_, f)| f.field_type != "hidden")
                                                .collect();
                                            if let Some((idx, field)) = editable_fields
                                                .iter()
                                                .find(|(_, f)| f.display_index == field_id)
                                            {
                                                self.focused_field = *idx;
                                                self.input_mode = InputMode::FormInput;
                                                let label = if !field.placeholder.is_empty() {
                                                    &field.placeholder
                                                } else {
                                                    &field.name
                                                };
                                                self.status_message = format!(
                                                    "Editing [F{}]: {}",
                                                    field.display_index, label
                                                );
                                                return;
                                            }
                                        }
                                    }
                                }
                            }
                        }

                        if let Some(id) = found_link_id {
                            self.link_input = id.to_string();
                            self.log(format!("Following link {}", id));
                            self.follow_link().await;
                            return;
                        }
                    }
                }
            }
        }
        // Scroll Wheel
        match mouse.kind {
            event::MouseEventKind::ScrollDown => self.scroll_down(),
            event::MouseEventKind::ScrollUp => self.scroll_up(),
            _ => {}
        }
    }

    async fn handle_key(&mut self, key: KeyEvent) -> Result<bool, anyhow::Error> {
        // Toggle Debug Console with F12 - Allow on Press
        if key.code == KeyCode::F(12) && key.kind == KeyEventKind::Press {
            self.show_debug = !self.show_debug;
            return Ok(false);
        }

        // Filter out Release events to prevent double input
        if key.kind != KeyEventKind::Press {
            return Ok(false);
        }

        // Toggle Help with ? - Global handler
        if key.code == KeyCode::Char('?') {
            self.show_help = !self.show_help;
            return Ok(false);
        }

        // Close overlays with Esc - Global handler
        if key.code == KeyCode::Esc {
            if self.show_help {
                self.show_help = false;
                return Ok(false);
            }
            if self.show_debug {
                self.show_debug = false;
                return Ok(false);
            }
            if self.show_bookmarks {
                self.show_bookmarks = false;
                return Ok(false);
            }
            if self.show_history {
                self.show_history = false;
                return Ok(false);
            }
        }

        // Note: Verbose key logging removed - only log important actions

        // Overlay Navigation (Bookmarks / History)
        if self.show_bookmarks || self.show_history {
            let items = if self.show_bookmarks {
                &self.bookmarks
            } else {
                &self.history
            };
            let len = items.len();

            match key.code {
                KeyCode::Esc => {
                    self.show_bookmarks = false;
                    self.show_history = false;
                    self.status_message = "Ready".to_string();
                }
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.selected_index > 0 {
                        self.selected_index -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if len > 0 && self.selected_index < len - 1 {
                        self.selected_index += 1;
                    }
                }
                KeyCode::Enter => {
                    if len > 0 && self.selected_index < len {
                        let url = items[self.selected_index].clone();
                        self.url_input = url;
                        self.show_bookmarks = false;
                        self.show_history = false;
                        self.navigate().await;
                    }
                }
                KeyCode::Char('d') | KeyCode::Delete => {
                    // Delete bookmark
                    if self.show_bookmarks && len > 0 && self.selected_index < len {
                        self.bookmarks.remove(self.selected_index);
                        if self.selected_index >= self.bookmarks.len() && self.selected_index > 0 {
                            self.selected_index -= 1;
                        }
                        self.save_bookmarks();
                        self.status_message = "Bookmark deleted".to_string();
                    }
                }
                _ => {}
            }
            return Ok(false);
        }

        match self.input_mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') | KeyCode::Char('Q')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    return Ok(true)
                }
                KeyCode::Char('c') | KeyCode::Char('C')
                    if key.modifiers.contains(KeyModifiers::CONTROL) =>
                {
                    return Ok(true)
                }
                // Close overlays with Esc
                KeyCode::Esc => {
                    if self.show_help {
                        self.show_help = false;
                    } else if self.show_debug {
                        self.show_debug = false;
                    } else {
                        self.status_message = "Ready".to_string();
                    }
                }
                KeyCode::Char('/') => {
                    self.input_mode = InputMode::Searching;
                    self.search_query.clear();
                    self.status_message = "Search mode...".to_string();
                }
                KeyCode::Char('f') => {
                    self.input_mode = InputMode::LinkFollow;
                    self.link_input.clear();
                    self.status_message = "Enter Link ID (e.g. 1):".to_string();
                }
                // Allow direct number typing to start Link Follow mode
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    self.input_mode = InputMode::LinkFollow;
                    self.link_input.clear();
                    self.link_input.push(c);
                    self.status_message = format!("Enter Link ID: {}", self.link_input);
                }
                KeyCode::Char('e') | KeyCode::Char('g') => {
                    self.input_mode = InputMode::EditingUrl;
                    self.status_message = "Editing URL...".to_string();
                }
                KeyCode::Char('r') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                    self.navigate().await;
                }
                KeyCode::Char('s') => {
                    // Cycle Search Engine
                    self.search_engine = match self.search_engine {
                        SearchEngine::DuckDuckGo => SearchEngine::Brave,
                        SearchEngine::Brave => SearchEngine::Swisscows,
                        SearchEngine::Swisscows => SearchEngine::Qwant,
                        SearchEngine::Qwant => SearchEngine::Google,
                        SearchEngine::Google => SearchEngine::Bing,
                        SearchEngine::Bing => SearchEngine::DuckDuckGo,
                    };
                    let engine_name = match self.search_engine {
                        SearchEngine::Google => "Google",
                        SearchEngine::DuckDuckGo => "DuckDuckGo",
                        SearchEngine::Bing => "Bing",
                        SearchEngine::Brave => "Brave",
                        SearchEngine::Swisscows => "Swisscows",
                        SearchEngine::Qwant => "Qwant",
                    };
                    self.status_message = format!("Search Engine set to: {}", engine_name);
                }
                KeyCode::Char('h') | KeyCode::Left => {
                    self.go_back().await;
                }
                KeyCode::Char('l') | KeyCode::Right => {
                    self.go_forward().await;
                }
                KeyCode::Char('j') | KeyCode::Down => {
                    self.scroll_down();
                }
                KeyCode::Char('k') | KeyCode::Up => {
                    self.scroll_up();
                }
                KeyCode::PageDown => {
                    for _ in 0..self.viewport_height {
                        self.scroll_down();
                    }
                }
                KeyCode::PageUp => {
                    for _ in 0..self.viewport_height {
                        self.scroll_up();
                    }
                }
                KeyCode::Char('?') => {
                    self.show_help = !self.show_help;
                }
                // Bookmarks
                KeyCode::Char('b') => {
                    self.show_bookmarks = !self.show_bookmarks;
                    self.show_history = false;
                    self.selected_index = 0;
                    self.status_message = if self.show_bookmarks {
                        "Bookmarks (Enter to go, Esc to close)".to_string()
                    } else {
                        "Ready".to_string()
                    };
                }
                KeyCode::Char('B') => {
                    // Add current URL to bookmarks
                    if !self.url_input.is_empty() && !self.bookmarks.contains(&self.url_input) {
                        self.bookmarks.push(self.url_input.clone());
                        self.status_message = format!("Bookmarked: {}", self.url_input);
                        self.save_bookmarks();
                    } else {
                        self.status_message = "Already bookmarked or no URL".to_string();
                    }
                }
                // History Viewer
                KeyCode::Char('H') => {
                    self.show_history = !self.show_history;
                    self.show_bookmarks = false;
                    self.selected_index = 0;
                    self.status_message = if self.show_history {
                        "History (Enter to go, Esc to close)".to_string()
                    } else {
                        "Ready".to_string()
                    };
                }
                // Tab key: Enter form input mode and cycle through fields
                KeyCode::Tab => {
                    let editable_fields: Vec<_> = self
                        .form_fields
                        .iter()
                        .filter(|f| f.field_type != "hidden")
                        .collect();

                    if !editable_fields.is_empty() {
                        // Find next editable field
                        if self.input_mode == InputMode::FormInput {
                            // Cycle to next field
                            self.focused_field = (self.focused_field + 1) % editable_fields.len();
                        } else {
                            // Enter form mode, focus first field
                            self.focused_field = 0;
                            self.input_mode = InputMode::FormInput;
                        }

                        if let Some(field) = editable_fields.get(self.focused_field) {
                            let label = if !field.placeholder.is_empty() {
                                &field.placeholder
                            } else {
                                &field.name
                            };
                            self.status_message = format!(
                                "Editing [F{}]: {} (Enter=submit, Esc=cancel)",
                                field.display_index, label
                            );
                        }
                    } else {
                        self.status_message = "No form fields on this page".to_string();
                    }
                }
                // F1-F9: Jump directly to edit form field by display_index
                KeyCode::F(n) if (1..=9).contains(&n) => {
                    let target_idx = n as usize;
                    let editable_fields: Vec<_> = self
                        .form_fields
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| f.field_type != "hidden")
                        .collect();

                    // Find the field with matching display_index
                    if let Some((actual_idx, field)) = editable_fields
                        .iter()
                        .find(|(_, f)| f.display_index == target_idx)
                    {
                        self.focused_field = *actual_idx;
                        self.input_mode = InputMode::FormInput;
                        let label = if !field.placeholder.is_empty() {
                            &field.placeholder
                        } else {
                            &field.name
                        };
                        self.status_message = format!(
                            "Editing [F{}]: {} = \"{}\"",
                            field.display_index, label, field.value
                        );
                    } else {
                        self.status_message = format!("No form field F{} on this page", n);
                    }
                }
                // Theme cycling
                KeyCode::Char('t') => {
                    self.theme = match self.theme {
                        Theme::Dark => Theme::Light,
                        Theme::Light => Theme::Retro,
                        Theme::Retro => Theme::Ocean,
                        Theme::Ocean => Theme::Dark,
                    };
                    let theme_name = match self.theme {
                        Theme::Dark => "Dark",
                        Theme::Light => "Light",
                        Theme::Retro => "Retro (Green)",
                        Theme::Ocean => "Ocean (Blue)",
                    };
                    self.status_message = format!("Theme: {}", theme_name);
                }
                // Tab: New tab with Ctrl+T (shift+T for new tab)
                KeyCode::Char('T') => {
                    self.create_new_tab();
                    if self.tabs.len() >= 9 {
                        self.status_message = "Max 9 tabs".to_string();
                    }
                }
                // Close tab with Shift+W
                KeyCode::Char('W') => {
                    if self.tabs.len() > 1 {
                        self.tabs.remove(self.active_tab);
                        if self.active_tab >= self.tabs.len() {
                            self.active_tab = self.tabs.len() - 1;
                        }
                        // Load new active tab
                        if let Some(tab) = self.tabs.get(self.active_tab) {
                            self.url_input = tab.url.clone();
                            self.content = tab.content.clone();
                            self.links = tab.links.clone();
                            self.scroll_offset = tab.scroll_offset;
                        }
                        self.status_message =
                            format!("Tab closed. Now on tab {}", self.active_tab + 1);
                    } else {
                        self.status_message = "Cannot close last tab".to_string();
                    }
                }
                // Number keys 1-9: If multiple tabs exist, switch tabs. Otherwise fall through to link following.
                KeyCode::Char(c @ '1'..='9') if self.tabs.len() > 1 => {
                    let target = (c as u8 - b'1') as usize;
                    if target < self.tabs.len() && target != self.active_tab {
                        // Save current tab state
                        if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                            tab.url = self.url_input.clone();
                            tab.content = self.content.clone();
                            tab.links = self.links.clone();
                            tab.scroll_offset = self.scroll_offset;
                        }
                        // Switch to target tab
                        self.active_tab = target;
                        if let Some(tab) = self.tabs.get(self.active_tab) {
                            self.url_input = tab.url.clone();
                            self.content = tab.content.clone();
                            self.links = tab.links.clone();
                            self.scroll_offset = tab.scroll_offset;
                        }
                        self.status_message =
                            format!("Tab {} of {}", self.active_tab + 1, self.tabs.len());
                    }
                }
                _ => {}
            },
            InputMode::EditingUrl => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.cursor_pos = 0;
                    self.navigate().await;
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.cursor_pos = 0;
                    self.status_message = "Ready".to_string();
                }
                KeyCode::Left => {
                    if self.cursor_pos > 0 {
                        self.cursor_pos -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.cursor_pos < self.url_input.len() {
                        self.cursor_pos += 1;
                    }
                }
                KeyCode::Home => {
                    self.cursor_pos = 0;
                }
                KeyCode::End => {
                    self.cursor_pos = self.url_input.len();
                }
                KeyCode::Backspace => {
                    if self.cursor_pos > 0 {
                        self.url_input.remove(self.cursor_pos - 1);
                        self.cursor_pos -= 1;
                    }
                }
                KeyCode::Delete => {
                    if self.cursor_pos < self.url_input.len() {
                        self.url_input.remove(self.cursor_pos);
                    }
                }
                KeyCode::Char(c) => {
                    self.url_input.insert(self.cursor_pos, c);
                    self.cursor_pos += 1;
                }
                _ => {}
            },
            InputMode::Searching => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.perform_search();
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.status_message = "Ready".to_string();
                }
                KeyCode::Backspace => {
                    self.search_query.pop();
                }
                KeyCode::Char(c) => {
                    self.search_query.push(c);
                }
                _ => {}
            },
            InputMode::LinkFollow => match key.code {
                KeyCode::Enter => {
                    self.input_mode = InputMode::Normal;
                    self.follow_link().await;
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.status_message = "Ready".to_string();
                }
                KeyCode::Backspace => {
                    self.link_input.pop();
                    if self.link_input.is_empty() {
                        self.status_message = "Enter Link ID:".to_string();
                    } else {
                        self.status_message = format!("Enter Link ID: {}", self.link_input);
                    }
                }
                KeyCode::Char(c) if c.is_ascii_digit() => {
                    self.link_input.push(c);
                    self.status_message = format!("Enter Link ID: {}", self.link_input);
                }
                _ => {}
            },
            InputMode::FormInput => match key.code {
                KeyCode::Tab => {
                    // Cycle to next field
                    let editable_count = self
                        .form_fields
                        .iter()
                        .filter(|f| f.field_type != "hidden")
                        .count();
                    if editable_count > 0 {
                        self.focused_field = (self.focused_field + 1) % editable_count;
                        let editable_fields: Vec<_> = self
                            .form_fields
                            .iter()
                            .filter(|f| f.field_type != "hidden")
                            .collect();
                        if let Some(field) = editable_fields.get(self.focused_field) {
                            let label = if !field.placeholder.is_empty() {
                                &field.placeholder
                            } else {
                                &field.name
                            };
                            self.status_message = format!(
                                "Editing [F{}]: {} = \"{}\"",
                                field.display_index, label, field.value
                            );
                        }
                    }
                }
                KeyCode::Enter => {
                    // Submit the form
                    self.submit_form().await;
                }
                KeyCode::Esc => {
                    self.input_mode = InputMode::Normal;
                    self.status_message = "Form editing cancelled".to_string();
                }
                KeyCode::Backspace => {
                    // Remove char from focused field
                    let editable_count = self
                        .form_fields
                        .iter()
                        .filter(|f| f.field_type != "hidden")
                        .count();
                    if self.focused_field < editable_count {
                        let mut idx = 0;
                        for field in self.form_fields.iter_mut() {
                            if field.field_type != "hidden" {
                                if idx == self.focused_field {
                                    field.value.pop();
                                    let label = if !field.placeholder.is_empty() {
                                        &field.placeholder
                                    } else if !field.name.is_empty() {
                                        &field.name
                                    } else {
                                        "Input"
                                    };
                                    self.status_message = format!(
                                        "[F{} {}]: {}█",
                                        field.display_index, label, field.value
                                    );
                                    break;
                                }
                                idx += 1;
                            }
                        }
                    }
                }
                KeyCode::Char(c) => {
                    // Type into focused field
                    let editable_count = self
                        .form_fields
                        .iter()
                        .filter(|f| f.field_type != "hidden")
                        .count();
                    if self.focused_field < editable_count {
                        let mut idx = 0;
                        for field in self.form_fields.iter_mut() {
                            if field.field_type != "hidden" {
                                if idx == self.focused_field {
                                    field.value.push(c);
                                    // Show field label, typed text, and cursor
                                    let label = if !field.placeholder.is_empty() {
                                        &field.placeholder
                                    } else if !field.name.is_empty() {
                                        &field.name
                                    } else {
                                        "Input"
                                    };
                                    self.status_message = format!(
                                        "[F{} {}]: {}█",
                                        field.display_index, label, field.value
                                    );
                                    break;
                                }
                                idx += 1;
                            }
                        }
                    }
                }
                _ => {}
            },
        }
        Ok(false)
    }

    async fn navigate(&mut self) {
        let raw_input = self.url_input.trim();
        if raw_input.is_empty() {
            return;
        }

        // Smart Search / URL Logic
        // If it looks like a URL, use it. Otherwise search using selected engine.
        // Heuristic: Has space? -> Search. No dot? -> Search.
        let target_url = if raw_input.contains(' ') || !raw_input.contains('.') {
            let query = raw_input.replace(" ", "+");
            match self.search_engine {
                SearchEngine::Google => format!("https://www.google.com/search?q={}", query),
                SearchEngine::DuckDuckGo => {
                    format!("https://html.duckduckgo.com/html/?q={}", query)
                }
                SearchEngine::Bing => format!("https://www.bing.com/search?q={}", query),
                SearchEngine::Brave => format!("https://search.brave.com/search?q={}", query),
                SearchEngine::Swisscows => format!("https://swisscows.com/en/web?query={}", query),
                SearchEngine::Qwant => format!("https://www.qwant.com/?q={}&t=web", query),
            }
        } else {
            raw_input.to_string()
        };

        self.is_loading = true;
        self.status_message = format!("Loading {}...", target_url);

        match self.client.fetch_url(&target_url).await {
            Ok((final_url, html_content)) => {
                // Update History
                if self.history.is_empty() {
                    self.history.push(final_url.clone());
                    self.history_index = 0;
                } else if self.history[self.history_index] != final_url {
                    // If we are earlier in history, truncate future
                    self.history.truncate(self.history_index + 1);
                    self.history.push(final_url.clone());
                    self.history_index = self.history.len() - 1;
                }

                self.url_input = final_url
                    .clone()
                    .trim_matches('"')
                    .trim_matches('\'')
                    .replace("\"", "")
                    .to_string();

                // Meta Refresh Detection
                let meta_re = regex::Regex::new(r#"(?i)<meta\s+http-equiv=["']?refresh["']?\s+content=["']?\d+;\s*URL=([^"']+)["']?"#).unwrap();
                if let Some(captures) = meta_re.captures(&html_content) {
                    let redirect_url = captures[1].to_string();
                    let decoded_url = html_escape::decode_html_entities(&redirect_url).to_string();
                    // Strip any quotes from the decoded URL
                    let clean_url = decoded_url
                        .trim()
                        .trim_matches('"')
                        .trim_matches('\'')
                        .replace("\"", "")
                        .to_string();
                    self.log(format!("Meta Refresh detected -> {}", clean_url));
                    self.url_input = clean_url;
                    // Recurse / Re-navigate (Box::pin due to async recursion if we were strict, but we can just loop or call local)
                    // Simple approach: modify url and return? No, we need to load.
                    // We can't easily recurse async traits or fns without boxing.
                    // Instead, let's just trigger another fetch here effectively by simulating a loop or re-calling?
                    // To avoid recursion limits, we can just call self.navigate() BUT it requires boxing.
                    // Hack: Just re-call it. Assuming shallow depth.
                    // Ideally we'd wrap the fetch in a loop inside navigate.
                    // But to keep diff minimal, let's return and let the next loop handle it? No, run_step calls navigate? No navigate is called by key handler.
                    Box::pin(self.navigate()).await;
                    return;
                }

                // Render with Dynamic Width
                // Use screen width - 6 (borders/scrollbars/padding/safe margin)
                // Increased margin to prevent scroll artifacts
                let render_width = (self.viewport_width as usize).saturating_sub(6).max(40);

                let result = render_html_to_text(&html_content, render_width);
                self.content = result.lines;
                self.links = result.links;
                self.form_fields = result.form_fields;
                self.form_action = result.form_action;
                self.form_method = result.form_method;
                self.focused_field = 0;

                // Auto-focus first editable field if any exist
                let editable_count = self
                    .form_fields
                    .iter()
                    .filter(|f| f.field_type != "hidden" && f.field_type != "submit")
                    .count();

                self.scroll_offset = 0;
                if editable_count > 0 {
                    self.status_message = format!(
                        "Loaded ({} links, {} form fields) - Press Tab to edit forms",
                        self.links.len(),
                        editable_count
                    );
                } else {
                    self.status_message = format!("Loaded ({} links)", self.links.len());
                }
            }
            Err(e) => {
                self.status_message = format!("Error: {:#}", e);
                self.content = vec![format!("Failed to load: {:#}", e)];
                self.links.clear();
                self.form_fields.clear();
            }
        }
        self.is_loading = false;
    }

    async fn submit_form(&mut self) {
        // Build query string from all form fields
        let mut params: Vec<String> = Vec::new();

        for field in &self.form_fields {
            if !field.name.is_empty() {
                // URL encode the value
                let encoded_value = field
                    .value
                    .replace(" ", "+")
                    .replace("&", "%26")
                    .replace("=", "%3D")
                    .replace("\"", "%22")
                    .replace("#", "%23");
                params.push(format!("{}={}", field.name, encoded_value));
            }
        }

        let query_string = params.join("&");

        // Clean form action - strip quotes, whitespace, and HTML entities
        let clean_action = self.form_action.as_ref().map(|a| {
            let mut cleaned = a.trim().to_string();
            // Remove surrounding quotes
            cleaned = cleaned.trim_matches('"').trim_matches('\'').to_string();
            // Decode any remaining HTML entities
            cleaned = html_escape::decode_html_entities(&cleaned).to_string();
            // Remove any remaining quotes
            cleaned = cleaned.replace("\"", "").replace("'", "");
            // Fix double protocol issue
            if cleaned.contains("https://https") || cleaned.contains("http://http") {
                if let Some(pos) = cleaned.rfind("http") {
                    cleaned = cleaned[pos..].to_string();
                }
            }
            cleaned
        });

        self.log(format!("Form action before clean: {:?}", self.form_action));
        self.log(format!("Form action after clean: {:?}", clean_action));

        // Get base URL from current page
        let base_url = {
            let parts: Vec<&str> = self.url_input.split('/').collect();
            if parts.len() >= 3 {
                format!("{}//{}", parts[0], parts[2])
            } else {
                self.url_input.clone()
            }
        };

        // Determine submission URL
        let submit_url = if let Some(ref action) = clean_action {
            if action.starts_with("http://") || action.starts_with("https://") {
                // Absolute URL
                if query_string.is_empty() {
                    action.clone()
                } else {
                    format!("{}?{}", action, query_string)
                }
            } else if action.starts_with("//") {
                // Protocol-relative URL
                let protocol = if self.url_input.starts_with("https") {
                    "https:"
                } else {
                    "http:"
                };
                if query_string.is_empty() {
                    format!("{}{}", protocol, action)
                } else {
                    format!("{}{}?{}", protocol, action, query_string)
                }
            } else if action.starts_with("/") {
                // Root-relative URL
                if query_string.is_empty() {
                    format!("{}{}", base_url, action)
                } else {
                    format!("{}{}?{}", base_url, action, query_string)
                }
            } else {
                // Relative URL or just a path
                if query_string.is_empty() {
                    format!("{}/{}", base_url, action)
                } else {
                    format!("{}/{}?{}", base_url, action, query_string)
                }
            }
        } else {
            // No action, submit to current URL
            if query_string.is_empty() {
                self.url_input.clone()
            } else if self.url_input.contains("?") {
                format!("{}&{}", self.url_input, query_string)
            } else {
                format!("{}?{}", self.url_input, query_string)
            }
        };

        self.log(format!(
            "Form submit: {} -> {}",
            self.form_method, submit_url
        ));
        self.url_input = submit_url;
        self.input_mode = InputMode::Normal;
        self.navigate().await;
    }

    async fn go_back(&mut self) {
        if self.history_index > 0 {
            self.history_index -= 1;
            self.url_input = self.history[self.history_index].clone();
            self.navigate().await;
        } else {
            self.status_message = "No earlier history".to_string();
        }
    }

    async fn go_forward(&mut self) {
        if self.history_index + 1 < self.history.len() {
            self.history_index += 1;
            self.url_input = self.history[self.history_index].clone();
            self.navigate().await;
        } else {
            self.status_message = "No later history".to_string();
        }
    }

    async fn follow_link(&mut self) {
        if let Ok(idx) = self.link_input.parse::<usize>() {
            if idx > 0 && idx <= self.links.len() {
                let link = self.links[idx - 1].clone();
                // Improved Relative URL Handling:
                // Check if it starts with 'http'
                let target = if link.starts_with("http") {
                    link
                } else if link.starts_with("//") {
                    format!("https:{}", link)
                } else if link.starts_with("/") {
                    // Join with host domain
                    // Quick extract of protocol+domain
                    if let Some(pos) = self.url_input.find("://") {
                        if let Some(end_host) =
                            self.url_input[pos + 3..].find('/').map(|i| i + pos + 3)
                        {
                            format!("{}{}", &self.url_input[..end_host], link)
                        } else {
                            format!("{}{}", self.url_input, link)
                        }
                    } else {
                        // Fallback?
                        format!("{}{}", self.url_input, link)
                    }
                } else {
                    // purely relative
                    if self.url_input.ends_with('/') {
                        format!("{}{}", self.url_input, link)
                    } else {
                        // Strip last part?
                        // "foo.com/bar/baz" + "qux" -> "foo.com/bar/qux"? or "foo.com/bar/baz/qux"?
                        // Usually relative joins to current "directory".
                        // Simplest is just append to full url /
                        format!("{}/{}", self.url_input, link)
                    }
                };

                // Clean any stray quotes from the URL
                self.url_input = target
                    .trim_matches('"')
                    .trim_matches('\'')
                    .replace("\"", "")
                    .to_string();
                self.navigate().await;
            } else {
                self.status_message = "Invalid Link ID".to_string();
            }
        } else {
            self.status_message = "Invalid Input".to_string();
        }
    }

    fn scroll_down(&mut self) {
        if !self.content.is_empty()
            && self.scroll_offset < (self.content.len() as u16).saturating_sub(1)
        {
            self.scroll_offset += 1;
        }
    }

    fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset -= 1;
        }
    }

    fn create_new_tab(&mut self) {
        if self.tabs.len() < 9 {
            // Save current tab state
            if let Some(tab) = self.tabs.get_mut(self.active_tab) {
                tab.url = self.url_input.clone();
                tab.content = self.content.clone();
                tab.links = self.links.clone();
                tab.scroll_offset = self.scroll_offset;
            }

            // Get engine name for new tab
            let engine_name = match self.search_engine {
                SearchEngine::Google => "Google",
                SearchEngine::DuckDuckGo => "DuckDuckGo",
                SearchEngine::Bing => "Bing",
                SearchEngine::Brave => "Brave",
                SearchEngine::Swisscows => "Swisscows",
                SearchEngine::Qwant => "Qwant",
            };

            // Create new tab with proper start page
            let new_tab = Tab::new(engine_name);
            self.content = new_tab.content.clone();
            self.links = new_tab.links.clone();
            self.tabs.push(new_tab);
            self.active_tab = self.tabs.len() - 1;
            self.url_input = String::new();
            self.scroll_offset = 0;
            self.form_fields.clear();
            self.status_message = format!("Tab {} created (1-9 to switch)", self.active_tab + 1);
        }
    }

    fn perform_search(&mut self) {
        if self.search_query.is_empty() {
            return;
        }
        for (i, line) in self.content.iter().enumerate() {
            if line
                .to_lowercase()
                .contains(&self.search_query.to_lowercase())
            {
                self.scroll_offset = i as u16;
                self.status_message = format!("Found at line {}", i + 1);
                return;
            }
        }
        self.status_message = format!("'{}' not found", self.search_query);
    }
}

const ASCII_ART: &str = r#"
███╗   ███╗███████╗████████╗██╗  ██╗ ██████╗ ██████╗ ██╗    ██╗██╗███████╗███████╗
████╗ ████║██╔════╝╚══██╔══╝██║  ██║██╔═══██╗██╔══██╗██║    ██║██║██╔════╝██╔════╝
██╔████╔██║█████╗     ██║   ███████║██║   ██║██║  ██║██║ █  ██║██║███████╗█████╗  
██║╚██╔╝██║██╔══╝     ██║   ██╔══██║██║   ██║██║  ██║█████████║██║╚════██║██╔══╝  
██║ ╚═╝ ██║███████╗   ██║   ██║  ██║╚██████╔╝██████╔╝  ██████ ║██║███████║███████╗
╚═╝     ╚═╝╚══════╝   ╚═╝   ╚═╝  ╚═╝ ╚═════╝ ╚═════╝  ╚══╝╚══╝╚══╝╚══════╝╚══════╝
                                                      >> METHODWISE.COM // ONLINE
"#;

// Theme color definitions
struct ThemeColors {
    primary: Color,      // Main accent color
    secondary: Color,    // Secondary accent
    text: Color,         // Text color
    border: Color,       // Border color
    bg_highlight: Color, // Background highlight
}

impl ThemeColors {
    fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark => ThemeColors {
                primary: Color::Cyan,
                secondary: Color::Blue,
                text: Color::White,
                border: Color::Cyan,
                bg_highlight: Color::DarkGray,
            },
            Theme::Light => ThemeColors {
                primary: Color::Blue,
                secondary: Color::Magenta,
                text: Color::Black,
                border: Color::Blue,
                bg_highlight: Color::Gray,
            },
            Theme::Retro => ThemeColors {
                primary: Color::Green,
                secondary: Color::Yellow,
                text: Color::Green,
                border: Color::Green,
                bg_highlight: Color::Black,
            },
            Theme::Ocean => ThemeColors {
                primary: Color::LightBlue,
                secondary: Color::Cyan,
                text: Color::White,
                border: Color::LightBlue,
                bg_highlight: Color::DarkGray,
            },
        }
    }
}

pub fn ui(f: &mut Frame, app: &BrowserApp) {
    let colors = ThemeColors::from_theme(app.theme);

    // 1. Main Layout: Header (ASCII), Address Bar, Tab Bar, Content, Footer
    let vertical_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8), // ASCII Art Area
            Constraint::Length(3), // Navbar
            Constraint::Length(1), // Tab Bar
            Constraint::Min(1),    // Main Content
            Constraint::Length(1), // Status Bar
        ])
        .split(f.area());

    // 2. Render ASCII Art Header
    let ascii_text = Paragraph::new(ASCII_ART)
        .style(
            Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
        )
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(ascii_text, vertical_chunks[0]);

    // 3. Navbar (Split into Back | Fwd | URL | Home | New Tab)
    let nav_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Length(7), // [ <]
            Constraint::Length(7), // [ >]
            Constraint::Min(10),   // Address
            Constraint::Length(7), // [ H]
            Constraint::Length(7), // [ +]
        ])
        .split(vertical_chunks[1]);

    // Back Button
    let back_btn = Paragraph::new(" < ")
        .style(
            Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.border)),
        );
    f.render_widget(back_btn, nav_chunks[0]);

    // Fwd Button
    let fwd_btn = Paragraph::new(" > ")
        .style(
            Style::default()
                .fg(colors.primary)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.border)),
        );
    f.render_widget(fwd_btn, nav_chunks[1]);

    // Address Bar (Middle)
    let input_style = if app.input_mode == InputMode::EditingUrl {
        Style::default().fg(Color::Yellow).bg(colors.bg_highlight)
    } else {
        Style::default().fg(colors.text)
    };

    let url_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(colors.border))
        .title(" URL / Search ");

    // Render URL with cursor at correct position when editing
    let url_display = if app.input_mode == InputMode::EditingUrl {
        let mut display = String::new();
        let chars: Vec<char> = app.url_input.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            if i == app.cursor_pos {
                display.push('█');
            }
            display.push(*c);
        }
        if app.cursor_pos >= chars.len() {
            display.push('█');
        }
        display
    } else {
        app.url_input.clone()
    };

    let url_paragraph = Paragraph::new(url_display.as_str())
        .style(input_style)
        .scroll((
            0,
            if url_display.len() > (nav_chunks[2].width as usize - 2) {
                (url_display.len() as u16).saturating_sub(nav_chunks[2].width - 2)
            } else {
                0
            },
        ))
        .block(url_block);
    f.render_widget(url_paragraph, nav_chunks[2]);

    // Home Button (Right)
    let home_btn = Paragraph::new(" H")
        .style(
            Style::default()
                .fg(colors.secondary)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.border)),
        );
    f.render_widget(home_btn, nav_chunks[3]);

    // New Tab Button
    let new_tab_btn = Paragraph::new(" +")
        .style(
            Style::default()
                .fg(Color::Green)
                .add_modifier(Modifier::BOLD),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(colors.border)),
        );
    f.render_widget(new_tab_btn, nav_chunks[4]);

    // 4. Tab Bar
    let tab_bar: String = app
        .tabs
        .iter()
        .enumerate()
        .map(|(i, tab)| {
            let title = if tab.url.is_empty() {
                "New Tab".to_string()
            } else {
                tab.url.chars().take(15).collect::<String>()
            };
            if i == app.active_tab {
                format!("[*{}: {}]", i + 1, title)
            } else {
                format!("[{}: {}]", i + 1, title)
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let tab_paragraph = Paragraph::new(tab_bar)
        .style(Style::default().fg(colors.primary))
        .alignment(ratatui::layout::Alignment::Left);
    f.render_widget(tab_paragraph, vertical_chunks[2]);

    // 5. Content Area
    let content_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Thick)
        .border_style(Style::default().fg(colors.border))
        .title(" Web View ");
    f.render_widget(content_block.clone(), vertical_chunks[3]);

    let inner_area = vertical_chunks[3].inner(Margin {
        vertical: 1,
        horizontal: 1,
    });

    // Calculate content slice
    let height = inner_area.height as usize;
    let start_idx = app.scroll_offset as usize;
    let end_idx = (start_idx + height).min(app.content.len());

    let page_content: Vec<Line> = if start_idx < app.content.len() {
        app.content[start_idx..end_idx]
            .iter()
            .map(|l| {
                if !app.search_query.is_empty()
                    && l.to_lowercase().contains(&app.search_query.to_lowercase())
                {
                    Line::from(Span::styled(
                        l,
                        Style::default().fg(Color::Black).bg(Color::Yellow),
                    ))
                } else {
                    Line::from(l.as_str())
                }
            })
            .collect()
    } else {
        vec![]
    };

    let paragraph = Paragraph::new(page_content).wrap(Wrap { trim: false });
    f.render_widget(paragraph, inner_area);

    // 5. Scrollbar
    let scrollbar = Scrollbar::default()
        .orientation(ScrollbarOrientation::VerticalRight)
        .begin_symbol(Some("▲"))
        .end_symbol(Some("▼"))
        .track_symbol(Some("│"))
        .thumb_symbol("█");

    let mut scrollbar_state = ScrollbarState::new(app.content.len().saturating_sub(height))
        .position(app.scroll_offset as usize);

    f.render_stateful_widget(
        scrollbar,
        vertical_chunks[2].inner(Margin {
            vertical: 1,
            horizontal: 0,
        }), // Render inside content block
        &mut scrollbar_state,
    );

    // Form Input Overlay - show current input prominently
    if app.input_mode == InputMode::FormInput {
        let editable_fields: Vec<_> = app
            .form_fields
            .iter()
            .filter(|f| f.field_type != "hidden")
            .collect();

        if let Some(field) = editable_fields.get(app.focused_field) {
            let label = if !field.placeholder.is_empty() {
                &field.placeholder
            } else if !field.name.is_empty() {
                &field.name
            } else {
                "Input"
            };

            let input_area = centered_rect(60, 10, f.area());
            f.render_widget(Clear, input_area);

            let input_block = Block::default()
                .title(format!(" Form Input: {} ", label))
                .borders(Borders::ALL)
                .border_type(BorderType::Double)
                .border_style(Style::default().fg(Color::Cyan));

            let input_text = format!("{}█", field.value);
            let input_para = Paragraph::new(input_text)
                .style(
                    Style::default()
                        .fg(Color::Yellow)
                        .add_modifier(Modifier::BOLD),
                )
                .block(input_block)
                .wrap(Wrap { trim: false });

            f.render_widget(input_para, input_area);
        }
    }

    // 6. Status Bar
    let mode_style = match app.input_mode {
        InputMode::Normal => Style::default()
            .bg(Color::Blue)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        InputMode::EditingUrl => Style::default()
            .bg(Color::Yellow)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
        InputMode::Searching => Style::default()
            .bg(Color::Magenta)
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
        InputMode::LinkFollow => Style::default()
            .bg(Color::Green)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
        InputMode::FormInput => Style::default()
            .bg(Color::Cyan)
            .fg(Color::Black)
            .add_modifier(Modifier::BOLD),
    };

    let scroll_info = if !app.content.is_empty() {
        let pct = (app.scroll_offset as f32 / app.content.len() as f32 * 100.0) as usize;
        format!("{}%", pct)
    } else {
        "TOP".to_string()
    };

    let status_line = Line::from(vec![
        Span::styled(
            format!(
                " {:^8} ",
                match app.input_mode {
                    InputMode::Normal => "NORMAL",
                    InputMode::EditingUrl => "EDIT",
                    InputMode::Searching => "SEARCH",
                    InputMode::LinkFollow => "LINK",
                    InputMode::FormInput => "FORM",
                }
            ),
            mode_style,
        ),
        Span::raw(" "),
        Span::styled(
            app.status_message.as_str(),
            Style::default().fg(colors.primary),
        ),
        Span::raw(" │ "),
        Span::styled(
            format!(" {} ", scroll_info),
            Style::default().fg(colors.secondary),
        ),
        Span::raw(" │ "),
        Span::styled(
            "?:Help  Tab:Forms  b:Bkmrk  H:Hist  1-9:Tabs",
            Style::default().fg(Color::DarkGray),
        ),
    ]);

    let status_block = Block::default().style(Style::default().bg(Color::Black));

    let status_paragraph = Paragraph::new(status_line).block(status_block);

    f.render_widget(status_paragraph, vertical_chunks[3]);

    // Overlays (Search, Link, Help) - Keep largely same but styled
    if app.input_mode == InputMode::Searching {
        let search_area = centered_rect(60, 20, f.area());
        f.render_widget(Clear, search_area);
        let search_text = format!("Search: {}", app.search_query);
        let search_block = Paragraph::new(search_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Find in Page ")
                    .border_style(Style::default().fg(Color::Magenta)),
            )
            .style(Style::default().fg(Color::White));
        f.render_widget(search_block, search_area);
    }

    if app.input_mode == InputMode::LinkFollow {
        let link_area = centered_rect(40, 10, f.area());
        f.render_widget(Clear, link_area);
        let link_text = format!("Link ID: {}", app.link_input);
        let link_block = Paragraph::new(link_text)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Follow Link ")
                    .border_style(Style::default().fg(Color::Green)),
            )
            .style(
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(link_block, link_area);
    }

    if app.show_help {
        let help_area = centered_rect(60, 70, f.area());
        f.render_widget(Clear, help_area);

        let help_text = vec![
            "Methodwise Help",
            "═══════════════════════════════",
            "",
            "NAVIGATION",
            "  j/k, ↑/↓   : Scroll Up/Down",
            "  h/l        : Back / Forward",
            "  PgUp/PgDn  : Page Scroll",
            "",
            "ACTIONS",
            "  e / g      : Edit URL / Search",
            "  Enter      : Load URL / Submit",
            "  0-9        : Follow Link (Direct)",
            "  f          : Follow Link (Type ID)",
            "  /          : Search in page",
            "",
            "FORMS",
            "  Tab        : Enter Form Mode",
            "  F1-F9      : Edit Form Field directly",
            "  Click      : Click on [F1:...] to edit",
            "",
            "TABS (when 2+ tabs)",
            "  Shift+T    : New Tab (max 9)",
            "  Shift+W    : Close Tab",
            "  1-9        : Switch to Tab #",
            "",
            "FEATURES",
            "  t          : Cycle Theme",
            "  b          : Toggle Bookmarks",
            "  B          : Add Bookmark",
            "  H          : Toggle History",
            "  F12        : Debug Console",
            "  ?          : Toggle Help",
            "",
            "Ctrl+Q to Quit",
        ];

        let help_p = Paragraph::new(help_text.join("\n"))
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Keyboard Shortcuts ")
                    .border_style(Style::default().fg(colors.primary)),
            )
            .style(Style::default().fg(colors.text));

        f.render_widget(help_p, help_area);
    }

    if app.show_debug {
        let debug_area = centered_rect(80, 50, f.area());
        f.render_widget(Clear, debug_area);

        let logs: Vec<Line> = app
            .debug_log
            .iter()
            .rev()
            .map(|l| Line::from(Span::styled(l, Style::default().fg(Color::Yellow))))
            .collect();

        let debug_p = Paragraph::new(logs)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Debug Log ")
                    .border_style(Style::default().fg(Color::Red)),
            )
            .wrap(Wrap { trim: true });

        f.render_widget(debug_p, debug_area);
    }

    // Bookmarks Overlay
    if app.show_bookmarks {
        let overlay_area = centered_rect(60, 60, f.area());
        f.render_widget(Clear, overlay_area);

        let items: Vec<Line> = if app.bookmarks.is_empty() {
            vec![Line::from(Span::styled(
                "No bookmarks yet. Press 'B' to add one.",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            app.bookmarks
                .iter()
                .enumerate()
                .map(|(i, url)| {
                    let style = if i == app.selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Cyan)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(Span::styled(format!(" {} ", url), style))
                })
                .collect()
        };

        let bm_p = Paragraph::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" Bookmarks (b) | d=delete | Enter=go ")
                .border_style(Style::default().fg(Color::Green)),
        );
        f.render_widget(bm_p, overlay_area);
    }

    // History Overlay
    if app.show_history {
        let overlay_area = centered_rect(60, 60, f.area());
        f.render_widget(Clear, overlay_area);

        let items: Vec<Line> = if app.history.is_empty() {
            vec![Line::from(Span::styled(
                "No history yet.",
                Style::default().fg(Color::DarkGray),
            ))]
        } else {
            app.history
                .iter()
                .rev()
                .enumerate()
                .map(|(i, url)| {
                    let actual_idx = app.history.len() - 1 - i;
                    let style = if actual_idx == app.selected_index {
                        Style::default()
                            .fg(Color::Black)
                            .bg(Color::Magenta)
                            .add_modifier(Modifier::BOLD)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    Line::from(Span::styled(format!(" {} ", url), style))
                })
                .collect()
        };

        let hist_p = Paragraph::new(items).block(
            Block::default()
                .borders(Borders::ALL)
                .title(" History (H) | Enter=go ")
                .border_style(Style::default().fg(Color::Magenta)),
        );
        f.render_widget(hist_p, overlay_area);
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}
