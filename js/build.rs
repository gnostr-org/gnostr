use std::fs;
use std::path::Path;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    watch_js_sources(Path::new("src/js"));
}

fn watch_js_sources(dir: &Path) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(err) => {
            panic!("failed to read {}: {}", dir.display(), err);
        }
    };

    for entry in entries {
        let entry = entry.unwrap_or_else(|err| {
            panic!("failed to read entry in {}: {}", dir.display(), err);
        });
        let path = entry.path();
        if path.is_dir() {
            watch_js_sources(&path);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) == Some("js") {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
