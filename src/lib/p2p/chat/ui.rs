//use crate::ui::solarized_dark;
//use crate::ui::solarized_light;

//use libp2p::{gossipsub, mdns, noise, swarm::NetworkBehaviour,
// swarm::SwarmEvent, tcp, yamux}; use ratatui::prelude::*;
use std::{
    env,
    error::Error,
    fs,
    io,
    process::Command,
    io::Write,
    sync::{Arc, Mutex},
    time::Duration,
};

use ratatui::{
    backend::{Backend, CrosstermBackend},
    crossterm::{
        cursor::Show,
        event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
        execute,
        terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    },
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Color,
    text::{Line, Span}, // Added Span here
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Wrap},
    Frame,
    Terminal,
};
use ratatui::{prelude::Stylize, style::Style};
use regex::Regex;
use std::rc::Rc;
use textwrap::{fill, Options};
use tui_input::{backend::crossterm::EventHandler, Input};
use uuid::Uuid;

use crate::ui::{
    draw_scrollbar,
    style::{SharedTheme, Theme},
    Orientation,
};
use crate::p2p::chat::msg::{self, MsgKind};

struct TerminalCleanup;

impl Drop for TerminalCleanup {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(io::stdout(), LeaveAlternateScreen, DisableMouseCapture, Show);
    }
}

#[derive(Default, PartialEq, Eq)] // Add PartialEq, Eq for comparison
pub enum AppMode {
    #[default]
    Normal,
    Editing,
    Command,
    Shell,
    SelectingDiff {
        diff_messages: Vec<msg::Msg>, // Filtered list of diff messages
        selected_index: usize,        // Index of the currently selected diff
        scroll_state: usize,          // Scroll position for the diff list
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ShellBatchCommand {
    label: String,
    command: String,
}

struct SuspendedTerminal;

impl SuspendedTerminal {
    fn suspend() -> io::Result<Self> {
        disable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, LeaveAlternateScreen, DisableMouseCapture, Show)?;
        Ok(Self)
    }
}

impl Drop for SuspendedTerminal {
    fn drop(&mut self) {
        let mut stdout = io::stdout();
        let _ = execute!(stdout, EnterAlternateScreen, EnableMouseCapture, Show);
        let _ = enable_raw_mode();
    }
}

/// App holds the state of the application
pub struct App {
    /// Current value of the input box
    pub input: Input,
    /// Current input mode
    pub mode: AppMode, // Renamed from input_mode to mode
    /// History of recorded messages
    pub messages: Arc<Mutex<Vec<msg::Msg>>>,
    pub _on_input_enter: Option<Box<dyn FnMut(msg::Msg)>>,
    pub msgs_scroll: usize,
    pub topic: String,
    pub show_side_panel: bool,
    pub show_help: bool,
    pub shell_output: Vec<String>,
}

impl Default for App {
    fn default() -> Self {
        App {
            input: Input::default(),
            mode: AppMode::default(), // Use new AppMode
            messages: Default::default(),
            _on_input_enter: None,
            msgs_scroll: usize::MAX,
            topic: String::from("gnostr"),
            show_side_panel: false,
            show_help: false,
            shell_output: Vec::new(),
        }
    }
}

impl App {
    pub fn on_submit<F: FnMut(msg::Msg) + 'static>(&mut self, hook: F) {
        self._on_input_enter = Some(Box::new(hook));
    }

    pub fn add_message(&self, msg: msg::Msg) {
        let mut msgs = self.messages.lock().unwrap();
        Self::add_msg(&mut msgs, msg);
    }

    fn add_msg(msgs: &mut Vec<msg::Msg>, msg: msg::Msg) {
        msgs.push(msg);
    }

    pub fn add_msg_fn(&self) -> Box<dyn FnMut(msg::Msg) + 'static + Send> {
        let m = self.messages.clone();
        Box::new(move |msg| {
            let mut msgs = m.lock().unwrap();
            Self::add_msg(&mut msgs, msg);
        })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        let _cleanup_guard = TerminalCleanup;
        // setup terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // run app
        run_app(&mut terminal, self)?;
        terminal.show_cursor()?;

