use std::{
    collections::BTreeSet,
    error::Error,
    fs,
    io,
    io::Write,
    io::stdout,
    path::{Path, PathBuf},
    process::Command,
    sync::mpsc::{self, Receiver},
    sync::OnceLock,
    thread,
    time::Duration,
};

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use filetreelist::{FileTree, MoveSelection};
use ratatui::{
    backend::CrosstermBackend,
    prelude::*,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
};
use gnostr_relay::App as RelayApp;

use crate::{nip34_browser, upstream, workflow};

static LOCAL_RELAY_BOOTSTRAPPED: OnceLock<()> = OnceLock::new();

pub fn collect_checkout_paths(dir: impl AsRef<Path>) -> io::Result<Vec<PathBuf>> {
    let output = Command::new("git")
        .args(["ls-files", "--cached", "--others", "--exclude-standard"])
        .current_dir(dir)
        .output()?;

    if !output.status.success() {
        return Err(io::Error::other(format!(
            "git ls-files failed with exit code {:?}\nstdout: {}\nstderr: {}",
            output.status.code(),
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        )));
    }

    let mut paths: Vec<PathBuf> = String::from_utf8(output.stdout)
        .map_err(io::Error::other)?
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .map(PathBuf::from)
        .collect();

    paths.sort();
    paths.dedup();
    Ok(paths)
}

fn build_tree(paths: &[PathBuf]) -> io::Result<FileTree> {
    let path_refs: Vec<&Path> = paths.iter().map(PathBuf::as_path).collect();
    let collapsed: BTreeSet<&String> = BTreeSet::new();
    filetreelist::FileTree::new(&path_refs, &collapsed).map_err(|err| io::Error::other(err.to_string()))
}

struct App {
    checkout_dir: PathBuf,
    tree: FileTree,
    content_scroll: u16,
    show_tree: bool,
    show_toolbar: bool,
    show_help: bool,
    force_full_repaint: bool,
    nip34_browser: Option<nip34_browser::Nip34Browser>,
    proposal_task: Option<ProposalTask>,
    status_line: String,
}

enum ProposalUpdate {
    Log(String),
    Done(Result<String, String>),
}

struct ProposalTask {
    receiver: Receiver<ProposalUpdate>,
    logs: Vec<String>,
    result: Option<Result<String, String>>,
}

impl ProposalTask {
    fn new(receiver: Receiver<ProposalUpdate>) -> Self {
        Self {
            receiver,
            logs: Vec::new(),
            result: None,
        }
    }

    fn drain(&mut self) -> Option<Result<String, String>> {
        let was_finished = self.result.is_some();
        while let Ok(update) = self.receiver.try_recv() {
            match update {
                ProposalUpdate::Log(line) => self.logs.push(line),
                ProposalUpdate::Done(result) => self.result = Some(result),
            }
        }

        if !was_finished {
            self.result.clone()
        } else {
            None
        }
    }

    fn is_active(&self) -> bool {
        self.result.is_none()
    }
}

impl App {
    fn new(checkout_dir: PathBuf, paths: Vec<PathBuf>) -> io::Result<Self> {
        let branch = workflow::current_branch(&checkout_dir).unwrap_or_else(|_| String::from("HEAD"));
        Ok(Self {
            checkout_dir,
            tree: build_tree(&paths)?,
            content_scroll: 0,
            show_tree: true,
            show_toolbar: true,
            show_help: false,
            force_full_repaint: true,
            nip34_browser: None,
            proposal_task: None,
            status_line: status_for_branch(&branch),
        })
    }

    fn reload(&mut self) -> io::Result<()> {
        let selected = self.selected_entry_path();
        let paths = collect_checkout_paths(&self.checkout_dir)?;
        self.tree = build_tree(&paths)?;
        if let Some(selected) = selected {
            let _ = self.tree.select_file(selected.as_path());
        }
        self.content_scroll = 0;
        let branch = workflow::current_branch(&self.checkout_dir).unwrap_or_else(|_| String::from("HEAD"));
        self.status_line = status_for_branch(&branch);
        Ok(())
    }

    fn selected_entry_path(&self) -> Option<PathBuf> {
        let visible = self.tree.visual_selection().map(|v| v.count).unwrap_or(0).max(1);
        self.tree
            .iterate(0, visible)
            .find(|(_, selected)| *selected)
            .map(|(item, _)| item.info().full_path().to_path_buf())
    }

