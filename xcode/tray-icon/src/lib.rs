use std::ffi::OsStr;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use portable_pty::{native_pty_system, CommandBuilder, PtySize};
pub use tao::event_loop::{ControlFlow, EventLoop};
pub use tray_icon::{
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    TrayIconBuilder,
};

pub const DEFAULT_TOOLTIP: &str = "Sovereign Menu Bar Utility";

pub struct TrayContext {
    pub menu: Menu,
    pub gnostr_item: MenuItem,
    pub quit_item: MenuItem,
}

pub fn load_icon_from_svg(svg: &[u8]) -> tray_icon::Icon {
    load_icon_from_svg_tinted(svg, [255, 0, 255, 255])
}

pub fn load_icon_from_svg_tinted(svg: &[u8], tint: [u8; 4]) -> tray_icon::Icon {
    let mut options = usvg::Options::default();
    options.fontdb_mut().load_system_fonts();

    let tree = usvg::Tree::from_data(svg, &options).expect("load tray icon svg");
    let size = tree.size().to_int_size();
    let mut pixmap = tiny_skia::Pixmap::new(size.width(), size.height()).expect("create pixmap");
    resvg::render(
        &tree,
        tiny_skia::Transform::default(),
        &mut pixmap.as_mut(),
    );

    for pixel in pixmap.data_mut().chunks_mut(4) {
        if pixel[3] != 0 {
            pixel[0] = tint[0];
            pixel[1] = tint[1];
            pixel[2] = tint[2];
        }
    }

    tray_icon::Icon::from_rgba(pixmap.take(), size.width(), size.height()).expect("build tray icon")
}

pub fn load_gnostr_icon() -> tray_icon::Icon {
    load_icon_from_svg(include_bytes!("../../icons/gnostr.svg"))
}

pub fn load_gnostr_icon_tinted(tint: [u8; 4]) -> tray_icon::Icon {
    load_icon_from_svg_tinted(include_bytes!("../../icons/gnostr.svg"), tint)
}

pub fn system_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    command.stdin(Stdio::null());
    command.stdout(Stdio::null());
    command.stderr(Stdio::null());
    command
}

pub fn pty_command(program: impl AsRef<OsStr>) -> CommandBuilder {
    CommandBuilder::new(program)
}

pub fn run_command_in_pty(command: CommandBuilder) -> io::Result<String> {
    let pty_system = native_pty_system();
    let portable_pty::PtyPair { master, slave } = pty_system
        .openpty(PtySize {
        rows: 24,
        cols: 80,
        pixel_width: 0,
        pixel_height: 0,
    })
    .map_err(|error| io::Error::other(error.to_string()))?;

    let mut child = slave
        .spawn_command(command)
        .map_err(|error| io::Error::other(error.to_string()))?;
    drop(slave);

    let mut reader = master
        .try_clone_reader()
        .map_err(|error| io::Error::other(error.to_string()))?;
    let mut output = String::new();
    reader.read_to_string(&mut output)?;
    let _ = child.wait();
    Ok(output)
}

pub fn gnostr_command_line() -> String {
    let gnostr = resolve_command_path("gnostr")
        .map(|path| path.to_string_lossy().to_string())
        .unwrap_or_else(|| "gnostr".to_string());
    let gitdir = resolve_gnostr_gitdir().unwrap_or_else(|| ".".to_string());
    format!("{gnostr} --gitdir {} tui", shell_string(&gitdir))
}

pub fn launch_gnostr_in_terminal() -> io::Result<std::process::Child> {
    spawn_terminal_command(gnostr_command_line())
}

pub fn resolve_gnostr_gitdir() -> Option<String> {
    if let Ok(value) = std::env::var("GNOSTR_GITDIR") {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return Some(trimmed.to_string());
        }
    }

    let cwd = std::env::current_dir().ok()?;
    for dir in cwd.as_path().ancestors() {
        if dir.join(".git").exists() {
            return Some(dir.to_string_lossy().to_string());
        }
    }

    None
}

pub fn command_exists(program: impl AsRef<OsStr>) -> bool {
    resolve_command_path(program).is_some()
}

