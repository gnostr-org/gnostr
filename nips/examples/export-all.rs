use clap::Parser;
use rust_embed::Embed;
use std::env;
use std::fs;
use std::fs::File;
use std::io;
use std::io::{stdout, Write};
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::process::Stdio;
use termimad::crossterm::{
    cursor::{Hide, Show},
    event::{self, Event, KeyCode::*, KeyEvent},
    queue,
    style::Color::*,
    terminal::{self, Clear, ClearType, EnterAlternateScreen, LeaveAlternateScreen},
};
use termimad::*;
use tracing_subscriber::{fmt, layer::SubscriberExt, EnvFilter, Registry};

#[derive(Embed)]
#[folder = "."]
#[include = "*.md"]
#[exclude = "*.DS_Store"]
#[exclude = "target/*"]
#[exclude = "src"]
#[exclude = "src/*"]
#[exclude = ".git"]
#[exclude = ".git/*"]
#[exclude = ".github/*"]
#[exclude = ".gitignore"]
#[exclude = ".justfile"]
#[exclude = ".nojekyll"]
#[exclude = "build.rs"]
#[exclude = "dist-workspace.toml"]
#[exclude = "error.log"]
#[exclude = "output.log"]
#[exclude = "post-commit-history"]
#[exclude = "script.sh"]
#[exclude = "template/Makefile"]
#[exclude = "template/default_config.conf"]
#[exclude = "template/install_script.sh"]
#[exclude = "test_files/tabbed.txtbuild.rs"]
#[exclude = "Cargo.lock"]
#[exclude = "Cargo.toml"]
#[exclude = "LICENSE"]
#[exclude = "Makefile"]
struct Template;

/// A simple tool to view embedded Markdown files.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Enable debug logging.
    #[clap(short, long)]
    debug: bool,

    /// List all embedded files.
    #[clap(short, long)]
    list_embedded: bool,

    /// Show the contents of an embedded file using a pager.
    #[clap(short, long, value_name = "NIP")]
    show: Option<String>,

    /// Export all embedded files to the current directory.
    #[clap(long)]
    export: bool,
}

fn make_executable(script_path: &Path) -> io::Result<()> {
    let mut permissions = fs::metadata(script_path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o111);
    fs::set_permissions(script_path, permissions)?;
    tracing::debug!("Made '{}' executable.", script_path.display());
    Ok(())
}

fn execute_script(script_path: &Path) -> io::Result<()> {
    tracing::debug!("Executing script: {}", script_path.display());
    let log_file = File::create("output.log")?;
    let error_file = File::create("error.log")?;
    let mut command = Command::new(script_path);
    command
        .stdout(Stdio::from(log_file))
        .stderr(Stdio::from(error_file));
    let status = command.spawn()?.wait()?;
    if status.success() {
        tracing::debug!("Script '{}' executed successfully.", script_path.display());
        Ok(())
    } else {
        eprintln!(
            "Script '{}' failed with exit code: {:?}",
            script_path.display(),
            status.code()
        );
        Err(io::Error::new(
            io::ErrorKind::Other,
            format!(
                "Script execution failed with exit code: {:?}",
                status.code()
            ),
        ))
    }
}

fn canonicalize_path(path: &Path) -> io::Result<PathBuf> {
    let absolute_path = if path.is_relative() {
        let current_dir = env::current_dir()?;
        current_dir.join(path)
    } else {
        path.to_path_buf()
    };
    fs::canonicalize(absolute_path)
}

fn extract(filename: &str, output_dir: &Path) -> io::Result<()> {
    match Template::get(filename) {
        Some(embedded_file) => {
            let output_path = output_dir.join(filename);
            if let Some(parent) = output_path.parent() {
                fs::create_dir_all(parent)?;
            }
            let mut outfile = File::create(&output_path)?;
            outfile.write_all(embedded_file.data.as_ref())?;
            tracing::debug!(
                "Successfully exported '{}' to '{}'",
                filename,
                output_path.display()
            );
            Ok(())
        }
        None => Err(io::Error::new(
            io::ErrorKind::NotFound,
            format!("Embedded file '{}' not found!", filename),
        )),
    }
}