    fn selected_file_path(&self) -> Option<PathBuf> {
        self.tree
            .selected_file()
            .map(|info| self.checkout_dir.join(info.full_path()))
    }

    fn move_selection(&mut self, direction: MoveSelection) {
        if self.tree.move_selection(direction) {
            self.content_scroll = 0;
        }
    }

    fn page_move(&mut self, down: bool) {
        for _ in 0..10 {
            if !self.tree.move_selection(if down {
                MoveSelection::Down
            } else {
                MoveSelection::Up
            }) {
                break;
            }
        }
        self.content_scroll = 0;
    }
}

fn status_for_branch(branch: &str) -> String {
    format!(
        "{branch}  e: edit  p: propose  P: nip34 browser  g: git ui  n: new branch  c: checkout  r: refresh  \\: help  .: toolbar  q: quit"
    )
}

fn content_for_selection(app: &App) -> (String, String) {
    if let Some(selected_file) = app.selected_file_path() {
        let title = selected_file
            .strip_prefix(&app.checkout_dir)
            .unwrap_or(selected_file.as_path())
            .display()
            .to_string();
        let content = fs::read_to_string(&selected_file)
            .unwrap_or_else(|_| format!("Error reading file: {}", selected_file.display()));
        return (title, content);
    }

    if let Some(selected_entry) = app.selected_entry_path() {
        let title = selected_entry.display().to_string();
        return (title.clone(), format!("Directory selected: {title}"));
    }

    (String::from("Content"), String::from("No file selected."))
}

fn render_tree(frame: &mut Frame, app: &App, area: Rect) {
    let visible = app.tree.visual_selection().map(|v| v.count).unwrap_or(0).max(1);
    let items: Vec<ListItem> = app
        .tree
        .iterate(0, visible)
        .map(|(item, selected)| {
            let indent = "  ".repeat(item.info().indent() as usize);
            let marker = if item.kind().is_path() {
                if item.kind().is_path_collapsed() {
                    "▸ "
                } else {
                    "▾ "
                }
            } else {
                "• "
            };
            let label = format!("{indent}{marker}{}", item.info().path_str());
            let style = if selected {
                Style::default().fg(Color::Black).bg(Color::LightGreen)
            } else if item.kind().is_path() {
                Style::default().fg(Color::Yellow)
            } else {
                Style::default()
            };

            ListItem::new(label).style(style)
        })
        .collect();

    let list = List::new(items).block(Block::default().borders(Borders::ALL).title("Files"));
    frame.render_widget(list, area);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let header = Paragraph::new(Text::from(vec![
        Line::from(""),
        Line::from(Span::styled(
            "gnostr/nips",
            Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD),
        )),
        Line::from(""),
        Line::from(""),
        Line::from(""),
    ]))
    .alignment(Alignment::Center);
    frame.render_widget(header, area);
}

fn toolbar_context(app: &App) -> (String, Style) {
    if let Some(file_path) = app.selected_file_path() {
        let rel = file_path
            .strip_prefix(&app.checkout_dir)
            .unwrap_or(file_path.as_path())
            .display()
            .to_string();
        return (format!("file: {rel}"), Style::default().fg(Color::Green));
    }

    if let Some(entry_path) = app.selected_entry_path() {
        let rel = entry_path
            .strip_prefix(&app.checkout_dir)
            .unwrap_or(entry_path.as_path())
            .display()
            .to_string();
        return (format!("directory: {rel}"), Style::default().fg(Color::Yellow));
    }

    (String::from("no selection"), Style::default().fg(Color::DarkGray))
}

fn toolbar_action(key: &str, label: &str, enabled: bool) -> Span<'static> {
    let style = if enabled {
        Style::default().fg(Color::White).add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::DarkGray)
    };
    Span::styled(format!("{key} {label}"), style)
}

fn render_toolbar(frame: &mut Frame, app: &App, area: Rect) {
    let (context, context_style) = toolbar_context(app);
    let file_selected = app.selected_file_path().is_some();

    let lines = vec![
        Line::from(vec![
            Span::styled(&app.status_line, Style::default().fg(Color::White)),
        ]),
        Line::from(vec![
            Span::styled("Context: ", Style::default().fg(Color::Gray)),
            Span::styled(context, context_style),
        ]),
        Line::from(vec![
            toolbar_action("e", "edit", file_selected),
            Span::raw("  "),
            toolbar_action("p", "propose", file_selected),
            Span::raw("  "),
            toolbar_action("n", "new branch", true),
            Span::raw("  "),
            toolbar_action("c", "checkout", true),
            Span::raw("  "),
            toolbar_action("P", "nip34 browser", true),
            Span::raw("  "),
            toolbar_action("g", "git ui", true),
            Span::raw("  "),
            toolbar_action("Enter", "toggle tree", true),
            Span::raw("  "),
            toolbar_action("r", "refresh", true),
            Span::raw("  "),
            toolbar_action("\\", "help", true),
            Span::raw("  "),
            toolbar_action(".", "toolbar", true),
        ]),
    ];

    let toolbar = Paragraph::new(Text::from(lines)).wrap(Wrap { trim: false });
    frame.render_widget(toolbar, area);
}

