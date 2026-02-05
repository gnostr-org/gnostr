use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;

#[derive(Copy, Clone)]
pub struct Paths<'a> {
    statics_in_dir: &'a Path,
    statics_out_dir: &'a Path,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("An error occurred within the rgit build script:\n\n{:?}", e);
        std::process::exit(1);
    }
}

fn run() -> anyhow::Result<()> {
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?);
    let statics_in_dir = manifest_dir;

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").context("OUT_DIR not set by rustc")?);
    let statics_out_dir = out_dir;

    let paths = Paths {
        statics_in_dir: &statics_in_dir,
        statics_out_dir: &statics_out_dir,
    };

    build_scss(paths).context("Failed to build CSS stylesheets")?;
    build_js(paths).context("Failed to build JS bundle")?;


    //
    let manifest_dir =
        PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").context("CARGO_MANIFEST_DIR not set")?);
    let statics_in_dir = manifest_dir;

    let out_dir = PathBuf::from(std::env::var("OUT_DIR").context("OUT_DIR not set by rustc")?);
    //
    let out_dir = Path::new(".");
    let statics_out_dir = out_dir;

    let paths = Paths {
        statics_in_dir: &statics_in_dir,
        statics_out_dir: &statics_out_dir,
    };

    build_scss(paths).context("Failed to build CSS stylesheets")?;
    build_js(paths).context("Failed to build JS bundle")?;

    println!("cargo:rerun-if-changed=build.rs");
    Ok(())
}

fn build_scss(paths: Paths) -> anyhow::Result<()> {

    //gnostr-gnit
    let in_dir = paths.statics_in_dir.join("src/lib/sass");
    let out_dir = paths.statics_out_dir.join("src/lib/css");
    std::fs::create_dir_all(&out_dir).context("Failed to create output directory")?;

    println!("cargo:rerun-if-changed={}", in_dir.display());

    let input_file = in_dir.join("style.scss");
    let output_file = out_dir.join("style.css");
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

    //gnostr-web
    let input_file = in_dir.join("nostr-styles.scss");
    let output_file = out_dir.join("styles.css");
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

    Ok(())
}

fn build_js(paths: Paths) -> anyhow::Result<()> {
    let in_dir = paths.statics_in_dir.join("src/lib/js");
    let ui_in_dir = in_dir.join("ui");
    let out_dir = paths.statics_out_dir.join("src/lib/js");
    std::fs::create_dir_all(&out_dir).context("Failed to create output directory for JS")?;

    println!("cargo:rerun-if-changed={}", in_dir.display());
    println!("cargo:rerun-if-changed={}", ui_in_dir.display());

    let mut all_js_content = String::new();

    // Explicitly add util.js first
    let util_js_path = in_dir.join("util.js");
    println!("cargo:rerun-if-changed={}", util_js_path.display());
    let util_js_content = std::fs::read_to_string(&util_js_path).context(format!(
        "Failed to read JS file: {}",
        util_js_path.display()
    ))?;
    all_js_content.push_str(&util_js_content);
    all_js_content.push_str("\n"); // Add newline for concatenation

    // Collect and sort JS files from statics/js, excluding util.js for deterministic builds
    let mut js_files: Vec<PathBuf> = std::fs::read_dir(&in_dir)
        .context("Failed to read statics/js directory")?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| {
            path.is_file()
                && path.extension().map_or(false, |ext| ext == "js")
                && *path != util_js_path
        })
        .collect();
    js_files.sort();

    for path in js_files {
        println!("cargo:rerun-if-changed={}", path.display());
        let content = std::fs::read_to_string(&path)
            .context(format!("Failed to read JS file: {}", path.display()))?;
        all_js_content.push_str(&content);
        all_js_content.push_str("\n"); // Add newline for concatenation
    }

    // Collect and sort JS files from statics/js/ui for deterministic builds
    let mut ui_js_files: Vec<PathBuf> = std::fs::read_dir(&ui_in_dir)
        .context("Failed to read statics/js/ui directory")?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && path.extension().map_or(false, |ext| ext == "js"))
        .collect();
    ui_js_files.sort();

    for path in ui_js_files {
        println!("cargo:rerun-if-changed={}", path.display());
        let content = std::fs::read_to_string(&path)
            .context(format!("Failed to read JS file: {}", path.display()))?;
        all_js_content.push_str(&content);
        all_js_content.push_str("\n"); // Add newline for concatenation
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

    //We output to CARGO_MANIFEST_DIR/src/lib/js/ also

    Ok(())
}