        Ok(())
    }
}

fn gen_color_by_hash(s: &str) -> Color {
    static LIGHT_COLORS: [Color; 5] = [
        Color::LightMagenta,
        Color::LightGreen,
        Color::LightYellow,
        Color::LightBlue,
        Color::LightCyan,
        // Color::White,
    ];
    let h = s.bytes().fold(0, |acc, b| acc ^ b as usize);
    LIGHT_COLORS[h % LIGHT_COLORS.len()]
}

fn process_and_add_diff_message(app: &mut App, input_text: String) {
    let diff_content = input_text.strip_prefix("--diff ").unwrap_or(&input_text);
    let message_id = Uuid::new_v4().to_string();
    let lines: Vec<String> = diff_content.lines().map(|s| s.to_string()).collect();
    let total_chunks = lines.len(); // Each line is a chunk for diffs

    for (sequence_num, line) in lines.into_iter().enumerate() {
        let m = msg::Msg::default()
            .set_content(line, 0)
            .set_kind(MsgKind::GitDiff)
            .set_message_id(message_id.clone())
            .set_sequence_num(sequence_num)
            .set_total_chunks(total_chunks);
        app.add_message(m.clone());
        if let Some(ref mut hook) = app._on_input_enter {
            hook(m);
        }
    }
}

fn push_shell_output(app: &mut App, lines: impl IntoIterator<Item = String>) {
    for line in lines {
        app.shell_output.push(line);
    }
    let max_lines = 200usize;
    if app.shell_output.len() > max_lines {
        let drop_count = app.shell_output.len() - max_lines;
        app.shell_output.drain(0..drop_count);
    }
}

fn run_shell_command(command: &str) -> io::Result<Vec<String>> {
    #[cfg(target_family = "unix")]
    let output = {
        let shell = env::var("SHELL").unwrap_or_else(|_| "/bin/sh".to_string());
        Command::new(shell).arg("-lc").arg(command).output()?
    };
    #[cfg(target_family = "windows")]
    let output = Command::new("cmd").args(["/C", command]).output()?;

    let mut lines = vec![format!("$ {}", command)];
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    for line in stdout.lines() {
        lines.push(line.to_string());
    }
    for line in stderr.lines() {
        lines.push(format!("stderr: {}", line));
    }
    if !output.status.success() {
        lines.push(format!("exit status: {}", output.status));
    }
    Ok(lines)
}

fn preferred_editor() -> String {
    env::var("GIT_EDITOR")
        .ok()
        .or_else(|| env::var("VISUAL").ok())
        .or_else(|| env::var("EDITOR").ok())
        .unwrap_or_else(|| "vi".to_string())
}

fn open_editor_buffer(initial_contents: &str) -> io::Result<String> {
    let _terminal = SuspendedTerminal::suspend()?;
    let mut file = tempfile::NamedTempFile::new()?;
    file.write_all(initial_contents.as_bytes())?;
    file.flush()?;

    let editor = preferred_editor();
    let mut editor_parts =
        shellwords::split(&editor).map_err(|e| io::Error::other(e.to_string()))?;
    let command = editor_parts
        .first()
        .cloned()
        .ok_or_else(|| io::Error::other("EDITOR is empty"))?;
    let args = editor_parts.drain(1..).collect::<Vec<_>>();
    let status = Command::new(command).args(args).arg(file.path()).status()?;
    if !status.success() {
        return Err(io::Error::other(format!("editor exited with status {}", status)));
    }

    fs::read_to_string(file.path())
}

fn parse_editor_buffer(buffer: &str) -> Vec<String> {
    buffer
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(ToOwned::to_owned)
        .collect()
}