pub fn resolve_command_path(program: impl AsRef<OsStr>) -> Option<PathBuf> {
    let program = Path::new(program.as_ref());

    if program.components().count() > 1 {
        return is_executable_path(program).then(|| program.to_path_buf());
    }

    let Some(paths) = std::env::var_os("PATH") else {
        return None;
    };

    for dir in std::env::split_paths(&paths) {
        if has_command_in_dir(&dir, program) {
            return Some(dir.join(program));
        }
    }

    None
}

pub fn tray_icon_command(program: impl AsRef<OsStr>, tint: [u8; 4]) -> Command {
    let mut command = system_command(program);
    command.env("TRAY_ICON_TINT", tint_hex(tint));
    command
}

pub fn terminal_command(command_line: impl AsRef<str>) -> Command {
    let command_line = command_line.as_ref().trim().to_string();

    #[cfg(target_os = "macos")]
    {
        let mut command = system_command("osascript");
        let script = format!(
            "tell application \"Terminal\"\n  activate\n  do script {}\nend tell",
            applescript_string(&command_line)
        );
        command.arg("-e").arg(script);
        return command;
    }

    #[cfg(target_os = "windows")]
    {
        let mut command = system_command("cmd");
        command.args([
            "/C",
            "start",
            "",
            "powershell",
            "-NoExit",
            "-NoProfile",
            "-Command",
            &command_line,
        ]);
        return command;
    }

    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let script = format!(
            "if command -v x-terminal-emulator >/dev/null 2>&1; then\n  exec x-terminal-emulator -e sh -lc {}\nelif command -v gnome-terminal >/dev/null 2>&1; then\n  exec gnome-terminal -- sh -lc {}\nelif command -v konsole >/dev/null 2>&1; then\n  exec konsole -e sh -lc {}\nelif command -v kitty >/dev/null 2>&1; then\n  exec kitty sh -lc {}\nelif command -v xterm >/dev/null 2>&1; then\n  exec xterm -e sh -lc {}\nfi\nexec sh -lc {}",
            shell_string(&command_line),
            shell_string(&command_line),
            shell_string(&command_line),
            shell_string(&command_line),
            shell_string(&command_line),
            shell_string(&command_line),
        );
        let mut command = system_command("sh");
        command.args(["-lc", &script]);
        return command;
    }

    #[allow(unreachable_code)]
    {
        system_command("sh")
    }
}

pub fn spawn_terminal_command(command_line: impl AsRef<str>) -> std::io::Result<std::process::Child> {
    terminal_command(command_line).spawn()
}

pub fn tray_icon_tint_from_env() -> [u8; 4] {
    std::env::var("TRAY_ICON_TINT")
        .ok()
        .and_then(|value| parse_tint(&value))
        .unwrap_or([255, 0, 255, 255])
}

pub fn parse_tint(input: &str) -> Option<[u8; 4]> {
    let value = input.trim();
    if value.is_empty() {
        return None;
    }

    let hex = value.strip_prefix('#').unwrap_or(value);
    match hex.len() {
        6 => {
            let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
            Some([red, green, blue, 255])
        }
        8 => {
            let red = u8::from_str_radix(&hex[0..2], 16).ok()?;
            let green = u8::from_str_radix(&hex[2..4], 16).ok()?;
            let blue = u8::from_str_radix(&hex[4..6], 16).ok()?;
            let alpha = u8::from_str_radix(&hex[6..8], 16).ok()?;
            Some([red, green, blue, alpha])
        }
        _ => None,
    }
}

pub fn tint_hex(tint: [u8; 4]) -> String {
    format!(
        "#{:02x}{:02x}{:02x}{:02x}",
        tint[0], tint[1], tint[2], tint[3]
    )
}

fn shell_string(input: &str) -> String {
    format!("'{}'", input.replace('\'', r"'\''"))
}

