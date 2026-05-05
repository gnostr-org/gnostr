use std::{
    env,
    error::Error,
    ffi::OsStr,
    io::{self, Read, Write},
    path::Path,
    process::Command,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use gnostr_asyncgit::{
    filehash::{get_relay_urls, publish_patch_event},
    sync::{commit::commit, stage_add_file, RepoPath},
    types::Keys,
};
use portable_pty::{native_pty_system, CommandBuilder, PtySize};

fn repo_path(repo_dir: &Path) -> Result<RepoPath, Box<dyn Error>> {
    let repo_dir = repo_dir
        .to_str()
        .ok_or_else(|| std::io::Error::other("invalid repository path"))?;
    Ok(repo_dir.into())
}

pub fn launch_editor(file_path: &Path) -> Result<(), Box<dyn Error>> {
    let editor = env::var("EDITOR").unwrap_or_else(|_| "vi".to_string());
    let mut parts = editor.split_whitespace();
    let program = parts.next().unwrap_or("vi");
    let args: Vec<&str> = parts.collect();

    let status = Command::new(program)
        .args(args)
        .arg(file_path)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!(
            "editor exited with status {status}"
        ))
        .into())
    }
}

pub fn current_branch(repo_dir: &Path) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::other("failed to read current branch").into());
    }

    let branch = String::from_utf8(output.stdout)?;
    Ok(branch.trim().to_string())
}

pub fn checkout_branch(repo_dir: &Path, branch: &str) -> Result<(), Box<dyn Error>> {
    let status = Command::new("git")
        .args(["checkout", branch])
        .current_dir(repo_dir)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("git checkout {branch} failed")).into())
    }
}

pub fn create_branch(repo_dir: &Path, branch: &str) -> Result<(), Box<dyn Error>> {
    let status = Command::new("git")
        .args(["checkout", "-b", branch])
        .current_dir(repo_dir)
        .status()?;

    if status.success() {
        Ok(())
    } else {
        Err(std::io::Error::other(format!("git checkout -b {branch} failed")).into())
    }
}