fn run_editor_batch() -> io::Result<Vec<String>> {
    let template = [
        "# Enter one shell command per line.",
        "# Blank lines and comments are ignored.",
        "# Save and exit to run the batch.",
        "",
    ]
    .join("\n");
    let buffer = open_editor_buffer(&template)?;
    let commands = parse_editor_buffer(&buffer);

    if commands.is_empty() {
        return Ok(vec!["batch editor: no commands to run".to_string()]);
    }

    let mut output = vec![format!("batch editor: {} command(s)", commands.len())];
    for (idx, command) in commands.iter().enumerate() {
        output.push(format!("[{}] {}", idx + 1, command));
        match run_shell_command(command) {
            Ok(lines) => output.extend(lines.into_iter().map(|line| format!("[{}] {}", idx + 1, line))),
            Err(err) => {
                output.push(format!("[{}] shell error: {}", idx + 1, err));
                output.push(format!("batch stopped at line {}", idx + 1));
                break;
            }
        }
    }

    Ok(output)
}

fn parse_shell_batch_command(input: &str) -> io::Result<Option<Vec<ShellBatchCommand>>> {
    let text = input.trim();
    if text.is_empty() {
        return Ok(None);
    }

    let re = Regex::new(r":(?P<label>[0-9]+|N)\s+").map_err(|e| io::Error::other(e.to_string()))?;
    let mut markers = Vec::new();

    for caps in re.captures_iter(text) {
        let m = caps
            .get(0)
            .ok_or_else(|| io::Error::other("invalid batch marker"))?;
        if m.start() == 0 || text[..m.start()].chars().last().is_some_and(char::is_whitespace) {
            markers.push((
                m.start(),
                m.end(),
                caps.name("label")
                    .ok_or_else(|| io::Error::other("missing batch label"))?
                    .as_str()
                    .to_string(),
            ));
        }
    }

    if markers.is_empty() || markers[0].0 != 0 {
        Ok(None)
    } else {
        let mut commands = Vec::new();
        for (index, (_start, end, label)) in markers.iter().enumerate() {
            let command_start = *end;
            let command_end = markers
                .get(index + 1)
                .map(|(next_start, _, _)| *next_start)
                .unwrap_or(text.len());
            let command = text[command_start..command_end].trim();
            if command.is_empty() {
                return Ok(None);
            }
            commands.push(ShellBatchCommand {
                label: label.clone(),
                command: command.to_string(),
            });
        }
        Ok(Some(commands))
    }
}

fn run_shell_batch(commands: &[ShellBatchCommand]) -> io::Result<Vec<String>> {
    let mut output = vec![format!("batch: {} command(s)", commands.len())];
    for command in commands {
        output.push(format!("[:{}] {}", command.label, command.command));
        match run_shell_command(&command.command) {
            Ok(lines) => {
                output.extend(lines.into_iter().map(|line| format!("[:{}] {}", command.label, line)));
            }
            Err(err) => {
                output.push(format!("[:{}] shell error: {}", command.label, err));
                output.push(format!("batch stopped at :{}", command.label));
                break;
            }
        }
    }
    Ok(output)
}

fn execute_colon_command(app: &mut App, command_text: &str) -> io::Result<Option<msg::Msg>> {
    let command_text = command_text.trim();
    if command_text.is_empty() {
        return Ok(None);
    }

    if command_text == "help" || command_text == "h" {
        app.show_help = true;
        return Ok(None);
    }

    if command_text == "q" || command_text == "quit" {
        return Ok(Some(
            msg::Msg::default()
                .set_content("quit".to_string(), 0)
                .set_kind(MsgKind::System),
        ));
    }

    if command_text == "shell" || command_text == "sh" {
        if !matches!(app.mode, AppMode::Shell) {
            app.shell_output.clear();
        }
        app.mode = AppMode::Shell;
        push_shell_output(
            app,
            [
                "shell mode ready".to_string(),
                "use :exit or :x to return to chat".to_string(),
            ],
        );
        return Ok(None);
    }

    if command_text == "B" {
        app.mode = AppMode::Shell;
        app.shell_output.clear();
        let lines = run_editor_batch()?;
        push_shell_output(app, lines);
        return Ok(None);
    }

    if let Some(shell_command) = command_text.strip_prefix('!') {
        if !matches!(app.mode, AppMode::Shell) {
            app.shell_output.clear();
        }
        app.mode = AppMode::Shell;
        let lines = if let Some(batch) = parse_shell_batch_command(shell_command.trim())? {
            run_shell_batch(&batch)?
        } else {
            run_shell_command(shell_command.trim())?
        };
        push_shell_output(app, lines);
        return Ok(None);
    }

    Ok(Some(
        msg::Msg::default()
            .set_content(format!("unknown command: :{}", command_text), 0)
            .set_kind(MsgKind::System),
    ))
}