fn render_proposal_popup(frame: &mut Frame, app: &App, area: Rect) {
    let mut lines = Vec::new();
    lines.push(Line::from(vec![
        Span::styled("Proposal log: ", Style::default().fg(Color::Gray)),
        Span::styled(
            match app.proposal_task.as_ref().and_then(|task| task.result.as_ref()) {
                None => "running".to_string(),
                Some(Ok(hash)) => format!("done {hash}"),
                Some(Err(err)) => format!("failed {err}"),
            },
            Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
        ),
    ]));

    if let Some(task) = app.proposal_task.as_ref() {
        let start = task.logs.len().saturating_sub(3);
        for line in task.logs.iter().skip(start) {
            lines.push(Line::from(Span::raw(line.clone())));
        }
    }

    lines.push(Line::from(vec![
        Span::styled("Esc", Style::default().fg(Color::Yellow)),
        Span::raw(" dismiss  "),
        Span::styled("p", Style::default().fg(Color::Yellow)),
        Span::raw(" submit"),
    ]));

    let popup = Paragraph::new(Text::from(lines))
        .block(Block::default().borders(Borders::ALL).title("Console Logger"))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, area);
    frame.render_widget(popup, area);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_help(frame: &mut Frame) {
    let area = centered_rect(70, 60, frame.area());
    frame.render_widget(Clear, area);
    let text = [
        "nips help",
        "",
        "e  edit selected file in $EDITOR",
        "p  stage + commit + publish nip34 proposal",
        "P  open live nip34 browser",
        "g  open asyncgit's full git TUI",
        "n  create and checkout a new branch",
        "c  checkout an existing branch",
        "r  refresh the upstream checkout",
        "\\  toggle this help",
        ".  toggle the tool bar",
        "q  quit",
    ]
    .join("\n");

    let help = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Help"))
        .wrap(Wrap { trim: false });
    frame.render_widget(help, area);
}

fn render_nip34_browser(frame: &mut Frame, browser: &nip34_browser::Nip34Browser) {
    let area = centered_rect(86, 78, frame.area());
    browser.render(frame, area);
}

fn bootstrap_local_relay() -> io::Result<()> {
    LOCAL_RELAY_BOOTSTRAPPED.get_or_init(|| {
        thread::spawn(|| {
            let runtime = match tokio::runtime::Runtime::new() {
                Ok(runtime) => runtime,
                Err(err) => {
                    eprintln!("relay runtime error: {err}");
                    return;
                }
            };

            runtime.block_on(async move {
                match RelayApp::create(Some("config/gnostr.toml"), true, Some("NOSTR".to_owned()), None) {
                    Ok(mut app_data) => {
                        app_data.setting.write().add_nip(34);
                        if let Err(err) = gnostr_relay::run_app_with_endpoint(app_data).await {
                            eprintln!("relay startup error: {err}");
                        }
                    }
                    Err(err) => {
                        eprintln!("relay config error: {err}");
                    }
                }
            });
        });
    });

    wait_for_local_relay(Duration::from_secs(10))
}

fn wait_for_local_relay(timeout: Duration) -> io::Result<()> {
    let started = std::time::Instant::now();
    loop {
        if !gnostr_asyncgit::types::local_relay_urls().is_empty() {
            return Ok(());
        }
        if started.elapsed() >= timeout {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "local relay did not publish an endpoint",
            ));
        }
        thread::sleep(Duration::from_millis(100));
    }
}

