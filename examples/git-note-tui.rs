use std::{
    path::{Path, PathBuf},
    process::Command,
};

use anyhow::{Context, Result};
use clap::{Parser, ValueEnum};
use git2::{Oid, Repository, Signature};

#[derive(Parser, Debug)]
struct Args {
    /// Path to the git repository
    #[arg(long, value_name = "PATH")]
    repo: PathBuf,

    /// Object id to attach the note to
    #[arg(long, value_name = "OID")]
    object: String,

    /// Note body
    #[arg(long, default_value = "testing git notes")]
    message: String,

    /// Notes ref to use
    #[arg(long, value_name = "REF")]
    notes_ref: Option<String>,

    /// Which demo to run
    #[arg(long, value_enum, default_value_t = Demo::Both)]
    demo: Demo,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, ValueEnum)]
enum Demo {
    System,
    Git2,
    Both,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let repo = Repository::open(&args.repo)?;
    let oid = Oid::from_str(&args.object).context("invalid object id")?;
    let notes_ref = args
        .notes_ref
        .clone()
        .unwrap_or_else(|| repo.note_default_ref().unwrap_or_else(|_| "refs/notes/commits".into()));

    match args.demo {
        Demo::System => system_git_notes(&repo, oid, &notes_ref, &args.message)?,
        Demo::Git2 => git2_git_notes(&repo, oid, &notes_ref, &args.message)?,
        Demo::Both => {
            system_git_notes(&repo, oid, &notes_ref, &args.message)?;
            git2_git_notes(&repo, oid, &notes_ref, &args.message)?;
        }
    }

    Ok(())
}

fn repo_workdir(repo: &Repository) -> Result<&Path> {
    repo.workdir()
        .or_else(|| repo.path().parent())
        .context("repository has no workdir")
}

fn system_git_notes(repo: &Repository, oid: Oid, notes_ref: &str, message: &str) -> Result<()> {
    let workdir = repo_workdir(repo)?;

    println!("== system git notes ==");
    run_git(
        workdir,
        &["notes", "--ref", notes_ref, "add", "-m", message, &oid.to_string()],
    )?;
    run_git(
        workdir,
        &["notes", "--ref", notes_ref, "show", &oid.to_string()],
    )?;
    run_git(workdir, &["notes", "--ref", notes_ref, "list"])?;
    run_git(
        workdir,
        &["notes", "--ref", notes_ref, "remove", &oid.to_string()],
    )?;

    Ok(())
}

fn git2_git_notes(repo: &Repository, oid: Oid, notes_ref: &str, message: &str) -> Result<()> {
    println!("== git2 git notes ==");

    let sig = signature(repo)?;
    repo.note(&sig, &sig, Some(notes_ref), oid, message, true)?;

    if let Some(note) = repo.find_note(Some(notes_ref), oid).ok() {
        println!("show: {}", note.message().unwrap_or_default());
    }

    for entry in repo.notes(Some(notes_ref))? {
        let (note_id, annotated_id) = entry?;
        println!("list: note@{} -> {}", note_id, annotated_id);
    }

    repo.note_delete(oid, Some(notes_ref), &sig, &sig)?;
    Ok(())
}

fn signature(repo: &Repository) -> Result<Signature<'_>> {
    repo.signature().context("missing git signature")
}

fn run_git(workdir: &Path, args: &[&str]) -> Result<()> {
    let output = Command::new("git").current_dir(workdir).args(args).output()?;
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.trim().is_empty() {
        print!("{stdout}");
    }
    if !stderr.trim().is_empty() {
        eprint!("{stderr}");
    }

    if output.status.success() {
        Ok(())
    } else {
        anyhow::bail!("git {:?} failed", args)
    }
}