fn help_text() -> Vec<&'static str> {
    vec![
        "GNOSTR CHAT HELP",
        "",
        "Keys",
        "  \\  open/close this help",
        "  :  enter command mode",
        "  e/i enter edit mode",
        "  Esc leave edit mode or close help",
        "  q quit",
        "  d open diff picker",
        "  arrows scroll messages",
        "  Ctrl-C quit immediately",
        "",
        "Commands",
        "  :help / :h show this help",
        "  :shell / :sh open shell mode",
        "  :B open $EDITOR buffer; run one shell command per line",
        "  :!<cmd> run a shell command and stay in shell mode",
        "  :1 cmd :2 cmd ... :N cmd batch shell commands in order",
        "  :exit / :x close shell mode",
        "  :q / :quit exit chat",
        "  /clone <blossom-url> [dest]",
        "  /git clone <blossom-url> [dest]",
        "  /blossom clone <blossom-url> [dest]",
        "",
        "Blossom URLs",
        "  blossom://<host>/<pubkey-hex>/<repo>",
        "  blossom+https://<host>/<pubkey-hex>/<repo>",
        "",
        "Behavior",
        "  /clone runs locally and is not broadcast.",
        "  Plain chat messages are fanned out to both p2p swarms.",
        "  --diff <patch> creates a structured diff message.",
    ]
}

fn help_lines() -> Vec<Line<'static>> {
    help_text().into_iter().map(Line::from).collect()
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ]
            .as_ref(),
        )
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints(
            [
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ]
            .as_ref(),
        )
        .split(popup_layout[1])[1]
}

#[cfg(test)]
mod tests {
    use super::{execute_colon_command, help_text, App};

    #[test]
    fn help_text_mentions_clone_and_help_keys() {
        let text = help_text().join("\n");
        assert!(text.contains("/clone <blossom-url> [dest]"));
        assert!(text.contains("open/close this help"));
        assert!(text.contains(":shell / :sh open an interactive shell"));
        assert!(text.contains("Plain chat messages are fanned out to both p2p swarms."));
    }

    #[test]
    fn colon_help_enables_help_overlay() {
        let mut app = App::default();
        let result = execute_colon_command(&mut app, "help").expect("command should parse");
        assert!(result.is_none());
        assert!(app.show_help);
    }

    #[test]
    fn colon_shell_enters_shell_mode() {
        let mut app = App::default();
        let result = execute_colon_command(&mut app, "shell").expect("command should parse");
        assert!(result.is_none());
        assert!(matches!(app.mode, AppMode::Shell));
    }

    #[test]
    fn batch_shell_command_is_parsed() {
        let batch = parse_shell_batch_command(":1 ls :2 cat README.md :3 git status :N echo done")
            .expect("parse batch")
            .expect("expected batch");
        assert_eq!(batch.len(), 4);
        assert_eq!(batch[0].label, "1");
        assert_eq!(batch[0].command, "ls");
        assert_eq!(batch[3].label, "N");
        assert_eq!(batch[3].command, "echo done");
    }

