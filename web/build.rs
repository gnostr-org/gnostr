use std::{
    io::Write,
    path::{Path, PathBuf},
};

use anyhow::Context;
use std::collections::BTreeMap;

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

    let assets = gnostr_js::get_js_assets();
    let mut ordered_assets: BTreeMap<String, &'static [u8]> = BTreeMap::new();
    for (name, bytes) in assets {
        ordered_assets.insert(name, bytes);
    }

    let mut all_js_content = String::new();
    let mut append_asset = |name: &str| -> anyhow::Result<()> {
        let content = ordered_assets
            .get(name)
            .copied()
            .with_context(|| format!("Missing JS asset: {name}"))?;
        let content = std::str::from_utf8(content).with_context(|| format!("JS asset {name} is not UTF-8"))?;
        all_js_content.push_str(content);
        all_js_content.push('\n');
        Ok(())
    };

    append_asset("util.js")?;

    let mut root_js_files: Vec<String> = ordered_assets
        .keys()
        .filter(|name| name.ends_with(".js") && !name.contains('/') && name.as_str() != "util.js")
        .cloned()
        .collect();
    root_js_files.sort();
    for name in root_js_files {
        append_asset(&name)?;
    }

    let mut ui_js_files: Vec<String> = ordered_assets
        .keys()
        .filter(|name| name.starts_with("ui/") && name.ends_with(".js"))
        .cloned()
        .collect();
    ui_js_files.sort();
    for name in ui_js_files {
        append_asset(&name)?;
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
