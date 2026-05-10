use std::{
    io::Write,
    process::Command,
    path::{Path, PathBuf},
};

use chrono::TimeZone;
use anyhow::Context;

#[derive(Copy, Clone)]
struct Paths<'a> {
    statics_in_dir: &'a Path,
    statics_out_dir: &'a Path,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("An error occurred within the gnostr-web build script:\n\n{:?}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    report_build_name();
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?);
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").context("OUT_DIR not set by rustc")?);

    let paths = Paths {
        statics_in_dir: &manifest_dir,
        statics_out_dir: &out_dir,
    };

    build_scss(paths).context("Failed to build CSS stylesheets")?;
    build_js(paths).context("Failed to build JS bundle")?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn report_build_name() {
    let now = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => chrono::Local
            .timestamp_opt(val.parse::<i64>().unwrap(), 0)
            .unwrap(),
        Err(_) => chrono::Local::now(),
    };
    let build_date = now.date_naive();
    let build_name = if std::env::var("GITUI_RELEASE").is_ok() {
        format!("{}-{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    } else {
        format!(
            "{}-{} {} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            build_date,
            get_git_hash()
        )
    };

    println!("cargo:warning=buildname '{build_name}'");
    println!("cargo:rustc-env=GITUI_BUILD_NAME={build_name}");
}

fn get_git_hash() -> String {
    if let Ok(commit) = std::env::var("BUILD_GIT_COMMIT_ID") {
        return commit[..7].to_string();
    }

    let commit = Command::new("git")
        .arg("rev-parse")
        .arg("--short=7")
        .arg("--verify")
        .arg("HEAD")
        .output();

    if let Ok(commit_output) = commit {
        let commit_string = String::from_utf8_lossy(&commit_output.stdout);
        return commit_string.lines().next().unwrap_or("").into();
    }

    panic!("Can not get git commit: {}", commit.unwrap_err());
}

fn build_scss(paths: Paths) -> anyhow::Result<()> {
    let in_dir = paths.statics_in_dir.join("src/sass");
    let out_dir = paths.statics_out_dir.join("src/lib/css");
    std::fs::create_dir_all(&out_dir).context("Failed to create output directory")?;

    println!("cargo:rerun-if-changed={}", in_dir.display());

    for (input_name, output_name) in [("style.scss", "style.css"), ("nostr-styles.scss", "styles.css")] {
        let input_file = in_dir.join(input_name);
        let output_file = out_dir.join(output_name);
        let format = rsass::output::Format {
            style: rsass::output::Style::Compressed,
            ..rsass::output::Format::default()
        };

        let output_content =
            rsass::compile_scss_path(&input_file, format).context("Failed to compile SASS")?;

        let mut output_file = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .open(output_file)
            .context("Failed to open output file")?;
        output_file
            .write_all(&output_content)
            .context("Failed to write compiled CSS to output")?;
    }

    Ok(())
}

fn build_js(paths: Paths) -> anyhow::Result<()> {
    let out_dir = paths.statics_out_dir.join("src/lib/js");
    std::fs::create_dir_all(&out_dir).context("Failed to create output directory for JS")?;

    let mut all_js_content = String::new();
    let js_dir = paths.statics_in_dir.join("../js/src/js");
    let mut append_asset = |path: PathBuf| -> anyhow::Result<()> {
        let content = std::fs::read(&path)
            .with_context(|| format!("Failed to read JS asset {}", path.display()))?;
        let content = std::str::from_utf8(&content)
            .with_context(|| format!("JS asset {} is not UTF-8", path.display()))?;
        all_js_content.push_str(content);
        all_js_content.push('\n');
        Ok(())
    };

    append_asset(js_dir.join("util.js"))?;

    let mut root_js_files: Vec<PathBuf> = std::fs::read_dir(&js_dir)
        .with_context(|| format!("Failed to read JS source dir {}", js_dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("js"))
        .filter(|path| path.file_name().and_then(|name| name.to_str()) != Some("util.js"))
        .collect();
    root_js_files.sort();
    for path in root_js_files {
        append_asset(path)?;
    }

    let ui_dir = js_dir.join("ui");
    let mut ui_js_files: Vec<PathBuf> = std::fs::read_dir(&ui_dir)
        .with_context(|| format!("Failed to read UI JS dir {}", ui_dir.display()))?
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.is_file() && path.extension().and_then(|ext| ext.to_str()) == Some("js"))
        .collect();
    ui_js_files.sort();
    for path in ui_js_files {
        append_asset(path)?;
    }

    let output_file = out_dir.join("bundle.js");
    let mut output_file = std::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(output_file)
        .context("Failed to open output JS bundle file")?;
    output_file
        .write_all(all_js_content.as_bytes())
        .context("Failed to write compiled JS bundle to output")?;

    Ok(())
}