    #[test]
    fn batch_shell_command_preserves_quotes() {
        let batch = parse_shell_batch_command(r#":1 echo "hello world" :2 git status"#)
            .expect("parse batch")
            .expect("expected batch");
        assert_eq!(batch[0].command, r#"echo "hello world""#);
        assert_eq!(batch[1].command, "git status");
    }

    #[test]
    fn editor_buffer_ignores_comments_and_blanks() {
        let commands = parse_editor_buffer(
            "# comment\n\nls -la\n  # another comment\ncat README.md\n",
        );
        assert_eq!(commands, vec!["ls -la", "cat README.md"]);
    }
}

fn run_app<B: Backend>(terminal: &mut Terminal<B>, app: &mut App) -> io::Result<()> {
    let tick_rate = Duration::from_millis(100);
    loop {
        terminal.draw(|f| ui(f, app))?;

        if !event::poll(tick_rate)? {
            continue;
        }

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('c')
                && key.modifiers.contains(event::KeyModifiers::CONTROL)
            {
                return Ok(());
            }

            if app.show_help {
                match key.code {
                    KeyCode::Char('\\') | KeyCode::Esc | KeyCode::Char('q') => {
                        app.show_help = false;
                    }
                    _ => {}
                }
                continue;
            }

            match app.mode {
                // Changed from app.input_mode
                AppMode::Normal => match key.code {
                KeyCode::Char('e') | KeyCode::Char('i') => {
                    app.mode = AppMode::Editing; // Changed from app.input_mode
                    app.msgs_scroll = usize::MAX;
                }
                KeyCode::Char(':') => {
                    app.mode = AppMode::Command;
                    app.input.reset();
                }
                KeyCode::Char('q') => {
                    return Ok(());
                }
                    KeyCode::Char('\\') => {
                        app.show_help = true;
                    }
                    KeyCode::Char('d') => {
                        // New keybinding for selecting diffs
                        let all_messages = app.messages.lock().unwrap();
                        let diff_messages: Vec<msg::Msg> = all_messages
                            .iter()
                            .filter(|m| m.kind == MsgKind::GitDiff)
                            .cloned() // Clone to move into the new state
                            .collect();

                        if !diff_messages.is_empty() {
                            app.mode = AppMode::SelectingDiff {
                                diff_messages,
                                selected_index: 0,
                                scroll_state: 0,
                            };
                        }
                    }
                    KeyCode::Up => {
                        let l = app.messages.lock().unwrap().len();
                        app.msgs_scroll = app.msgs_scroll.saturating_sub(1).min(l);
                    }
                    KeyCode::Down => {
                        let l = app.messages.lock().unwrap().len();
                        app.msgs_scroll = app.msgs_scroll.saturating_add(1).min(l);
                    }
                    KeyCode::Esc => {
                        app.msgs_scroll = usize::MAX;
                        app.msgs_scroll = usize::MAX;
                        app.input.reset();
                    }
                    _ => {
                        app.msgs_scroll = usize::MAX;
                    }
                },
                AppMode::Editing => match key.code {
                    // Changed from InputMode::Editing
                    KeyCode::Enter => {
                        if !app.input.value().trim().is_empty() {
                            let input_text = app.input.value().to_owned();
                            if input_text.starts_with("--diff ") {
                                process_and_add_diff_message(app, input_text);
                            } else if input_text.starts_with('/') {
                                let m = msg::Msg::default()
                                    .set_content(input_text, 0)
                                    .set_kind(MsgKind::Command);
                                app.add_message(m.clone());
                                if let Some(ref mut hook) = app._on_input_enter {
                                    hook(m);
                                }
                            } else {
                                let wrapped_lines: Vec<String> = fill(&input_text, 80)
                                    .split('\n')
                                    .map(|s| s.to_string())
                                    .collect();
                                let total_chunks = wrapped_lines.len();
                                let message_id = Uuid::new_v4().to_string();

                                for (sequence_num, line) in wrapped_lines.into_iter().enumerate() {
                                    if !line.trim().is_empty() {
                                        let m = msg::Msg::default()
                                            .set_content(line, 0)
                                            .set_message_id(message_id.clone())
                                            .set_sequence_num(sequence_num)
                                            .set_total_chunks(total_chunks);
                                        app.add_message(m.clone());
                                        if let Some(ref mut hook) = app._on_input_enter {
                                            hook(m);
                                        }
                                    }
                                }
                            }
                        }
                        app.input.reset();
                    }
                    KeyCode::Esc => {
                        app.mode = AppMode::Normal; // Changed from app.input_mode
                        app.msgs_scroll = app.messages.lock().unwrap().len();
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key));
                    }
                },
                AppMode::Command => match key.code {
                    KeyCode::Enter => {
                        let command_text = app.input.value().to_owned();
                        app.input.reset();
                        app.mode = AppMode::Normal;

                        match execute_colon_command(app, &command_text) {
                            Ok(Some(msg)) => {
                                if msg.content[0] == "quit" {
                                    return Ok(());
                                }
                                app.add_message(msg.clone());
                                if let Some(ref mut hook) = app._on_input_enter {
                                    hook(msg);
                                }
                            }
                            Ok(None) => {}
                            Err(err) => {
                                app.add_message(
                                    msg::Msg::default()
                                        .set_content(err.to_string(), 0)
                                        .set_kind(MsgKind::System),
                                );
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.input.reset();
                        app.mode = AppMode::Normal;
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key));
                    }
                },
                AppMode::Shell => match key.code {
                    KeyCode::Enter => {
                        let command_text = app.input.value().to_owned();
                        app.input.reset();

                        if matches!(command_text.trim(), ":exit" | ":x" | "exit" | "x") {
                            app.mode = AppMode::Normal;
                            app.shell_output.clear();
                        } else if !command_text.trim().is_empty() {
                            match parse_shell_batch_command(&command_text) {
                                Ok(Some(batch)) => match run_shell_batch(&batch) {
                                    Ok(lines) => push_shell_output(app, lines),
                                    Err(err) => push_shell_output(
                                        app,
                                        [format!("shell error: {}", err)],
                                    ),
                                },
                                Ok(None) => match run_shell_command(&command_text) {
                                    Ok(lines) => push_shell_output(app, lines),
                                    Err(err) => push_shell_output(
                                        app,
                                        [format!("shell error: {}", err)],
                                    ),
                                },
                                Err(err) => push_shell_output(
                                    app,
                                    [format!("shell error: {}", err)],
                                ),
                            }
                        }
                    }
                    KeyCode::Esc => {
                        app.input.reset();
                        app.mode = AppMode::Normal;
                    }
                    _ => {
                        app.input.handle_event(&Event::Key(key));
                    }
                },
                AppMode::SelectingDiff {
                    ref mut diff_messages,
                    ref mut selected_index,
                    scroll_state: _,
                } => {
                    match key.code {
                        KeyCode::Up => {
                            if *selected_index > 0 {
                                *selected_index -= 1;
                            }
                        }
                        KeyCode::Down => {
                            if *selected_index < diff_messages.len() - 1 {
                                *selected_index += 1;
                            }
                        }
                        KeyCode::Enter => {
                            // Display the selected diff
                            let selected_diff = diff_messages[*selected_index].clone();
                            // Clear existing messages and add the selected diff for display
                            let mut all_messages = app.messages.lock().unwrap();
                            let oneshot_messages: Vec<msg::Msg> = all_messages
                                .iter()
                                .filter(|m| m.kind == MsgKind::OneShot)
                                .cloned()
                                .collect();

                            // TODO: remove duplicates by p2p message id
                            // all_messages.clear(); // Clear existing messages
                            all_messages.extend(oneshot_messages); // Add existing OneShot messages back
                            all_messages.push(selected_diff); // Add the selected diff
                                                              // TODO: handle better
                                                              // app.msgs_scroll = usize::MAX; // Scroll to bottom

                            app.mode = AppMode::Normal; // Exit selection mode
                        }
                        KeyCode::Esc => {
                            app.mode = AppMode::Normal; // Exit selection mode
                        }
                        _ => {}
                    }
                }
            }
        }
    }
}