fn ui(frame: &mut Frame, app: &mut App) {
    let mut constraints = vec![Constraint::Length(5), Constraint::Min(0)];
    if app.proposal_task.is_some() {
        constraints.push(Constraint::Length(7));
    }
    if app.show_toolbar {
        constraints.push(Constraint::Length(3));
    }

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints(constraints)
        .split(frame.area());

    render_header(frame, chunks[0]);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(if app.show_tree {
            vec![Constraint::Percentage(35), Constraint::Percentage(65)]
        } else {
            vec![Constraint::Percentage(100)]
        })
        .split(chunks[1]);

    if app.show_tree {
        render_tree(frame, app, body[0]);
    }

    let (content_title, content_text) = content_for_selection(app);
    let content = Paragraph::new(content_text)
        .block(Block::default().borders(Borders::ALL).title(content_title))
        .wrap(Wrap { trim: false })
        .scroll((app.content_scroll, 0));

    frame.render_widget(content, body[if app.show_tree { 1 } else { 0 }]);

    if app.show_help {
        render_help(frame);
    }

    if let Some(browser) = app.nip34_browser.as_ref() {
        render_nip34_browser(frame, browser);
    }

    if app.proposal_task.is_some() {
        render_proposal_popup(frame, app, chunks[2]);
    }

    if app.show_toolbar {
        let toolbar_index = if app.proposal_task.is_some() { 3 } else { 2 };
        render_toolbar(frame, app, chunks[toolbar_index]);
    }
}

fn editor_target(app: &App) -> Option<PathBuf> {
    app.selected_file_path()
}

fn bottom_rect(percent_x: u16, height: u16, area: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(0), Constraint::Length(height)])
        .split(area);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn prompt_input(prompt: &str) -> io::Result<Option<String>> {
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;

    let result = (|| -> io::Result<Option<String>> {
        print!("{prompt}: ");
        stdout().flush()?;
        let mut input = String::new();
        let read = io::stdin().read_line(&mut input)?;
        if read == 0 {
            return Ok(None);
        }

        let trimmed = input.trim().to_string();
        if trimmed.is_empty() {
            Ok(None)
        } else {
            Ok(Some(trimmed))
        }
    })();

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;
    result
}

fn start_proposal_task(app: &mut App) {
    if app.proposal_task.as_ref().is_some_and(ProposalTask::is_active) {
        app.status_line = String::from("proposal already running");
        return;
    }

    if let Some(file_path) = editor_target(app) {
        let (tx, rx) = mpsc::channel();
        let checkout_dir = app.checkout_dir.clone();
        app.proposal_task = Some(ProposalTask::new(rx));
        app.status_line = String::from("pushing proposal...");
        thread::spawn(move || {
            let result = workflow::submit_proposal_with_log(&checkout_dir, &file_path, |line| {
                let _ = tx.send(ProposalUpdate::Log(line));
            })
            .map_err(|err| err.to_string());

            let _ = tx.send(ProposalUpdate::Done(result));
        });
    } else {
        app.status_line = String::from("select a file to propose");
    }
}

