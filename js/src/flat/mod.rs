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

pub fn run_with_args(args: Args) -> Result<()> {
    let tmp_dir = tempfile::Builder::new().prefix("flatten_").tempdir()?;
    let repo_path = scan::clone_repo(tmp_dir.path(), &args.repo_url)?;
    let files = scan::collect_files(&repo_path, args.max_bytes)?;
    let html = render::build_html(&args.repo_url, &files);
    let out = args.out.unwrap_or_else(|| PathBuf::from("repo_flat.html"));
    fs::write(&out, html)?;

    println!("✓ Flattened HTML generated at: {:?}", out);
    Ok(())
}