//as popup widget is constructed in chat_details/mos.rs
fn ui(f: &mut Frame, app: &App) {
    let theme: SharedTheme = Rc::new(Theme::default());
    let main_layout_constraints = if app.show_side_panel {
        vec![Constraint::Percentage(70), Constraint::Percentage(30)]
    } else {
        vec![Constraint::Percentage(100)]
    };

    let main_layout_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(main_layout_constraints)
        .split(f.area());

    let message_and_input_area = main_layout_chunks[0];

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(
            [
                Constraint::Length(3), // Header height
                Constraint::Fill(5),   // Messages height
                Constraint::Length(3), // Input height
            ]
            .as_ref(),
        )
        .split(message_and_input_area);

    // Header Widget
    let header_text = vec![Line::from(app.topic.as_str())];
    let header =
        Paragraph::new(header_text).block(Block::default().borders(Borders::ALL).title("Topic"));
    f.render_widget(header, chunks[0]);

    // Messages Widget
    let height = chunks[1].height; // Re-introduce height variable
    let msgs = app.messages.lock().unwrap();
    let scroll_pos = if app.msgs_scroll == usize::MAX {
        msgs.len()
    } else {
        app.msgs_scroll.min(msgs.len())
    };
    let show_scrollbar = msgs.len() > height as usize;
    let message_area_width = chunks[1].width.saturating_sub(if show_scrollbar { 1 } else { 0 });

    let mut messages: Vec<ListItem> = Vec::new();

    for msg in msgs.iter().rev() {
        match msg.kind {
            MsgKind::Chat => {
                let mut chat_spans: Vec<ratatui::text::Span> = Vec::new();
                let (prefix, indent) = if msg.from == *msg::USER_NAME {
                    (
                        format!("{}{} ", &msg.from, ">"),
                        " ".repeat(msg.from.len() + 2),
                    )
                } else {
                    (
                        format!(" {}{}", &msg.from, "> "),
                        " ".repeat(msg.from.len() + 3),
                    )
                };
                let prefix_style = Style::default().fg(gen_color_by_hash(&msg.from));
                chat_spans.push(ratatui::text::Span::styled(prefix.clone(), prefix_style));

                let content_width = message_area_width.saturating_sub(prefix.len() as u16);
                let wrapped_content =
                    textwrap::wrap(&msg.content[0], Options::new(content_width as usize));

                for (idx, segment) in wrapped_content.into_iter().enumerate() {
                    if idx > 0 {
                        chat_spans.push(ratatui::text::Span::raw("\n"));
                        chat_spans.push(ratatui::text::Span::raw(indent.clone()));
                    }
                    chat_spans.push(ratatui::text::Span::raw(segment.to_string()));
                }
                messages.push(ListItem::new(Line::from(chat_spans)));
            }
            MsgKind::GitDiff => {
                for line_content in msg.content.iter() {
                    // Iterate directly over pre-wrapped lines
                    let style = if line_content.starts_with('+') {
                        Style::default().fg(Color::Green)
                    } else if line_content.starts_with('-') {
                        Style::default().fg(Color::Red)
                    } else if line_content.starts_with('@') || line_content.starts_with('\\') {
                        Style::default().fg(Color::Cyan)
                    } else {
                        Style::default().fg(Color::White)
                    };
                    messages.push(ListItem::new(Line::from(Span::styled(
                        line_content.clone(),
                        style,
                    ))));
                }
            }
            _ => {
                // For other MsgKind, directly convert to ListItem
                messages.push(ListItem::new(ratatui::text::Text::from(Line::from(msg))));
            }
        }
    }
    messages.truncate(height as usize); // Take only the visible number of lines

    let messages = List::new(messages)
        .direction(ratatui::widgets::ListDirection::BottomToTop)
        .block(Block::default().borders(Borders::NONE));
    f.render_widget(messages, chunks[1]);

    if show_scrollbar {
        draw_scrollbar(
            f,
            chunks[1],
            &theme,
            msgs.len(),
            scroll_pos,
            Orientation::Vertical,
        );
    }

    if let AppMode::SelectingDiff {
        diff_messages,
        selected_index,
        scroll_state: _,
    } = &app.mode
    {
        let popup_area = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(f.area())[1];

        let popup_area = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(20),
                Constraint::Percentage(60),
                Constraint::Percentage(20),
            ])
            .split(popup_area)[1];

        f.render_widget(Clear, popup_area); // Clear the area first

        let items: Vec<ListItem> = diff_messages
            .iter()
            .enumerate()
            .map(|(i, msg)| {
                let mut summary = String::new();
                if let Some(first_line) = msg.content.first() {
                    // Take a snippet of the first line as a summary
                    summary = first_line
                        .chars()
                        .take(popup_area.width as usize - 4)
                        .collect(); // -4 for borders
                }
                let content = if i == *selected_index {
                    format!("> {}", summary)
                } else {
                    format!("  {}", summary)
                };
                ListItem::new(content).style(Style::default().fg(Color::Yellow))
            })
            .collect();

        let mut list_state = ListState::default();
        list_state.select(Some(*selected_index));

        let diff_list = List::new(items)
            .block(Block::default().borders(Borders::ALL).title("Select Diff"))
            .highlight_style(Style::default().fg(Color::Black).bg(Color::White));

        f.render_stateful_widget(diff_list, popup_area, &mut list_state);
    }

    // Input Widget
    let width = chunks[2].width.max(3) - 3; // Use width of the input chunk
    let scroll = app.input.visual_scroll(width as usize);

    let input_str = app.input.value();
    let mut spans = Vec::new();
    let default_input_style = match app.mode {
        AppMode::Normal => Style::default(),
        AppMode::Editing => Style::default().fg(Color::Cyan),
        AppMode::Command => Style::default().fg(Color::Yellow),
        AppMode::Shell => Style::default().fg(Color::Green),
        AppMode::SelectingDiff { .. } => Style::default().fg(Color::DarkGray), /* Indicate non-editable */
    };

    for (i, c) in input_str.chars().enumerate() {
        if (i + 1) % 80 == 0 {
            // Highlight every 80th character
            spans.push(ratatui::text::Span::styled(
                c.to_string(),
                default_input_style.fg(Color::Red),
            ));
        } else {
            spans.push(ratatui::text::Span::styled(
                c.to_string(),
                default_input_style,
            ));
        }
    }

    let input_line = Line::from(spans);

    let input = Paragraph::new(input_line)
        .style(default_input_style)
        .scroll((0, scroll as u16))
        .block(Block::default().borders(Borders::ALL).title(match app.mode {
            AppMode::Command => "Command",
            AppMode::Shell => "Shell",
            _ => "Input",
        }));
    f.render_widget(input, chunks[2]);

    match app.mode {
        AppMode::Normal => {}
        AppMode::Editing => f.set_cursor_position((
            chunks[2].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            chunks[2].y + 1,
        )),
        AppMode::Command => f.set_cursor_position((
            chunks[2].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            chunks[2].y + 1,
        )),
        AppMode::Shell => f.set_cursor_position((
            chunks[2].x + ((app.input.visual_cursor()).max(scroll) - scroll) as u16 + 1,
            chunks[2].y + 1,
        )),
        AppMode::SelectingDiff { .. } => {} // No cursor in this mode
    }

    // Render side panel if active
    if app.show_side_panel {
        let side_panel_area = main_layout_chunks[1];
        let side_panel = Block::default()
            .borders(Borders::ALL)
            .title("Side Panel")
            .fg(Color::White);
        f.render_widget(side_panel, side_panel_area);
    }

    if app.show_help {
        let help_area = centered_rect(80, 80, f.area());
        f.render_widget(Clear, help_area);
        let help_block = Block::default()
            .borders(Borders::ALL)
            .title("Help")
            .fg(Color::White);
        let help = Paragraph::new(help_lines())
            .block(help_block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        f.render_widget(help, help_area);
    }

    if matches!(app.mode, AppMode::Shell) {
        let shell_area = centered_rect(90, 70, f.area());
        f.render_widget(Clear, shell_area);
        let shell_block = Block::default()
            .borders(Borders::ALL)
            .title("Shell")
            .fg(Color::Green);
        let transcript: Vec<Line<'static>> = app
            .shell_output
            .iter()
            .map(|line| Line::from(line.clone()))
            .collect();
        let shell = Paragraph::new(transcript)
            .block(shell_block)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false });
        f.render_widget(shell, shell_area);
    }
}