#[cfg(target_os = "macos")]
fn applescript_string(input: &str) -> String {
    let escaped = input
        .replace('\\', r"\\")
        .replace('"', r#"\""#)
        .replace('\n', r"\n");
    format!("\"{}\"", escaped)
}

fn has_command_in_dir(dir: &Path, program: &Path) -> bool {
    #[cfg(target_os = "windows")]
    {
        let pathext = std::env::var_os("PATHEXT")
            .unwrap_or_else(|| ".EXE;.CMD;.BAT;.COM".into())
            .to_string_lossy()
            .into_owned();

        let extensions = pathext
            .split(';')
            .map(str::trim)
            .filter(|ext| !ext.is_empty())
            .collect::<Vec<_>>();

        if program.extension().is_some() {
            return is_executable_path(&dir.join(program));
        }

        for ext in extensions {
            let candidate = dir.join(format!("{}{}", program.display(), ext));
            if is_executable_path(&candidate) {
                return true;
            }
        }

        false
    }

    #[cfg(not(target_os = "windows"))]
    {
        is_executable_path(&dir.join(program))
    }
}

fn is_executable_path(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };

    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }

    #[cfg(not(unix))]
    {
        true
    }
}

pub fn default_tray_context() -> TrayContext {
    let menu = Menu::new();
    let gnostr_item = MenuItem::new("gnostr", true, None);
    let separator = PredefinedMenuItem::separator();
    let quit_item = MenuItem::new("Quit App", true, None);

    menu.append(&gnostr_item).unwrap();
    menu.append(&separator).unwrap();
    menu.append(&quit_item).unwrap();

    TrayContext {
        menu,
        gnostr_item,
        quit_item,
    }
}

pub fn run_default_tray_app() -> ! {
    run_tray_app(
        DEFAULT_TOOLTIP,
        load_gnostr_icon_tinted(tray_icon_tint_from_env()),
        default_tray_context(),
    )
}

pub fn run_tray_app(tooltip: &str, icon: tray_icon::Icon, context: TrayContext) -> ! {
    let event_loop = EventLoop::new();
    let tray_menu = context.menu;
    let gnostr_item = context.gnostr_item;
    let quit_item = context.quit_item;

    let _tray_icon = TrayIconBuilder::new()
        .with_menu(Box::new(tray_menu))
        .with_tooltip(tooltip)
        .with_icon(icon)
        .build()
        .unwrap();

    let menu_channel = MenuEvent::receiver();

    event_loop.run(move |_event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        if let Ok(event) = menu_channel.try_recv() {
            if event.id == quit_item.id() {
                println!("Exiting gracefully.");
                *control_flow = ControlFlow::Exit;
            } else if event.id == gnostr_item.id() {
                if let Err(error) = launch_gnostr_in_terminal() {
                    eprintln!("failed to launch gnostr: {error}");
                }
            }
        }
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_rgb_hex() {
        assert_eq!(parse_tint("#ff00ff"), Some([255, 0, 255, 255]));
    }

    #[test]
    fn parses_rgba_hex() {
        assert_eq!(parse_tint("ff00ff80"), Some([255, 0, 255, 128]));
    }

    #[test]
    fn rejects_invalid_values() {
        assert_eq!(parse_tint("not-a-color"), None);
    }

    #[test]
    fn formats_tint_as_hex() {
        assert_eq!(tint_hex([255, 0, 255, 255]), "#ff00ffff");
    }

    #[test]
    fn builds_tray_command() {
        let command = tray_icon_command("tray-icon", [255, 0, 255, 255]);
        assert_eq!(command.get_program(), std::ffi::OsStr::new("tray-icon"));
    }

    #[test]
    fn builds_terminal_command() {
        let command = terminal_command("echo hi");
        #[cfg(target_os = "macos")]
        assert_eq!(command.get_program(), std::ffi::OsStr::new("osascript"));
        #[cfg(target_os = "windows")]
        assert_eq!(command.get_program(), std::ffi::OsStr::new("cmd"));
        #[cfg(all(unix, not(target_os = "macos")))]
        assert_eq!(command.get_program(), std::ffi::OsStr::new("sh"));
    }

    #[test]
    fn detects_program_presence_for_current_shell() {
        assert!(command_exists("sh") || command_exists("cmd"));
    }
}