pub fn launch_git_tui(repo_dir: &Path) -> Result<(), Box<dyn Error>> {
    let workspace_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or_else(|| io::Error::other("failed to resolve workspace root"))?;
    let pty_system = native_pty_system();
    let (cols, rows) = crossterm::terminal::size().unwrap_or((120, 30));
    let pair = pty_system.openpty(PtySize {
        rows,
        cols,
        pixel_width: 0,
        pixel_height: 0,
    })?;

    let mut command = CommandBuilder::new("cargo");
    command.args([
        "run",
        "--quiet",
        "--manifest-path",
        workspace_root
            .join("Cargo.toml")
            .to_str()
            .ok_or_else(|| io::Error::other("invalid Cargo.toml path"))?,
        "-p",
        "gnostr-asyncgit",
        "--bin",
        "git-tui",
        "--features",
        "tui",
    ]);
    command.cwd(repo_dir);
    command.env("TERM", "xterm-256color");
    command.env("COLORTERM", "truecolor");

    let mut child = pair.slave.spawn_command(command)?;
    let mut reader = pair.master.try_clone_reader()?;
    let writer = pair.master.take_writer()?;
    let writer = Arc::new(std::sync::Mutex::new(writer));
    let done = Arc::new(AtomicBool::new(false));

    let output_done = Arc::clone(&done);
    let output_handle = thread::spawn(move || {
        let mut stdout = io::stdout();
        let mut buf = [0u8; 8192];
        while !output_done.load(Ordering::SeqCst) {
            match reader.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => {
                    if stdout.write_all(&buf[..n]).is_err() {
                        break;
                    }
                    if stdout.flush().is_err() {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    });

    let mut result: Result<(), Box<dyn Error>> = Ok(());
    loop {
        match crossterm::event::poll(Duration::from_millis(50)) {
            Ok(true) => match crossterm::event::read() {
                Ok(crossterm::event::Event::Key(key)) => {
                    let bytes = match key.code {
                        crossterm::event::KeyCode::Char(c) => {
                            let mut out = Vec::new();
                            if key.modifiers.contains(crossterm::event::KeyModifiers::CONTROL) {
                                if let Some(ctrl) = c.to_ascii_lowercase().to_digit(36) {
                                    out.push((ctrl as u8).saturating_sub(9));
                                }
                            } else {
                                out.extend_from_slice(c.encode_utf8(&mut [0; 4]).as_bytes());
                            }
                            out
                        }
                        crossterm::event::KeyCode::Enter => vec![b'\r'],
                        crossterm::event::KeyCode::Tab => vec![b'\t'],
                        crossterm::event::KeyCode::Backspace => vec![0x7f],
                        crossterm::event::KeyCode::Esc => vec![0x1b],
                        crossterm::event::KeyCode::Left => b"\x1b[D".to_vec(),
                        crossterm::event::KeyCode::Right => b"\x1b[C".to_vec(),
                        crossterm::event::KeyCode::Up => b"\x1b[A".to_vec(),
                        crossterm::event::KeyCode::Down => b"\x1b[B".to_vec(),
                        crossterm::event::KeyCode::Home => b"\x1b[H".to_vec(),
                        crossterm::event::KeyCode::End => b"\x1b[F".to_vec(),
                        crossterm::event::KeyCode::PageUp => b"\x1b[5~".to_vec(),
                        crossterm::event::KeyCode::PageDown => b"\x1b[6~".to_vec(),
                        crossterm::event::KeyCode::Delete => b"\x1b[3~".to_vec(),
                        _ => Vec::new(),
                    };

                    if !bytes.is_empty() {
                        let mut guard = writer.lock().map_err(|_| io::Error::other("pty writer lock poisoned"))?;
                        guard.write_all(&bytes)?;
                        guard.flush()?;
                    }
                }
                Ok(crossterm::event::Event::Resize(cols, rows)) => {
                    pair.master.resize(PtySize {
                        rows,
                        cols,
                        pixel_width: 0,
                        pixel_height: 0,
                    })?;
                }
                Ok(_) => {}
                Err(err) => {
                    result = Err(Box::new(err));
                    break;
                }
            },
            Ok(false) => {}
            Err(err) => {
                result = Err(Box::new(err));
                break;
            }
        }

        if let Some(status) = child.try_wait()? {
            if !status.success() {
                result = Err(io::Error::other(format!("git-tui exited with {status}")).into());
            }
            break;
        }
    }

    done.store(true, Ordering::SeqCst);
    let _ = output_handle.join();
    let _ = child.wait();
    result
}

fn patch_content(repo_dir: &Path, commit_id: &str) -> Result<String, Box<dyn Error>> {
    let output = Command::new("git")
        .args([
            "show",
            "--format=medium",
            "--patch",
            "--no-ext-diff",
            "--no-color",
            commit_id,
        ])
        .current_dir(repo_dir)
        .output()?;

    if !output.status.success() {
        return Err(std::io::Error::other("failed to generate patch content").into());
    }

    Ok(String::from_utf8(output.stdout)?)
}

pub fn submit_proposal(repo_dir: &Path, file_path: &Path) -> Result<String, Box<dyn Error>> {
    let repo = repo_path(repo_dir)?;
    let rel_path = file_path.strip_prefix(repo_dir)?;
    stage_add_file(&repo, rel_path)?;

    let subject = rel_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("nips update");
    let commit_id = commit(&repo, &format!("update {subject}"))?;
    let commit_hash = commit_id.to_string();
    let keys = Keys::parse(format!("{:0>64}", commit_hash))
        .ok_or_else(|| std::io::Error::other("failed to derive keys from commit hash"))?;
    let relay_urls = get_relay_urls();
    let patch = patch_content(repo_dir, &commit_hash)?;
    let repo_name = repo_dir
        .file_name()
        .and_then(OsStr::to_str)
        .unwrap_or("nips")
        .to_string();

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        publish_patch_event(&keys, &relay_urls, &repo_name, &commit_hash, &patch, None).await
    })?;

    Ok(commit_hash)
}
