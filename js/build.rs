use chrono::TimeZone;
use serde_json::json;
use std::fs;
use std::path::Path;
use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");

    let now = match std::env::var("SOURCE_DATE_EPOCH") {
        Ok(val) => chrono::Local
            .timestamp_opt(val.parse::<i64>().unwrap(), 0)
            .unwrap(),
        Err(_) => chrono::Local::now(),
    };
    let build_date = now.date_naive();
    let build_name = if std::env::var("GITUI_RELEASE").is_ok() {
        format!("{}@{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
    } else {
        format!(
            "{}@{} {} ({})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            build_date,
            get_git_hash()
        )
    };

    println!("cargo:warning=buildname '{build_name}'");
    println!("cargo:rustc-env=GITUI_BUILD_NAME={build_name}");
    let app_metadata = json!({
        "name": "gnostr",
        "description": env!("CARGO_PKG_DESCRIPTION"),
        "website": env!("CARGO_PKG_HOMEPAGE"),
        "repository": env!("CARGO_PKG_REPOSITORY"),
        "build_name": build_name,
        "kind": 31990,
    });
    println!("cargo:rustc-env=GITUI_APP_NAME=gnostr");
    println!(
        "cargo:rustc-env=GITUI_APP_METADATA_JSON={}",
        app_metadata.to_string()
    );
    write_nip89_app_asset(&build_name);
    write_js_hashes_asset();
    watch_sources(Path::new("src/js"), &["js"]);
    watch_sources(Path::new("src/css"), &["css"]);
    watch_sources(Path::new("src"), &["html"]);
}

fn write_nip89_app_asset(build_name: &str) {
    let out_dir = match std::env::var("OUT_DIR") {
        Ok(dir) => dir,
        Err(error) => {
            println!("cargo:warning=skipping NIP-89 app asset: {error}");
            return;
        }
    };

    let description = env!("CARGO_PKG_DESCRIPTION");
    let metadata = serde_json::json!({
        "kind": 31990,
        "pubkey": "",
        "content": {
            "name": "gnostr",
            "about": if description.is_empty() { "git+nostr workflow utility" } else { description },
            "website": env!("CARGO_PKG_HOMEPAGE"),
            "repository": env!("CARGO_PKG_REPOSITORY"),
            "picture": "/images/logo.svg",
            "version": env!("CARGO_PKG_VERSION"),
            "build_name": build_name,
        },
        "tags": [
            ["d", "gnostr"],
            ["k", "0"],
            ["k", "1"],
            ["k", "3"],
            ["k", "4"],
            ["k", "10002"],
            ["k", "31989"],
            ["k", "31990"],
            ["web", "/settings"],
        ]
    });

    let path = Path::new(&out_dir).join("nip89-app.json");
    let data = match serde_json::to_vec_pretty(&metadata) {
        Ok(data) => data,
        Err(error) => {
            println!("cargo:warning=skipping NIP-89 app asset: {error}");
            return;
        }
    };

    if let Err(error) = fs::write(&path, data) {
        println!("cargo:warning=failed to write NIP-89 app asset: {error}");
    } else {
        println!("cargo:rerun-if-changed={}", path.display());
    }
}

fn write_js_hashes_asset() {
    let mut entries: Vec<(String, String)> = Vec::new();
    collect_js_files(Path::new("src/js"), Path::new("src/js"), &mut entries);

    let mut map = serde_json::Map::new();
    for (rel, hash) in entries {
        map.insert(rel, serde_json::Value::String(hash));
    }
    let data = match serde_json::to_vec_pretty(&map) {
        Ok(data) => data,
        Err(error) => {
            println!("cargo:warning=failed to serialize JS hashes: {error}");
            return;
        }
    };

    let src_path = Path::new("src/js/hashes.json");
    if let Err(error) = fs::write(&src_path, &data) {
        println!("cargo:warning=failed to write JS hashes asset: {error}");
    } else {
        println!("cargo:rerun-if-changed={}", src_path.display());
    }

    if let Ok(out_dir) = std::env::var("OUT_DIR") {
        let out_path = Path::new(&out_dir).join("hashes.json");
        if let Err(error) = fs::write(&out_path, &data) {
            println!("cargo:warning=failed to write JS hashes asset (out): {error}");
        } else {
            println!("cargo:rerun-if-changed={}", out_path.display());
        }
    }
}

fn collect_js_files(base: &Path, dir: &Path, out: &mut Vec<(String, String)>) {
    use sha2::{Digest, Sha256};

    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_js_files(base, &path, out);
            continue;
        }
        if path.extension().and_then(|e| e.to_str()) != Some("js") {
            continue;
        }
        let Ok(bytes) = fs::read(&path) else {
            continue;
        };
        let mut hasher = Sha256::new();
        hasher.update(&bytes);
        let hash = hasher
            .finalize()
            .iter()
            .map(|b| format!("{:02x}", b))
            .collect::<String>();

        let rel = path
            .strip_prefix(base)
            .unwrap_or(&path)
            .to_string_lossy()
            .replace('\\', "/");
        out.push((rel, hash));
    }
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

fn watch_sources(dir: &Path, extensions: &[&str]) {
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
            watch_sources(&path, extensions);
            continue;
        }

        if path
            .extension()
            .and_then(|ext| ext.to_str())
            .is_some_and(|ext| extensions.contains(&ext))
        {
            println!("cargo:rerun-if-changed={}", path.display());
        }
    }
}