pub fn run_default() -> Result<(), Box<dyn Error>> {
    bootstrap_local_relay()?;
    let checkout_dir = upstream::ensure_checkout()?;
    let mut app = App::new(checkout_dir.clone(), collect_checkout_paths(&checkout_dir)?)?;
    if upstream::worktree_dirty(&checkout_dir)? {
        app.status_line = String::from("dirty checkout: fetch only, local edits preserved");
    }

    stdout().execute(EnterAlternateScreen)?;
    enable_raw_mode()?;

    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let result = (|| -> Result<(), Box<dyn Error>> {
        loop {
            if let Some(browser) = app.nip34_browser.as_mut() {
                browser.drain();
            }

            if let Some(task) = app.proposal_task.as_mut() {
                if let Some(result) = task.drain() {
                    app.status_line = match result {
                        Ok(hash) => format!("submitted proposal {hash}"),
                        Err(err) => format!("proposal failed: {err}"),
                    };
                    app.force_full_repaint = true;
                }
            }

            if app.force_full_repaint {
                terminal.clear()?;
                app.force_full_repaint = false;
            }
            terminal.draw(|f| ui(f, &mut app))?;

            if !event::poll(Duration::from_millis(100))? {
                continue;
            }

            if let Event::Key(key) = event::read()? {
                if key.kind != KeyEventKind::Press {
                    continue;
                }

                if let Some(browser) = app.nip34_browser.as_mut() {
                    if browser.handle_key(key) {
                        app.nip34_browser = None;
                        app.force_full_repaint = true;
                    }
                    continue;
                }

                let shift_p = matches!(key.code, KeyCode::Char('P'))
                    || (matches!(key.code, KeyCode::Char('p'))
                        && key.modifiers.contains(KeyModifiers::SHIFT));
                if shift_p {
                    app.nip34_browser = Some(nip34_browser::Nip34Browser::spawn());
                    app.force_full_repaint = true;
                    continue;
                }

                if matches!(key.code, KeyCode::Esc)
                    && app
                        .proposal_task
                        .as_ref()
                        .is_some_and(|task| task.result.is_some())
                {
                    app.proposal_task = None;
                    app.force_full_repaint = true;
                    continue;
                }

                match key.code {
                    KeyCode::Char('q') => break,
                    KeyCode::Char('\\') => app.show_help = !app.show_help,
                    KeyCode::Char('.') => app.show_toolbar = !app.show_toolbar,
                    KeyCode::Esc | KeyCode::Enter => app.show_tree = !app.show_tree,
                    KeyCode::Left => {
                        app.move_selection(MoveSelection::Left);
                        app.show_tree = false;
                    }
                    KeyCode::Right => {
                        app.move_selection(MoveSelection::Right);
                        app.show_tree = false;
                    }
                    KeyCode::Down | KeyCode::Char('j') => app.move_selection(MoveSelection::Down),
                    KeyCode::Up | KeyCode::Char('k') => app.move_selection(MoveSelection::Up),
                    KeyCode::PageDown => app.page_move(true),
                    KeyCode::PageUp => app.page_move(false),
                    KeyCode::Char('r') => match upstream::ensure_checkout() {
                        Ok(_) => match app.reload() {
                            Ok(_) => app.status_line = String::from("refreshed upstream checkout"),
                            Err(err) => app.status_line = format!("refresh failed: {err}"),
                        },
                        Err(err) => app.status_line = format!("refresh failed: {err}"),
                    },
                    KeyCode::Char('e') => {
                        if let Some(file_path) = editor_target(&app) {
                            disable_raw_mode()?;
                            stdout().execute(LeaveAlternateScreen)?;
                            let editor_result = workflow::launch_editor(&file_path);
                            stdout().execute(EnterAlternateScreen)?;
                            enable_raw_mode()?;
                            app.force_full_repaint = true;
                            app.status_line = match editor_result {
                                Ok(_) => match app.reload() {
                                    Ok(_) => format!("edited {}", file_path.display()),
                                    Err(err) => format!("edited {}; reload failed: {err}", file_path.display()),
                                },
                                Err(err) => format!("editor error: {err}"),
                            };
                        } else {
                            app.status_line = String::from("select a file to edit");
                        }
                    }
                    KeyCode::Char('n') => {
                        if let Some(branch) = prompt_input("Create and checkout branch")? {
                            disable_raw_mode()?;
                            stdout().execute(LeaveAlternateScreen)?;
                            let branch_result = workflow::create_branch(&app.checkout_dir, &branch);
                            stdout().execute(EnterAlternateScreen)?;
                            enable_raw_mode()?;
                            app.force_full_repaint = true;
                            app.status_line = match branch_result {
                                Ok(_) => {
                                    let _ = app.reload();
                                    format!("created branch {branch}")
                                }
                                Err(err) => format!("create branch failed: {err}"),
                            };
                        }
                    }
                    KeyCode::Char('c') => {
                        if let Some(branch) = prompt_input("Checkout branch")? {
                            disable_raw_mode()?;
                            stdout().execute(LeaveAlternateScreen)?;
                            let branch_result = workflow::checkout_branch(&app.checkout_dir, &branch);
                            stdout().execute(EnterAlternateScreen)?;
                            enable_raw_mode()?;
                            app.force_full_repaint = true;
                            app.status_line = match branch_result {
                                Ok(_) => {
                                    let _ = app.reload();
                                    format!("checked out {branch}")
                                }
                                Err(err) => format!("checkout failed: {err}"),
                            };
                        }
                    }
                    KeyCode::Char('g') => {
                        disable_raw_mode()?;
                        stdout().execute(LeaveAlternateScreen)?;
                        let git_result = workflow::launch_git_tui(&app.checkout_dir);
                        stdout().execute(EnterAlternateScreen)?;
                        enable_raw_mode()?;
                        app.force_full_repaint = true;
                        app.status_line = match git_result {
                            Ok(_) => String::from("returned from asyncgit TUI"),
                            Err(err) => format!("git ui error: {err}"),
                        };
                        let _ = app.reload();
                    }
                    KeyCode::Char('p') => {
                        start_proposal_task(&mut app);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    })();

    stdout().execute(LeaveAlternateScreen)?;
    disable_raw_mode()?;
    terminal.show_cursor()?;

    result
}