fn view_area() -> Area {
    let mut area = Area::full_screen();
    area.pad_for_max_width(120);
    area
}

fn run_app(skin: MadSkin, nip: String) -> Result<(), Error> {
    let mut w = stdout();
    queue!(w, EnterAlternateScreen)?;
    terminal::enable_raw_mode()?;
    queue!(w, Hide)?;
    let mut view = MadView::from(nip.to_owned(), view_area(), skin);
    loop {
        view.write_on(&mut w)?;
        w.flush()?;
        match event::read() {
            Ok(Event::Key(KeyEvent { code, .. })) => match code {
                Up => view.try_scroll_lines(-1),
                Down => view.try_scroll_lines(1),
                PageUp => view.try_scroll_pages(-1),
                PageDown => view.try_scroll_pages(1),
                Char('q') | Esc => break,
                _ => {}
            },
            Ok(Event::Resize(..)) => {
                queue!(w, Clear(ClearType::All))?;
                view.resize(&view_area());
            }
            _ => {}
        }
    }
    terminal::disable_raw_mode()?;
    queue!(w, Show)?;
    queue!(w, LeaveAlternateScreen)?;
    w.flush()?;
    Ok(())
}

fn make_skin() -> MadSkin {
    let mut skin = MadSkin::default();
    skin.table.align = Alignment::Center;
    skin.set_headers_fg(AnsiValue(178));
    skin.bold.set_fg(Yellow);
    skin.italic.set_fg(Magenta);
    skin.scrollbar.thumb.set_fg(AnsiValue(178));
    skin.code_block.align = Alignment::Center;
    skin
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let level_filter = if args.debug {
        EnvFilter::new("debug")
    } else {
        EnvFilter::from_default_env()
    };

    let subscriber = Registry::default()
        .with(fmt::layer().with_writer(std::io::stdout))
        .with(level_filter);
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");

    tracing::debug!("Parsed arguments: {:?}", args);

    if args.list_embedded {
        tracing::debug!("Embedded files:");
        for file in Template::iter() {
            println!("{}", file.as_ref());
        }
        return Ok(());
    }

    if args.export {
        tracing::info!("Exporting all embedded files to the current directory...");
        let current_dir = env::current_dir()?;
        let mut export_count = 0;
        for file in Template::iter() {
            match extract(file.as_ref(), &current_dir) {
                Ok(_) => {
                    export_count += 1;
                }
                Err(e) => {
                    eprintln!("Error exporting '{}': {}", file.as_ref(), e);
                }
            }
        }
        tracing::info!("Successfully exported {} embedded files.", export_count);
        return Ok(());
    }

    if let Some(nip_arg) = &args.show {
        let filename = if nip_arg.ends_with(".md") {
            nip_arg.clone()
        } else {
            format!("{:02}.md", nip_arg)
        };

        match Template::get(&filename) {
            Some(embedded_file) => {
                let content = String::from_utf8_lossy(embedded_file.data.as_ref());
                let skin = make_skin();
                let _res = run_app(skin, (&content).to_string());
                return Ok(());
            }
            None => {
                eprintln!("Error: Embedded NIP file '{}' not found!", filename);
                std::process::exit(1);
            }
        }
    }

    tracing::debug!(
        "Canonical path of '.': {}",
        canonicalize_path(Path::new("."))?.display()
    );
    tracing::debug!(
        "Canonical path of 'src': {}",
        canonicalize_path(Path::new("src"))?.display()
    );

    #[cfg(windows)]
    let absolute_path_str = "C:\\Windows\\System32\\cmd.exe";
    #[cfg(not(windows))]
    let absolute_path_str = "/bin/ls";

    tracing::debug!(
        "Canonical path of '{}': {}",
        absolute_path_str,
        canonicalize_path(Path::new(absolute_path_str))?.display()
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use rust_embed::RustEmbed;
    use std::fs;
    use std::path::PathBuf;
    use std::thread;
    use std::time::Duration;

    fn create_test_file(path: &PathBuf, content: &str) -> io::Result<()> {
        fs::create_dir_all(path.parent().unwrap())?;
        fs::write(path, content)?;
        Ok(())
    }

    #[test]
    fn test_make_test_file() -> io::Result<()> {
        let test_file_content = "Line with\ttab.\nAnother\t\ttabbed line.\n\tLeading tab.\n\\tThis line starts with a literal backslash-t.\nThis line has a tab in the middle:	like this.";
        let script_path = Path::new("tabbed.txt");
        let test_files = Path::new("test_files");
        let full_path = test_files.join(script_path);
        tracing::debug!("{}", full_path.display());
        create_test_file(&full_path, test_file_content)?;
        thread::sleep(Duration::from_secs(1));
        Ok(())
    }

    #[test]
    fn test_preserve_tabs() -> io::Result<()> {
        test_make_test_file()?;
        let test_file_content = "Line with\ttab.\nAnother\t\ttabbed line.\n\tLeading tab.\n\\tThis line starts with a literal backslash-t.\nThis line has a tab in the middle:	like this.";

        #[derive(RustEmbed)]
        #[folder = "./test_files"]
        #[exclude = "*.DS_Store"]
        struct EmbeddedAssets;

        for file in EmbeddedAssets::iter() {
            println!("Found asset: {}", file.as_ref());

            match EmbeddedAssets::get(file.as_ref()) {
                Some(embedded_file) => {
                    let content = String::from_utf8_lossy(embedded_file.data.as_ref());
                    println!("Contents of {}:\n{}", file.as_ref(), content);
                    let install_default_conf = Command::new("echo")
                        .arg(content.as_ref())
                        .status()
                        .expect("Failed to execute install script");
                    if install_default_conf.success() {
                        println!("Installation script executed successfully.");
                    } else {
                        eprintln!("Installation script failed.");
                    }
                }
                None => {
                    eprintln!("Error: {} not found in embedded assets!", file.as_ref());
                }
            }
        }

        if let Some(file) = EmbeddedAssets::get("tabbed.txt") {
            let embedded_content = String::from_utf8_lossy(file.data.as_ref()).to_string();
            assert_eq!(embedded_content, test_file_content);
            assert!(embedded_content.contains("\t"));
            assert!(embedded_content.contains("	"));
            assert!(embedded_content.lines().nth(0).unwrap().contains('\t'));
            assert!(embedded_content.lines().nth(0).unwrap().contains("	"));
            assert!(embedded_content.lines().nth(1).unwrap().contains('\t'));
            assert!(embedded_content.lines().nth(1).unwrap().contains("	"));
            assert!(embedded_content.lines().nth(1).unwrap().contains("\t\t"));
            assert!(embedded_content.lines().nth(1).unwrap().contains("		"));
            assert!(embedded_content.lines().nth(2).unwrap().contains('\t'));
            assert!(embedded_content.lines().nth(2).unwrap().contains("	"));
            assert!(!embedded_content.lines().nth(3).unwrap().starts_with('\t'));
            assert!(!embedded_content.lines().nth(3).unwrap().starts_with("\t"));
            assert!(!embedded_content.lines().nth(3).unwrap().starts_with("	"));
            assert!(embedded_content.lines().nth(0).unwrap().contains("h\t"));
            assert!(embedded_content.lines().nth(1).unwrap().contains("r\t"));
            assert!(embedded_content.lines().nth(2).unwrap().contains("\tL"));
        } else {
            println!("Failed to embed 'tabbed.txt'");
        }
        Ok(())
    }

    #[test]
    fn remove_dir_all_custom() -> io::Result<()> {
        let path = Path::new("test_files");
        if path.is_dir() {
            fs::remove_dir_all(path)?;
            fs::create_dir_all(path)?;
        } else if path.is_file() {
            fs::remove_file(path)?;
            fs::create_dir_all(path)?;
        } else {
            fs::create_dir_all(path)?;
        }
        test_make_test_file()?;
        Ok(())
    }
}
