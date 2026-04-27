pub mod render;
pub mod scan;

use anyhow::Result;
use clap::Parser;
use std::fs;
use std::path::PathBuf;

const MAX_DEFAULT_BYTES: u64 = 51200; // 50 KiB

#[derive(Parser, Debug)]
#[command(author, version, about = "Strictly-compliant Repo Flattener")]
pub struct Args {
    pub repo_url: String,
    #[arg(short, long)]
    pub out: Option<PathBuf>,
    #[arg(long, default_value_t = MAX_DEFAULT_BYTES)]
    pub max_bytes: u64,
}

pub fn run() -> Result<()> {
    run_with_args(Args::parse())
}

pub fn build_html(repo_url: &str, max_bytes: u64) -> Result<String> {
    let tmp_dir = tempfile::Builder::new().prefix("flatten_").tempdir()?;
    let repo = scan::clone_repo(tmp_dir.path(), repo_url)?;
    let files = scan::collect_files(&repo.path, max_bytes)?;
    Ok(render::build_html(
        repo_url,
        &files,
        &default_output_stem(repo_url, &repo.short_hash),
    ))
}

pub fn run_with_args(args: Args) -> Result<()> {
    let tmp_dir = tempfile::Builder::new().prefix("flatten_").tempdir()?;
    let repo = scan::clone_repo(tmp_dir.path(), &args.repo_url)?;
    let files = scan::collect_files(&repo.path, args.max_bytes)?;
    let html = render::build_html(
        &args.repo_url,
        &files,
        &default_output_stem(&args.repo_url, &repo.short_hash),
    );
    let out = args
        .out
        .unwrap_or_else(|| default_output_path(&args.repo_url, &repo.short_hash));
    fs::write(&out, html)?;

    println!("✓ Flattened HTML generated at: {:?}", out);
    Ok(())
}

fn default_output_path(repo_url: &str, short_hash: &str) -> PathBuf {
    PathBuf::from(format!("{}.html", default_output_stem(repo_url, short_hash)))
}

fn default_output_stem(repo_url: &str, short_hash: &str) -> String {
    if short_hash == "unknown" {
        return "repo_flat".to_string();
    }

    format!("{}@{short_hash}", sanitize_filename(repo_url))
}

fn sanitize_filename(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut prev_underscore = false;

    for ch in input.chars() {
        let safe = matches!(ch, 'a'..='z' | 'A'..='Z' | '0'..='9' | '.' | '-' | '_');
        let mapped = if safe { ch } else { '_' };
        if mapped == '_' {
            if prev_underscore {
                continue;
            }
            prev_underscore = true;
        } else {
            prev_underscore = false;
        }
        out.push(mapped);
    }

    while out.ends_with('_') {
        out.pop();
    }

    if out.is_empty() {
        "repo".to_string()
    } else {
        out
    }
}
